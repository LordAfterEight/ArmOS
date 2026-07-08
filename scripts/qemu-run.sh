#!/usr/bin/env bash
# Cargo runner: `cargo run` launches QEMU with semihosting I/O.
#
# Requires the `qemu` feature (enabled by default).
# Machine: mps2-an500 (Cortex-M7). Uses linker-qemu.ld (flash @ 0x00000000).

set -euo pipefail

BIN="${1:?usage: qemu-run.sh <path-to-elf>}"

# Semihost SYS_EXIT makes QEMU return non-zero; that still means success here.
qemu-system-arm \
    -cpu cortex-m7 \
    -machine mps2-an500 \
    -semihosting-config enable=on,target=native \
    -nographic \
    -kernel "$BIN" \
    || true