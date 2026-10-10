#!/usr/bin/env bash
# Build, assemble, and create macOS release DMG:
#
#   $DIST/minerust-<version>-macos-<arch>.dmg
#
# Usage: packaging/macos/package.sh [--arch arm64|x86_64|universal] [--skip-build]
set -euo pipefail

# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/macos"

ARCH="arm64"
SKIP_BUILD=0
while [ $# -gt 0 ]; do
  case "$1" in
    --arch) ARCH="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h | --help)
      echo "Usage: $0 [--arch arm64|x86_64|universal] [--skip-build]"
      exit 0
      ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

case "$ARCH" in
  universal) TARGETS=(aarch64-apple-darwin x86_64-apple-darwin) ;;
  aarch64 | arm64) ARCH="arm64"; TARGETS=(aarch64-apple-darwin) ;;
  x86_64) TARGETS=(x86_64-apple-darwin) ;;
  *) echo "unsupported architecture $ARCH" >&2; exit 2 ;;
esac

export MACOSX_DEPLOYMENT_TARGET=11.0
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
WORK="$CARGO_TARGET_DIR/macos-package"
APP="$WORK/MineRust.app"
DMG="$DIST/minerust-$VERSION-macos-$ARCH.dmg"

echo "==> MineRust $VERSION for macOS ($ARCH), identity: $IDENTITY"

# ---- build -------------------------------------------------------------------------------------
if [ "$SKIP_BUILD" = 0 ]; then
  # If running natively on macOS without cross-target flags specified
  if [ "$(uname -s)" = "Darwin" ] && [ "$ARCH" != "universal" ]; then
    (cd "$ROOT" && cargo build --release --locked)
  else
    args=()
    for t in "${TARGETS[@]}"; do args+=(--target "$t"); done
    (cd "$ROOT" && cargo build --release --locked "${args[@]}")
  fi
fi

rm -rf "$WORK"
mkdir -p "$WORK/bin"

if [ "$ARCH" = "universal" ]; then
  inputs=()
  for t in "${TARGETS[@]}"; do inputs+=("$CARGO_TARGET_DIR/$t/release/minerust"); done
  lipo -create -output "$WORK/bin/minerust" "${inputs[@]}"
else
  if [ -f "$CARGO_TARGET_DIR/${TARGETS[0]}/release/minerust" ]; then
    /bin/cp -f "$CARGO_TARGET_DIR/${TARGETS[0]}/release/minerust" "$WORK/bin/minerust"
  else
    /bin/cp -f "$CARGO_TARGET_DIR/release/minerust" "$WORK/bin/minerust"
  fi
fi

# ---- assemble MineRust.app --------------------------------------------------------------------
echo "==> assembling $APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
/bin/cp -f "$WORK/bin/minerust" "$APP/Contents/MacOS/minerust"
/bin/cp -rf "$ROOT/assets" "$APP/Contents/Resources/"
/bin/cp -f "$ROOT/assets/icons/minerust.icns" "$APP/Contents/Resources/minerust.icns"

sed -e "s/@VERSION@/$VERSION/g" "$HERE/Info.plist.in" > "$APP/Contents/Info.plist"
if command -v plutil >/dev/null; then
  plutil -lint "$APP/Contents/Info.plist"
fi
printf 'APPL????' > "$APP/Contents/PkgInfo"

# ---- codesign ---------------------------------------------------------------------------------
if command -v codesign >/dev/null; then
  echo "==> signing bundle with identity: $IDENTITY"
  codesign --force --sign "$IDENTITY" "$APP/Contents/MacOS/minerust"
  codesign --force --sign "$IDENTITY" "$APP"
fi

# ---- create DMG -------------------------------------------------------------------------------
echo "==> building $DMG"
STAGE="$WORK/dmg"
mkdir -p "$STAGE"
/bin/cp -R "$APP" "$STAGE/MineRust.app"
ln -sf /Applications "$STAGE/Applications"

rm -f "$DMG" "$WORK/raw.dmg"
if command -v hdiutil >/dev/null; then
  # makehybrid + convert prevents CI runner hangs ("Resource busy")
  hdiutil makehybrid -hfs -hfs-volume-name "MineRust" -hfs-openfolder "$STAGE" -o "$WORK/raw.dmg" "$STAGE"
  hdiutil convert "$WORK/raw.dmg" -format UDZO -imagekey zlib-level=9 -o "$DMG"
  rm -f "$WORK/raw.dmg"
  if command -v codesign >/dev/null; then
    codesign --force --sign "$IDENTITY" "$DMG"
  fi
  echo "wrote $DMG"
else
  echo "warning: hdiutil not available on non-macOS host, packaging DMG tarball fallback for testing"
  tar -C "$STAGE" -czf "$DIST/minerust-$VERSION-macos-$ARCH.tar.gz" MineRust.app
fi

echo "==> macOS packaging complete"
