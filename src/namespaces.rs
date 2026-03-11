use nix::unistd::sethostname;
use std::fs;

/// Sets the hostname for the container.
/// This only affects the container because it runs inside a new UTS namespace.
pub fn set_hostname(hostname: &str) {
    if let Err(e) = sethostname(hostname) {
        eprintln!("❌ Failed to set hostname: {}", e);
        std::process::exit(1);
    }
}

/// Maps the user and group IDs between the host and the container.
/// By mapping UID 0 (root inside container) to the host's actual unprivileged UID,
/// we achieve "Rootless Containers" where the process thinks it's root but is actually harmless.
pub fn setup_user_mapping(pid: u32, host_uid: u32, host_gid: u32) {
    let uid_map = format!("0 {} 1\n", host_uid);
    let gid_map = format!("0 {} 1\n", host_gid);

    // Write UID mapping
    let uid_path = format!("/proc/{}/uid_map", pid);
    if let Err(e) = fs::write(&uid_path, uid_map.as_bytes()) {
        eprintln!("❌ Failed to write uid_map: {}", e);
    }

    // Must deny setgroups before writing gid_map for unprivileged accounts
    let setgroups_path = format!("/proc/{}/setgroups", pid);
    if let Err(e) = fs::write(&setgroups_path, b"deny") {
        eprintln!("⚠️ Failed to write setgroups (may not be supported/needed on this kernel): {}", e);
    }

    // Write GID mapping
    let gid_path = format!("/proc/{}/gid_map", pid);
    if let Err(e) = fs::write(&gid_path, gid_map.as_bytes()) {
        eprintln!("❌ Failed to write gid_map: {}", e);
    }
    
    println!("🔐 Namespaces: Mapped Container Root (0) to Host User ({}/{})", host_uid, host_gid);
}
