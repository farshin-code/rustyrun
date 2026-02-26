# Step 1: Architecture and CLI Foundation

Welcome to the development of **rustyrun**, a container runtime written in Rust. This project aims to replicate the core functionality of runtimes like `runc` by interacting directly with Linux kernel APIs to isolate processes.

---

## 1. Architectural Plan

A container runtime cannot simply call an isolation function and continue running normally. Because Rust is multi-threaded by default, changing namespaces (like PID and Mount) in a running process can lead to undefined behavior. 

To solve this, `rustyrun` uses a **Multi-Stage Execution Model (The "Fork" Architecture)**:

*   **Stage 1 (The Parent/Host):** The main Rust process. It parses CLI arguments, sets up network interfaces, configures Cgroups (resource limits), and spawns a child process.
*   **Stage 2 (The Child/Init):** The spawned process running inside the new namespaces. It sets up the isolated environment (mounts, hostname), drops privileges, and finally replaces itself with the user's requested program (e.g., `/bin/sh`) using the `execve` syscall.

---

## 2. Core Components

The runtime is divided into four logical managers:

1.  **Namespace Manager (Isolation):** Uses `unshare` to create isolated environments for Hostname (UTS), Process Trees (PID), Mounts, IPC, and Network.
2.  **Filesystem & Mount Manager (Rootfs):** Constructs the container's view of the hard drive. It mounts pseudo-filesystems (`/proc`, `/sys`) and uses `pivot_root` to securely swap the host's root filesystem with the container's root filesystem.
3.  **Cgroup Manager (Resource Control):** Interacts with `/sys/fs/cgroup` to limit CPU, Memory, and PIDs for the containerized process.
4.  **Security & Execution Engine:** Finalizes the environment, drops host privileges, and executes the target binary.

---

## 3. Key Design Decisions

### `unshare` vs. `clone`
While production runtimes like `runc` use `clone` (via C-bindings) to create a process and isolate it simultaneously, `rustyrun` will start by using `unshare`. 
*   **Why?** `unshare` is much easier to use safely in Rust. We can use Rust's standard `std::process::Command` to spawn a child, and use a `.pre_exec()` hook to call `unshare` just before the target binary runs.

### `chroot` vs. `pivot_root`
*   **Why start with `chroot`?** `pivot_root` is the secure, production-standard way to isolate a filesystem, but it has strict, undocumented kernel rules that are hard to debug. We will start with `chroot` to get basic isolation working, and upgrade to `pivot_root` later for real security.

---

## 4. Project Structure

We are keeping the modules flat inside the `src/` directory for simplicity:

```text
rustyrun/
├── Cargo.toml
└── src/
    ├── main.rs          # Entry point and CLI parsing
    ├── config.rs        # Configuration structures
    ├── container.rs     # Process orchestration (Parent vs. Child)
    ├── namespaces.rs    # Isolation syscalls
    ├── mounts.rs        # Filesystem logic
    ├── cgroups.rs       # Resource management
    └── errors.rs        # Centralized error handling
```

---

## 5. Initial Code Implementation

We started by setting up the project metadata and building the Command-Line Interface (CLI) using the `clap` crate.

### `Cargo.toml`
We defined the project metadata and set the Rust edition to `2024` to utilize the latest stable language features. We also added `clap` for CLI parsing.

```toml
[package]
name = "rustyrun"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
```

### `src/config.rs`
This file defines the `ContainerConfig` struct, which acts as the blueprint for our container. It groups all user inputs together so we can easily pass them to our execution engine later.

```rust
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
```

### `src/main.rs`
This is the entry point. It uses `clap` macros to automatically generate a CLI based on the `Args` struct. It parses the user's input, builds the `ContainerConfig`, and prepares to hand off execution to the container module.

```rust
mod config;

use clap::Parser;
use config::ContainerConfig;
use std::path::PathBuf;

/// A simple container runtime written in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the root filesystem (e.g., /tmp/alpine)
    #[arg(short, long)]
    rootfs: PathBuf,

    /// Command to run inside the container
    #[arg(short, long, default_value = "/bin/sh")]
    command: String,

    /// Hostname for the container
    #[arg(long, default_value = "rustyrun-container")]
    hostname: String,
}

fn main() {
    // 1. Parse the command-line arguments provided by the user
    let args = Args::parse();

    // 2. Convert the arguments into our internal configuration struct
    let config = ContainerConfig::new(args.rootfs, args.command, args.hostname);

    // 3. Print the configuration to verify it works
    println!("🚀 Starting rustyrun...");
    println!("{:#?}", config);

    // TODO: Pass `config` to the container module
}
```

---

## 6. How to Test

You can test the CLI foundation by running the following commands in your terminal:

```bash
# View the auto-generated help menu
cargo run -- --help

# Run the app with custom arguments
cargo run -- --rootfs /tmp/alpine-rootfs --command /bin/bash --hostname my-test-box
```