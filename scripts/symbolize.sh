#!/usr/bin/env bash
# Offline symbolization for UART hex PCs printed by the panic handler.
#
# Usage:
#   ./scripts/symbolize.sh target/thumbv7em-none-eabihf/release/ArmOS 0x08001234 0x08001560

set -euo pipefail

ELF="${1:?usage: symbolize.sh <elf> <pc> [pc...]}"
shift

if [[ $# -eq 0 ]]; then
    echo "Provide at least one program counter (hex)." >&2
    exit 1
fi

if command -v llvm-symbolizer >/dev/null 2>&1; then
    SYM=llvm-symbolizer
elif command -v llvm-addr2line >/dev/null 2>&1; then
    SYM=llvm-addr2line
else
    SYM=addr2line
fi

for pc in "$@"; do
    echo "==> $pc"
    "$SYM" -e "$ELF" -f -C "$pc" || true
done