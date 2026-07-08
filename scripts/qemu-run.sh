#!/usr/bin/env bash
# Cargo runner: `cargo run` launches QEMU with semihosting I/O.
#
# Requires the `qemu` feature (enabled by default).
# Machine: mps2-an500 (Cortex-M7). Uses linker-qemu.ld (flash @ 0x00000000).
#
# `exec` replaces this shell with QEMU so host Ctrl+C goes straight to QEMU
# (same as running `qemu-system-arm -kernel …` directly). With `-nographic`,
# QEMU's own quit sequence is Ctrl+A then X if Ctrl+C does not reach it.

set -euo pipefail

BIN="${1:?usage: qemu-run.sh <path-to-elf>}"

exec qemu-system-arm \
    -cpu cortex-m7 \
    -machine mps2-an500 \
    -semihosting-config enable=on,target=native \
    -nographic \
    -kernel "$BIN"