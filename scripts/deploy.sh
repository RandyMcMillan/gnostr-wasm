#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

usage() {
  cat <<'EOF'
Usage: scripts/deploy.sh [--out-dir DIR] [--tag TAG] [--skip-build]

Builds the desktop wrapper and wasm app, then stages release artifacts under
the chosen output directory. If --tag is provided, the archives are uploaded to
the matching GitHub release with the gh CLI.
EOF
}

out_dir="dist/releases"
tag=""
skip_build=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      out_dir="$2"
      shift 2
      ;;
    --tag)
      tag="$2"
      shift 2
      ;;
    --skip-build)
      skip_build=true
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

root="$(repo_root)"
cd "$root"
mkdir -p "$out_dir"

if [[ "$skip_build" == false ]]; then
  "$root/scripts/build.sh"
fi

host="$(host_triple)"
release_dir="$out_dir/$host"
cloud_bin="target/release/gnostr-cloud"
wasm_dist="dist"

if [[ ! -x "$cloud_bin" ]]; then
  echo "Missing $cloud_bin; run scripts/build.sh first or omit --skip-build." >&2
  exit 1
fi

if [[ ! -d "$wasm_dist" ]]; then
  echo "Missing $wasm_dist; run scripts/build.sh first or omit --skip-build." >&2
  exit 1
fi

rm -rf "$release_dir"
mkdir -p "$release_dir/gnostr-wasm"

cp "$cloud_bin" "$release_dir/gnostr-cloud"
cp -R "$wasm_dist"/. "$release_dir/gnostr-wasm/"

cloud_archive="$out_dir/gnostr-cloud-$host.tar.gz"
wasm_archive="$out_dir/gnostr-wasm-$host.tar.gz"

tar -C "$release_dir" -czf "$cloud_archive" gnostr-cloud
tar -C "$release_dir/gnostr-wasm" -czf "$wasm_archive" .

printf 'Staged %s\n' "$release_dir"
printf 'Packed %s\n' "$cloud_archive"
printf 'Packed %s\n' "$wasm_archive"

if [[ -n "$tag" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    echo "gh is required to upload release artifacts." >&2
    exit 1
  fi

  if ! gh release view "$tag" >/dev/null 2>&1; then
    gh release create "$tag" --title "$tag" --notes "Automated release for $tag"
  fi

  gh release upload "$tag" "$cloud_archive" "$wasm_archive" --clobber
fi
