#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
APPIMAGE_FILE="${HOME}/.local/share/kitodo/KitoDo.AppImage"
ICON_FILE="${HOME}/.local/share/icons/hicolor/512x512/apps/kitodo.png"
PIXMAP_FILE="${HOME}/.local/share/pixmaps/kitodo.png"
ICON_64="${HOME}/.local/share/icons/hicolor/64x64/apps/kitodo.png"
ICON_128="${HOME}/.local/share/icons/hicolor/128x128/apps/kitodo.png"
ICON_256="${HOME}/.local/share/icons/hicolor/256x256/apps/kitodo.png"
REMOVED=0

if [[ -f "${BIN_DIR}/kitodo" ]]; then
  rm -f "${BIN_DIR}/kitodo"
  echo "[kitodo] Eliminado: ${BIN_DIR}/kitodo"
  REMOVED=1
fi

if [[ -f "${BIN_DIR}/kitodo-cli" ]]; then
  rm -f "${BIN_DIR}/kitodo-cli"
  echo "[kitodo] Eliminado: ${BIN_DIR}/kitodo-cli"
  REMOVED=1
fi

if [[ -f "${APPIMAGE_FILE}" ]]; then
  rm -f "${APPIMAGE_FILE}"
  echo "[kitodo] Eliminado: ${APPIMAGE_FILE}"
  REMOVED=1
fi

if [[ -f "${APP_DIR}/kitodo.desktop" ]]; then
  rm -f "${APP_DIR}/kitodo.desktop"
  echo "[kitodo] Eliminado: ${APP_DIR}/kitodo.desktop"
  REMOVED=1
fi

if [[ -f "${ICON_FILE}" ]]; then
  rm -f "${ICON_FILE}"
  echo "[kitodo] Eliminado: ${ICON_FILE}"
  REMOVED=1
fi

if [[ -f "${PIXMAP_FILE}" ]]; then
  rm -f "${PIXMAP_FILE}"
  echo "[kitodo] Eliminado: ${PIXMAP_FILE}"
  REMOVED=1
fi

if [[ -f "${ICON_64}" ]]; then
  rm -f "${ICON_64}"
  echo "[kitodo] Eliminado: ${ICON_64}"
  REMOVED=1
fi

if [[ -f "${ICON_128}" ]]; then
  rm -f "${ICON_128}"
  echo "[kitodo] Eliminado: ${ICON_128}"
  REMOVED=1
fi

if [[ -f "${ICON_256}" ]]; then
  rm -f "${ICON_256}"
  echo "[kitodo] Eliminado: ${ICON_256}"
  REMOVED=1
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APP_DIR}" >/dev/null 2>&1 || true
fi

if [[ "${REMOVED}" -eq 0 ]]; then
  echo "[kitodo] No se encontraron binarios instalados en ${BIN_DIR}."
else
  echo "[kitodo] Desinstalación local completada."
fi
