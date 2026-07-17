# STM32H745 custom QEMU for ArmOS

In-tree QEMU machine **`stm32h745-carrier`** used by `cargo run`.

## Layout

| Path | Purpose |
|------|---------|
| `patches/` | Applied on top of QEMU v10.1.0 |
| `scripts/ensure-qemu.sh` | Fetch + build if missing (called by Cargo) |
| `scripts/fetch-qemu.sh` | Clone QEMU + `git am` patches |
| `scripts/build-qemu.sh` | Configure/ninja arm-softmmu |
| `scripts/install-global.sh` | Put `qemu-system-arm` on PATH (`~/.local/bin`) |
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

## Install system-wide (user PATH)

So any shell can run `qemu-system-arm -machine stm32h745-carrier …`:

```bash
./scripts/install-global.sh
# new shell, or:  export PATH="$HOME/.local/bin:$PATH"
qemu-system-arm -machine help | grep stm32h745
```

Installs a launcher at `~/.local/bin/qemu-system-arm` that points at this tree’s
build and sets library paths for staged deps. Rebuilds via `ensure-qemu.sh` /
`build-qemu.sh` are picked up automatically. Distro QEMU remains at
`/usr/bin/qemu-system-arm` if you need it by absolute path.

Optional:

```bash
ARMOS_QEMU_INSTALL_NAME=qemu-system-arm-armos ./scripts/install-global.sh  # no shadow
ARMOS_QEMU_INSTALL_BIN=/usr/local/bin sudo -E ./scripts/install-global.sh  # system-wide
```
