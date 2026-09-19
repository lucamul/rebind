#!/usr/bin/env bash
# Downloads the prebuilt PDFium dynamic library for the current
# platform into vendor/pdfium/. Not committed to source control (it's
# ~7MB and platform-specific) — run this once after cloning.
#
# Source: https://github.com/bblanchon/pdfium-binaries (MIT-licensed
# build/packaging around Google's BSD-3-Clause PDFium). Pinned to a
# known-good release; bump PDFIUM_TAG deliberately, not blindly.
set -euo pipefail

PDFIUM_TAG="chromium/8057"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT/vendor/pdfium"

os="$(uname -s)"
arch="$(uname -m)"

case "$os-$arch" in
  Darwin-arm64) asset="pdfium-mac-arm64.tgz"; lib_name="libpdfium.dylib" ;;
  Darwin-x86_64) asset="pdfium-mac-x64.tgz"; lib_name="libpdfium.dylib" ;;
  Linux-x86_64) asset="pdfium-linux-x64.tgz"; lib_name="libpdfium.so" ;;
  Linux-aarch64) asset="pdfium-linux-arm64.tgz"; lib_name="libpdfium.so" ;;
  *)
    echo "fetch-pdfium: no known PDFium asset for $os-$arch" >&2
    echo "see https://github.com/bblanchon/pdfium-binaries/releases/tag/${PDFIUM_TAG/\//%2F}" >&2
    exit 1
    ;;
esac

url="https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_TAG}/${asset}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "fetch-pdfium: downloading $asset ($os-$arch) from $PDFIUM_TAG..."
curl -fsSL "$url" -o "$tmp/$asset"

mkdir -p "$OUT_DIR/lib"
tar xzf "$tmp/$asset" -C "$tmp"
cp "$tmp/lib/$lib_name" "$OUT_DIR/lib/"
cp "$tmp/LICENSE" "$OUT_DIR/LICENSE"

echo "fetch-pdfium: installed $OUT_DIR/lib/$lib_name"
