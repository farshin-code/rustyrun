# Step 4: Resource Limits with Cgroups v2

In this step, we introduced Cgroups (Control Groups) to restrict the resources our isolated container can consume. Without cgroups, a container process could accidentally (or maliciously) consume 100% of the host machine's RAM and CPU.

By implementing this, we emulate the exact behavior of `runc` and Docker.

---

## 1. What are Cgroups v2?

Cgroups are a Linux kernel feature that limits, accounts for, and isolates the resource usage of a collection of processes. 

Modern Linux distributions (like Ubuntu 22.04) use the unified Cgroup v2 hierarchy, usually mounted at `/sys/fs/cgroup`. To limit a process, you interact with specific files inside this directory tree.

---

## 2. Managing Cgroups (`src/cgroups.rs`)

We created a custom `Cgroup` manager. When the host process starts a container, it performs three major actions:

1.  **Creation:** 
    It creates a new directory specific to the container based on its hostname: `/sys/fs/cgroup/rustyrun-<hostname>`.
2.  **Setting Limits:** 
    If the user passed a memory limit (e.g., `--memory 50` for 50MB), it multiplies that by `1024 * 1024` to convert it to bytes, and writes it to the `memory.max` file.
3.  **Cleanup:** 
    When the container exits, it calls `remove_dir` to delete the cgroup from the kernel.

---

## 3. Eliminating the "Race Condition" (`container.rs`)

One of the biggest challenges in writing a container runtime is avoiding race conditions. 

If we spawned our child process, let it start executing, and *then* tried to add it to the Cgroup from the host, the child might consume massive amounts of memory in the milliseconds before it is attached.

To fix this, we used Rust's `CommandExt::pre_exec` hook. This closure executes *inside* the child process immediately after it is forked, but *before* it runs `/proc/self/exe` or `unshare`.

```rust
child.pre_exec(move || {
    // 1. Attach this specific process to the Cgroup
    let pid = std::process::id();
    std::fs::write(&cgroup_procs_path, pid.to_string()).unwrap();

    // 2. NOW we unshare namespaces
    unshare(flags).unwrap();
    Ok(())
});
```

Because Cgroup limits are inherited, by attaching the first `child`, our `init` grandchild (and any program the user runs, like `/bin/sh`) is automatically restricted!

---

## 4. How to Test It

You can run your container with a strict memory limit. In this example, we restrict the container to 50 Megabytes of RAM.

```bash
# Don't forget to run with sudo for cgroup manipulation
sudo cargo run -- run \
  --rootfs ./root-file-systems/alpine-rootfs \
  --command /bin/sh \
  --memory 50
```

Inside the shell, if you were to write a small script that rapidly allocates memory (like a Python `a = ['string' * 1000] * 10000`), the Linux kernel's Out-Of-Memory (OOM) killer would immediately step in and kill only your container process, keeping your host completely safe!