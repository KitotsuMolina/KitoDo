#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${ROOT_DIR}"

pnpm run build:web
cargo build --manifest-path src-tauri/Cargo.toml --release --bin kitodo-server
pnpm exec electron-builder --linux AppImage --publish never
