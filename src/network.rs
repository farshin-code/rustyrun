use std::process::Command;
use uuid::Uuid;

/// Manages the Virtual Ethernet (veth) bridge for the container.
pub struct Network {
    pub veth_host: String,
    pub veth_guest: String,
}

impl Network {
    /// Generates unique names for the veth pair.
    pub fn new() -> Self {
        // veth names are limited to 15 characters in Linux.
        // We use a short UUID to ensure they don't collide.
        let id = &Uuid::new_v4().to_string()[..6];
        Self {
            veth_host: format!("vethH{}", id),
            veth_guest: format!("vethG{}", id),
        }
    }

    /// Sets up the veth pair and attaches the guest end to the child's network namespace.
    /// This must be called from the HOST process, AFTER the child PID is known.
    pub fn setup_veth_pair(&self, child_pid: u32) {
        println!("🌐 Network: Setting up veth pair ({} <--> {}) for child PID {}", 
                 self.veth_host, self.veth_guest, child_pid);

        // 1. Create the veth pair.
        // This is like creating a virtual wire with two ends.
        run_ip(&["link", "add", &self.veth_host, "type", "veth", "peer", "name", &self.veth_guest]);

        // 2. Bring the host end UP
        run_ip(&["link", "set", &self.veth_host, "up"]);

        // 3. Move the guest end into the child's network namespace.
        // The child is running inside a new CLONE_NEWNET namespace.
        // By giving the 'ip' command the child's PID, it knows which namespace to push it into.
        run_ip(&["link", "set", &self.veth_guest, "netns", &child_pid.to_string()]);
    }

    /// Configures the network interfaces inside the container.
    /// This must be called from the INIT process (inside the new namespace).
    pub fn configure_guest(&self) {
        println!("🌐 Network: Configuring container interfaces...");

        // 1. Bring up the loopback interface (localhost)
        // Without this, services that bind to 127.0.0.1 will fail.
        run_ip(&["link", "set", "lo", "up"]);

        // 2. Rename the guest veth to the standard 'eth0'
        run_ip(&["link", "set", &self.veth_guest, "name", "eth0"]);

        // 3. Assign an IP address to eth0 (we use a static private IP for simplicity)
        run_ip(&["addr", "add", "10.0.0.2/24", "dev", "eth0"]);

        // 4. Bring eth0 UP so it can transmit data
        run_ip(&["link", "set", "eth0", "up"]);
        
        // 5. Add a default route (gateway). 
        // We assume the host side will be configured with 10.0.0.1.
        run_ip(&["route", "add", "default", "via", "10.0.0.1"]);
    }

    /// Cleans up the network namespace.
    /// Deleting one end of a veth pair automatically deletes the other end.
    pub fn clean(&self) {
        run_ip(&["link", "del", &self.veth_host]);
    }
}

/// Helper function to execute `ip` commands easily.
fn run_ip(args: &[&str]) {
    // Note: In an actual production runtime, we would use the `netlink` crate
    // instead of shelling out to the `ip` binary, as shelling out is slower
    // and depends on the host having the `iproute2` package installed.
    // For this educational purpose, shelling out is much easier to read and understand.
    let status = Command::new("/sbin/ip")
        .args(args)
        .status()
        .expect("❌ Failed to execute 'ip' command. Is iproute2 installed?");

    if !status.success() {
        eprintln!("⚠️ Network warning: 'ip {:?}' failed with {}", args, status);
    }
}
