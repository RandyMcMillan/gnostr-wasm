#!/usr/bin/env bash

set -euo pipefail

##lsof -nP -iTCP:8080 -sTCP:LISTEN

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

root="$(repo_root)"
cd "$root"

ensure_wasm_target
ensure_cargo_bundle
ensure_trunk

trunk build --release --dist dist
cargo bundle --bin gnostr-wasm --release --features wry-app

printf 'Built %s\n' "target/release/bundle/"
printf 'Built %s\n' "dist/"
