# Step 3: PIDs and Filesystems (The Double Fork)

In this step, we turned our basic chroot jail into a true container by solving two critical problems: mounting the `/proc` filesystem and achieving true PID isolation.

---

## 1. Preparing the Environment (Root Filesystems)

Before testing the container, you need a realistic Linux root filesystem so the container has actual commands to run (like `/bin/sh`, `ls`, or `ps`). 

We have set up a `root-file-systems/` directory to hold these. **Do not commit these files to Git!** They are ignored by the `.gitignore` policy.

### Downloading an Alpine RootFS
Alpine is highly recommended for testing because it is extremely small (~5MB).
```bash
mkdir -p root-file-systems/alpine-rootfs
cd root-file-systems/alpine-rootfs
wget https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz
sudo tar -xzf alpine-minirootfs-3.19.1-x86_64.tar.gz
rm alpine-minirootfs-3.19.1-x86_64.tar.gz
```

---

## 2. Mounting `/proc`

Linux system tools (like `ps` and `top`) do not work by magically knowing what the kernel is doing. They work by reading text files out of the `/proc` pseudo-filesystem. 

If we trap a process in a new root directory without mounting `/proc`, those tools crash.

We updated `src/mounts.rs` to automatically mount this right after the `chroot` happens:

```rust
// Create the directory if it doesn't exist in the Alpine folder
let proc_path = Path::new("/proc");
fs::create_dir_all(proc_path).unwrap();

// Ask the kernel to mount the "proc" pseudo-filesystem here
mount(
    Some("proc"),
    proc_path,
    Some("proc"),
    MsFlags::empty(),
    None::<&str>
).unwrap();
```

---

## 3. The "Double Fork" (Achieving PID 1)

This is the most complex part of using `unshare`. 

According to Linux kernel rules: when you call `unshare(CLONE_NEWPID)`, the process calling it is **not** moved into the new PID namespace. Instead, only its future *children* will be placed in the new namespace. 

If we didn't fix this, our container would have its own mount and hostname, but if you ran `ps aux`, you would see all the host's background processes!

We updated `src/container.rs` and `src/main.rs` to implement the **Double Fork** architecture:

1.  **The Host (`rustyrun run ...`)**
    * Spawns `child` using `Command::new("/proc/self/exe")`.
    * Uses `pre_exec` to call `unshare` with `CLONE_NEWPID` (and all other namespaces).
2.  **The Intermediate Child (`rustyrun child ...`)**
    * Wakes up. It has new mount/UTS namespaces, but is *still* in the host's PID namespace.
    * It cannot run the user's app yet. Instead, it spawns *another* process (`init`).
3.  **The Init Grandchild (`rustyrun init ...`)**
    * Wakes up. Because it is the child of an unshared process, the Kernel officially puts it into the new PID namespace.
    * It is officially **PID 1** inside the container.
    * It sets the hostname, mounts `/proc`, `chroot`s, and `execve`s into the user's requested command (e.g., `/bin/sh`).

---

## 4. How to Test It

Now that we have PID isolation and `/proc`, you can test it:

```bash
# You must run this with sudo because chroot and mount require root privileges
sudo cargo run -- run --rootfs ./root-file-systems/alpine-rootfs --command /bin/sh
```

Once inside the shell, run these commands to verify it works:
*   `hostname` (Should print `rustyrun-container`)
*   `ps aux` (Should only show your `/bin/sh` process as PID 1, and the `ps` command itself. You will NOT see host processes!)
*   `touch /hello.txt` (This file will only exist inside your `alpine-rootfs` folder on the host).
