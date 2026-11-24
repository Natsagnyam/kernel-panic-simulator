#!/usr/bin/env bash
set -euo pipefail

ROOT="kernel-panic-detector"

mkdir -p "$ROOT/kernel/src"
mkdir -p "$ROOT/egui-ui/src"
mkdir -p "$ROOT/scripts"

cat <<'EOF_README' > "$ROOT/README.md"
# Kernel Panic Detector

A Linux kernel fault-injection module paired with a Rust `egui` control panel.

## Group: Let me panic KERNEL
- **Let me try a memory leak**
- **Let me try a race condition**
- **Let me try a number / 0**
- **Let me try an out-of-bounds C pointer**

Includes a toggleable ML detector that can be enabled or disabled during kernel panic testing, following a C-first kernel engineering approach.
EOF_README

cat <<'EOF_MAKE' > "$ROOT/kernel/Makefile"
obj-m += panic_lab.o
panic_lab-objs := src/panic_lab_main.o src/faults.o

KDIR ?= /lib/modules/$(shell uname -r)/build
PWD  := $(shell pwd)

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
EOF_MAKE

cat <<'EOF_MAIN_C' > "$ROOT/kernel/src/panic_lab_main.c"
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("NATSKA");
MODULE_DESCRIPTION("Kernel Panic Detector and Fault Injection Lab (C-first)");

static int __init panic_lab_init(void)
{
    pr_info("panic_lab: module loaded\n");
    return 0;
}

static void __exit panic_lab_exit(void)
{
    pr_info("panic_lab: module unloaded\n");
}

module_init(panic_lab_init);
module_exit(panic_lab_exit);
EOF_MAIN_C

cat <<'EOF_FAULTS_C' > "$ROOT/kernel/src/faults.c"
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/slab.h>

/* 1. Memory leak trigger */
void trigger_memory_leak(void)
{
    void *leaked = kmalloc(4096, GFP_KERNEL);
    pr_warn("panic_lab: allocated 4096 bytes at %p without freeing\n", leaked);
}

/* 2. Race condition trigger */
void trigger_race_condition(void)
{
    pr_warn("panic_lab: race condition test hook invoked\n");
}

/* 3. Divide by zero trigger */
int trigger_divide_by_zero(int numerator, int denominator)
{
    if (denominator == 0) {
        pr_emerg("panic_lab: divide by zero condition\n");
        return 0;
    }
    return numerator / denominator;
}

/* 4. Invalid / out-of-bounds pointer access trigger */
void trigger_bad_pointer(void)
{
    volatile int *bad_ptr = (volatile int *)0xdeadbeefULL;
    pr_emerg("panic_lab: dereferencing invalid pointer %p\n", bad_ptr);
    (void)*bad_ptr;
}
EOF_FAULTS_C

cat <<'EOF_CARGO' > "$ROOT/egui-ui/Cargo.toml"
[package]
name = "panic-detector-egui"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.28"
egui = "0.28"
EOF_CARGO

cat <<'EOF_MAIN_RS' > "$ROOT/egui-ui/src/main.rs"
use eframe::egui;

#[derive(Default)]
struct PanicLabApp {
    ml_detector_enabled: bool,
    status_log: String,
}

impl eframe::App for PanicLabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Kernel Panic Detector");
            ui.checkbox(&mut self.ml_detector_enabled, "Enable ML Anomaly Detector");
            ui.separator();

            ui.group(|ui| {
                ui.label("Let me panic KERNEL");

                if ui.button("Let me try a memory leak").clicked() {
                    self.status_log = "Dispatched: Let me try a memory leak".to_string();
                }
                if ui.button("Let me try a race condition").clicked() {
                    self.status_log = "Dispatched: Let me try a race condition".to_string();
                }
                if ui.button("Let me try a number / 0").clicked() {
                    self.status_log = "Dispatched: Let me try a number / 0".to_string();
                }
                if ui.button("Let me try an out-of-bounds C pointer").clicked() {
                    self.status_log = "Dispatched: Let me try an out-of-bounds C pointer".to_string();
                }
            });

            ui.separator();
            ui.label(format!(
                "ML Detector: {}",
                if self.ml_detector_enabled { "ENABLED" } else { "DISABLED (raw test mode)" }
            ));
            ui.label(format!("Status: {}", self.status_log));
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Kernel Panic Detector",
        options,
        Box::new(|_cc| Ok(Box::new(PanicLabApp::default()))),
    )
}
EOF_MAIN_RS

cat <<'EOF_BUILD' > "$ROOT/scripts/build_all.sh"
#!/usr/bin/env bash
set -euo pipefail

make -C kernel
cargo build --manifest-path egui-ui/Cargo.toml
EOF_BUILD
chmod +x "$ROOT/scripts/build_all.sh"

echo "Created project tree at: $ROOT"
