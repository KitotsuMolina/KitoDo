#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ICON_SRC="${ROOT_DIR}/assets/kitodo-icon.png"
TAURI_ICON_DIR="${ROOT_DIR}/src-tauri/icons"
TAURI_ICON_TARGET="${TAURI_ICON_DIR}/icon.png"

if [[ ! -f "${ICON_SRC}" ]]; then
  echo "[kitodo] Error: no se encontró ${ICON_SRC}" >&2
  exit 1
fi

install -d "${TAURI_ICON_DIR}"

if command -v magick >/dev/null 2>&1; then
  magick "${ICON_SRC}" \
    -resize 512x512 \
    -background none \
    -gravity center \
    -extent 512x512 \
    "${TAURI_ICON_TARGET}"
else
  cp "${ICON_SRC}" "${TAURI_ICON_TARGET}"
fi

echo "[kitodo] Icono Tauri sincronizado: ${TAURI_ICON_TARGET}"
