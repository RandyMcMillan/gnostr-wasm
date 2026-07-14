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
bundle_root="target/release/bundle"
wasm_dist="dist"

cloud_artifact="$(bundle_artifact_path gnostr-cloud "$bundle_root")"
wasm_artifact="$(bundle_artifact_path gnostr-wasm "$bundle_root")"

if [[ -z "$cloud_artifact" ]]; then
  echo "Missing gnostr-cloud bundle in $bundle_root; run scripts/build.sh first or omit --skip-build." >&2
  exit 1
fi

if [[ -z "$wasm_artifact" ]]; then
  echo "Missing gnostr-wasm bundle in $bundle_root; run scripts/build.sh first or omit --skip-build." >&2
  exit 1
fi

if [[ ! -d "$wasm_dist" ]]; then
  echo "Missing $wasm_dist; run scripts/build.sh first or omit --skip-build." >&2
  exit 1
fi

rm -rf "$release_dir"
mkdir -p "$release_dir"

cloud_name="$(basename "$cloud_artifact")"
wasm_name="$(basename "$wasm_artifact")"

cp -R "$cloud_artifact" "$release_dir/$cloud_name"
cp -R "$wasm_artifact" "$release_dir/$wasm_name"
cp -R "$wasm_dist" "$release_dir/gnostr-wasm-web"

cloud_archive="$out_dir/gnostr-cloud-$host.tar.gz"
wasm_archive="$out_dir/gnostr-wasm-$host.tar.gz"
web_archive="$out_dir/gnostr-wasm-web-$host.tar.gz"

tar -C "$release_dir" -czf "$cloud_archive" "$cloud_name"
tar -C "$release_dir" -czf "$wasm_archive" "$wasm_name"
tar -C "$release_dir" -czf "$web_archive" gnostr-wasm-web

printf 'Staged %s\n' "$release_dir"
printf 'Packed %s\n' "$cloud_archive"
printf 'Packed %s\n' "$wasm_archive"
printf 'Packed %s\n' "$web_archive"

if [[ -n "$tag" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    echo "gh is required to upload release artifacts." >&2
    exit 1
  fi

  if ! gh release view "$tag" >/dev/null 2>&1; then
    gh release create "$tag" --title "$tag" --notes "Automated release for $tag"
  fi

  gh release upload "$tag" "$cloud_archive" "$wasm_archive" "$web_archive" --clobber
fi
