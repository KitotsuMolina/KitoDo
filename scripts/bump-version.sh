#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Uso: ./scripts/bump-version.sh <version>" >&2
  echo "Ejemplo: ./scripts/bump-version.sh 0.5.0" >&2
  exit 1
fi

VERSION="$1"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Version invalida. Usa formato semver: X.Y.Z" >&2
  exit 1
fi

cd "${ROOT_DIR}"

sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" package.json
sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" src-tauri/Cargo.toml
sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" src-tauri/tauri.conf.json
sed -i "s/^# KitoDo v.*/# KitoDo v${VERSION}/" README.md

echo "[kitodo] Version actualizada a ${VERSION}"
echo "[kitodo] Siguiente paso sugerido:"
echo "git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json README.md"
echo "git commit -m \"chore: release v${VERSION}\""
echo "git tag v${VERSION}"
