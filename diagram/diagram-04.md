# Architecture Diagram: Filesystem Isolation (`pivot_root` vs `chroot`)

This diagram illustrates how `rustyrun` transitions from using the insecure `chroot` command to the highly secure `pivot_root` syscall to isolate the container's filesystem.

```mermaid
journey
    title The pivot_root File System Swap
    
    section Initial State (Child Process)
      Host Root (/): 5: Host FS
      Alpine Folder (/tmp/alpine): 5: Target FS
      
    section 1. Private & Bind Mounts
      Remount / as MS_PRIVATE: 3: Prevents leaks to host
      Bind Mount Alpine to Itself: 3: Satisfies kernel separate-mount rule
      
    section 2. The Pivot (The Swap)
      Call pivot_root(alpine, alpine/.oldroot): 7: Kernel level swap
      Alpine becomes the new (/): 7: Success
      Host Root moves to (/.oldroot): 7: Success
      
    section 3. Cleanup & Lock In
      Change Dir to (/): 5: CWD is secure
      Unmount (/.oldroot): 5: Host FS detached entirely
      Delete /.oldroot: 5: Escape is now impossible
      
    section Final State
      New Container Root (/): 7: Securely Isolated
      Host Root: 1: Inaccessible!
```

---

### Sequence View

```mermaid
sequenceDiagram
    participant Child as Init Process (PID 1)
    participant FS as Mount Namespace
    participant Kernel as Linux Kernel

    Note over Child, Kernel: Process is in a NEW Mount Namespace (CLONE_NEWNS)
    
    Child->>Kernel: `mount("/", MS_PRIVATE | MS_REC)`
    Note right of Kernel: Prevents container mount events from reaching the host
    
    Child->>Kernel: `mount("/tmp/alpine", "/tmp/alpine", MS_BIND)`
    Note right of Kernel: Kernel rule: New Root must be a distinct mount point
    
    Child->>FS: `mkdir("/tmp/alpine/.oldroot")`
    
    Child->>Kernel: `pivot_root("/tmp/alpine", "/tmp/alpine/.oldroot")`
    Note over FS: SWAP OCCURS.<br/>Container's `/` is now the Alpine folder.<br/>Host's `/` is shifted to `/.oldroot`
    
    Child->>FS: `chdir("/")`
    Note right of Child: Enter the new root logically
    
    Child->>Kernel: `umount2("/.oldroot", MNT_DETACH)`
    Note right of Kernel: Host filesystem is completely detached from this namespace
    
    Child->>FS: `rmdir("/.oldroot")`
    Note over Child, FS: The container is mathematically locked in.<br/>No path leads back to the host.
```
