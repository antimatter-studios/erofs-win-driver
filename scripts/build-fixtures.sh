#!/usr/bin/env bash
# build-fixtures.sh OUT_DIR -- build the EROFS images test-matrix.json
# names, into OUT_DIR, with the reference mkfs.erofs (erofs-utils) (#6).
#
# Not with the sibling crate's own writer: an image and a driver that are
# both ours agree with each other whatever the format says, so the matrix
# would prove only that they are consistent. mkfs.erofs is the tool every
# real EROFS image is made with. It is a Linux tool, so CI runs this on
# ubuntu-latest and hands the images to the Windows job.
#
# Outputs:
#   OUT_DIR/erofs-basic.img  -- test.txt, subdir/nested.txt; label rsmkfs
#
# The file contents are fixed: test-matrix.json pins their sizes and
# sha256 sums.
set -euo pipefail

[ $# -eq 1 ] || { echo "build-fixtures.sh: usage: build-fixtures.sh OUT_DIR" >&2; exit 2; }
out="$1"
command -v mkfs.erofs >/dev/null || {
    echo "build-fixtures.sh: mkfs.erofs not found. Install erofs-utils (Linux)." >&2
    exit 1
}
mkdir -p "$out"

src=$(mktemp -d)
trap 'rm -rf "$src"' EXIT
printf 'hello from erofs\n' > "$src/test.txt"
mkdir -p "$src/subdir"
printf 'nested\n' > "$src/subdir/nested.txt"

# -b 4096: test-matrix.json expects a 4 KiB block size whatever the page
# size of the machine building the image.
mkfs.erofs -b 4096 -L rsmkfs "$out/erofs-basic.img" "$src" >/dev/null
echo "built $out/erofs-basic.img"
