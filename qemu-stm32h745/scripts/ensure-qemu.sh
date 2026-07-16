#!/usr/bin/env bash
# Ensure qemu-system-arm with stm32h745-carrier exists. Prints binary path on stdout.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ROOT}/qemu/build/qemu-system-arm"

if [[ "${FORCE_QEMU_REBUILD:-0}" != "1" && -x "${BIN}" ]]; then
  export LD_LIBRARY_PATH="${ROOT}/.deps/prefix/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  if "${BIN}" -machine help 2>/dev/null | grep -q 'stm32h745-carrier'; then
    echo "${BIN}"
    exit 0
  fi
fi

echo "==> preparing custom QEMU under qemu-stm32h745/" >&2
"${ROOT}/scripts/bootstrap-deps.sh" || true
"${ROOT}/scripts/fetch-qemu.sh"
"${ROOT}/scripts/build-qemu.sh"
echo "${BIN}"
