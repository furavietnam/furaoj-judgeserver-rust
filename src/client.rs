// Logic: Network client communicating with FuraOJ Bridge, performing handshake, and handling submissions.
// Input: JudgeConfig configuration and runtime compiler availability.
// Output: Persistent connection loop streaming live evaluation packets.

use crate::compiler::compile_source;
use crate::config::JudgeConfig;
use crate::grader::{grade_single_case, load_problem_testcases, TestCaseResult};
use crate::protocol::{decode_packet, encode_packet};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

// Logic: Establishes resilient TCP connection to Bridge daemon, executes handshake, and processes packets.
// Input: config (JudgeConfig reference).
// Output: Runs indefinitely; returns error if non-recoverable error occurs.
pub async fn run_client_loop(config: JudgeConfig) {
    let target = format!("{}:{}", config.judge.bridge_host, config.judge.bridge_port);

    loop {
        tracing::info!("Connecting to Bridge daemon at {target}...");
        match TcpStream::connect(&target).await {
            Ok(stream) => {
                tracing::info!("Connected to Bridge. Executing authentication handshake...");
                if let Err(e) = handle_connection(stream, &config).await {
                    tracing::warn!("Bridge session terminated with error: {e}");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to connect to Bridge at {target}: {e}. Retrying in 2s...");
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
}

// Logic: Manages duplex packet streaming over an active TCP socket.
// Input: stream (TcpStream), config (&JudgeConfig).
// Output: Result<(), Box<dyn std::error::Error>> when connection finishes.
async fn handle_connection(
    mut stream: TcpStream,
    config: &JudgeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut read_half, mut write_half) = stream.split();

    // 1. Send Handshake packet
    let all_executors = crate::compiler::get_registry().all_keys();
    let handshake_payload = json!({
        "name": "handshake",
        "id": config.judge.name,
        "key": config.judge.key,
        "problems": HashMap::<String, u64>::new(),
        "executors": all_executors,
        "version": "2.0.0"
    });

    let encoded_handshake = encode_packet(&handshake_payload.to_string())?;
    write_half.write_all(&encoded_handshake).await?;
    tracing::info!("Handshake packet transmitted to Bridge.");

    // 2. Packet read loop
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(_) = read_half.read_exact(&mut len_buf).await {
            tracing::info!("Bridge closed connection.");
            break;
        }

        let packet_len = u32::from_be_bytes(len_buf) as usize;
        let mut compressed_buf = vec![0u8; packet_len];
        read_half.read_exact(&mut compressed_buf).await?;

        let decoded_json = decode_packet(&compressed_buf)?;
        let packet: Value = serde_json::from_str(&decoded_json)?;

        let packet_name = packet["name"].as_str().unwrap_or_default();

        match packet_name {
            "ping" => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let pong = json!({
                    "name": "ping-response",
                    "when": now
                });
                let encoded = encode_packet(&pong.to_string())?;
                write_half.write_all(&encoded).await?;
            }
            "submission-request" => {
                let sub_id = packet["submission_id"].as_i64().unwrap_or(0);
                let prob_id = packet["problem_id"].as_i64().unwrap_or(1);
                let language = packet["language"].as_str().unwrap_or("cpp");
                let source_code = packet["source_code"].as_str().unwrap_or("");
                let time_limit = packet["time_limit"].as_u64().unwrap_or(1000);

                // Acknowledge submission
                let ack = json!({
                    "name": "submission-acknowledged",
                    "submission_id": sub_id
                });
                write_half.write_all(&encode_packet(&ack.to_string())?).await?;

                // Compile source
                let work_dir = std::env::temp_dir().join(format!("furaoj_sub_{sub_id}"));
                let compile_res = compile_source(language, source_code, &work_dir);

                match compile_res {
                    Err(err_msg) => {
                        let ce_packet = json!({
                            "name": "grading-end",
                            "submission_id": sub_id,
                            "verdict": "CE",
                            "score": 0,
                            "error_message": err_msg,
                            "test_cases": []
                        });
                        write_half.write_all(&encode_packet(&ce_packet.to_string())?).await?;
                    }
                    Ok(bin_path) => {
                        // Grading begin
                        let begin_packet = json!({
                            "name": "grading-begin",
                            "submission_id": sub_id
                        });
                        write_half.write_all(&encode_packet(&begin_packet.to_string())?).await?;

                        let problem_dir = Path::new(&config.judge.problem_dir);
                        let test_cases = load_problem_testcases(problem_dir, prob_id);

                        let mut results: Vec<TestCaseResult> = Vec::new();
                        let mut total_score = 0;
                        let mut final_verdict = "AC".to_string();
                        let mut max_time = 0;
                        let mut max_mem = 0;

                        for tc in &test_cases {
                            let res = grade_single_case(&bin_path, language, tc, time_limit).await;
                            total_score += res.points;
                            if res.time_ms > max_time {
                                max_time = res.time_ms;
                            }
                            if res.memory_kb > max_mem {
                                max_mem = res.memory_kb;
                            }
                            if res.verdict != "AC" && final_verdict == "AC" {
                                final_verdict = res.verdict.clone();
                            }

                            // Stream case status
                            let case_packet = json!({
                                "name": "test-case-status",
                                "submission_id": sub_id,
                                "case_index": res.index,
                                "verdict": res.verdict,
                                "time_ms": res.time_ms,
                                "memory_kb": res.memory_kb,
                                "points": res.points
                            });
                            write_half.write_all(&encode_packet(&case_packet.to_string())?).await?;
                            results.push(res);
                        }

                        // Final grading packet
                        let end_packet = json!({
                            "name": "grading-end",
                            "submission_id": sub_id,
                            "verdict": final_verdict,
                            "score": total_score,
                            "time_ms": max_time,
                            "memory_kb": max_mem,
                            "test_cases": results
                        });
                        write_half.write_all(&encode_packet(&end_packet.to_string())?).await?;

                        // Clean up temporary files
                        let _ = std::fs::remove_dir_all(&work_dir);
                    }
                }
            }
            "terminate-submission" => {
                let sub_id = packet["submission_id"].as_i64().unwrap_or(0);
                let term = json!({
                    "name": "submission-terminated",
                    "submission_id": sub_id
                });
                write_half.write_all(&encode_packet(&term.to_string())?).await?;
            }
            _ => {
                tracing::debug!("Received unrecognized packet: {packet_name}");
            }
        }
    }

    Ok(())
}
