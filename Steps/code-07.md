# Step 7: Container Networking (veth pairs)

In this step, we add network isolation and connectivity to our container using virtual ethernet interfaces (`veth` pairs).

## Concept
When a container is purely isolated via `CLONE_NEWNET` (New Network Namespace), it has its own empty routing table and no network interfaces—not even a loopback interface (`lo`). This means it can't communicate with the outside world or the host.

To bridge this gap, Linux provides `veth` (virtual ethernet) devices. They act like a virtual wire: packets sent into one side of the `veth` pair come out the other. 

## The Architecture
1. **Create the Pair:** We create a pair on the host (`rusty-host-xxx` and `rusty-guest-xxx`).
2. **Move into Namespace:** We move the guest side of the pair into the new container's network namespace (which is tied to a specific process PID).
3. **Configure Host:** On the host, we assign an IP (e.g., `10.0.0.1/24`) and bring the interface UP.
4. **Configure Guest:** Inside the container, we assign the other interface a different IP (e.g., `10.0.0.2/24`), set up the local loopback (`lo`), and add a default route pointing to the host's IP so the container knows how to send traffic out.

## Implementation
We created `src/network.rs` to handle executing `/sbin/ip` commands to manage all this boilerplate networking structure.

In `src/container.rs`, we intercept the flow:
*   **Host Process:** Just after spawning the `child` (which unshared the `CLONE_NEWNET` namespace), we attach the guest `veth` to the child's `PID`.
*   **Init Process:** Inside the container (PID 1), we turn up the networking interface, assign the IP, and configure routes *before* we execute `pivot_root`, because `pivot_root` would hide the host's `/sbin/ip` executable if we don't have our own networking binaries inside the generic Alpine rootfs.
