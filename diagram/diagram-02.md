# Architecture Diagram: The "Double Fork" Execution Model

This diagram illustrates how `rustyrun` solves the PID Namespace limitation using a "double fork" architecture. This is necessary because `unshare(CLONE_NEWPID)` only applies to future children, not the calling process.

```mermaid
sequenceDiagram
    participant User as User (Terminal)
    participant Host as Host Process (run)
    participant Kernel as Linux Kernel
    participant Child as Child Process (child)
    participant Init as Init Process (init)
    participant App as Target App (/bin/sh)

    User->>Host: `sudo rustyrun run ...`
    
    Note over Host: Prepares to spawn first child
    
    Host->>Kernel: `Command::new("/proc/self/exe child")`
    
    Note over Host,Kernel: pre_exec hook runs
    Host->>Kernel: `unshare(CLONE_NEWNS | CLONE_NEWUTS | CLONE_NEWPID | ...)`
    Kernel-->>Host: Namespaces created (PID ns is marked for future children)
    
    Host->>Kernel: `spawn()`
    
    Note over Child: Wakes up in new UTS/Mount/Net ns.
    Note over Child: STILL IN HOST PID NAMESPACE.
    
    Kernel->>Child: Starts `rustyrun child ...`
    
    Note over Child: Child must fork again so the new process enters the pending PID namespace.
    
    Child->>Kernel: `Command::new("/proc/self/exe init").spawn()`
    
    Note over Kernel: Spawns new process. Because it is the child of the unshared process, it enters the new PID namespace and becomes PID 1.
    
    Kernel->>Init: Starts `rustyrun init ...`
    
    Note over Init: Wakes up. It is officially PID 1 inside the container.
    
    Init->>Kernel: `sethostname("rustyrun-container")`
    Init->>Kernel: `chroot("/path/to/rootfs")`
    Init->>Kernel: `chdir("/")`
    Init->>Kernel: `mount("proc", "/proc", "proc", ...)`
    
    Note over Init: Environment is ready.
    
    Init->>Kernel: `execve("/bin/sh")`
    
    Note over Kernel: Replaces `rustyrun init` with `/bin/sh`
    
    Kernel->>App: Starts `/bin/sh` (Still PID 1)
    
    Note over App: App is isolated. `ps` will only show processes in this container.
    
    App-->>User: Interactive Shell
    
    Note right of App: App execution...
    
    App->>Kernel: `exit()`
    Kernel-->>Child: Init process exited
    Child->>Kernel: `exit(status)`
    Kernel-->>Host: Child process exited
    Host-->>User: Returns to host terminal
```
