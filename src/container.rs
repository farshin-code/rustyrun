use crate::config::ContainerConfig;
use nix::sched::{unshare, CloneFlags};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// The entry point for the host process.
/// This function spawns a new child process that will become the container.
pub fn start(config: ContainerConfig) {
    println!("🚀 Host: Starting container process...");

    // We use a trick here: we tell the current executable to run itself again,
    // but we pass the "child" subcommand instead of "run".
    // This allows us to execute our own Rust code inside the new namespaces.
    let mut child = Command::new("/proc/self/exe");

    // Pass the arguments to the child process
    child.arg("child");
    child.arg("--rootfs").arg(&config.rootfs);
    child.arg("--command").arg(&config.command);
    child.arg("--hostname").arg(&config.hostname);

    // This is the magic part: The pre_exec hook runs in the child process
    // *after* it forks, but *before* it executes the new binary (/proc/self/exe).
    // This is where we isolate the process from the host.
    unsafe {
        child.pre_exec(|| {
            // Create new namespaces for Mount, UTS (hostname), IPC, and Network.
            // Note: We are NOT creating a new PID namespace here yet, because
            // unshare(CLONE_NEWPID) only applies to *children* of the calling process.
            // We will handle PID isolation later when we upgrade to `clone`.
            let flags = CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWNET;

            if let Err(e) = unshare(flags) {
                eprintln!("❌ Failed to unshare namespaces: {}", e);
                std::process::exit(1);
            }
            Ok(())
        });
    }

    // Spawn the child process and wait for it to finish
    let mut process = child.spawn().expect("❌ Failed to spawn child process");
    let status = process.wait().expect("❌ Failed to wait on child process");

    println!("🛑 Host: Container exited with status: {}", status);
}

/// The entry point for the child process.
/// This function runs *inside* the newly created namespaces.
pub fn init_child(config: ContainerConfig) {
    println!("📦 Child: Initializing container environment...");

    // 1. Set the hostname (UTS namespace)
    crate::namespaces::set_hostname(&config.hostname);

    // 2. Set up the filesystem (Mount namespace)
    crate::mounts::setup_rootfs(&config.rootfs);

    // 3. Execute the target command
    println!("🚀 Child: Executing command: {}", config.command);

    // We use CommandExt::exec() here. This is crucial!
    // It completely replaces the current Rust process (rustyrun child)
    // with the target program (e.g., /bin/sh).
    // If exec() succeeds, it never returns.
    let err = Command::new(&config.command).exec();

    // If we reach this line, exec() failed.
    eprintln!("❌ Child: Failed to execute command: {}", err);
    std::process::exit(1);
}
