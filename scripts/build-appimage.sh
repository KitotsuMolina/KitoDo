#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYNC_ICON_SCRIPT="${ROOT_DIR}/scripts/sync-tauri-icon.sh"

echo "[kitodo] Root: ${ROOT_DIR}"
cd "${ROOT_DIR}"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "[kitodo] Error: pnpm no está instalado." >&2
  exit 1
fi

if [[ -x "${SYNC_ICON_SCRIPT}" ]]; then
  echo "[kitodo] Sincronizando icono Tauri..."
  "${SYNC_ICON_SCRIPT}"
fi

echo "[kitodo] Compilando AppImage..."
pnpm run tauri:build

APPIMAGE_PATH="$(find "${ROOT_DIR}/src-tauri/target/release/bundle/appimage" -maxdepth 1 -type f -name '*.AppImage' | sort | tail -n 1)"

if [[ -z "${APPIMAGE_PATH}" ]]; then
  echo "[kitodo] Error: no se encontró AppImage generado." >&2
  exit 1
fi

echo "[kitodo] AppImage listo: ${APPIMAGE_PATH}"
