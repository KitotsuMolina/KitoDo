#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
APPIMAGE_DIR="${HOME}/.local/share/kitodo"
APPIMAGE_NAME="KitoDo.AppImage"
APPIMAGE_TARGET="${APPIMAGE_DIR}/${APPIMAGE_NAME}"
ICON_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"
PIXMAP_DIR="${HOME}/.local/share/pixmaps"
ICON_SRC="${ROOT_DIR}/assets/kitodo-icon.png"
PREPARE_SCRIPT="${ROOT_DIR}/packaging/flatpak/prepare.sh"
BUILD_SCRIPT="${ROOT_DIR}/scripts/build-appimage.sh"

echo "[kitodo] Root: ${ROOT_DIR}"
cd "${ROOT_DIR}"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "[kitodo] Error: pnpm no está instalado." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "[kitodo] Error: cargo no está instalado." >&2
  exit 1
fi

"${BUILD_SCRIPT}"

APPIMAGE_SOURCE="$(find "${ROOT_DIR}/src-tauri/target/release/bundle/appimage" -maxdepth 1 -type f -name '*.AppImage' | sort | tail -n 1)"

if [[ -z "${APPIMAGE_SOURCE}" ]]; then
  echo "[kitodo] Error: no se encontró AppImage para instalar." >&2
  exit 1
fi

echo "[kitodo] Instalando AppImage en ${APPIMAGE_DIR}..."
install -d "${APPIMAGE_DIR}" "${BIN_DIR}"
install -m755 "${APPIMAGE_SOURCE}" "${APPIMAGE_TARGET}"

cat > "${BIN_DIR}/kitodo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
  export GDK_BACKEND=x11
fi

exec "${HOME}/.local/share/kitodo/KitoDo.AppImage" "$@"
EOF
chmod +x "${BIN_DIR}/kitodo"

cat > "${BIN_DIR}/kitodo-cli" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
exec "${HOME}/.local/bin/kitodo" "$@"
EOF
chmod +x "${BIN_DIR}/kitodo-cli"

if [[ -f "${ICON_SRC}" ]]; then
  echo "[kitodo] Instalando icono..."
  install -d "${ICON_DIR}" "${PIXMAP_DIR}"
  install -m644 "${ICON_SRC}" "${ICON_DIR}/kitodo.png"
  install -m644 "${ICON_SRC}" "${PIXMAP_DIR}/kitodo.png"

  if [[ -x "${PREPARE_SCRIPT}" ]] && command -v magick >/dev/null 2>&1; then
    echo "[kitodo] Generando iconos 64/128/256 con prepare.sh..."
    "${PREPARE_SCRIPT}"

    install -d \
      "${HOME}/.local/share/icons/hicolor/64x64/apps" \
      "${HOME}/.local/share/icons/hicolor/128x128/apps" \
      "${HOME}/.local/share/icons/hicolor/256x256/apps"

    install -m644 "${ROOT_DIR}/packaging/flatpak/icons/hicolor/64x64/apps/io.github.KitotsuMolina.KitoDo.png" \
      "${HOME}/.local/share/icons/hicolor/64x64/apps/kitodo.png"
    install -m644 "${ROOT_DIR}/packaging/flatpak/icons/hicolor/128x128/apps/io.github.KitotsuMolina.KitoDo.png" \
      "${HOME}/.local/share/icons/hicolor/128x128/apps/kitodo.png"
    install -m644 "${ROOT_DIR}/packaging/flatpak/icons/hicolor/256x256/apps/io.github.KitotsuMolina.KitoDo.png" \
      "${HOME}/.local/share/icons/hicolor/256x256/apps/kitodo.png"
  fi
else
  echo "[kitodo] Aviso: no se encontró icono en ${ICON_SRC}"
fi

echo "[kitodo] Creando desktop entry..."
install -d "${APP_DIR}"
cat > "${APP_DIR}/kitodo.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=KitoDo
Comment=Fast launcher-style tasks app
Exec=kitodo
Icon=kitodo
Terminal=false
Categories=Utility;Office;Productivity;
StartupNotify=true
EOF

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APP_DIR}" >/dev/null 2>&1 || true
fi

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
  echo "[kitodo] Aviso: ${BIN_DIR} no está en PATH."
  echo "[kitodo] Añádelo a tu shell rc (por ejemplo ~/.bashrc):"
  echo "export PATH=\"${BIN_DIR}:\$PATH\""
fi

echo "[kitodo] Instalación local completada."
echo "[kitodo] Ejecuta: kitodo"
