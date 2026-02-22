# Overview

Docker solved the classic **“Dependency Hell”** problem.

Before Docker, even though containerization concepts already existed, running an application on another machine often meant moving the entire codebase and recreating the environment manually. This led to the well-known:

> “It works on my machine” problem.

Docker changed this by:

* Standardizing environments
* Simplifying DevOps workflows
* Replacing hundreds of pages of setup documentation with a short Dockerfile
* Reducing the need to deploy full Virtual Machines for each application

It also played a major role in enabling the **microservices revolution**.

---

## Did Docker Invent Containers?

**No — Docker made containers usable and popular.**

The underlying ideas evolved over decades:

### Early building blocks

* **Unix `chroot` (late 1970s)**
  Made a process believe a directory was the root of the filesystem.

* **FreeBSD Jails**
  Added:

  * Network isolation
  * OS-level virtualization

* **Solaris Zones**
  Introduced:

  * Strong stability
  * Fine-grained resource control

---

## LXC (Linux Containers)

In the early 2000s, Linux added the two key kernel features that make containers possible:

### 1. Namespaces → isolation

Give a process its own private view of the system.

### 2. Cgroups → resource control

Limit and measure:

* CPU
* Memory
* I/O
* etc.

### LXC (2008)

LXC was the first practical way to create real Linux containers without extra tooling.

Early Docker versions used LXC, but later Docker replaced it with:

* **libcontainer**
* **runc**

This allowed Docker to become a **portable, standardized runtime**.

---

# The Core Idea: How Containers Work

A Linux system has two main parts:

### 1. Kernel

* Talks to hardware
* Shared by all containers

### 2. User space (root filesystem / rootfs)

Contains directories like:

```
/bin
/etc
/lib
/proc
```

---

## Containers vs Virtual Machines

### Virtual Machine

* Full guest OS
* Own kernel
* Heavy

### Container

* Shares host kernel
* Has its own root filesystem
* Lightweight

The **root filesystem (rootfs)** defines the container’s OS “personality”.

Example:

* **Alpine Linux → tiny rootfs → ideal for small containers**

You can download an Alpine mini rootfs and see:

* `/bin` → command binaries (`cp`, `cat`, `date`, …)
* `/lib`
* `/etc`
* etc.

---

# Key Concepts

### Process

A running application.

### Namespace

Makes a process think it is alone in a specific system resource.

### `unshare`

A userspace tool that tells the kernel:

> “Run this process in new namespaces.”

---

## Types of Namespaces

* **UTS** → hostname
* **PID** → process tree
* **MNT** → mount points
* **NET** → networking
* **USER** → user IDs
* **IPC** → shared memory

---

# Creating a Container Using Only Linux

This demonstrates how Docker works under the hood.

---

## Step 1 — Create the Directory Structure

```bash
mkdir my-container-01
mkdir my-container-01/rootfs   # container filesystem
mkdir my-container-01/ns       # namespace references
```

---

## Step 2 — Create New Namespaces

Run a shell inside new namespaces:

```bash
sudo unshare \
  --uts \
  --pid \
  --mount \
  --net \
  --ipc \
  --fork \
  --mount-proc \
  bash
```

This gives the shell:

* Its own hostname
* Its own process tree
* Its own mount table
* Its own network stack
* Its own IPC space

⚠️ At this point, the namespace exists **only while this shell is running**.

---

## Step 3 — Make the Namespace Persistent

Namespaces are exposed through:

```
/proc/<PID>/ns/
```

### Find the PID from the host

```bash
ps -ef | grep -E 'unshare|bash' | grep -v grep
```

Example:

```
root 100007 bash   ← use this PID
```

---

### Create mount points

```bash
sudo touch my-container-01/ns/{pid,mnt,uts,net,ipc}
```

### Bind the namespace handles

```bash
sudo mount --bind /proc/100007/ns/pid my-container-01/ns/pid
sudo mount --bind /proc/100007/ns/mnt my-container-01/ns/mnt
sudo mount --bind /proc/100007/ns/uts my-container-01/ns/uts
sudo mount --bind /proc/100007/ns/net my-container-01/ns/net
sudo mount --bind /proc/100007/ns/ipc my-container-01/ns/ipc
```

Now:

✅ The namespace stays alive
✅ You can close the original shell

---

## Step 4 — Re-enter the Container

```bash
sudo nsenter \
  --uts=my-container-01/ns/uts \
  --pid=my-container-01/ns/pid \
  --mount=my-container-01/ns/mnt \
  --net=my-container-01/ns/net \
  --ipc=my-container-01/ns/ipc \
  bash
```

You are back inside the same isolated environment.


