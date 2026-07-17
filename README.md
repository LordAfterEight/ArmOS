# ArmOS

Bare-metal OS for **STM32H745BIT6** (Cortex-M7).

## Quick start (custom QEMU)

```bash
git clone <this-repo> ArmOS
cd ArmOS
cargo run --release
```

Boot architecture:

```text
MCU flash  →  bootloader  →  jumps to NOR
NOR (XIP)  →  ArmOS       →  uses FMC SDRAM as main RAM
```

The first `cargo build` / `cargo run`:

1. Builds **bootloader** (MCU flash `@ 0x08000000`, DTCM only) and **ArmOS** (NOR XIP `@ 0x90000000`, SDRAM RAM).
2. Fetches/builds in-tree QEMU with **`stm32h745-carrier`**.
3. Runs: `-kernel bootloader` + `-machine …,os-image=ArmOS` (serial + SDL LTDC).
   NOR/SDRAM stay unmapped until the bootloader enables FMC + QUADSPI.

### Host packages (Fedora)

```bash
sudo dnf install gcc ninja-build meson pixman-devel glib2-devel \
  zlib-devel libslirp-devel python3 SDL2-devel
rustup target add thumbv7em-none-eabihf
```

Without root, `qemu-stm32h745/scripts/bootstrap-deps.sh` can stage headers into `.deps/`.

### Useful commands

| Command | Meaning |
|---------|---------|
| `cargo run --release` | Bootloader + OS on `stm32h745-carrier` |
| `cargo build --release --bin bootloader --bin ArmOS` | Both images |
| `cargo hw` | Release OS image (for NOR programming) |
| `cargo mps2` | Legacy `mps2-an500` + semihosting |
| `ARMOS_QEMU_DISPLAY=none cargo run --release` | Headless |
| `ARMOS_QEMU_ICOUNT=shift=1,align=on cargo run --release` | Try real-time ~480 MHz (warns if host can’t keep up) |
| `./qemu-stm32h745/scripts/install-global.sh` | Install `qemu-system-arm` → `~/.local/bin` (carrier machine) |

### Manual QEMU (any OS ELF linked for NOR)

```bash
# Once: put ArmOS QEMU on PATH (stm32h745-carrier)
./qemu-stm32h745/scripts/install-global.sh
# new shell, or: export PATH="$HOME/.local/bin:$PATH"

qemu-system-arm \
  -machine stm32h745-carrier,os-image=target/thumbv7em-none-eabihf/release/ArmOS \
  -kernel target/thumbv7em-none-eabihf/release/bootloader \
  -serial mon:stdio -display sdl
```

### Flash to hardware (SWD + IS25LP01GJ)

See **[docs/flashing.md](docs/flashing.md)** and **[qemu-stm32h745/docs/board-carrier.md](qemu-stm32h745/docs/board-carrier.md)** (pinmux from netlist).

1. **Fix `QSPI_NCS`** on the schematic (CE# is currently not wired to the MCU)
2. Program **bootloader** → MCU flash `@ 0x08000000` via J4 SWD
3. Program **ArmOS** → **IS25LP01GJ** NOR via QSPI (external loader or one-shot burner)
4. Reset → bootloader → XIP `@ 0x90000000` + SDRAM

## Layout

| Path | Role |
|------|------|
| `src/bin/bootloader.rs` | Stage-0 (MCU flash) |
| `src/main.rs` | ArmOS entry (NOR XIP) |
| `linker-boot.ld` | Bootloader: MCU flash + DTCM |
| `linker.ld` | OS: NOR + SDRAM |
| `qemu-stm32h745/` | Custom QEMU machine |
| `scripts/qemu-run.sh` | Cargo runner |

See `qemu-stm32h745/docs/memory-map.md` and `how-the-machine-works.md`.
