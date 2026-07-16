# STM32H745 custom QEMU for ArmOS

In-tree QEMU machine **`stm32h745-carrier`** used by `cargo run`.

## Layout

| Path | Purpose |
|------|---------|
| `patches/` | Applied on top of QEMU v10.1.0 |
| `scripts/ensure-qemu.sh` | Fetch + build if missing (called by Cargo) |
| `scripts/fetch-qemu.sh` | Clone QEMU + `git am` patches |
| `scripts/build-qemu.sh` | Configure/ninja arm-softmmu |
| `qemu/` | Full QEMU tree (**gitignored**, created on first build) |
| `docs/` | Memory map, peripherals, how it works |

## First-time deps (host)

Fedora example:

```bash
sudo dnf install gcc ninja-build meson pixman-devel glib2-devel \
  zlib-devel libslirp-devel python3 SDL2-devel
```

Without root, `scripts/bootstrap-deps.sh` can pull RPMs into `.deps/prefix`.

## Manual build

```bash
./scripts/ensure-qemu.sh
./qemu/build/qemu-system-arm -machine help | grep stm32h745
```

Normally you never need this — `cargo build` / `cargo run` from the ArmOS root call `ensure-qemu.sh` automatically.
