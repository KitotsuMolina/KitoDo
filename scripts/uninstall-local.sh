#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
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

if [[ "${REMOVED}" -eq 0 ]]; then
  echo "[kitodo] No se encontraron binarios instalados en ${BIN_DIR}."
else
  echo "[kitodo] Desinstalación local completada."
fi
