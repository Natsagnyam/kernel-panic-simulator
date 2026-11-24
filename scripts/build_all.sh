#!/usr/bin/env bash
set -euo pipefail

make -C kernel
cargo build --manifest-path egui-ui/Cargo.toml
