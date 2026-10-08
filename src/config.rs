// Logic: Strongly typed configuration models and JSON loader for Rust Judge Server.
// Input: JSON configuration file content or filesystem path.
// Output: Deserialized JudgeConfig data structure.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeSettings {
    pub name: String,
    pub key: String,
    pub bridge_host: String,
    pub bridge_port: u16,
    pub problem_dir: String,
    pub sandbox_tmpfs_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsSettings {
    pub max_cpu_time: f64,
    pub max_memory_mb: u64,
    pub max_output_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    pub judge: JudgeSettings,
    pub limits: LimitsSettings,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            judge: JudgeSettings {
                name: "default-judge".to_string(),
                key: "judge_secret_authentication_key".to_string(),
                bridge_host: "127.0.0.1".to_string(),
                bridge_port: 9999,
                problem_dir: "./problems".to_string(),
                sandbox_tmpfs_size_mb: 64,
            },
            limits: LimitsSettings {
                max_cpu_time: 10.0,
                max_memory_mb: 1024,
                max_output_mb: 16,
            },
        }
    }
}

// Logic: Reads configuration file from disk or falls back to built-in default settings.
// Input: path (&Path config file path).
// Output: Result<JudgeConfig, Box<dyn std::error::Error>> containing resolved configuration.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<JudgeConfig, Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    if path_ref.exists() {
        let content = fs::read_to_string(path_ref)?;
        let config: JudgeConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        Ok(JudgeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = JudgeConfig::default();
        assert_eq!(cfg.judge.name, "default-judge");
        assert_eq!(cfg.judge.bridge_port, 9999);
    }
}
