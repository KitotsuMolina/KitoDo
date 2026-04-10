#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYNC_ICON_SCRIPT="${ROOT_DIR}/scripts/sync-tauri-icon.sh"
BUILD_OUTPUT_DIR="${ROOT_DIR}/src-tauri/target/release/bundle/appimage"
APPDIR="${ROOT_DIR}/AppDir"
APPIMAGE_NAME="KitoDo_amd64.AppImage"

echo "[kitodo] Root: ${ROOT_DIR}"
cd "${ROOT_DIR}"

for cmd in pnpm cargo curl; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "[kitodo] Error: falta dependencia requerida: ${cmd}" >&2
    exit 1
  fi
done

if [[ -f "${SYNC_ICON_SCRIPT}" ]]; then
  echo "[kitodo] Sincronizando icono Tauri..."
  bash "${SYNC_ICON_SCRIPT}"
fi

echo "[kitodo] Construyendo frontend..."
pnpm build

echo "[kitodo] Compilando binario release..."
cargo build --manifest-path src-tauri/Cargo.toml --release --bin kitodo

echo "[kitodo] Preparando AppDir..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/512x512/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/lib/webkit2gtk-4.1"
mkdir -p "${APPDIR}/usr/lib64"

install -m755 src-tauri/target/release/kitodo "${APPDIR}/usr/bin/kitodo"
cat > "${APPDIR}/usr/bin/kitodo-launcher" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export GDK_BACKEND=x11
SCRIPT_PATH="$(readlink -f "$0")"
SCRIPT_DIR="$(dirname "${SCRIPT_PATH}")"

if [[ -x "${SCRIPT_DIR}/kitodo" ]]; then
  APPDIR_ROOT="${SCRIPT_DIR}/.."
  KITODO_BIN="${SCRIPT_DIR}/kitodo"
elif [[ -x "${SCRIPT_DIR}/usr/bin/kitodo" ]]; then
  APPDIR_ROOT="${SCRIPT_DIR}"
  KITODO_BIN="${SCRIPT_DIR}/usr/bin/kitodo"
else
  echo "[kitodo] No se encontro kitodo dentro del AppImage" >&2
  exit 1
fi

APPDIR_ROOT="$(readlink -f "${APPDIR_ROOT}")"
export LD_LIBRARY_PATH="${APPDIR_ROOT}/lib:${APPDIR_ROOT}/usr/lib:${APPDIR_ROOT}/lib64:${APPDIR_ROOT}/usr/lib64:${LD_LIBRARY_PATH:-}"
exec "${KITODO_BIN}" "$@"
EOF
chmod +x "${APPDIR}/usr/bin/kitodo-launcher"
install -m644 packaging/appimage/io.github.KitotsuMolina.KitoDo.desktop "${APPDIR}/usr/share/applications/io.github.KitotsuMolina.KitoDo.desktop"
install -m644 src-tauri/icons/icon.png "${APPDIR}/usr/share/icons/hicolor/512x512/apps/io.github.KitotsuMolina.KitoDo.png"
install -m644 packaging/flatpak/icons/io.github.KitotsuMolina.KitoDo.svg "${APPDIR}/usr/share/icons/hicolor/scalable/apps/io.github.KitotsuMolina.KitoDo.svg"

for helper in WebKitNetworkProcess WebKitWebProcess WebKitGPUProcess; do
  if [[ -f "/usr/lib/webkit2gtk-4.1/${helper}" ]]; then
    install -m755 "/usr/lib/webkit2gtk-4.1/${helper}" "${APPDIR}/usr/lib/webkit2gtk-4.1/${helper}"
  fi
done

if [[ -d "/usr/lib/webkit2gtk-4.1/injected-bundle" ]]; then
  mkdir -p "${APPDIR}/usr/lib/webkit2gtk-4.1/injected-bundle"
  cp -a /usr/lib/webkit2gtk-4.1/injected-bundle/. "${APPDIR}/usr/lib/webkit2gtk-4.1/injected-bundle/"
fi

ln -sfn usr/lib "${APPDIR}/lib"
ln -sfn usr/lib64 "${APPDIR}/lib64"

echo "[kitodo] Descargando linuxdeploy..."
curl -L -o linuxdeploy-x86_64.AppImage https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy/linuxdeploy-x86_64.AppImage
curl -L -o linuxdeploy-plugin-gtk.sh https://raw.githubusercontent.com/tauri-apps/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
curl -L -o linuxdeploy-plugin-appimage-x86_64.AppImage https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage linuxdeploy-plugin-gtk.sh linuxdeploy-plugin-appimage-x86_64.AppImage

echo "[kitodo] Extrayendo linuxdeploy para evitar strip embebido incompatible..."
rm -rf linuxdeploy-root squashfs-root
APPIMAGE_EXTRACT_AND_RUN=1 ./linuxdeploy-x86_64.AppImage --appimage-extract >/dev/null
mv squashfs-root linuxdeploy-root
rm -f linuxdeploy-root/usr/bin/strip
cp linuxdeploy-plugin-gtk.sh linuxdeploy-root/
cp linuxdeploy-plugin-appimage-x86_64.AppImage linuxdeploy-root/

echo "[kitodo] Preparando runtime GTK/WebKit en AppDir..."
export ARCH=x86_64
export APPIMAGE_EXTRACT_AND_RUN=1
./linuxdeploy-root/AppRun \
  --appdir "${APPDIR}" \
  -e "${APPDIR}/usr/bin/kitodo" \
  -d packaging/appimage/io.github.KitotsuMolina.KitoDo.desktop \
  -i packaging/flatpak/icons/io.github.KitotsuMolina.KitoDo.svg \
  --plugin gtk

# El plugin GTK reescribe rutas internas de WebKitGTK a ././/lib/... .
# Restauramos las rutas esperadas y ajustamos RUNPATH para que los helpers
# puedan cargar libwebkit2gtk-4.1.so.0 desde ../.
find "${APPDIR}"/usr/lib* -name 'libwebkit*.so*' -type f -print0 | while IFS= read -r -d '' file; do
  perl -0pi -e 's#\./\.//lib/webkit2gtk-4\.1/injected-bundle/#/usr/lib/webkit2gtk-4.1/injected-bundle/#g; s#\./\.//lib/webkit2gtk-4\.1#/usr/lib/webkit2gtk-4.1#g' "$file"
done

PATCH_ELF="${ROOT_DIR}/linuxdeploy-root/usr/bin/patchelf"
if [[ -x "${PATCH_ELF}" ]]; then
  for helper in WebKitNetworkProcess WebKitWebProcess WebKitGPUProcess; do
    helper_path="${APPDIR}/usr/lib/webkit2gtk-4.1/${helper}"
    if [[ -f "${helper_path}" ]]; then
      "${PATCH_ELF}" --set-rpath '$ORIGIN:$ORIGIN/..' "${helper_path}"
    fi
  done

  injected_bundle="${APPDIR}/usr/lib/webkit2gtk-4.1/injected-bundle/libwebkit2gtkinjectedbundle.so"
  if [[ -f "${injected_bundle}" ]]; then
    "${PATCH_ELF}" --set-rpath '$ORIGIN:$ORIGIN/..:$ORIGIN/../..' "${injected_bundle}"
  fi
fi

echo "[kitodo] Empaquetando AppImage..."
APPIMAGETOOL="${ROOT_DIR}/linuxdeploy-root/plugins/linuxdeploy-plugin-appimage/usr/bin/appimagetool"
OUTPUT="${APPIMAGE_NAME}" "${APPIMAGETOOL}" "${APPDIR}" "${APPIMAGE_NAME}"

mkdir -p "${BUILD_OUTPUT_DIR}"
mv -f "${ROOT_DIR}/${APPIMAGE_NAME}" "${BUILD_OUTPUT_DIR}/${APPIMAGE_NAME}"

echo "[kitodo] AppImage listo: ${BUILD_OUTPUT_DIR}/${APPIMAGE_NAME}"
