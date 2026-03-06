# Architecture Diagram: Populating the Container OS

This diagram illustrates how `rustyrun` populates the container's root file system with the three essential pseudo-filesystems (`/proc`, `/sys`, and `/dev`) immediately after isolating the root directory via `pivot_root`. This step transforms an empty directory of files into a functioning, dynamic operating system.

```mermaid
graph TD
    %% Define styles
    classDef container fill:#eef,stroke:#333,stroke-width:2px;
    classDef device fill:#ffe,stroke:#f90,stroke-width:1px;
    classDef memory fill:#efe,stroke:#090,stroke-width:1px;
    classDef kernel fill:#fee,stroke:#900,stroke-width:1px;

    Host[Host Kernel] --> |Provides VFS Interfaces| ContainerRoot
    
    subgraph ContainerRoot ["Container Filesystem (/)"]
        class ContainerRoot container
        
        BinFolder["/bin <br> <small>(from Alpine tarball)</small>"]
        EtcFolder["/etc <br> <small>(from Alpine tarball)</small>"]
        
        ProcMount{"/proc"}
        SysMount{"/sys"}
        DevMount{"/dev"}
        
        ProcMount --> |"Mount: proc"| ProcDetails["PID Info, <br> System Uptime"]
        class ProcDetails kernel
        
        SysMount --> |"Mount: sysfs <br> Flag: READ-ONLY"| SysDetails["Hardware Info, <br> Kernel Params"]
        class SysDetails kernel
        
        DevMount --> |"Mount: tmpfs <br> Size: 64MB"| DevDetails["In-Memory Space <br> for Device Nodes"]
        class DevDetails memory
    end
    
    %% Annotations
    noteProc>"Required for tools like 'ps' and 'top'"] -.- ProcMount
    noteSys>"Required by system utilities. Mounted read-only for security."] -.- SysMount
    noteDev>"A private RAM disk. Later populated with /dev/null, /dev/urandom."] -.- DevMount
```
