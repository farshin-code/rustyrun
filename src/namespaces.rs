use nix::unistd::sethostname;

/// Sets the hostname for the container.
/// This only affects the container because it runs inside a new UTS namespace.
pub fn set_hostname(hostname: &str) {
    if let Err(e) = sethostname(hostname) {
        eprintln!("❌ Failed to set hostname: {}", e);
        std::process::exit(1);
    }
}
