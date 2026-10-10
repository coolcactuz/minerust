#!/usr/bin/env bash
# Shared setup for MineRust packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                workspace root
#   VERSION             package version from Cargo.toml (override: MINERUST_VERSION)
#   DIST                output directory for release artifacts (default: $ROOT/dist/release)
#   MINERUST_BUILD_SHA  git commit baked into metadata
#   MINERUST_BUILD_DATE UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR    cargo target directory (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# Read version from Cargo.toml [package] section
read_package_version() {
  awk '
    /^\[package\]/ { in_pkg = 1; next }
    /^\[/ && !/^\[package\]/ { in_pkg = 0 }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
}

VERSION="${MINERUST_VERSION:-$(read_package_version)}"
if [ -z "$VERSION" ]; then
  echo "error: could not read [package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

if [ -z "${MINERUST_BUILD_SHA:-}" ]; then
  MINERUST_BUILD_SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || true)"
fi
export MINERUST_BUILD_SHA
export MINERUST_BUILD_DATE="${MINERUST_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning or stderr message
warn() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    echo "::warning::$*"
  else
    echo "warning: $*" >&2
  fi
}

# Copy documentation and licensing assets into a package target
copy_docs() {
  local dest="$1"
  mkdir -p "$dest"
  for f in README.md LICENSE LICENSE-MIT LICENSE-APACHE; do
    if [ -f "$ROOT/$f" ]; then
      /bin/cp -f "$ROOT/$f" "$dest/"
    fi
  done
}
