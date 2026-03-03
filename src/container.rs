use crate::config::ContainerConfig;
use nix::sched::{unshare, CloneFlags};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// The entry point for the host process.
/// This function spawns a new child process that will become the container.
pub fn start(config: ContainerConfig) {
    println!("🚀 Host: Starting container process...");

    // 1. Set up Cgroups for resource limitations
    let cgroup = crate::cgroups::Cgroup::new(&config.hostname);
    if let Some(limit_mb) = config.memory_mb {
        cgroup.set_memory_limit(limit_mb);
    }
    
    // We get the path so we can write to it from within the child's pre_exec hook
    let cgroup_procs_path = cgroup.procs_path();

    let mut child = Command::new("/proc/self/exe");

    // Pass the arguments to the `child` subcommand
    child.arg("child");
    child.arg("--rootfs").arg(&config.rootfs);
    child.arg("--command").arg(&config.command);
    child.arg("--hostname").arg(&config.hostname);

    unsafe {
        child.pre_exec(move || {
            // A. Attach this process to the Cgroup BEFORE we unshare or fork.
            // This guarantees that all children (like our `init` process) will
            // inherit these resource limits automatically.
            let pid = std::process::id();
            if let Err(e) = std::fs::write(&cgroup_procs_path, pid.to_string()) {
                eprintln!("❌ Failed to attach PID to cgroup: {}", e);
                std::process::exit(1);
            }

            // B. Unshare everything INCLUDING the PID namespace.
            // Remember: unshare(CLONE_NEWPID) only affects FUTURE children.
            let flags = CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWNET
                | CloneFlags::CLONE_NEWPID;

            if let Err(e) = unshare(flags) {
                eprintln!("❌ Failed to unshare namespaces: {}", e);
                std::process::exit(1);
            }
            Ok(())
        });
    }

    let mut process = child.spawn().expect("❌ Failed to spawn child process");
    let status = process.wait().expect("❌ Failed to wait on child process");

    // 2. Clean up Cgroups
    cgroup.clean();

    println!("🛑 Host: Container exited with status: {}", status);
}

/// The intermediate child process.
/// This process is inside the new namespaces (UTS, Mount, etc.) but it is
/// NOT in the new PID namespace yet because it just called unshare().
/// It must spawn one more process (the Grandchild) to be PID 1.
pub fn child(config: ContainerConfig) {
    println!("👶 Child: Forking again to enter new PID namespace...");

    let mut init = Command::new("/proc/self/exe");

    // Pass the arguments to the `init` subcommand
    init.arg("init");
    init.arg("--rootfs").arg(&config.rootfs);
    init.arg("--command").arg(&config.command);
    init.arg("--hostname").arg(&config.hostname);

    // We don't need any pre_exec hooks here because we are already unshared.
    // The simple act of spawning creates a new process which will inherit
    // all namespaces AND be placed into the new PID namespace.
    let mut process = init.spawn().expect("❌ Failed to spawn init process");
    let status = process.wait().expect("❌ Failed to wait on init process");

    std::process::exit(status.code().unwrap_or(1));
}

/// The final Init process (PID 1 inside the container).
/// This function finalizes the environment and executes the user's app.
pub fn init(config: ContainerConfig) {
    println!("📦 Init (PID 1): Finalizing container environment...");

    // 1. Set the hostname (UTS namespace)
    crate::namespaces::set_hostname(&config.hostname);

    // 2. Set up the filesystem (Mount namespace and /proc)
    crate::mounts::setup_rootfs(&config.rootfs);

    // 3. Execute the target command
    println!("🚀 Init (PID 1): Executing command: {}", config.command);

    let err = Command::new(&config.command).exec();

    eprintln!("❌ Init: Failed to execute command: {}", err);
    std::process::exit(1);
}
