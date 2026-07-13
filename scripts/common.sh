#!/usr/bin/env bash

set -euo pipefail

repo_root() {
  cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd
}

ensure_wasm_target() {
  if rustup target list --installed | grep -qx 'wasm32-unknown-unknown'; then
    return
  fi

  rustup target add wasm32-unknown-unknown
}

ensure_trunk() {
  if command -v trunk >/dev/null 2>&1; then
    return
  fi

  cargo install trunk --locked
}

host_triple() {
  rustc -vV | awk '/^host: / { print $2; exit }'
}

