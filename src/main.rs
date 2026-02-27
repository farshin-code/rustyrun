mod config;
mod container;
mod mounts;
mod namespaces;

use clap::{Parser, Subcommand};
use config::ContainerConfig;
use std::path::PathBuf;

/// A simple container runtime written in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a new container
    Run {
        /// Path to the root filesystem (e.g., /tmp/alpine)
        #[arg(short, long)]
        rootfs: PathBuf,

        /// Command to run inside the container
        #[arg(short, long, default_value = "/bin/sh")]
        command: String,

        /// Hostname for the container
        #[arg(long, default_value = "rustyrun-container")]
        hostname: String,
    },
    /// Hidden command used internally to initialize the container environment
    #[command(hide = true)]
    Child {
        #[arg(short, long)]
        rootfs: PathBuf,

        #[arg(short, long)]
        command: String,

        #[arg(long)]
        hostname: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            rootfs,
            command,
            hostname,
        } => {
            let config = ContainerConfig::new(rootfs, command, hostname);
            println!("🚀 Starting rustyrun...");
            container::start(config);
        }
        Commands::Child {
            rootfs,
            command,
            hostname,
        } => {
            let config = ContainerConfig::new(rootfs, command, hostname);
            container::init_child(config);
        }
    }
}