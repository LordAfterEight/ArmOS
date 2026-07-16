#!/usr/bin/env bash
# Cargo runner for ArmOS.
#
# Default ELF (linker.ld @ 0x08000000):
#   stm32h745-carrier from qemu-stm32h745/ (fetch+build on first use)
#
# mps2 ELF (linker-qemu.ld @ 0x00000000, --features mps2):
#   system qemu-system-arm -machine mps2-an500 + semihosting
#
# Quit custom machine: close SDL window, or Ctrl-A then X on serial.

set -euo pipefail

BIN="${1:?usage: qemu-run.sh <path-to-elf>}"
shift || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Detect load address: hardware/custom QEMU uses flash @ 0x08000000.
is_stm32_map() {
  if command -v llvm-readelf >/dev/null 2>&1; then
    llvm-readelf -l "$BIN" 2>/dev/null | grep -qE '0x0*8000000'
    return $?
  fi
  if command -v arm-none-eabi-readelf >/dev/null 2>&1; then
    arm-none-eabi-readelf -l "$BIN" 2>/dev/null | grep -qE '0x0*8000000'
    return $?
  fi
  if command -v readelf >/dev/null 2>&1; then
    readelf -l "$BIN" 2>/dev/null | grep -qE '0x0*8000000'
    return $?
  fi
  # Fallback: default to custom machine
  return 0
}

run_mps2() {
  echo "==> mps2-an500 + semihosting (legacy)" >&2
  exec qemu-system-arm \
    -cpu cortex-m7 \
    -machine mps2-an500 \
    -semihosting-config enable=on,target=native \
    -nographic \
    -kernel "$BIN" \
    "$@"
}

run_carrier() {
  local ensure="${ROOT}/qemu-stm32h745/scripts/ensure-qemu.sh"
  if [[ ! -x "${ensure}" ]]; then
    echo "error: missing ${ensure}" >&2
    exit 1
  fi

  local qemu_bin
  qemu_bin="$("${ensure}")"

  local deps_lib="${ROOT}/qemu-stm32h745/.deps/prefix/usr/lib64"
  if [[ -d "${deps_lib}" ]]; then
    export LD_LIBRARY_PATH="${deps_lib}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  fi

  local display_args=(-display sdl)
  if [[ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]]; then
    echo "warning: no DISPLAY/WAYLAND_DISPLAY — using -display none" >&2
    display_args=(-display none)
  fi
  case "${ARMOS_QEMU_DISPLAY:-}" in
    none|off) display_args=(-display none) ;;
    sdl) display_args=(-display sdl) ;;
    gtk) display_args=(-display gtk) ;;
    "") ;;
    *) display_args=(-display "${ARMOS_QEMU_DISPLAY}") ;;
  esac

  echo "==> stm32h745-carrier (${qemu_bin})" >&2
  exec "${qemu_bin}" \
    -machine stm32h745-carrier \
    -kernel "${BIN}" \
    -serial mon:stdio \
    "${display_args[@]}" \
    "$@"
}

if [[ "${ARMOS_MPS2:-0}" == "1" ]] || ! is_stm32_map; then
  run_mps2 "$@"
else
  run_carrier "$@"
fi
