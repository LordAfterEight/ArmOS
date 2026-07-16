# ArmOS

Bare-metal OS for **STM32H745BIT6** (Cortex-M7).

## Quick start (custom QEMU)

```bash
git clone <this-repo> ArmOS
cd ArmOS
cargo run --release
```

The first `cargo build` / `cargo run`:

1. Builds the firmware with the **hardware memory map** (flash `@ 0x08000000`).
2. Fetches QEMU **v10.1.0**, applies patches under `qemu-stm32h745/patches/`, and compiles `qemu-system-arm` with **`stm32h745-carrier`** (cached under `qemu-stm32h745/qemu/`, gitignored).
3. Launches the ELF with USART on stdio and an SDL window for LTDC (800×480).

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
| `cargo run --release` | Build + run on `stm32h745-carrier` |
| `cargo build --release` | Build firmware (+ ensure QEMU exists) |
| `cargo hw` | Alias: release build for flashing |
| `cargo mps2` | Legacy `mps2-an500` + semihosting (system QEMU) |
| `ARMOS_SKIP_QEMU_BUILD=1 cargo build` | Firmware only; do not fetch/build QEMU |
| `ARMOS_QEMU_DISPLAY=none cargo run --release` | Headless (no SDL window) |

### Flash to hardware

```bash
cargo build --release   # produces target/thumbv7em-none-eabihf/release/ArmOS.{elf,bin,hex}
# then probe-rs / openocd / your SWD tool
```

## Layout

| Path | Role |
|------|------|
| `src/` | Kernel / drivers / UI |
| `linker.ld` | Hardware + custom QEMU map |
| `linker-qemu.ld` | Legacy mps2 map (`--features mps2`) |
| `qemu-stm32h745/` | Custom QEMU machine (patches + scripts) |
| `scripts/qemu-run.sh` | Cargo runner |

See `qemu-stm32h745/docs/how-the-machine-works.md` for how the emulator machine works.
