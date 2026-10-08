// Logic: Source code compilation manager for C, C++, Rust, and syntax validation for Python.
// Input: Language identifier, source code string, and output working directory.
// Output: Result<std::path::PathBuf, String> with compiled binary path or compiler diagnostic message.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Logic: Compiles source code into an executable binary or prepares script file.
// Input: language (&str identifier), source_code (&str source), work_dir (&Path scratch location).
// Output: Result<PathBuf, String> path to executable or compilation error diagnostics.
pub fn compile_source(
    language: &str,
    source_code: &str,
    work_dir: &Path,
) -> Result<PathBuf, String> {
    fs::create_dir_all(work_dir).map_err(|e| format!("Failed to create work directory: {e}"))?;

    match language.to_lowercase().as_str() {
        "cpp" | "c++" => {
            let src_file = work_dir.join("solution.cpp");
            let bin_file = work_dir.join("solution.out");
            fs::write(&src_file, source_code).map_err(|e| e.to_string())?;

            let output = Command::new("g++")
                .args([
                    "-O3",
                    "-std=c++20",
                    "-Wall",
                    "-Wextra",
                    src_file.to_str().unwrap(),
                    "-o",
                    bin_file.to_str().unwrap(),
                ])
                .output()
                .map_err(|e| format!("Failed to invoke g++ compiler: {e}"))?;

            if output.status.success() {
                Ok(bin_file)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(stderr)
            }
        }
        "c" => {
            let src_file = work_dir.join("solution.c");
            let bin_file = work_dir.join("solution.out");
            fs::write(&src_file, source_code).map_err(|e| e.to_string())?;

            let output = Command::new("gcc")
                .args([
                    "-O3",
                    "-std=c11",
                    "-Wall",
                    src_file.to_str().unwrap(),
                    "-o",
                    bin_file.to_str().unwrap(),
                ])
                .output()
                .map_err(|e| format!("Failed to invoke gcc compiler: {e}"))?;

            if output.status.success() {
                Ok(bin_file)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(stderr)
            }
        }
        "rust" | "rs" => {
            let src_file = work_dir.join("solution.rs");
            let bin_file = work_dir.join("solution.out");
            fs::write(&src_file, source_code).map_err(|e| e.to_string())?;

            let output = Command::new("rustc")
                .args([
                    "-O",
                    src_file.to_str().unwrap(),
                    "-o",
                    bin_file.to_str().unwrap(),
                ])
                .output()
                .map_err(|e| format!("Failed to invoke rustc compiler: {e}"))?;

            if output.status.success() {
                Ok(bin_file)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(stderr)
            }
        }
        "python" | "py" | "python3" => {
            let src_file = work_dir.join("solution.py");
            fs::write(&src_file, source_code).map_err(|e| e.to_string())?;

            let output = Command::new("python3")
                .args(["-m", "py_compile", src_file.to_str().unwrap()])
                .output()
                .map_err(|e| format!("Failed to invoke python3 syntax validator: {e}"))?;

            if output.status.success() {
                Ok(src_file)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(stderr)
            }
        }
        other => Err(format!("Unsupported programming language: {other}")),
    }
}
