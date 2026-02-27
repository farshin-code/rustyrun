# Step 2: The Execution Engine (`container.rs`)

In this step, we implemented the core execution model of our container runtime. We used the "Fork" architecture to isolate a process using Linux namespaces.

---

## 1. The "Self-Execution" Trick

Because we are using `unshare` instead of `clone`, we face a challenge: `unshare(CLONE_NEWPID)` does not put the calling process into a new PID namespace; it only affects its *children*.

To solve this cleanly in Rust, we use a trick where the program calls itself:

1.  The user runs `rustyrun run ...`.
2.  The program spawns a child process using `std::process::Command::new("/proc/self/exe")`.
3.  It passes a hidden subcommand: `rustyrun child ...`.
4.  *Before* the child process starts executing the Rust binary again, we use a `pre_exec` hook to call `unshare`.
5.  The new process wakes up inside the new namespaces, sets up the environment, and finally calls `execve` to replace itself with the user's requested command (e.g., `/bin/sh`).

---

## 2. Updating the CLI (`main.rs`)

We updated our `clap` CLI to use subcommands.

*   `Run`: The public command the user interacts with.
*   `Child`: A hidden command used internally to initialize the isolated environment.

```rust
#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a new container
    Run { /* args */ },
    
    /// Hidden command used internally
    #[command(hide = true)]
    Child { /* args */ },
}
```

---

## 3. The Host Logic (`container::start`)

This function runs in the parent process. It prepares the child process and injects the `unshare` syscall.

```rust
pub fn start(config: ContainerConfig) {
    let mut child = Command::new("/proc/self/exe");
    child.arg("child").arg("--rootfs").arg(&config.rootfs) /* ... */;

    unsafe {
        child.pre_exec(|| {
            let flags = CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWNET;

            if let Err(e) = unshare(flags) {
                eprintln!("❌ Failed to unshare namespaces: {}", e);
                std::process::exit(1);
            }
            Ok(())
        });
    }

    let mut process = child.spawn().expect("Failed to spawn");
    process.wait().expect("Failed to wait");
}
```

---

## 4. The Child Logic (`container::init_child`)

This function runs *inside* the newly created namespaces. It finalizes the environment and executes the target binary.

```rust
pub fn init_child(config: ContainerConfig) {
    // 1. Set the hostname (UTS namespace)
    crate::namespaces::set_hostname(&config.hostname);

    // 2. Set up the filesystem (Mount namespace)
    crate::mounts::setup_rootfs(&config.rootfs);

    // 3. Execute the target command
    // CommandExt::exec() completely replaces the current Rust process
    let err = Command::new(&config.command).exec();
    
    eprintln!("❌ Child: Failed to execute command: {}", err);
    std::process::exit(1);
}
```

---

## 5. Supporting Modules

To keep the code modular, we created two helper files:

### `namespaces.rs`
Provides a safe wrapper around `libc::sethostname` to change the container's hostname without affecting the host (thanks to the UTS namespace).

### `mounts.rs`
Handles the filesystem isolation. For now, we use `chroot` to change the root directory to the provided `rootfs` path. We will upgrade this to `pivot_root` in a future step for better security.

---

## 6. Dependencies Added

We added two crucial crates to `Cargo.toml`:
*   `libc`: For raw C bindings to Linux syscalls.
*   `nix`: For safe Rust wrappers around Unix APIs (like `unshare` and `CloneFlags`).
