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

    // TODO: In the next steps, we will pass `config` to our container module
    // container::start(config);
}