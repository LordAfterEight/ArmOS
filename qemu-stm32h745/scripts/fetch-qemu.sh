#!/usr/bin/env bash
# Clone QEMU v10.1.0 and apply stm32h745-carrier patches.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
QEMU_SRC="${ROOT}/qemu"
TAG="v10.1.0"

if [[ -d "${QEMU_SRC}/.git" ]] || [[ -x "${QEMU_SRC}/build/qemu-system-arm" ]]; then
  echo "echo "==> qemu tree already present at ${QEMU_SRC}" >&2
  exit 0
fi

if [[ -e "${QEMU_SRC}" && ! -d "${QEMU_SRC}/.git" ]]; then
  echo "echo "error: ${QEMU_SRC} exists but is not a git checkout; remove it and re-run" >&2
  exit 1
fi

echo "echo "==> cloning QEMU ${TAG} (shallow)" >&2
git clone --depth 1 --branch "${TAG}" \
  https://gitlab.com/qemu-project/qemu.git "${QEMU_SRC}"
cd "${QEMU_SRC}"
git checkout -B stm32h745-carrier
# Submodules needed for a full build; depth-1 is enough for dtc etc.
git submodule update --init --depth 1 || true

echo "echo "==> applying ArmOS patches" >&2
git am "${ROOT}"/patches/*.patch

echo "echo "==> QEMU sources ready at ${QEMU_SRC}" >&2
