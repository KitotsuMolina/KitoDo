#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ICON_SRC="${ROOT_DIR}/assets/kitodo-icon.png"
TAURI_ICON_DIR="${ROOT_DIR}/src-tauri/icons"
TAURI_ICON_TARGET="${TAURI_ICON_DIR}/icon.png"

echo "[kitodo] Root: ${ROOT_DIR}"
cd "${ROOT_DIR}"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "[kitodo] Error: pnpm no está instalado." >&2
  exit 1
fi

if [[ -f "${ICON_SRC}" ]]; then
  install -d "${TAURI_ICON_DIR}"
  if command -v magick >/dev/null 2>&1; then
    echo "[kitodo] Sincronizando icono Tauri desde assets..."
    magick "${ICON_SRC}" -resize 512x512 "${TAURI_ICON_TARGET}"
  else
    cp "${ICON_SRC}" "${TAURI_ICON_TARGET}"
  fi
fi

echo "[kitodo] Compilando AppImage..."
pnpm run tauri:build

APPIMAGE_PATH="$(find "${ROOT_DIR}/src-tauri/target/release/bundle/appimage" -maxdepth 1 -type f -name '*.AppImage' | sort | tail -n 1)"

if [[ -z "${APPIMAGE_PATH}" ]]; then
  echo "[kitodo] Error: no se encontró AppImage generado." >&2
  exit 1
fi

echo "[kitodo] AppImage listo: ${APPIMAGE_PATH}"
