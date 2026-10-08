// Logic: Main entry point for FuraOJ Rust Judge Server daemon.
// Input: Command line options and judge_config.json.
// Output: Starts Tokio async runtime and executes the network judge worker client loop.

pub mod client;
pub mod compiler;
pub mod config;
pub mod grader;
pub mod protocol;
pub mod sandbox;

use config::load_config;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Initializing FuraOJ Rust Judge Server v2.0.0...");

    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "judge_config.json".to_string()
    };

    let config = load_config(Path::new(&config_path))?;
    tracing::info!(
        "Loaded configuration for judge worker '{}' targeting Bridge at {}:{}",
        config.judge.name,
        config.judge.bridge_host,
        config.judge.bridge_port
    );

    client::run_client_loop(config).await;

    Ok(())
}
