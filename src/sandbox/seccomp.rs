// Logic: Configures Seccomp-BPF system call filtering to restrict untrusted code execution.
// Input: None.
// Output: Result<(), Box<dyn std::error::Error>> after loading BPF filter rules into kernel.

use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};

// Logic: Constructs and loads Seccomp-BPF syscall whitelist filter to prevent dangerous syscalls.
// Input: None.
// Output: Result<(), Box<dyn std::error::Error>> indicating filter installation status.
pub fn apply_seccomp_filter() -> Result<(), Box<dyn std::error::Error>> {
    // Default action: Kill process on any unlisted system call
    let mut filter = ScmpFilterContext::new_filter(ScmpAction::KillProcess)?;

    let allowed_syscalls = [
        "read",
        "write",
        "close",
        "stat",
        "fstat",
        "lstat",
        "lseek",
        "mmap",
        "mprotect",
        "munmap",
        "brk",
        "rt_sigaction",
        "rt_sigprocmask",
        "rt_sigreturn",
        "ioctl",
        "pread64",
        "pwrite64",
        "readv",
        "writev",
        "access",
        "pipe",
        "select",
        "sched_yield",
        "mremap",
        "nanosleep",
        "getitimer",
        "setitimer",
        "getpid",
        "exit",
        "uname",
        "fcntl",
        "fsync",
        "fdatasync",
        "getcwd",
        "time",
        "futex",
        "set_thread_area",
        "get_thread_area",
        "io_setup",
        "io_destroy",
        "io_getevents",
        "io_submit",
        "io_cancel",
        "exit_group",
        "epoll_create",
        "epoll_ctl",
        "epoll_wait",
        "set_tid_address",
        "restart_syscall",
        "clock_gettime",
        "clock_getres",
        "clock_nanosleep",
        "tgkill",
        "openat",
        "newfstatat",
        "set_robust_list",
        "get_robust_list",
        "rseq",
        "prlimit64",
        "arch_prctl",
        "getrusage",
        "gettimeofday",
        "getuid",
        "geteuid",
        "getgid",
        "getegid",
    ];

    for name in &allowed_syscalls {
        if let Ok(syscall) = ScmpSyscall::from_name(name) {
            let _ = filter.add_rule(ScmpAction::Allow, syscall);
        }
    }

    // Load filter into current thread and inherited children
    filter.load()?;
    Ok(())
}
