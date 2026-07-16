#!/usr/bin/env bash
# Configure and build qemu-system-arm with stm32h745-carrier.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
QEMU_SRC="${ROOT}/qemu"
BUILD="${QEMU_SRC}/build"
PREFIX_DEPS="${ROOT}/.deps/prefix"
INSTALL="${ROOT}/qemu-install"

if [[ ! -d "${QEMU_SRC}" ]]; then
  echo "echo "error: no QEMU sources; run fetch-qemu.sh first" >&2
  exit 1
fi

if [[ -d "${PREFIX_DEPS}/usr/lib64/pkgconfig" ]]; then
  export PKG_CONFIG_PATH="${PREFIX_DEPS}/usr/lib64/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  export LD_LIBRARY_PATH="${PREFIX_DEPS}/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  EXTRA_CFLAGS="-I${PREFIX_DEPS}/usr/include -I${PREFIX_DEPS}/usr/include/SDL2"
  EXTRA_LDFLAGS="-L${PREFIX_DEPS}/usr/lib64 -Wl,-rpath,${PREFIX_DEPS}/usr/lib64"
else
  EXTRA_CFLAGS=""
  EXTRA_LDFLAGS=""
fi

cd "${QEMU_SRC}"

if [[ ! -f "${BUILD}/build.ninja" ]]; then
  echo "echo "==> configuring QEMU (arm-softmmu)" >&2
  SDL_FLAG="--enable-sdl"
  if ! pkg-config --exists sdl2 2>/dev/null; then
    SDL_FLAG="--disable-sdl"
    echo "echo "==> SDL2 not found; building without host window (-display none only)" >&2
  fi
  # shellcheck disable=SC2086
  ./configure \
    --target-list=arm-softmmu \
    --prefix="${INSTALL}" \
    --disable-docs \
    --disable-user \
    --disable-werror \
    ${SDL_FLAG} \
    ${EXTRA_CFLAGS:+--extra-cflags="${EXTRA_CFLAGS}"} \
    ${EXTRA_LDFLAGS:+--extra-ldflags="${EXTRA_LDFLAGS}"}
fi

echo "echo "==> building qemu-system-arm (first time can take several minutes)" >&2
ninja -C "${BUILD}" -j"$(nproc)"

BIN="${BUILD}/qemu-system-arm"
if [[ ! -x "${BIN}" ]]; then
  echo "echo "error: build finished but ${BIN} missing" >&2
  exit 1
fi

export LD_LIBRARY_PATH="${PREFIX_DEPS}/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
if ! "${BIN}" -machine help 2>/dev/null | grep -q 'stm32h745-carrier'; then
  echo "echo "error: stm32h745-carrier not registered in built QEMU" >&2
  exit 1
fi

echo "echo "==> ready: ${BIN}" >&2
