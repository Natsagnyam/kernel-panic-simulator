# Kernel Panic Simulator (`kernel-panic-detector`)

A Linux kernel fault-injection laboratory (`panic_lab`) and companion Rust `egui` desktop monitor ("Let me panic KERNEL") designed to simulate and detect low-level kernel defects—including **race conditions**, **memory leaks**, and **hard faults**—with DKMS packaging for Ubuntu (`noble`).

## Overview

Modern systems engineers need repeatable, observable ways to trigger and diagnose kernel-level failures without guessing what happened inside ring 0. This project pairs a C kernel module with a Rust `egui` monitoring interface:

* **Fault Simulation (`kernel/`):** Injects controlled failure modes—such as concurrent access race conditions, unfreed allocation memory leaks, and hard fault / panic conditions—so you can observe how the kernel and diagnostic tools respond.
* **Anomaly Detection & Live Telemetry (`egui-ui/`):** Streams and parses live kernel log output (`dmesg`) alongside anomaly scoring to surface unusual behavior, allocation growth, or fault signatures as they occur.
* **DKMS Packaging (`debian/`):** Packages the kernel module cleanly via DKMS so it can be built and maintained across kernel updates on Ubuntu `noble`.

> **Safety Warning:** Simulating hard faults or kernel panics can destabilize or crash a running system. Always test inside a virtual machine, isolated test rig, or non-production environment, and save open work first.

## Test Instructions

### 1. Build and Run the Rust `egui` Monitor

1. Open a terminal in the `egui-ui` directory:
   ```bash
   cd /home/natska/kernel-panic-detector/egui-ui
   cargo build --release
Launch the desktop monitor (elevated privileges may be needed if dmesg access is restricted on your system):

Bash
sudo ./target/release/*
(Or run via cargo run --release during development.)

Verify: The "Let me panic KERNEL" window opens cleanly, fills the available viewport without clipping controls, and begins polling kernel log events.

2. Build and Test the panic_lab Kernel Module
Build the module from the kernel/ directory:

Bash
cd /home/natska/kernel-panic-detector/kernel
make
Load the compiled module and watch live kernel messages in a second terminal:

Bash
sudo dmesg -w
Bash
sudo insmod panic_lab.ko
Trigger each fault scenario (adjust module parameters or control interfaces according to your panic_lab implementation in kernel/) and observe both dmesg and the egui monitor:

Race condition: Trigger concurrent access and verify synchronization warnings, lock contention, or unexpected state changes in dmesg and the UI score.

Memory leak: Trigger repeated allocations without release and check kernel slab/allocation counters and UI anomaly trends.

Hard fault: Trigger the hard-fault path on a test machine/VM and confirm the fault signature is logged before panic/halt.

Unload the module cleanly when finished testing:

Bash
sudo rmmod panic_lab
Verify: lsmod | grep panic_lab returns empty, and dmesg records clean module initialization and exit messages.

3. Verify DKMS Packaging (Ubuntu noble)
Install the built or PPA-published DKMS package:

Bash
sudo apt update
sudo apt install -y kernel-panic-detector-dkms
Check DKMS build and installation status:

Bash
dkms status
Verify: dkms status lists the module as installed for your current kernel version.

License
Dual-licensed under MIT OR GPL-2.0-only. See debian/copyright and source headers for details.
