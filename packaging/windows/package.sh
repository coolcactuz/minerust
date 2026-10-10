#!/usr/bin/env bash
# Build and package MineRust for Windows in bash / Git Bash:
#
#   $DIST/minerust-<version>-windows-<arch>-portable.zip
#
# Usage: packaging/windows/package.sh [--arch x86_64] [--skip-build]
set -euo pipefail

# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"

ARCH="x86_64"
SKIP_BUILD=0
while [ $# -gt 0 ]; do
  case "$1" in
    --arch) ARCH="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h | --help)
      echo "Usage: $0 [--arch x86_64] [--skip-build]"
      exit 0
      ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

BASENAME="minerust-$VERSION-windows-$ARCH-portable"
echo "==> MineRust $VERSION for Windows $ARCH (portable)"

WORK="$CARGO_TARGET_DIR/windows-package"
STAGE="$WORK/$BASENAME"
rm -rf "$WORK"
mkdir -p "$STAGE"

BIN="$CARGO_TARGET_DIR/release"
if [ -f "$CARGO_TARGET_DIR/$ARCH-pc-windows-msvc/release/minerust.exe" ]; then
  BIN="$CARGO_TARGET_DIR/$ARCH-pc-windows-msvc/release"
fi

/bin/cp -f "$BIN/minerust.exe" "$STAGE/"
/bin/cp -rf "$ROOT/assets" "$STAGE/"
mkdir -p "$STAGE/saves"
copy_docs "$STAGE"

ZIP="$DIST/$BASENAME.zip"
rm -f "$ZIP"
if command -v 7z >/dev/null; then
  (cd "$WORK" && 7z a "$ZIP" "$BASENAME")
elif command -v zip >/dev/null; then
  (cd "$WORK" && zip -r "$ZIP" "$BASENAME")
else
  echo "error: neither 7z nor zip found" >&2
  exit 1
fi

echo "wrote $ZIP"
