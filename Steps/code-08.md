# Step 8: User Namespaces (Rootless Containers)

In this step, we introduced strong privilege isolation by isolating the container inside a **User Namespace** using the `CLONE_NEWUSER` flag. 

## The Concept of Rootless Containers
If a container is compromised, the attacker becomes the `root` user *inside* that container. If your container engine doesn't define a User Namespace boundary, that attacker is actually executing as `root` (UID 0) on your host computer. Thus, an escape exploit instantly grants them complete server control.

Using **User Namespaces**, we map a user ID from the container to a different user ID on the host machine.

### The Mapping Strategy
We do a simple `0:1000:1` mapping. 
* Inside the container: Target process runs as **UID 0** (`root`). It gets full privileges over its own isolated namespaces (Mount, Network, UTS).
* On the Host: The process is observed as **UID 1000** (a standard unprivileged user).

If the attacker breaks out of the container, the host kernel checks their permissions and denies access because they are just a standard unprivileged user. This concept is commonly referred to as "Rootless Containers".

## The Implementation

1. **`clone(CLONE_NEWUSER)`:** We append the user namespace flag to our `unshare` command inside `src/container.rs`.
2. **Writing Maps to `/proc/<pid>/uid_map`:** The host process tracks the `child`'s process ID. It writes the map `0 <host_uid> 1` into the procfs metadata of the child. It does the exact same for `gid_map` with groups.
3. **`setgroups` Denial:** The Linux kernel requires us to write `deny` to the `/proc/<pid>/setgroups` file before saving our `gid_map`. This is a security measure to prevent unprivileged users from gaining access to secondary groups they weren't assigned.

With this applied, inside your Alpine shell `id -u` shows `0`. However, if you trigger a `sleep 100` and check `htop` on your host machine, you will see the sleep command owned by your normal user account instead of `root`.