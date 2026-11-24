# Kernel Panic Detector

A Linux kernel fault-injection module paired with a Rust `egui` control panel.

## Group: Let me panic KERNEL
- **Let me try a memory leak**
- **Let me try a race condition**
- **Let me try a number / 0**
- **Let me try an out-of-bounds C pointer**

Includes a toggleable ML detector that can be enabled or disabled during kernel panic testing, following a C-first kernel engineering approach.
