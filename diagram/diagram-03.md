# Architecture Diagram: Integrating Cgroups v2

This diagram illustrates how `rustyrun` implements resource limitations using Linux Cgroups v2, mimicking how `runc` and Docker handle it. The critical part is assigning the limit *before* the child process executes the user's workload.

```mermaid
sequenceDiagram
    participant User as User (Terminal)
    participant Host as Host Process (run)
    participant Cgroup as Linux Cgroup FS (/sys/fs/cgroup)
    participant Child as Child Process (child)
    participant Kernel as Linux Kernel

    User->>Host: `sudo rustyrun run --memory 50 --rootfs /tmp/alpine`

    Note over Host: 1. Setup Cgroups
    Host->>Cgroup: `mkdir /sys/fs/cgroup/rustyrun-container`
    Host->>Cgroup: `Write 52428800 to memory.max` (50MB)
    
    Host->>Kernel: `Command::new("rustyrun child").spawn()`
    
    Note over Kernel: pre_exec hook runs inside the new subprocess
    
    Kernel->>Child: Process starts (pre_exec execution)
    
    Note over Child: 2. Attach Process to Cgroup
    Child->>Cgroup: `Write Child PID to cgroup.procs`
    
    Note over Kernel: From this millisecond onward, the Child and ALL its future descendants are strictly limited to 50MB.
    
    Child->>Kernel: `unshare(CLONE_NEWNS | CLONE_NEWPID | ...)`
    
    Note over Child: Continues with Double Fork & execve...
    
    Child->>Kernel: `fork() -> Init -> execve("/bin/sh")`
    
    Note over Host: 3. Wait for Container to Exit
    Host->>Kernel: `wait()`
    
    Kernel-->>Host: Subprocesses exit
    
    Note over Host: 4. Cgroup Cleanup
    Host->>Cgroup: `rmdir /sys/fs/cgroup/rustyrun-container`
    
    Host-->>User: Returns to host terminal
```
