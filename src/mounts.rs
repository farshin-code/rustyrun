use nix::mount::{mount, MsFlags};
use nix::unistd::chroot;
use std::env::set_current_dir;
use std::fs;
use std::path::Path;

/// Sets up the container's root filesystem.
pub fn setup_rootfs(rootfs: &Path) {
    // 1. Change the root directory to the provided path
    if let Err(e) = chroot(rootfs) {
        eprintln!("❌ Failed to chroot to {}: {}", rootfs.display(), e);
        std::process::exit(1);
    }

    // 2. Change the current working directory to the new root
    if let Err(e) = set_current_dir("/") {
        eprintln!("❌ Failed to change directory to /: {}", e);
        std::process::exit(1);
    }

    // 3. Mount the proc pseudo-filesystem
    // This is required for tools like `ps`, `top`, and getting PID info.
    // Ensure the /proc directory exists in the new rootfs before mounting.
    let proc_path = Path::new("/proc");
    if !proc_path.exists() {
        if let Err(e) = fs::create_dir_all(proc_path) {
            eprintln!("❌ Failed to create /proc directory: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = mount(
        Some("proc"),      // source
        proc_path,         // target
        Some("proc"),      // filesystem type
        MsFlags::empty(),  // mount flags
        None::<&str>,      // data options
    ) {
        eprintln!("❌ Failed to mount /proc: {}", e);
        std::process::exit(1);
    }
}

