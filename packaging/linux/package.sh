#!/usr/bin/env bash
# Build and package MineRust for Linux (AppImage and tar.gz):
#
#   $DIST/minerust-<version>-linux-<arch>.AppImage
#   $DIST/minerust-<version>-linux-<arch>.tar.gz
#
# Usage: packaging/linux/package.sh [--skip-build] [--formats "appimage tar"]
set -euo pipefail

# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/linux"
APP_ID=com.coolcactuz.minerust

SKIP_BUILD=0
FORMATS="appimage tar"
while [ $# -gt 0 ]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --formats) FORMATS="$2"; shift 2 ;;
    -h | --help)
      echo "Usage: $0 [--skip-build] [--formats \"appimage tar\"]"
      exit 0
      ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ;;
  aarch64 | arm64) ARCH=aarch64 ;;
  *) echo "unsupported architecture $ARCH" >&2; exit 2 ;;
esac

BASENAME="minerust-$VERSION-linux-$ARCH"
echo "==> MineRust $VERSION for Linux $ARCH ($FORMATS)"

if [ "$SKIP_BUILD" = 0 ]; then
  (cd "$ROOT" && cargo build --release --locked)
fi

BIN="$CARGO_TARGET_DIR/release"
WORK="$CARGO_TARGET_DIR/linux-package"
STAGE="$WORK/root"
rm -rf "$WORK"
mkdir -p "$STAGE/usr/bin" "$STAGE/usr/share/applications" "$STAGE/usr/share/icons/hicolor/256x256/apps"

# Copy binary
install -Dm755 "$BIN/minerust" "$STAGE/usr/bin/minerust"
if [ -f "$BIN/bench_cpu" ]; then
  install -Dm755 "$BIN/bench_cpu" "$STAGE/usr/bin/bench_cpu"
fi
strip "$STAGE/usr/bin/minerust" 2>/dev/null || true

# Copy assets and icon
/bin/cp -rf "$ROOT/assets" "$STAGE/usr/bin/"
install -Dm644 "$HERE/$APP_ID.desktop" "$STAGE/usr/share/applications/$APP_ID.desktop"
install -Dm644 "$ROOT/assets/icons/minerust-256.png" "$STAGE/usr/share/icons/hicolor/256x256/apps/minerust.png"
copy_docs "$STAGE/usr/share/doc/minerust"

has() { case " $FORMATS " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }

# ---- .tar.gz ------------------------------------------------------------------------------------
if has tar; then
  echo "==> building tar.gz"
  mkdir -p "$WORK/tar/$BASENAME"
  /bin/cp -f "$STAGE/usr/bin/minerust" "$WORK/tar/$BASENAME/"
  if [ -f "$STAGE/usr/bin/bench_cpu" ]; then
    /bin/cp -f "$STAGE/usr/bin/bench_cpu" "$WORK/tar/$BASENAME/"
  fi
  /bin/cp -rf "$ROOT/assets" "$WORK/tar/$BASENAME/"
  copy_docs "$WORK/tar/$BASENAME"
  tar -C "$WORK/tar" -czf "$DIST/$BASENAME.tar.gz" "$BASENAME"
  echo "wrote $DIST/$BASENAME.tar.gz"
fi

# ---- AppImage -----------------------------------------------------------------------------------
if has appimage; then
  echo "==> building AppImage"
  APPDIR="$WORK/MineRust.AppDir"
  /bin/cp -R "$STAGE" "$APPDIR"
  
  # Set up AppRun script
  cat << 'EOF' > "$APPDIR/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="$HERE/usr/bin:$PATH"
export LD_LIBRARY_PATH="$HERE/usr/lib:$LD_LIBRARY_PATH"
cd "$HERE/usr/bin"
exec "$HERE/usr/bin/minerust" "$@"
EOF
  chmod +x "$APPDIR/AppRun"

  # Root AppDir integration
  /bin/cp -f "$HERE/$APP_ID.desktop" "$APPDIR/$APP_ID.desktop"
  /bin/cp -f "$HERE/$APP_ID.desktop" "$APPDIR/minerust.desktop"
  /bin/cp -f "$ROOT/assets/icons/minerust-256.png" "$APPDIR/minerust.png"
  ln -sf minerust.png "$APPDIR/.DirIcon"
  ln -sf usr/bin/assets "$APPDIR/assets"

  TOOL="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
  if [ -z "$TOOL" ]; then
    TOOL="$CARGO_TARGET_DIR/appimagetool-$ARCH.AppImage"
    if [ ! -x "$TOOL" ]; then
      echo "==> downloading appimagetool for $ARCH"
      curl -fsSL -o "$TOOL" "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage"
      chmod +x "$TOOL"
    fi
  fi

  OUT="$(cd "$DIST" && pwd)/$BASENAME.AppImage"
  APPDIR="$(cd "$APPDIR" && pwd)"
  TOOL="$(cd "$(dirname "$TOOL")" && pwd)/$(basename "$TOOL")"

  # Build AppImage with extract-and-run runtime fallback for CI containers
  (cd "$DIST" && ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 "$TOOL" --no-appstream "$APPDIR" "$OUT")
  echo "wrote $OUT"
fi

echo "==> Linux packaging complete"
ls -lh "$DIST"/minerust*
