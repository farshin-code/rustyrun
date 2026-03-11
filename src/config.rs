use std::path::PathBuf;

/// Holds the configuration for the container we want to run.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// The path to the directory that will become the container's root (/)
    pub rootfs: PathBuf,
    /// The command to execute inside the container (e.g., "/bin/sh")
    pub command: String,
    /// The hostname to assign to the container
    pub hostname: String,
    /// The maximum memory limit in Megabytes (optional)
    pub memory_mb: Option<u64>,
    /// The generated guest veth interface name
    pub veth_guest: String,
}

impl ContainerConfig {
    pub fn new(rootfs: PathBuf, command: String, hostname: String, memory_mb: Option<u64>, veth_guest: String) -> Self {
        Self {
            rootfs,
            command,
            hostname,
            memory_mb,
            veth_guest,
        }
    }
}
