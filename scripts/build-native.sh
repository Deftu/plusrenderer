#!/bin/sh
# build-native.sh <out> [rust-target...]  (default: host) -> <out>/natives/plusrenderer/<os>-<arch>/<lib>
# Needs rust-src on the pinned nightly, which rebuilds std without its backtrace printer (a third of the library).
set -e
cd "$(dirname "$0")/.."
out=${1:?usage: build-native.sh <out-dir> [rust-target...]}
shift
nightly=nightly-2026-07-21
[ $# -eq 0 ] && set -- "$(rustc +$nightly -vV | sed -n 's/^host: //p')"

for target in "$@"; do
    case $target in
        aarch64-apple-darwin) dir=macos-aarch64 lib=libplusrenderer_ffi.dylib ;;
        x86_64-apple-darwin) dir=macos-x86_64 lib=libplusrenderer_ffi.dylib ;;
        x86_64-unknown-linux-gnu) dir=linux-x86_64 lib=libplusrenderer_ffi.so ;;
        aarch64-unknown-linux-gnu) dir=linux-aarch64 lib=libplusrenderer_ffi.so ;;
        x86_64-pc-windows-msvc) dir=windows-x86_64 lib=plusrenderer_ffi.dll ;;
        aarch64-pc-windows-msvc) dir=windows-aarch64 lib=plusrenderer_ffi.dll ;;
        *) echo "no natives/ layout for $target" >&2; exit 1 ;;
    esac
    cargo +$nightly build -p plusrenderer-ffi --profile native --target "$target" \
        -Zbuild-std=std,panic_unwind -Zbuild-std-features=panic-unwind
    mkdir -p "$out/natives/plusrenderer/$dir"
    cp "target/$target/native/$lib" "$out/natives/plusrenderer/$dir/$lib"
    ls -lh "$out/natives/plusrenderer/$dir/$lib" | awk '{ print $9, $5 }'
done
