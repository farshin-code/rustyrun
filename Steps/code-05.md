# Step 5: Filesystem Security with `pivot_root`

In this step, we upgraded our container's file system isolation from the basic `chroot` to the production-standard `pivot_root`. This eliminates the ability for a malicious program to break out of the container and attack the host machine.

---

## 1. The Problem with `chroot`

`chroot` was introduced in 1979. It simply tells a process "Hey, pretend this directory is the root `/`." 

The problem is that the host filesystem is technically still mounted and accessible underneath. A malicious process can use basic kernel tricks (like opening a file descriptor out to a directory, calling `chroot` again into a nested folder, and then walking `../../` back up) to break out of the jail and gain full access to the host's hard drive.

---

## 2. Why `pivot_root` is Secure

`pivot_root` works at the Virtual File System (VFS) layer of the Linux kernel. Instead of just changing the process's perspective, it physically swaps the mount points inside the Mount Namespace.

Once `pivot_root` is executed, we can `umount` (unmount) the host filesystem entirely. When we do this, the host's files literally cease to exist from the perspective of the container's kernel environment. There is no path out because the outside world is gone.

---

## 3. The Details of the Implementation (`src/mounts.rs`)

Using `pivot_root` is famously tricky because the Linux kernel has undocumented, strict rules for it to succeed. If you get it wrong, it returns a cryptic `EINVAL` (Invalid Argument) error.

Here is the exact "dance" we implemented to satisfy the kernel:

1.  **Private Mount Propagation:** First, we remounted `/` as `MS_PRIVATE`. This ensures that when we eventually unmount the host filesystem in Step 6, we don't accidentally unmount it for the actual host machine (since we share mount trees until we specify `MS_PRIVATE`).
2.  **The Self-Bind Trick:** The kernel demands that the "new root" (our Alpine folder) must be a distinct mount point from the current root. We satisfy this mathematically by bind-mounting the Alpine folder to itself!
3.  **Create `.oldroot`:** We create a temporary folder inside the Alpine root to catch the host filesystem.
4.  **The Pivot:** We call `pivot_root`. In an instant, `/` becomes the Alpine folder, and the host's root is mapped to `/.oldroot`.
5.  **Change Directory:** We `chdir("/")` so our working directory is inside the new boundary securely.
6.  **Unmount and Clean:** We call `umount2("/.oldroot", MNT_DETACH)` to completely eject the host filesystem from the namespace, and delete the empty directory.

Our container is now completely, cleanly, and securely isolated!

---

## 4. How to Test It

Because `pivot_root` physically alters mounts, it is highly sensitive to the environment. Run it normally:

```bash
sudo cargo run -- run --rootfs ./root-file-systems/alpine-rootfs --command /bin/sh
```

If it drops you into the shell successfully, `pivot_root` worked! 
You can verify the security by running:
```bash
mount
```
You will see that the root `/` mount is your application's folder, and there are absolutely no mounts pointing back to your host machine's drives.