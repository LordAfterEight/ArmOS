#!/usr/bin/env bash
# Clone QEMU v10.1.0 and apply stm32h745-carrier patches.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
QEMU_SRC="${ROOT}/qemu"
TAG="v10.1.0"

if [[ -d "${QEMU_SRC}/.git" ]] || [[ -x "${QEMU_SRC}/build/qemu-system-arm" ]]; then
  echo "==> qemu tree already present at ${QEMU_SRC}" >&2
  exit 0
fi

if [[ -e "${QEMU_SRC}" && ! -d "${QEMU_SRC}/.git" ]]; then
  echo "error: ${QEMU_SRC} exists but is not a git checkout; remove it and re-run" >&2
  exit 1
fi

echo "==> cloning QEMU ${TAG} (shallow)" >&2
git clone --depth 1 --branch "${TAG}" \
  https://gitlab.com/qemu-project/qemu.git "${QEMU_SRC}"
cd "${QEMU_SRC}"
git checkout -B stm32h745-carrier
git submodule update --init --depth 1 || true

echo "==> applying ArmOS patches" >&2
git am "${ROOT}"/patches/*.patch

echo "==> QEMU sources ready at ${QEMU_SRC}" >&2
