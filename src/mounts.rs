use nix::unistd::chroot;
use std::env::set_current_dir;
use std::path::Path;

/// Sets up the container's root filesystem.
/// For now, we use `chroot` for simplicity. We will upgrade to `pivot_root` later.
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
}
