#!/usr/bin/env bash
# .ci/build.sh
# Builds a single component for a single Rust target using cargo-zigbuild.
# Replaces build_tarballs.jl + release.jl.
#
# Required env vars:
#   COMPONENT  — e.g. "spfc" or "spfc-target-ttf"
#   TARGET     — Rust target triple, e.g. "x86_64-unknown-linux-gnu"
#   TAG_NAME   — e.g. "cli-v1.0.0" (used only for the output filename)

set -euo pipefail

: "${COMPONENT:?COMPONENT env var is required}"
: "${TARGET:?TARGET env var is required}"
: "${TAG_NAME:?TAG_NAME env var is required}"

# Rust crate names use underscores; package flags use hyphens.
CRATE_NAME="${COMPONENT//-/_}"

echo "▶ Building component='$COMPONENT' crate='$CRATE_NAME' target='$TARGET' tag='$TAG_NAME'"

# ── Per-target naming conventions ────────────────────────────────────────────
case "$TARGET" in
  *windows*)
    EXE_EXT=".exe"
    LIB_PREFIX=""
    LIB_EXT=".dll"
    ;;
  *apple*)
    EXE_EXT=""
    LIB_PREFIX="lib"
    LIB_EXT=".dylib"
    ;;
  *)  # Linux, FreeBSD
    EXE_EXT=""
    LIB_PREFIX="lib"
    LIB_EXT=".so"
    ;;
esac

# ── RUSTFLAGS ─────────────────────────────────────────────────────────────────
# For cdylib targets on glibc/darwin/freebsd, avoid baking the CRT into the
# shared library (would duplicate it at runtime). Musl targets are statically
# linked by design so we leave them alone.
if [[ "$COMPONENT" != "spfc" && "$TARGET" != *musl* ]]; then
  export RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=-crt-static"
fi

# ── Build ─────────────────────────────────────────────────────────────────────
cargo zigbuild -p "$COMPONENT" --release --target "$TARGET"

# ── Collect binary into a staging dir ────────────────────────────────────────
STAGING="staging/${TARGET}"
mkdir -p "$STAGING" artifacts

if [[ "$COMPONENT" == "spfc" ]]; then
  cp "target/${TARGET}/release/spfc${EXE_EXT}" "$STAGING/"
else
  cp "target/${TARGET}/release/${LIB_PREFIX}${CRATE_NAME}${LIB_EXT}" "$STAGING/"
fi

# Always bundle the license
cp LICENSE-APACHE "$STAGING/"

# ── Package ───────────────────────────────────────────────────────────────────
TARBALL="artifacts/${COMPONENT}.${TAG_NAME}.${TARGET}.tar.gz"
tar -czf "$TARBALL" -C staging "${TARGET}"
rm -rf staging

echo "✅ Artifact ready: $TARBALL"