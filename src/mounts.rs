use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::unistd::pivot_root;
use std::env::set_current_dir;
use std::fs;
use std::path::Path;

/// Sets up the container's root filesystem.
pub fn setup_rootfs(rootfs: &Path) {
    // 1. Remount the root directory as private
    // This is required so that any mounts we do here don't leak back to the host machine.
    // The MS_REC flag makes it recursive (applies to all submounts).
    if let Err(e) = mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    ) {
        eprintln!("❌ Failed to remount / as private: {}", e);
        std::process::exit(1);
    }

    // 2. Bind mount the new root to itself
    // Kernel rule: pivot_root requires the new root and the old root to be on different mounts.
    // A bind mount to itself artificially satisfies this rule.
    if let Err(e) = mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    ) {
        eprintln!("❌ Failed to bind mount rootfs to itself: {}", e);
        std::process::exit(1);
    }

    // 3. Create a directory to hold the old root temporarily
    let old_root = rootfs.join(".oldroot");
    if let Err(e) = fs::create_dir_all(&old_root) {
        eprintln!("❌ Failed to create .oldroot folder: {}", e);
        std::process::exit(1);
    }

    // 4. Execute pivot_root
    // This physically swaps the mount namespace's root directory,
    // placing the host's root into the `.oldroot` directory.
    if let Err(e) = pivot_root(rootfs, &old_root) {
        eprintln!("❌ pivot_root failed: {}", e);
        std::process::exit(1);
    }

    // 5. Change the current working directory to the new root
    if let Err(e) = set_current_dir("/") {
        eprintln!("❌ Failed to change directory to /: {}", e);
        std::process::exit(1);
    }

    // 6. Unmount the old root filesystem
    // We use MNT_DETACH so we don't have to worry about processes still using it.
    if let Err(e) = umount2("/.oldroot", MntFlags::MNT_DETACH) {
        eprintln!("❌ Failed to unmount /.oldroot: {}", e);
        std::process::exit(1);
    }

    // 7. Remove the temporary .oldroot directory
    if let Err(e) = fs::remove_dir("/.oldroot") {
        eprintln!("❌ Failed to remove /.oldroot directory: {}", e);
        std::process::exit(1);
    }

    // 8. Mount pseudo-filesystems (/proc, /sys, /dev)
    mount_pseudo_filesystems();
}

/// Mounts essential Linux pseudo-filesystems inside the container.
fn mount_pseudo_filesystems() {
    // --- Mount /proc ---
    let proc_path = Path::new("/proc");
    if !proc_path.exists() {
        let _ = fs::create_dir_all(proc_path);
    }
    if let Err(e) = mount(
        Some("proc"),
        proc_path,
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    ) {
        eprintln!("❌ Failed to mount /proc: {}", e);
        std::process::exit(1);
    }

    // --- Mount /sys ---
    // /sys is mounted read-only for security, just like Docker does it.
    let sys_path = Path::new("/sys");
    if !sys_path.exists() {
        let _ = fs::create_dir_all(sys_path);
    }
    if let Err(e) = mount(
        Some("sysfs"),
        sys_path,
        Some("sysfs"),
        MsFlags::MS_RDONLY | MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
        None::<&str>,
    ) {
        eprintln!("❌ Failed to mount /sys: {}", e);
        // We don't exit here, as some minimal containers can survive without /sys
    }

    // --- Mount /dev (tmpfs) ---
    // A tmpfs is an in-memory filesystem. We use it for /dev because device
    // nodes should not persist to disk, and the container needs its own private /dev.
    let dev_path = Path::new("/dev");
    if !dev_path.exists() {
        let _ = fs::create_dir_all(dev_path);
    }
    if let Err(e) = mount(
        Some("tmpfs"),
        dev_path,
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_STRICTATIME,
        Some("mode=755,size=65536k"),
    ) {
        eprintln!("❌ Failed to mount tmpfs on /dev: {}", e);
        std::process::exit(1);
    }

    // Note for the future: A complete runtime would now manually execute mknod()
    // inside this tmpfs to create /dev/null, /dev/zero, /dev/urandom, etc.
    // For now, many basic CLI tools will function simply by having the tmpfs exist.
}

