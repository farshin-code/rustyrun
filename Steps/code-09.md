# Step 9: Union Filesystems (OverlayFS)

In this step, we introduced **OverlayFS**, eliminating the problem of containers permanently modifying their base disk image.

## The Problem
Up until now, `rustyrun` was using a plain folder (like `~/my-container-01`) directly as its `rootfs`.
If you ran `touch /hello.txt` inside the container, that file materialized literally on your host hard drive in that directory.

Problems with this:
1. Running multiple containers from the exact same disk image simultaneously causes file corruption, as they overwrite each other's configuration files and binaries.
2. Changes inside the container permanently pollute the base image for future containers.

## The Solution: Union Filesystem (OverlayFS)
Linux **OverlayFS** solves this by layering folders on top of each other. 
It requires 4 directories to work:
1. **LowerDir:** The base Alpine image (Read-Only).
2. **UpperDir:** An empty, temporary folder (Read-Write).
3. **WorkDir:** A temporary scratch-space folder used by the Linux kernel to organize files.
4. **Merged:** The resulting folder where the Upper and Lower layers are logically combined.

### How it behaves:
*   **Reads**: When a container reads `/bin/sh`, OverlayFS fetches it from the `LowerDir` (the Alpine base).
*   **Writes**: When a container creates `/hello.txt`, rather than modifying the `LowerDir`, OverlayFS intercepts the write and saves the file to the `UpperDir`.
*   **Deletes**: If the container deletes a file that belongs to the base image, OverlayFS creates a "whiteout" (a special invisible file) in the `UpperDir` to mask the file from the container's view, without actually deleting it from the base `LowerDir`.

## The Implementation
We modified the runtime Host Process (`container.start`) to orchestrate these layers:
1. It intercepts the user's `--rootfs` directory and treats it safely as the `LowerDir`.
2. It dynamically creates `/tmp/rustyrun-<hostname>/` and its subdirectories (`upper`, `work`, `merged`).
3. It executes the `mount -t overlay` syscall directly on the host, passing all the layers as configuration data.
4. Finally, it alters the string handed to the container Child Process, passing `/tmp/rustyrun-<hostname>/merged` as the actual `rootfs` instead of the original Alpine path.
5. When the container process quits, the Host cleans up and deletes `/tmp/rustyrun-<hostname>`, instantly erasing all the container's changes and leaving the base image untouched!
