use crate::config::ContainerConfig;
use crate::network::Network;
use nix::sched::{unshare, CloneFlags};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// The entry point for the host process.
/// This function spawns a new child process that will become the container.
pub fn start(config: ContainerConfig, network: Network) {
    println!("🚀 Host: Starting container process...");

    // 1. Set up Cgroups for resource limitations
    let cgroup = crate::cgroups::Cgroup::new(&config.hostname);
    if let Some(limit_mb) = config.memory_mb {
        cgroup.set_memory_limit(limit_mb);
    }
    let cgroup_procs_path = cgroup.procs_path();

    let mut child = Command::new("/proc/self/exe");

    // Pass the arguments to the `child` subcommand
    child.arg("child");
    child.arg("--rootfs").arg(&config.rootfs);
    child.arg("--command").arg(&config.command);
    child.arg("--hostname").arg(&config.hostname);
    child.arg("--veth-guest").arg(&config.veth_guest);

    unsafe {
        child.pre_exec(move || {
            let pid = std::process::id();
            if let Err(e) = std::fs::write(&cgroup_procs_path, pid.to_string()) {
                eprintln!("❌ Failed to attach PID to cgroup: {}", e);
                std::process::exit(1);
            }

            let flags = CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWNET
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWUSER;

            if let Err(e) = unshare(flags) {
                eprintln!("❌ Failed to unshare namespaces: {}", e);
                std::process::exit(1);
            }
            Ok(())
        });
    }

    let mut process = child.spawn().expect("❌ Failed to spawn child process");
    
    // MAPPING MAGIC: Map the container's root user (UID 0) to the host's actual user
    // This allows the container to think it's root without having native root powers on the host
    let host_uid = unsafe { libc::getuid() };
    let host_gid = unsafe { libc::getgid() };
    crate::namespaces::setup_user_mapping(process.id(), host_uid, host_gid);

    // NETWORK MAGIC: Now that the child process is spawned, it has a new Network Namespace.
    // We attach the host's veth cable to the child's namespace PID.
    network.setup_veth_pair(process.id());

    let status = process.wait().expect("❌ Failed to wait on child process");

    // 2. Clean up Cgroups & Network
    cgroup.clean();
    network.clean();

    println!("🛑 Host: Container exited with status: {}", status);
}

/// The intermediate child process.
pub fn child(config: ContainerConfig) {
    println!("👶 Child: Forking again to enter new PID namespace...");

    let mut init = Command::new("/proc/self/exe");

    // Pass the arguments to the `init` subcommand
    init.arg("init");
    init.arg("--rootfs").arg(&config.rootfs);
    init.arg("--command").arg(&config.command);
    init.arg("--hostname").arg(&config.hostname);
    init.arg("--veth-guest").arg(&config.veth_guest);

    let mut process = init.spawn().expect("❌ Failed to spawn init process");
    let status = process.wait().expect("❌ Failed to wait on init process");

    std::process::exit(status.code().unwrap_or(1));
}

/// The final Init process (PID 1 inside the container).
pub fn init(config: ContainerConfig) {
    println!("📦 Init (PID 1): Finalizing container environment...");

    // 1. Set the hostname (UTS namespace)
    crate::namespaces::set_hostname(&config.hostname);

    // 2. Configure the Network (IP, Loopback, routing)
    // We must do this BEFORE pivot_root because `ip` command exists on host, not necessarily in Alpine.
    // Because we are in the new Net Namespace, this only affects the container.
    let network = Network { 
        veth_host: String::new(), // Not needed inside container
        veth_guest: config.veth_guest,
    };
    network.configure_guest();

    // 3. Set up the filesystem (Mount namespace and /proc, /sys, /dev)
    crate::mounts::setup_rootfs(&config.rootfs);

    // 4. Execute the target command
    println!("🚀 Init (PID 1): Executing command: {}", config.command);

    let err = Command::new(&config.command).exec();

    eprintln!("❌ Init: Failed to execute command: {}", err);
    std::process::exit(1);
}
