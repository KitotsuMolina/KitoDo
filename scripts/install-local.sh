#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"

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

echo "[kitodo] Instalando dependencias frontend..."
pnpm install

echo "[kitodo] Compilando frontend..."
pnpm build

echo "[kitodo] Compilando binarios release..."
cargo build --release --manifest-path src-tauri/Cargo.toml --bin kitodo --bin kitodo-cli

echo "[kitodo] Instalando binarios en ${BIN_DIR}..."
install -d "${BIN_DIR}"
install -m755 src-tauri/target/release/kitodo "${BIN_DIR}/kitodo"
install -m755 src-tauri/target/release/kitodo-cli "${BIN_DIR}/kitodo-cli"

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
  echo "[kitodo] Aviso: ${BIN_DIR} no está en PATH."
  echo "[kitodo] Añádelo a tu shell rc (por ejemplo ~/.bashrc):"
  echo "export PATH=\"${BIN_DIR}:\$PATH\""
fi

echo "[kitodo] Instalación local completada."
echo "[kitodo] Ejecuta: kitodo"
