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

Here is a complete, step-by-step guide to building a Linux container from scratch using Alpine Linux. 

This guide focuses on process isolation and filesystem setup (including proc and sys), without touching networking yet.

---

### Step 1: Prepare the "Hard Drive" (Root Filesystem)
A container cannot use your host computer's commands. It needs its own isolated set of files, folders, and binaries (like sh, ls, etc.). We will use Alpine Linux because it is incredibly small (about 3MB).

**1. Create a folder for your container:**
```bash
mkdir ~/my-container-01
cd ~/my-container-01
```

**2. Download and extract the Alpine "Mini Root Filesystem":**

Download the alpine mini root filesystem from alpine website and extract it to my-container-01

*If you type `ls` now, you will see standard Linux folders like `bin`, `etc`, `dev`, `proc`, `sys`, and `usr`. This is your container's entire world.*

---

### Step 2: Isolate and Enter the Container
Now we use the `unshare` command to create Linux **Namespaces**. Namespaces are the kernel feature that hides the host computer from the container.

**Run this command to start your container:**
```bash
sudo unshare --mount --uts --ipc --pid --fork --root ~/my-container-01 /bin/sh
```

**Explanation of what just happened:**
* `--mount`: Creates a Mount Namespace. Any drives or filesystems we mount inside the container will *not* be seen by the host computer.
* `--uts`: Creates a Hostname Namespace. This allows the container to have its own computer name.
* `--ipc`: Creates an IPC Namespace. Prevents the container from talking directly to host processes via shared memory.
* `--pid --fork`: Creates a Process ID Namespace. This makes your new sh shell become **PID 1** (the master process) inside the container. It cannot see the host's processes.
* `--root`: Changes the root directory (`/`) to your Alpine folder. The container is now physically trapped in this folder.

*You are now inside the container! Your prompt will likely just be `/ #`.*

---

### Step 3: Mount proc (The Process API)
If you type `ps` right now, you will get an error or see nothing. That is because the proc folder is currently empty.

In Linux, proc is a "pseudo-filesystem". It doesn't exist on a hard drive; it is generated in RAM by the kernel to show running processes.

**Run this command inside the container:**
```sh
mount -t proc proc /proc
```

**Explanation:**
* `mount -t proc`: Tells the kernel to mount a filesystem of type `proc`.
* `proc /proc`: Mounts it to the proc directory.
* **Why it works:** Because you used `--pid` in Step 2, the kernel knows you are in an isolated PID namespace. When you mount proc, the kernel dynamically generates a *new* proc that only contains the processes running inside this specific container.

*Test it: Type `ps`. You will now see sh running as PID 1!*

---

### Step 4: Mount sys (The Hardware API)
If you type `lsblk` (to list disks) or `ip link` (to list network cards), they will fail. The kernel exposes hardware information through the sys directory, which is currently empty.

**Run these two commands inside the container:**
```sh
# 1. Mount the sysfs filesystem
mount -t sysfs sys /sys

# 2. Remount it as Read-Only (CRITICAL for security)
mount -o remount,ro /sys
```

**Explanation:**
* `mount -t sysfs`: Tells the kernel to mount the hardware information filesystem.
* **Why Read-Only (`ro`)?** sys allows you to change physical hardware settings (like turning off a CPU core or changing power limits). If a hacker breaks into your container, you do not want them modifying your host computer's hardware. Docker always mounts sys as read-only to prevent this.

*Test it: Type `ip link`. Even though we haven't set up networking yet, the command will now successfully run and show the default `lo` (loopback) interface.*

---

### Step 5: Test the Isolation
You now have a fully functioning, isolated Linux container. Try these commands to prove it is isolated from your main computer:

1. **Change the hostname:**
   ```sh
   hostname my-custom-container
   ```
   *(This changes the name inside the container, but your host computer's name remains untouched thanks to the `--uts` flag).*

2. **Check your processes:**
   ```sh
   top
   ```
   *(You will only see `sh` and `top`. The hundreds of background processes running on your host computer are completely invisible).*

### Step 6: Exit and Clean Up
To destroy the container, simply type:
```sh
exit
```
Because the sh process was PID 1, exiting it kills the container. The kernel automatically destroys the namespaces, and because we used a Mount Namespace (`--mount`), the proc and sys mounts are automatically unmounted and cleaned up for you.