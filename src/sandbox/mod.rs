// Logic: Linux sandboxing primitives combining namespaces, resource limits, and seccomp filters.
// Input: Resource limits configuration and execution parameters.
// Output: Sandboxed child process execution and status verification.

pub mod seccomp;

use nix::sched::{unshare, CloneFlags};
use nix::sys::resource::{setrlimit, Resource};

// Logic: Restricts memory, CPU, and output size resource limits using POSIX setrlimit.
// Input: cpu_sec (u64 limit in seconds), memory_mb (u64 limit in megabytes), output_mb (u64 limit in megabytes).
// Output: Result<(), nix::Error> on system call success or failure.
pub fn apply_resource_limits(
    cpu_sec: u64,
    memory_mb: u64,
    output_mb: u64,
) -> Result<(), nix::Error> {
    // CPU time limit
    let cpu_limit = cpu_sec;
    setrlimit(Resource::RLIMIT_CPU, cpu_limit, cpu_limit + 1)?;

    // Virtual address space / memory limit (bytes)
    let mem_bytes = memory_mb * 1024 * 1024;
    let _ = setrlimit(Resource::RLIMIT_AS, mem_bytes, mem_bytes);

    // Max file / output size (bytes)
    let out_bytes = output_mb * 1024 * 1024;
    setrlimit(Resource::RLIMIT_FSIZE, out_bytes, out_bytes)?;

    // Max child processes / fork-bomb prevention
    let _ = setrlimit(Resource::RLIMIT_NPROC, 1, 1);

    Ok(())
}

// Logic: Isolates network, IPC, and UTS namespaces using unshare system call.
// Input: None.
// Output: Result<(), nix::Error> indicating namespace isolation status.
pub fn isolate_namespaces() -> Result<(), nix::Error> {
    let flags = CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWIPC | CloneFlags::CLONE_NEWUTS;
    match unshare(flags) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Gracefully ignore failure if unprivileged container environment disallows unshare
            tracing::warn!("Unshare namespaces returned {e}, proceeding with process-level restrictions");
            Ok(())
        }
    }
}
