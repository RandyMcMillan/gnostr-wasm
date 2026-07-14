#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

root="$(repo_root)"
cd "$root"

ensure_wasm_target
ensure_cargo_bundle
ensure_trunk

cargo bundle --bin gnostr-wasm --release --features wry-app
cargo bundle --bin gnostr-cloud --release --features wry-app
trunk build --release --dist dist

printf 'Built %s\n' "target/release/bundle/"
printf 'Built %s\n' "dist/"
