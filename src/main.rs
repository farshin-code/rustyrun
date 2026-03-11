mod cgroups;
mod config;
mod container;
mod mounts;
mod namespaces;
mod network;

use clap::{Parser, Subcommand};
use config::ContainerConfig;
use network::Network;
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

        /// Memory limit in Megabytes
        #[arg(short, long)]
        memory: Option<u64>,
    },
    /// Hidden command used internally to set up namespaces
    #[command(hide = true)]
    Child {
        #[arg(short, long)]
        rootfs: PathBuf,

        #[arg(short, long)]
        command: String,

        #[arg(long)]
        hostname: String,

        #[arg(long)]
        veth_guest: String,
    },
    /// Hidden command used internally to be PID 1 in the new namespace
    #[command(hide = true)]
    Init {
        #[arg(short, long)]
        rootfs: PathBuf,

        #[arg(short, long)]
        command: String,

        #[arg(long)]
        hostname: String,

        #[arg(long)]
        veth_guest: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            rootfs,
            command,
            hostname,
            memory,
        } => {
            // Because Network setup happens on the host first, 
            // the Host generates the names and passes them via config.
            let network = Network::new();
            let config = ContainerConfig::new(rootfs, command, hostname, memory, network.veth_guest.clone());
            
            println!("🚀 Starting rustyrun...");
            container::start(config, network);
        }
        Commands::Child {
            rootfs,
            command,
            hostname,
            veth_guest,
        } => {
            let config = ContainerConfig::new(rootfs, command, hostname, None, veth_guest);
            container::child(config);
        }
        Commands::Init {
            rootfs,
            command,
            hostname,
            veth_guest,
        } => {
            let config = ContainerConfig::new(rootfs, command, hostname, None, veth_guest);
            container::init(config);
        }
    }
}