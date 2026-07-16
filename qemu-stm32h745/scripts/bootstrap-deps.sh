#!/usr/bin/env bash
# Optional local .deps when system -devel packages are missing.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEPS="${ROOT}/.deps"
PREFIX="${DEPS}/prefix"

if pkg-config --exists pixman-1 glib-2.0 2>/dev/null; then
  echo "==> system pkg-config deps OK" >&2
  exit 0
fi

mkdir -p "${DEPS}"
cd "${DEPS}"

echo "==> downloading build dependency RPMs" >&2
dnf download --resolve \
  pixman-devel glib2-devel zlib-devel libslirp-devel \
  libffi-devel pcre2-devel SDL2-devel 2>/dev/null \
  || dnf download --resolve \
    pixman-devel glib2-devel zlib-devel libslirp-devel \
    libffi-devel pcre2-devel 2>/dev/null \
  || true

echo "==> extracting into ${PREFIX}" >&2
rm -rf "${PREFIX}"
mkdir -p "${PREFIX}"

shopt -s nullglob
for rpm in *x86_64.rpm *noarch.rpm; do
  [[ -f "${rpm}" ]] || continue
  case "${rpm}" in cmake*) continue ;; esac
  rpm2cpio "${rpm}" | (cd "${PREFIX}" && cpio -idm 2>/dev/null) || true
done

for pc in "${PREFIX}"/usr/lib64/pkgconfig/*.pc; do
  [[ -f "${pc}" ]] || continue
  sed -i "s|^prefix=/usr|prefix=${PREFIX}/usr|" "${pc}"
done

export PKG_CONFIG_PATH="${PREFIX}/usr/lib64/pkgconfig"
if pkg-config --exists pixman-1 glib-2.0; then
  echo "==> local .deps/prefix ready" >&2
else
  echo "==> warning: install pixman-devel glib2-devel (and SDL2-devel for display)" >&2
fi
