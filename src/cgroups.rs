use std::fs;
use std::path::PathBuf;

/// Manages the Linux Cgroups (v2) for resource limitation.
pub struct Cgroup {
    path: PathBuf,
}

impl Cgroup {
    /// Creates a new cgroup directory for the container under the unified hierarchy.
    pub fn new(name: &str) -> Self {
        // We create a directory for our specific container
        let path = PathBuf::from(format!("/sys/fs/cgroup/rustyrun-{}", name));
        
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(&path) {
                eprintln!("❌ Failed to create cgroup directory: {}", e);
                std::process::exit(1);
            }
        }
        
        Self { path }
    }

    /// Sets the maximum memory limit in Megabytes.
    pub fn set_memory_limit(&self, limit_mb: u64) {
        let limit_bytes = limit_mb * 1024 * 1024;
        let mem_path = self.path.join("memory.max");
        
        if let Err(e) = fs::write(&mem_path, limit_bytes.to_string()) {
            eprintln!("❌ Failed to write memory limit to cgroup: {}", e);
        } else {
            println!("⚖️  Cgroup: Hard memory limit set to {} MB", limit_mb);
        }
    }

    /// Returns the path to the cgroup.procs file. 
    /// We will write the child process's PID into this file to attach it to the limits.
    pub fn procs_path(&self) -> PathBuf {
        self.path.join("cgroup.procs")
    }

    /// Cleans up the cgroup directory when the container exits.
    pub fn clean(&self) {
        if self.path.exists() {
            // Wait briefly to ensure the kernel has cleaned up the PID from cgroup.procs
            std::thread::sleep(std::time::Duration::from_millis(50));
            if let Err(e) = fs::remove_dir(&self.path) {
                eprintln!("⚠️  Cgroup: Could not clean up cgroup dir: {}", e);
            }
        }
    }
}
