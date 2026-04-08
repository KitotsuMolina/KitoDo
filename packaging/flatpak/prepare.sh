#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC_ICON="${ROOT_DIR}/assets/kitodo-icon.png"
APP_ID="io.github.KitotsuMolina.KitoDo"
OUT_ROOT="${ROOT_DIR}/packaging/flatpak/icons/hicolor"

if ! command -v magick >/dev/null 2>&1; then
  echo "[flatpak/prepare] Error: ImageMagick ('magick') no está instalado." >&2
  exit 1
fi

if [[ ! -f "${SRC_ICON}" ]]; then
  echo "[flatpak/prepare] Error: no se encontró ${SRC_ICON}" >&2
  exit 1
fi

for size in 64 128 256; do
  out_dir="${OUT_ROOT}/${size}x${size}/apps"
  out_file="${out_dir}/${APP_ID}.png"
  mkdir -p "${out_dir}"

  magick "${SRC_ICON}" \
    -background none \
    -gravity center \
    -resize "${size}x${size}" \
    -extent "${size}x${size}" \
    -strip \
    "${out_file}"

  echo "[flatpak/prepare] Generado ${out_file}"
done

echo "[flatpak/prepare] OK"
