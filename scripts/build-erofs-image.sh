#!/usr/bin/env bash
# build-erofs-image.sh RUN_ID IMAGE -- the matrix's image op (#6): put a fresh
# copy of IMAGE in $HOST_IMAGE_DIR/RUN_ID/ for one scenario.
#
# The images are scripts/build-fixtures.sh's, made by the reference
# mkfs.erofs. That is a Linux tool, so a Windows runner cannot make them;
# CI builds them on ubuntu-latest and points EROFS_PREBUILT_IMAGES at them.
# Without it, a Linux shell with mkfs.erofs builds them here; anywhere
# else this fails and says what to do, rather than running the matrix
# against nothing.
set -euo pipefail

[ $# -eq 2 ] || { echo "build-erofs-image.sh: usage: build-erofs-image.sh RUN_ID IMAGE" >&2; exit 2; }
run_id="$1"
image="$2"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dest="${HOST_IMAGE_DIR:?build-erofs-image.sh: HOST_IMAGE_DIR is not set; run through scripts/run-matrix.sh}/$run_id"

src_dir="${EROFS_PREBUILT_IMAGES:-}"
if [ -z "$src_dir" ]; then
    if command -v mkfs.erofs >/dev/null; then
        src_dir="$repo_root/.fixtures"
        [ -f "$src_dir/$image" ] || bash "$repo_root/scripts/build-fixtures.sh" "$src_dir"
    else
        echo "build-erofs-image.sh: no EROFS_PREBUILT_IMAGES, and the images need mkfs.erofs (Linux)." >&2
        echo "  Build them on Linux with: scripts/build-fixtures.sh DIR, then set EROFS_PREBUILT_IMAGES=DIR." >&2
        exit 1
    fi
fi
[ -f "$src_dir/$image" ] || { echo "build-erofs-image.sh: $src_dir has no $image" >&2; exit 1; }
mkdir -p "$dest"
cp "$src_dir/$image" "$dest/$image"
echo "[build-erofs-image] $image -> $dest"
