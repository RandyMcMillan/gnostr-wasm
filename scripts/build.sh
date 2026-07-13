#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

root="$(repo_root)"
cd "$root"

ensure_wasm_target
ensure_trunk

cargo build --release --bin gnostr-cloud --features wry-app
trunk build --release --dist dist

printf 'Built %s\n' "target/release/gnostr-cloud"
printf 'Built %s\n' "dist/"

