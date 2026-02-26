use std::path::PathBuf;

/// Holds the configuration for the container we want to run.
#[derive(Debug)]
pub struct ContainerConfig {
    /// The path to the directory that will become the container's root (/)
    pub rootfs: PathBuf,
    /// The command to execute inside the container (e.g., "/bin/sh")
    pub command: String,
    /// The hostname to assign to the container
    pub hostname: String,
}

impl ContainerConfig {
    pub fn new(rootfs: PathBuf, command: String, hostname: String) -> Self {
        Self {
            rootfs,
            command,
            hostname,
        }
    }
}