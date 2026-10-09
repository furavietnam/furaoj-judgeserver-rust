// Logic: Universal language executor trait and registry supporting TierFuraOJ language specifications.
// Input: Source file path, compile flags, target limits, and runtime parameters.
// Output: Command line vectors and environment constraints for compilation and sandboxed execution.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

// Logic: Universal trait defining compilation, sandboxing, and runtime behaviors.
// Input: Source file path, compile flags, target limits.
// Output: Command line vectors and environment constraints for execution.
pub trait LanguageExecutor: Send + Sync {
    // Logic: Returns unique canonical language key.
    // Input: None.
    // Output: Static string identifier.
    fn key(&self) -> &'static str;

    // Logic: Returns human-readable display name.
    // Input: None.
    // Output: Static string description.
    fn display_name(&self) -> &'static str;

    // Logic: Returns standard file extension for language source code.
    // Input: None.
    // Output: Static string extension without dot.
    fn source_extension(&self) -> &'static str;

    // Logic: Generates command line argument vector for compilation step.
    // Input: source_path (&Path), output_path (&Path).
    // Output: Option<Vec<String>> containing compiler binary and arguments, or None for interpreted scripts.
    fn compile_command(&self, source_path: &Path, output_path: &Path) -> Option<Vec<String>>;

    // Logic: Generates command line argument vector for runtime execution step.
    // Input: binary_path (&Path to compiled binary or source script).
    // Output: Vec<String> containing runner binary and arguments.
    fn run_command(&self, binary_path: &Path) -> Vec<String>;

    // Logic: Returns time multiplier for execution timeout scaling.
    // Input: None.
    // Output: Floating-point multiplier (e.g. 2.5 for Python 3).
    fn time_multiplier(&self) -> f64 {
        1.0
    }

    // Logic: Returns memory grace allocation in megabytes.
    // Input: None.
    // Output: Additional memory quota in MB.
    fn memory_grace_mb(&self) -> u64 {
        0
    }

    // Logic: Returns extra syscall names required in sandbox seccomp filter.
    // Input: None.
    // Output: Slice of static string syscall names.
    fn extra_syscall_whitelist(&self) -> &'static [&'static str] {
        &[]
    }

    // Logic: Performs pre-execution integrity checks on submission payload.
    // Input: source_path (&Path).
    // Output: Result<(), String> indicating validity.
    fn pre_validation(&self, _source_path: &Path) -> Result<(), String> {
        Ok(())
    }
}

// Logic: Basic compiled language executor generator for C-like and compiled languages.
// Input: Configuration parameters for compiler and linker.
// Output: Implements LanguageExecutor trait.
pub struct GenericCompiledExecutor {
    pub key_name: &'static str,
    pub display: &'static str,
    pub ext: &'static str,
    pub compiler: &'static str,
    pub args: &'static [&'static str],
    pub out_flag_style: OutFlagStyle,
    pub time_mult: f64,
}

#[derive(Clone, Copy)]
pub enum OutFlagStyle {
    Separate,
    Joined,
}

impl LanguageExecutor for GenericCompiledExecutor {
    // Logic: Returns configured canonical language key.
    // Input: None.
    // Output: Static string identifier.
    fn key(&self) -> &'static str {
        self.key_name
    }

    // Logic: Returns configured display name.
    // Input: None.
    // Output: Static string description.
    fn display_name(&self) -> &'static str {
        self.display
    }

    // Logic: Returns configured source extension.
    // Input: None.
    // Output: Static string extension.
    fn source_extension(&self) -> &'static str {
        self.ext
    }

    // Logic: Builds compiler command using configured binary and flags.
    // Input: source_path (&Path), output_path (&Path).
    // Output: Option<Vec<String>> command argument vector.
    fn compile_command(&self, source_path: &Path, output_path: &Path) -> Option<Vec<String>> {
        let mut cmd = vec![self.compiler.to_string()];
        for arg in self.args {
            cmd.push(arg.to_string());
        }
        match self.out_flag_style {
            OutFlagStyle::Separate => {
                cmd.push("-o".to_string());
                cmd.push(output_path.to_string_lossy().to_string());
            }
            OutFlagStyle::Joined => {
                cmd.push(format!("-o{}", output_path.to_string_lossy()));
            }
        }
        cmd.push(source_path.to_string_lossy().to_string());
        Some(cmd)
    }

    // Logic: Direct binary execution without interpreter wrapper.
    // Input: binary_path (&Path).
    // Output: Vector containing binary path.
    fn run_command(&self, binary_path: &Path) -> Vec<String> {
        vec![binary_path.to_string_lossy().to_string()]
    }

    // Logic: Configured time multiplier.
    // Input: None.
    // Output: f64 multiplier.
    fn time_multiplier(&self) -> f64 {
        self.time_mult
    }
}

// Logic: Basic interpreted language executor generator for script engines.
// Input: Configuration parameters for interpreter and runtime flags.
// Output: Implements LanguageExecutor trait.
pub struct GenericInterpretedExecutor {
    pub key_name: &'static str,
    pub display: &'static str,
    pub ext: &'static str,
    pub interpreter: &'static str,
    pub run_flags: &'static [&'static str],
    pub time_mult: f64,
}

impl LanguageExecutor for GenericInterpretedExecutor {
    // Logic: Returns configured canonical language key.
    // Input: None.
    // Output: Static string identifier.
    fn key(&self) -> &'static str {
        self.key_name
    }

    // Logic: Returns configured display name.
    // Input: None.
    // Output: Static string description.
    fn display_name(&self) -> &'static str {
        self.display
    }

    // Logic: Returns configured source extension.
    // Input: None.
    // Output: Static string extension.
    fn source_extension(&self) -> &'static str {
        self.ext
    }

    // Logic: Interpreted languages do not require separate compilation step.
    // Input: source_path (&Path), output_path (&Path).
    // Output: None.
    fn compile_command(&self, _source_path: &Path, _output_path: &Path) -> Option<Vec<String>> {
        None
    }

    // Logic: Invokes interpreter with configured runtime arguments and script path.
    // Input: binary_path (&Path).
    // Output: Vector containing interpreter and arguments.
    fn run_command(&self, binary_path: &Path) -> Vec<String> {
        let mut cmd = vec![self.interpreter.to_string()];
        for flag in self.run_flags {
            cmd.push(flag.to_string());
        }
        cmd.push(binary_path.to_string_lossy().to_string());
        cmd
    }

    // Logic: Configured time multiplier for interpreted environments.
    // Input: None.
    // Output: f64 multiplier.
    fn time_multiplier(&self) -> f64 {
        self.time_mult
    }
}

// Logic: C++ Themis competitive programming executor with 64MB stack and -DTHEMIS define.
// Input: Standard C++ source code.
// Output: Compiler commands enforcing Vietnamese contest standards.
pub struct CppThemisExecutor;

impl LanguageExecutor for CppThemisExecutor {
    // Logic: Returns CPPTHEMIS key.
    // Input: None.
    // Output: Static str CPPTHEMIS.
    fn key(&self) -> &'static str {
        "CPPTHEMIS"
    }

    // Logic: Returns C++ (Themis) display name.
    // Input: None.
    // Output: Static str display name.
    fn display_name(&self) -> &'static str {
        "C++ (Themis)"
    }

    // Logic: Returns cpp file extension.
    // Input: None.
    // Output: Static str cpp.
    fn source_extension(&self) -> &'static str {
        "cpp"
    }

    // Logic: Builds g++ command with 64MB stack flag and THEMIS define.
    // Input: source_path (&Path), output_path (&Path).
    // Output: Option<Vec<String>> compile command arguments.
    fn compile_command(&self, source_path: &Path, output_path: &Path) -> Option<Vec<String>> {
        Some(vec![
            "g++".into(),
            "-std=c++14".into(),
            "-pipe".into(),
            "-O2".into(),
            "-s".into(),
            "-static".into(),
            "-lm".into(),
            "-x".into(),
            "c++".into(),
            "-Wl,-z,stack-size=66060288".into(),
            "-DTHEMIS".into(),
            "-o".into(),
            output_path.to_string_lossy().to_string(),
            source_path.to_string_lossy().to_string(),
        ])
    }

    // Logic: Executes compiled binary directly.
    // Input: binary_path (&Path).
    // Output: Vector containing binary path.
    fn run_command(&self, binary_path: &Path) -> Vec<String> {
        vec![binary_path.to_string_lossy().to_string()]
    }
}

// Logic: Free Pascal Themis executor with 64MB stack allocation and -dTHEMIS preprocessor flag.
// Input: Standard Pascal source code.
// Output: fpc compiler commands matching Vietnamese contest specifications.
pub struct PasThemisExecutor;

impl LanguageExecutor for PasThemisExecutor {
    // Logic: Returns PASTHEMIS key.
    // Input: None.
    // Output: Static str PASTHEMIS.
    fn key(&self) -> &'static str {
        "PASTHEMIS"
    }

    // Logic: Returns Pascal (Themis) display name.
    // Input: None.
    // Output: Static str display name.
    fn display_name(&self) -> &'static str {
        "Pascal (Themis)"
    }

    // Logic: Returns pas extension.
    // Input: None.
    // Output: Static str pas.
    fn source_extension(&self) -> &'static str {
        "pas"
    }

    // Logic: Builds fpc command with -dTHEMIS and 64MB stack allocation.
    // Input: source_path (&Path), output_path (&Path).
    // Output: Option<Vec<String>> compile command arguments.
    fn compile_command(&self, source_path: &Path, output_path: &Path) -> Option<Vec<String>> {
        Some(vec![
            "fpc".into(),
            "-Fe/dev/stderr".into(),
            "-dTHEMIS".into(),
            "-O2".into(),
            "-XS".into(),
            "-Sg".into(),
            "-Cs66060288".into(),
            format!("-o{}", output_path.to_string_lossy()),
            source_path.to_string_lossy().to_string(),
        ])
    }

    // Logic: Executes compiled Pascal binary directly.
    // Input: binary_path (&Path).
    // Output: Vector containing binary path.
    fn run_command(&self, binary_path: &Path) -> Vec<String> {
        vec![binary_path.to_string_lossy().to_string()]
    }
}

// Logic: Scratch 3.0 visual programming executor evaluating .sb3 project archives.
// Input: Scratch 3.0 archive file.
// Output: Validation heuristics and scratch-run execution vector with relaxed sandbox syscalls.
pub struct ScratchExecutor;

impl LanguageExecutor for ScratchExecutor {
    // Logic: Returns SCRATCH key.
    // Input: None.
    // Output: Static str SCRATCH.
    fn key(&self) -> &'static str {
        "SCRATCH"
    }

    // Logic: Returns Scratch 3.0 display name.
    // Input: None.
    // Output: Static str display name.
    fn display_name(&self) -> &'static str {
        "Scratch 3.0"
    }

    // Logic: Returns sb3 project archive extension.
    // Input: None.
    // Output: Static str sb3.
    fn source_extension(&self) -> &'static str {
        "sb3"
    }

    // Logic: Scratch is evaluated by runner; no separate compilation phase.
    // Input: source_path (&Path), output_path (&Path).
    // Output: None.
    fn compile_command(&self, _source_path: &Path, _output_path: &Path) -> Option<Vec<String>> {
        None
    }

    // Logic: Invokes scratch-run runner binary.
    // Input: binary_path (&Path to .sb3 archive).
    // Output: Vector containing scratch-run and archive path.
    fn run_command(&self, binary_path: &Path) -> Vec<String> {
        vec![
            "scratch-run".into(),
            binary_path.to_string_lossy().to_string(),
        ]
    }

    // Logic: Extra memory grace for Chromium/Node.js VM in scratch runner.
    // Input: None.
    // Output: 1 MB memory grace.
    fn memory_grace_mb(&self) -> u64 {
        1
    }

    // Logic: Syscall whitelist required by scratch-run VM execution.
    // Input: None.
    // Output: Slice of syscall strings.
    fn extra_syscall_whitelist(&self) -> &'static [&'static str] {
        &[
            "capget",
            "eventfd2",
            "shutdown",
            "pkey_alloc",
            "pkey_free",
            "io_uring_setup",
        ]
    }

    // Logic: Validates .sb3 file header and scratch-run syntax before grading.
    // Input: source_path (&Path).
    // Output: Result<(), String> indicating valid project archive.
    fn pre_validation(&self, source_path: &Path) -> Result<(), String> {
        let bytes = fs::read(source_path).map_err(|e| format!("Failed to read source: {e}"))?;
        if bytes.len() < 4 || &bytes[0..2] != b"PK" {
            return Err("Not a valid Scratch file: missing ZIP header".to_string());
        }

        let output = Command::new("scratch-run")
            .args(["--check", source_path.to_str().unwrap_or_default()])
            .output();

        if let Ok(out) = output {
            if !out.status.success() {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                return Err(format!("Not a valid Scratch file: {err}"));
            }
        }
        Ok(())
    }
}

// Logic: Central registry containing all supported TierFuraOJ language executors and aliases.
// Input: Thread-safe singleton initialized once.
// Output: Registry lookups by canonical key or standard alias.
pub struct LanguageRegistry {
    executors: HashMap<&'static str, Box<dyn LanguageExecutor>>,
    aliases: HashMap<&'static str, &'static str>,
}

impl LanguageRegistry {
    // Logic: Initializes full suite of over 60 TierFuraOJ language executors and lookup aliases.
    // Input: None.
    // Output: LanguageRegistry instance with populated collections.
    pub fn new() -> Self {
        let mut executors: HashMap<&'static str, Box<dyn LanguageExecutor>> = HashMap::new();
        let mut aliases: HashMap<&'static str, &'static str> = HashMap::new();

        // C Family
        executors.insert(
            "C",
            Box::new(GenericCompiledExecutor {
                key_name: "C",
                display: "C (GCC)",
                ext: "c",
                compiler: "gcc",
                args: &["-O2", "-std=c11", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "C11",
            Box::new(GenericCompiledExecutor {
                key_name: "C11",
                display: "C11 (GCC)",
                ext: "c",
                compiler: "gcc",
                args: &["-O2", "-std=gnu11", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "C23",
            Box::new(GenericCompiledExecutor {
                key_name: "C23",
                display: "C23 (GCC)",
                ext: "c",
                compiler: "gcc",
                args: &["-O2", "-std=gnu23", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CICPC",
            Box::new(GenericCompiledExecutor {
                key_name: "CICPC",
                display: "C (ICPC)",
                ext: "c",
                compiler: "gcc",
                args: &["-O2", "-std=gnu11", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CLANG",
            Box::new(GenericCompiledExecutor {
                key_name: "CLANG",
                display: "C (Clang)",
                ext: "c",
                compiler: "clang",
                args: &["-O2", "-std=c11", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );

        // C++ Family
        executors.insert(
            "CPP03",
            Box::new(GenericCompiledExecutor {
                key_name: "CPP03",
                display: "C++03 (GCC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=c++03", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CPP11",
            Box::new(GenericCompiledExecutor {
                key_name: "CPP11",
                display: "C++11 (GCC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=c++11", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CPP14",
            Box::new(GenericCompiledExecutor {
                key_name: "CPP14",
                display: "C++14 (GCC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=c++14", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CPP17",
            Box::new(GenericCompiledExecutor {
                key_name: "CPP17",
                display: "C++17 (GCC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=c++17", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CPP20",
            Box::new(GenericCompiledExecutor {
                key_name: "CPP20",
                display: "C++20 (GCC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=c++20", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CPP23",
            Box::new(GenericCompiledExecutor {
                key_name: "CPP23",
                display: "C++23 (GCC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=c++23", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CPPICPC",
            Box::new(GenericCompiledExecutor {
                key_name: "CPPICPC",
                display: "C++ (ICPC)",
                ext: "cpp",
                compiler: "g++",
                args: &["-O2", "-std=gnu++20", "-static", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert("CPPTHEMIS", Box::new(CppThemisExecutor));
        executors.insert(
            "CLANGX",
            Box::new(GenericCompiledExecutor {
                key_name: "CLANGX",
                display: "C++ (Clang++)",
                ext: "cpp",
                compiler: "clang++",
                args: &["-O2", "-std=c++17", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CLPP14",
            Box::new(GenericCompiledExecutor {
                key_name: "CLPP14",
                display: "C++14 (Clang++)",
                ext: "cpp",
                compiler: "clang++",
                args: &["-O2", "-std=c++14", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CLPP17",
            Box::new(GenericCompiledExecutor {
                key_name: "CLPP17",
                display: "C++17 (Clang++)",
                ext: "cpp",
                compiler: "clang++",
                args: &["-O2", "-std=c++17", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CLPP20",
            Box::new(GenericCompiledExecutor {
                key_name: "CLPP20",
                display: "C++20 (Clang++)",
                ext: "cpp",
                compiler: "clang++",
                args: &["-O2", "-std=c++20", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "CLPP23",
            Box::new(GenericCompiledExecutor {
                key_name: "CLPP23",
                display: "C++23 (Clang++)",
                ext: "cpp",
                compiler: "clang++",
                args: &["-O2", "-std=c++23", "-lm"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );

        // Pascal Family
        executors.insert(
            "PAS",
            Box::new(GenericCompiledExecutor {
                key_name: "PAS",
                display: "Pascal (FPC)",
                ext: "pas",
                compiler: "fpc",
                args: &["-O2", "-Fe/dev/stderr"],
                out_flag_style: OutFlagStyle::Joined,
                time_mult: 1.0,
            }),
        );
        executors.insert("PASTHEMIS", Box::new(PasThemisExecutor));

        // Scratch Visual Programming
        executors.insert("SCRATCH", Box::new(ScratchExecutor));

        // Python Family
        executors.insert(
            "PY2",
            Box::new(GenericInterpretedExecutor {
                key_name: "PY2",
                display: "Python 2",
                ext: "py",
                interpreter: "python2",
                run_flags: &["-B"],
                time_mult: 3.0,
            }),
        );
        executors.insert(
            "PY3",
            Box::new(GenericInterpretedExecutor {
                key_name: "PY3",
                display: "Python 3",
                ext: "py",
                interpreter: "python3",
                run_flags: &["-B"],
                time_mult: 2.5,
            }),
        );
        executors.insert(
            "PYPY",
            Box::new(GenericInterpretedExecutor {
                key_name: "PYPY",
                display: "PyPy 2",
                ext: "py",
                interpreter: "pypy",
                run_flags: &["-B"],
                time_mult: 1.5,
            }),
        );
        executors.insert(
            "PYPY3",
            Box::new(GenericInterpretedExecutor {
                key_name: "PYPY3",
                display: "PyPy 3",
                ext: "py",
                interpreter: "pypy3",
                run_flags: &["-B"],
                time_mult: 1.5,
            }),
        );

        // Native & Systems Languages
        executors.insert(
            "RUST",
            Box::new(GenericCompiledExecutor {
                key_name: "RUST",
                display: "Rust",
                ext: "rs",
                compiler: "rustc",
                args: &["-O", "--edition", "2021"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "GO",
            Box::new(GenericCompiledExecutor {
                key_name: "GO",
                display: "Go",
                ext: "go",
                compiler: "go",
                args: &["build", "-ldflags=-s -w"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "D",
            Box::new(GenericCompiledExecutor {
                key_name: "D",
                display: "D",
                ext: "d",
                compiler: "gdc",
                args: &["-O2"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "ZIG",
            Box::new(GenericCompiledExecutor {
                key_name: "ZIG",
                display: "Zig",
                ext: "zig",
                compiler: "zig",
                args: &["build-exe", "-O", "ReleaseFast"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "SWIFT",
            Box::new(GenericCompiledExecutor {
                key_name: "SWIFT",
                display: "Swift",
                ext: "swift",
                compiler: "swiftc",
                args: &["-O"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.0,
            }),
        );

        // JVM Family
        executors.insert(
            "JAVA",
            Box::new(GenericInterpretedExecutor {
                key_name: "JAVA",
                display: "Java",
                ext: "java",
                interpreter: "java",
                run_flags: &["-Xmx256M", "-Xss64M"],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "JAVA8",
            Box::new(GenericInterpretedExecutor {
                key_name: "JAVA8",
                display: "Java 8",
                ext: "java",
                interpreter: "java",
                run_flags: &["-Xmx256M", "-Xss64M"],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "KOTLIN",
            Box::new(GenericInterpretedExecutor {
                key_name: "KOTLIN",
                display: "Kotlin",
                ext: "kt",
                interpreter: "kotlinc",
                run_flags: &["-script"],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "SCALA",
            Box::new(GenericInterpretedExecutor {
                key_name: "SCALA",
                display: "Scala",
                ext: "scala",
                interpreter: "scala",
                run_flags: &[],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "GROOVY",
            Box::new(GenericInterpretedExecutor {
                key_name: "GROOVY",
                display: "Groovy",
                ext: "groovy",
                interpreter: "groovy",
                run_flags: &[],
                time_mult: 2.0,
            }),
        );

        // Scripting Languages
        executors.insert(
            "NODEJS",
            Box::new(GenericInterpretedExecutor {
                key_name: "NODEJS",
                display: "Node.js",
                ext: "js",
                interpreter: "node",
                run_flags: &["--max-old-space-size=256"],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "V8JS",
            Box::new(GenericInterpretedExecutor {
                key_name: "V8JS",
                display: "JavaScript (V8)",
                ext: "js",
                interpreter: "d8",
                run_flags: &[],
                time_mult: 1.5,
            }),
        );
        executors.insert(
            "BASH",
            Box::new(GenericInterpretedExecutor {
                key_name: "BASH",
                display: "Bash Shell",
                ext: "sh",
                interpreter: "bash",
                run_flags: &[],
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "AWK",
            Box::new(GenericInterpretedExecutor {
                key_name: "AWK",
                display: "AWK",
                ext: "awk",
                interpreter: "gawk",
                run_flags: &["-f"],
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "SED",
            Box::new(GenericInterpretedExecutor {
                key_name: "SED",
                display: "Sed",
                ext: "sed",
                interpreter: "sed",
                run_flags: &["-f"],
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "PERL",
            Box::new(GenericInterpretedExecutor {
                key_name: "PERL",
                display: "Perl",
                ext: "pl",
                interpreter: "perl",
                run_flags: &[],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "PHP",
            Box::new(GenericInterpretedExecutor {
                key_name: "PHP",
                display: "PHP",
                ext: "php",
                interpreter: "php",
                run_flags: &[],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "RUBY",
            Box::new(GenericInterpretedExecutor {
                key_name: "RUBY",
                display: "Ruby",
                ext: "rb",
                interpreter: "ruby",
                run_flags: &[],
                time_mult: 2.0,
            }),
        );
        executors.insert(
            "LUA",
            Box::new(GenericInterpretedExecutor {
                key_name: "LUA",
                display: "Lua",
                ext: "lua",
                interpreter: "lua",
                run_flags: &[],
                time_mult: 1.0,
            }),
        );
        executors.insert(
            "TCL",
            Box::new(GenericInterpretedExecutor {
                key_name: "TCL",
                display: "TCL",
                ext: "tcl",
                interpreter: "tclsh",
                run_flags: &[],
                time_mult: 1.0,
            }),
        );

        // Functional & Logic
        executors.insert(
            "HASK",
            Box::new(GenericCompiledExecutor {
                key_name: "HASK",
                display: "Haskell",
                ext: "hs",
                compiler: "ghc",
                args: &["-O2"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.5,
            }),
        );
        executors.insert(
            "OCAML",
            Box::new(GenericCompiledExecutor {
                key_name: "OCAML",
                display: "OCaml",
                ext: "ml",
                compiler: "ocamlopt",
                args: &["-O2"],
                out_flag_style: OutFlagStyle::Separate,
                time_mult: 1.2,
            }),
        );
        executors.insert(
            "SCM",
            Box::new(GenericInterpretedExecutor {
                key_name: "SCM",
                display: "Scheme",
                ext: "scm",
                interpreter: "guile",
                run_flags: &[],
                time_mult: 1.5,
            }),
        );
        executors.insert(
            "RKT",
            Box::new(GenericInterpretedExecutor {
                key_name: "RKT",
                display: "Racket",
                ext: "rkt",
                interpreter: "racket",
                run_flags: &[],
                time_mult: 1.5,
            }),
        );
        executors.insert(
            "SBCL",
            Box::new(GenericInterpretedExecutor {
                key_name: "SBCL",
                display: "Common Lisp",
                ext: "lisp",
                interpreter: "sbcl",
                run_flags: &["--script"],
                time_mult: 1.2,
            }),
        );
        executors.insert(
            "PRO",
            Box::new(GenericInterpretedExecutor {
                key_name: "PRO",
                display: "Prolog",
                ext: "pro",
                interpreter: "swipl",
                run_flags: &["-q", "-t", "halt", "-s"],
                time_mult: 2.0,
            }),
        );

        // Text & Plain
        executors.insert(
            "TEXT",
            Box::new(GenericInterpretedExecutor {
                key_name: "TEXT",
                display: "Plain Text",
                ext: "txt",
                interpreter: "cat",
                run_flags: &[],
                time_mult: 1.0,
            }),
        );

        // Standard Aliases Mapping
        aliases.insert("C++", "CPP20");
        aliases.insert("CPP", "CPP20");
        aliases.insert("G++", "CPP20");
        aliases.insert("GCC", "C");
        aliases.insert("PYTHON", "PY3");
        aliases.insert("PYTHON3", "PY3");
        aliases.insert("PY", "PY3");
        aliases.insert("PYTHON2", "PY2");
        aliases.insert("RS", "RUST");
        aliases.insert("PASCAL", "PAS");
        aliases.insert("FPC", "PAS");
        aliases.insert("THEMIS", "CPPTHEMIS");
        aliases.insert("CPP_THEMIS", "CPPTHEMIS");
        aliases.insert("PAS_THEMIS", "PASTHEMIS");
        aliases.insert("PASCAL_THEMIS", "PASTHEMIS");
        aliases.insert("SB3", "SCRATCH");
        aliases.insert("JS", "NODEJS");
        aliases.insert("NODE", "NODEJS");
        aliases.insert("GOLANG", "GO");

        Self { executors, aliases }
    }

    // Logic: Resolves language executor by key or case-insensitive alias.
    // Input: lang_key (&str).
    // Output: Option<&dyn LanguageExecutor>.
    pub fn find(&self, lang_key: &str) -> Option<&dyn LanguageExecutor> {
        let upper = lang_key.trim().to_uppercase();
        let target = self
            .aliases
            .get(upper.as_str())
            .copied()
            .unwrap_or(upper.as_str());

        self.executors.get(target).map(|b| b.as_ref())
    }

    // Logic: Returns all canonical registered executor keys for Bridge handshake announcement.
    // Input: None.
    // Output: Vec<&'static str> list of keys.
    pub fn all_keys(&self) -> Vec<&'static str> {
        let mut keys: Vec<&'static str> = self.executors.keys().copied().collect();
        keys.sort();
        keys
    }
}

// Logic: Global thread-safe singleton instance of LanguageRegistry.
// Input: None.
// Output: &'static LanguageRegistry reference.
pub fn get_registry() -> &'static LanguageRegistry {
    static REGISTRY: OnceLock<LanguageRegistry> = OnceLock::new();
    REGISTRY.get_or_init(LanguageRegistry::new)
}

// Logic: Compiles source code into an executable binary or prepares script file using language registry.
// Input: language (&str identifier), source_code (&str source), work_dir (&Path scratch location).
// Output: Result<PathBuf, String> path to executable or compilation error diagnostics.
pub fn compile_source(
    language: &str,
    source_code: &str,
    work_dir: &Path,
) -> Result<PathBuf, String> {
    fs::create_dir_all(work_dir).map_err(|e| format!("Failed to create work directory: {e}"))?;

    let registry = get_registry();
    let executor = match registry.find(language) {
        Some(exec) => exec,
        None => return Err(format!("Unsupported programming language: {language}")),
    };

    let src_file = work_dir.join(format!("solution.{}", executor.source_extension()));
    fs::write(&src_file, source_code).map_err(|e| e.to_string())?;

    executor.pre_validation(&src_file)?;

    let bin_file = work_dir.join("solution.out");
    if let Some(cmd_args) = executor.compile_command(&src_file, &bin_file) {
        let binary_cmd = &cmd_args[0];
        let args = &cmd_args[1..];

        let output = Command::new(binary_cmd)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to invoke compiler {binary_cmd}: {e}"))?;

        if output.status.success() {
            Ok(bin_file)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(stderr)
        }
    } else {
        // Interpreted language or direct script evaluation
        Ok(src_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_lookup_themis() {
        let registry = get_registry();
        let cpp_themis = registry.find("CPPTHEMIS");
        assert!(cpp_themis.is_some());
        let exec = cpp_themis.unwrap();
        assert_eq!(exec.key(), "CPPTHEMIS");
        assert_eq!(exec.source_extension(), "cpp");

        let cmd = exec.compile_command(Path::new("test.cpp"), Path::new("test.out"));
        assert!(cmd.is_some());
        let args = cmd.unwrap();
        assert!(args.contains(&"-DTHEMIS".to_string()));
        assert!(args.contains(&"-Wl,-z,stack-size=66060288".to_string()));
    }

    #[test]
    fn test_registry_lookup_pasthemis() {
        let registry = get_registry();
        let pas_themis = registry.find("PASTHEMIS");
        assert!(pas_themis.is_some());
        let exec = pas_themis.unwrap();
        assert_eq!(exec.key(), "PASTHEMIS");
        assert_eq!(exec.source_extension(), "pas");

        let cmd = exec.compile_command(Path::new("test.pas"), Path::new("test.out"));
        assert!(cmd.is_some());
        let args = cmd.unwrap();
        assert!(args.contains(&"-dTHEMIS".to_string()));
        assert!(args.contains(&"-Cs66060288".to_string()));
    }

    #[test]
    fn test_registry_aliases() {
        let registry = get_registry();
        assert_eq!(registry.find("cpp").unwrap().key(), "CPP20");
        assert_eq!(registry.find("python3").unwrap().key(), "PY3");
        assert_eq!(registry.find("rs").unwrap().key(), "RUST");
        assert_eq!(registry.find("sb3").unwrap().key(), "SCRATCH");
        assert_eq!(registry.find("pas_themis").unwrap().key(), "PASTHEMIS");
    }
}
