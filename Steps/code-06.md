# Step 6: Completing the OS with `/sys` and `/dev`

When we download a rootfs tarball (like Alpine), we are only downloading static files (the binaries in `/bin`, the configs in `/etc`). However, a real Linux operating system requires dynamic interfaces to communicate with the kernel and hardware. 

Without these, basic system utilities, package managers, and web servers will crash.

In this step, we updated `src/mounts.rs` to wire up the three essential pseudo-filesystems immediately after the `pivot_root` isolation occurs.

---

## 1. The Kernel Interface (`/proc`)

We implemented this in a previous step, but it is the first critical piece. The `/proc` filesystem is how the Linux kernel exposes information about running processes (PIDs). Tools like `ps` and `top` parse the text files in this directory.

Because our process is inside a PID namespace, the kernel automatically filters `proc` when we mount it, ensuring the container only sees its own processes.

## 2. The Hardware Interface (`/sys`)

The `/sys` filesystem (`sysfs`) is a virtual filesystem provided by the kernel that exports information about various kernel subsystems, hardware devices, and associated device drivers.

Many background utilities and programming language runtimes check `/sys` to understand the environment they are running in (e.g., checking CPU topology or network device states).

### Security Considerations for `/sys`
Because `sysfs` allows changing kernel parameters on the host, a container should **never** be allowed to write to it. 
We mounted `/sys` with strict flags, exactly like Docker does:
*   `MS_RDONLY`: Read-only. The container cannot alter kernel states.
*   `MS_NOSUID`: Ignore set-user-identifier bits.
*   `MS_NOEXEC`: Do not allow execution of binaries in this filesystem.

```rust
mount(
    Some("sysfs"),
    sys_path,
    Some("sysfs"),
    MsFlags::MS_RDONLY | MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
    None::<&str>,
)
```

## 3. The Devices Interface (`/dev`)

The `/dev` directory contains special files known as "device nodes". These files allow applications to interact with device drivers (like hard drives, terminals, or random number generators). 

If a container's `/dev` is empty, common commands fail. For example, if a script tries to discard output by redirecting to `/dev/null`, it will crash. If a web server tries to generate an HTTPS certificate and needs randomness from `/dev/urandom`, it will hang or panic.

### The `tmpfs` Implementation
Device nodes should not be written to the actual physical hard drive (they are ephemeral). And importantly, a container needs its own private, empty `/dev` space so it doesn't accidentally interfere with the host's devices.

To achieve this, we mounted a **`tmpfs` (Temporary File System)** over the container's `/dev` folder. A `tmpfs` is essentially a RAM disk. It exists entirely in memory.

```rust
mount(
    Some("tmpfs"),
    dev_path,
    Some("tmpfs"),
    MsFlags::MS_NOSUID | MsFlags::MS_STRICTATIME,
    Some("mode=755,size=65536k"),
)
```

*Note: A complete, production-grade runtime like `runc` goes one step further. After mounting this empty `tmpfs`, it uses the `mknod()` syscall to manually inject safe devices into the RAM disk (like `null`, `zero`, `random`, `urandom`, and `tty`). For our instructional runtime, simply providing the `tmpfs` allows many basic assumptions of the OS to hold true.*

---

## 4. Testing the Environment

You can verify the new filesystems are present by starting the container and inspecting the mount table.

```bash
sudo cargo run -- run --rootfs ./root-file-systems/alpine-rootfs --command /bin/sh
```

Inside the container:
```bash
# View the list of active mounts
mount

# You should see output similar to this:
# proc on /proc type proc (rw,relatime)
# sysfs on /sys type sysfs (ro,nosuid,nodev,noexec,relatime)
# tmpfs on /dev type tmpfs (rw,nosuid,size=65536k,mode=755)
```