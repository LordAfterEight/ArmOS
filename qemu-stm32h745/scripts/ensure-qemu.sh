#!/usr/bin/env bash
# Ensure a qemu-system-arm with stm32h745-carrier exists (fetch + build if needed).
# Safe to call from cargo build.rs and the cargo runner on every invocation.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ROOT}/qemu/build/qemu-system-arm"
STAMP="${ROOT}/qemu/.armos-qemu-ready"

# Skip rebuild when binary already works (unless FORCE_QEMU_REBUILD=1).
if [[ "${FORCE_QEMU_REBUILD:-0}" != "1" && -x "${BIN}" ]]; then
  export LD_LIBRARY_PATH="${ROOT}/.deps/prefix/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  if "${BIN}" -machine help 2>/dev/null | grep -q 'stm32h745-carrier'; then
    # Print path for callers that want it
    echo "${BIN}"
    exit 0
  fi
fi

echo "==> preparing custom QEMU (stm32h745-carrier) under qemu-stm32h745/" >&2
"${ROOT}/scripts/bootstrap-deps.sh" || true
"${ROOT}/scripts/fetch-qemu.sh"
"${ROOT}/scripts/build-qemu.sh"
date -Iseconds > "${STAMP}"
echo "${BIN}"
