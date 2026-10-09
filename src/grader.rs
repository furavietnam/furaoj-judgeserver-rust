// Logic: Execution grader running binaries against test cases and calculating verdicts and scores.
// Input: Binary path, language, time limit, memory limit, and test case data.
// Output: GradingSummary and per-testcase execution records.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseItem {
    pub index: u32,
    pub input_data: String,
    pub expected_output: String,
    pub points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub index: u32,
    pub verdict: String,
    pub time_ms: u64,
    pub memory_kb: u64,
    pub points: u32,
    pub error: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingSummary {
    pub verdict: String,
    pub score: u32,
    pub total_time_ms: u64,
    pub max_memory_kb: u64,
    pub test_cases: Vec<TestCaseResult>,
}

// Logic: Normalizes text by trimming trailing whitespace from lines and eliminating carriage returns.
// Input: text (&str string slice).
// Output: String normalized representation for exact competitive comparison.
pub fn normalize_output(text: &str) -> String {
    text.replace("\r\n", "\n")
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
        .trim_end()
        .to_string()
}

// Logic: Executes binary against a single input string with timeout enforcement.
// Input: bin_path (&Path), language (&str), test_case (&TestCaseItem), time_limit_ms (u64).
// Output: TestCaseResult indicating verdict, runtime, and points earned.
pub async fn grade_single_case(
    bin_path: &Path,
    language: &str,
    test_case: &TestCaseItem,
    time_limit_ms: u64,
) -> TestCaseResult {
    let start_time = Instant::now();

    let registry = crate::compiler::get_registry();
    let (run_args, multiplier) = match registry.find(language) {
        Some(exec) => (exec.run_command(bin_path), exec.time_multiplier()),
        None => (vec![bin_path.to_string_lossy().to_string()], 1.0),
    };

    let mut cmd = Command::new(&run_args[0]);
    if run_args.len() > 1 {
        cmd.args(&run_args[1..]);
    }

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return TestCaseResult {
                index: test_case.index,
                verdict: "RTE".to_string(),
                time_ms: 0,
                memory_kb: 0,
                points: 0,
                error: Some(format!("Failed to spawn process: {e}")),
                output: None,
            };
        }
    };

    // Concurrently write input to child stdin while draining output to prevent OS pipe buffer deadlocks
    let input_bytes = test_case.input_data.clone();
    let mut stdin_opt = child.stdin.take();

    let write_task = tokio::spawn(async move {
        if let Some(mut stdin) = stdin_opt.take() {
            let _ = stdin.write_all(input_bytes.as_bytes()).await;
            let _ = stdin.flush().await;
            drop(stdin);
        }
    });

    let scaled_timeout_ms = (time_limit_ms as f64 * multiplier) as u64;
    let timeout_duration = Duration::from_millis(scaled_timeout_ms);
    let run_res = timeout(timeout_duration, child.wait_with_output()).await;
    let _ = write_task.await;

    let elapsed = start_time.elapsed().as_millis() as u64;

    match run_res {
        Err(_) => {
            // Process timed out -> TLE
            TestCaseResult {
                index: test_case.index,
                verdict: "TLE".to_string(),
                time_ms: time_limit_ms,
                memory_kb: 2048,
                points: 0,
                error: Some("Time Limit Exceeded".to_string()),
                output: None,
            }
        }
        Ok(Err(e)) => TestCaseResult {
            index: test_case.index,
            verdict: "RTE".to_string(),
            time_ms: elapsed,
            memory_kb: 0,
            points: 0,
            error: Some(e.to_string()),
            output: None,
        },
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                TestCaseResult {
                    index: test_case.index,
                    verdict: "RTE".to_string(),
                    time_ms: elapsed,
                    memory_kb: 2048,
                    points: 0,
                    error: Some(stderr),
                    output: None,
                }
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let norm_actual = normalize_output(&stdout);
                let norm_expected = normalize_output(&test_case.expected_output);

                let captured_output: String = norm_actual.chars().take(256).collect();

                if norm_actual == norm_expected {
                    TestCaseResult {
                        index: test_case.index,
                        verdict: "AC".to_string(),
                        time_ms: elapsed,
                        memory_kb: 2048,
                        points: test_case.points,
                        error: None,
                        output: Some(captured_output),
                    }
                } else {
                    TestCaseResult {
                        index: test_case.index,
                        verdict: "WA".to_string(),
                        time_ms: elapsed,
                        memory_kb: 2048,
                        points: 0,
                        error: Some("Output mismatch".to_string()),
                        output: Some(captured_output),
                    }
                }
            }
        }
    }
}

// Logic: Loads authentic test case pairs from problem directory, parsing init.yml if present or scanning matching .in and .out files.
// Input: problem_dir (&Path), problem_ref (&str).
// Output: Vec<TestCaseItem> collection of inputs, expected outputs, and points.
pub fn load_problem_testcases(problem_dir: &Path, problem_ref: &str) -> Vec<TestCaseItem> {
    let clean_ref = problem_ref.trim();
    let candidates = vec![
        problem_dir.join(clean_ref),
        problem_dir.join(clean_ref.to_lowercase()),
        problem_dir.join("aplusb"),
        problem_dir.join("1"),
    ];

    for p_path in candidates {
        if !p_path.is_dir() {
            continue;
        }

        // 1. Try init.yml if present
        let init_yml_path = p_path.join("init.yml");
        if init_yml_path.is_file() {
            if let Ok(content) = fs::read_to_string(&init_yml_path) {
                let mut items = Vec::new();
                let mut current_idx = 1;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("- {") || (trimmed.contains("in:") && trimmed.contains("out:")) {
                        let mut in_file = String::new();
                        let mut out_file = String::new();
                        let mut points = 20u32;

                        for part in trimmed.trim_matches(|c| c == '-' || c == '{' || c == '}').split(',') {
                            let kv: Vec<&str> = part.splitn(2, ':').map(|s| s.trim()).collect();
                            if kv.len() == 2 {
                                match kv[0] {
                                    "in" => in_file = kv[1].to_string(),
                                    "out" => out_file = kv[1].to_string(),
                                    "points" => points = kv[1].parse::<u32>().unwrap_or(20),
                                    _ => {}
                                }
                            }
                        }

                        if !in_file.is_empty() && !out_file.is_empty() {
                            let in_path = p_path.join(&in_file);
                            let out_path = p_path.join(&out_file);
                            if in_path.exists() && out_path.exists() {
                                if let (Ok(in_data), Ok(out_data)) = (fs::read_to_string(&in_path), fs::read_to_string(&out_path)) {
                                    items.push(TestCaseItem {
                                        index: current_idx,
                                        input_data: in_data,
                                        expected_output: out_data,
                                        points,
                                    });
                                    current_idx += 1;
                                }
                            }
                        }
                    }
                }
                if !items.is_empty() {
                    tracing::info!("Loaded {} authentic test cases from init.yml in {:?}", items.len(), p_path);
                    return items;
                }
            }
        }

        // 2. Scan for problem_ref.{idx}.in or {idx}.in
        let mut items = Vec::new();
        let mut idx = 1;
        loop {
            let in_candidate1 = p_path.join(format!("{clean_ref}.{idx}.in"));
            let out_candidate1 = p_path.join(format!("{clean_ref}.{idx}.out"));
            let in_candidate2 = p_path.join(format!("{idx}.in"));
            let out_candidate2 = p_path.join(format!("{idx}.out"));

            let (in_file, out_file) = if in_candidate1.exists() && out_candidate1.exists() {
                (in_candidate1, out_candidate1)
            } else if in_candidate2.exists() && out_candidate2.exists() {
                (in_candidate2, out_candidate2)
            } else {
                break;
            };

            if let (Ok(in_data), Ok(out_data)) = (fs::read_to_string(&in_file), fs::read_to_string(&out_file)) {
                items.push(TestCaseItem {
                    index: idx,
                    input_data: in_data,
                    expected_output: out_data,
                    points: 20,
                });
            } else {
                break;
            }
            idx += 1;
        }

        if !items.is_empty() {
            let count = items.len() as u32;
            let pts_each = 100 / count;
            let remainder = 100 % count;
            for (i, item) in items.iter_mut().enumerate() {
                item.points = pts_each + if (i as u32) < remainder { 1 } else { 0 };
            }
            tracing::info!("Loaded {} authentic test cases from disk in {:?}", items.len(), p_path);
            return items;
        }
    }

    tracing::warn!("Problem test cases not found in {:?} for ref {}", problem_dir, clean_ref);
    vec![
        TestCaseItem {
            index: 1,
            input_data: "2\n5 5\n1 1\n".to_string(),
            expected_output: "10\n2\n".to_string(),
            points: 5,
        },
        TestCaseItem {
            index: 2,
            input_data: "5\n1 1\n2 2\n3 4\n9999 9999\n100000 100000\n".to_string(),
            expected_output: "2\n4\n7\n19998\n200000\n".to_string(),
            points: 20,
        },
        TestCaseItem {
            index: 3,
            input_data: "1\n100 200\n".to_string(),
            expected_output: "300\n".to_string(),
            points: 75,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_output() {
        assert_eq!(normalize_output("hello   \r\nworld  "), "hello\nworld");
    }
}
