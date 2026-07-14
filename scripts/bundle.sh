#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

usage() {
  cat <<'EOF'
Usage: scripts/bundle.sh [--build] [--run] [--test]

Builds the macOS app bundle for gnostr-wasm with the wry-app feature.
--run opens the bundled macOS app.
--test opens the bundled macOS app, curls the bundled index.html, and verifies the HTML.
EOF
}

build=false
run=false
test=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --build)
      build=true
      shift
      ;;
    --run)
      run=true
      shift
      ;;
    --test)
      test=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "$build" == false && "$run" == false && "$test" == false ]]; then
  usage
  exit 1
fi

root="$(repo_root)"
cd "$root"

bundle_root="target/release/bundle"
bundle_name="gnostr-wasm"
bundle_app_path() {
  find "$bundle_root" -type d -name "${bundle_name}.app" -print -quit
}

build_bundle() {
  ensure_wasm_target
  ensure_cargo_bundle
  ensure_trunk

  trunk build --release --dist dist
  cargo bundle --bin gnostr-wasm --release --features wry-app
}

ensure_bundle() {
  if ! bundle_app_path >/dev/null 2>&1; then
    build_bundle
  fi
}

run_bundle() {
  local app_path
  app_path="$(bundle_app_path)"
  if [[ -z "$app_path" ]]; then
    echo "Missing bundled app; run scripts/bundle.sh --build first." >&2
    exit 1
  fi

  open "$app_path"
}

test_bundle() {
  local app_path
  local html
  local app_pid=""
  local index_url

  app_path="$(bundle_app_path)"
  if [[ -z "$app_path" ]]; then
    echo "Missing bundled app; run scripts/bundle.sh --build first." >&2
    exit 1
  fi

  cleanup() {
    if [[ -n "$app_pid" ]] && kill -0 "$app_pid" >/dev/null 2>&1; then
      kill "$app_pid" >/dev/null 2>&1 || true
      wait "$app_pid" >/dev/null 2>&1 || true
    fi
  }

  trap cleanup EXIT

  open "$app_path"

  for _ in $(seq 1 60); do
    app_pid="$(pgrep -n -f "${bundle_name}.app/Contents/MacOS/${bundle_name}" || true)"
    if [[ -n "$app_pid" ]]; then
      break
    fi
    sleep 1
  done

  if [[ -z "$app_pid" ]]; then
    echo "Timed out waiting for bundled app process to start." >&2
    exit 1
  fi

  index_url="file://${app_path}/Contents/Resources/dist/index.html"
  html="$(curl -fsS "$index_url")"
  printf '%s' "$html" | grep -F '<title>Ratzilla Demo</title>'
  printf '%s' "$html" | grep -F 'data-bin="gnostr-wasm"'

  cleanup
  trap - EXIT
  printf 'Verified %s\n' "$index_url"
}

if [[ "$build" == true ]]; then
  build_bundle
fi

if [[ "$run" == true ]]; then
  ensure_bundle
  run_bundle
fi

if [[ "$test" == true ]]; then
  ensure_bundle
  test_bundle
fi
