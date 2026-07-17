#!/usr/bin/env bash
# Cargo runner — stm32h745-carrier with hardware-gated NOR/SDRAM.
#
#   -kernel bootloader.elf
#   -machine stm32h745-carrier,os-image=ArmOS.elf
#
# Bootloader must enable FMC + QUADSPI before the OS image is visible/usable.
#
# Timing (important):
#   Machine SYSCLK is 480 MHz (virtual CPU/SysTick). That does NOT throttle how
#   fast TCG runs on the host — by default the guest is unrestricted TCG.
#
#   Optional wall-clock pacing (often too optimistic on a laptop + SDL):
#     ARMOS_QEMU_ICOUNT=shift=1,align=on   # ~480 MHz real-time target
#   With align=on, if the host cannot sustain that rate QEMU prints
#   "The guest is now late by …" and runs full-speed anyway (no useful throttle).
#   Prefer a larger shift if you want align without spam, e.g. shift=4 (~62 MHz).

set -euo pipefail

BIN="${1:?usage: qemu-run.sh <path-to-ArmOS-elf>}"
shift || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROFILE_DIR="$(dirname "${BIN}")"

# Default off: unrestricted TCG, no "guest is late" spam from align=on.
icount_args() {
  local spec="${ARMOS_QEMU_ICOUNT:-off}"
  case "${spec}" in
    ""|off|OFF|max|none|disable|disabled)
      return 0
      ;;
    *)
      printf '%s\n' "-icount" "${spec}"
      ;;
  esac
}

has_vma() {
  local pattern="$1"
  if command -v llvm-readelf >/dev/null 2>&1; then
    llvm-readelf -l "$BIN" 2>/dev/null | grep -qE "${pattern}"
    return $?
  fi
  if command -v readelf >/dev/null 2>&1; then
    readelf -l "$BIN" 2>/dev/null | grep -qE "${pattern}"
    return $?
  fi
  return 1
}

run_mps2() {
  echo "==> mps2-an500 + semihosting (legacy)" >&2
  exec qemu-system-arm \
    -cpu cortex-m7 -machine mps2-an500 \
    -semihosting-config enable=on,target=native \
    -nographic -kernel "$BIN" "$@"
}

ensure_bootloader() {
  local bl="${PROFILE_DIR}/bootloader"
  local cargo_bin="${CARGO:-cargo}"
  local profile_flag=(--release)
  if [[ "${PROFILE_DIR}" == *"/debug" ]]; then
    profile_flag=()
  fi

  # Always rebuild if missing, or if bootloader source / linker is newer than the
  # binary (stale stage-0 without pinmux leaves NOR/SDRAM unmapped → black UI).
  local need_build=0
  if [[ ! -x "${bl}" ]]; then
    need_build=1
  else
    local src
    for src in \
      "${ROOT}/src/bin/bootloader.rs" \
      "${ROOT}/linker-boot.ld" \
      "${ROOT}/Cargo.toml"
    do
      if [[ -f "${src}" && "${src}" -nt "${bl}" ]]; then
        need_build=1
        break
      fi
    done
  fi

  if [[ "${need_build}" -eq 1 ]]; then
    echo "==> building bootloader" >&2
    (cd "${ROOT}" && ARMOS_SKIP_QEMU_BUILD=1 "${cargo_bin}" build "${profile_flag[@]}" --bin bootloader) >&2
  fi
  [[ -x "${bl}" ]] || { echo "error: bootloader missing at ${bl}" >&2; exit 1; }
  echo "${bl}"
}

run_carrier() {
  local ensure="${ROOT}/qemu-stm32h745/scripts/ensure-qemu.sh"
  [[ -x "${ensure}" ]] || { echo "error: missing ${ensure}" >&2; exit 1; }
  local qemu_bin
  qemu_bin="$("${ensure}")"

  local deps_lib="${ROOT}/qemu-stm32h745/.deps/prefix/usr/lib64"
  if [[ -d "${deps_lib}" ]]; then
    export LD_LIBRARY_PATH="${deps_lib}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  fi

  local display_args=(-display sdl)
  if [[ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]]; then
    echo "warning: no DISPLAY — using -display none" >&2
    display_args=(-display none)
  fi
  case "${ARMOS_QEMU_DISPLAY:-}" in
    none|off) display_args=(-display none) ;;
    sdl) display_args=(-display sdl) ;;
    "") ;;
    *) display_args=(-display "${ARMOS_QEMU_DISPLAY}") ;;
  esac

  local icount=()
  # shellcheck disable=SC2207
  mapfile -t icount < <(icount_args)

  if has_vma '0x0*90000000'; then
    local bootloader
    bootloader="$(ensure_bootloader)"
    echo "==> stm32h745-carrier (hardware-gated NOR/SDRAM)" >&2
    echo "    bootloader: ${bootloader}" >&2
    echo "    os-image:   ${BIN}" >&2
    if ((${#icount[@]})); then
      echo "    icount:     ${icount[*]}" >&2
    fi
    exec "${qemu_bin}" \
      -machine "stm32h745-carrier,os-image=${BIN}" \
      -kernel "${bootloader}" \
      -serial mon:stdio \
      "${display_args[@]}" \
      "${icount[@]}" \
      "$@"
  fi

  echo "==> stm32h745-carrier: -kernel ${BIN} (no NOR XIP VMA)" >&2
  if ((${#icount[@]})); then
    echo "    icount:     ${icount[*]}" >&2
  fi
  exec "${qemu_bin}" \
    -machine stm32h745-carrier \
    -kernel "${BIN}" \
    -serial mon:stdio \
    "${display_args[@]}" \
    "${icount[@]}" \
    "$@"
}

if [[ "${ARMOS_MPS2:-0}" == "1" ]]; then
  run_mps2 "$@"
elif has_vma '0x0*90000000' || has_vma '0x0*8000000' || has_vma '0x0*C02F0000'; then
  run_carrier "$@"
else
  run_mps2 "$@"
fi
