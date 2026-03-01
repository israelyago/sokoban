#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APPIMAGE_DIR="$ROOT_DIR/packaging/appimage"
APPIMAGE_WORK_DIR="${APPIMAGE_WORK_DIR:-$APPIMAGE_DIR/.work}"
APPDIR="$APPIMAGE_WORK_DIR/AppDir"
BUILD_DIR="$APPIMAGE_WORK_DIR/build"
DIST_DIR="$ROOT_DIR/dist"
BIN_PATH="$ROOT_DIR/target/release/sokoban"
ICON_SOURCE="${ICON_SOURCE:-$ROOT_DIR/assets/box.png}"
LINUXDEPLOY_CANDIDATE="$ROOT_DIR/tools/linuxdeploy-x86_64.AppImage"
APPIMAGE_BUILDER_RECIPE="$APPIMAGE_DIR/AppImageBuilder.yml"

resolve_linuxdeploy() {
    if command -v linuxdeploy >/dev/null 2>&1; then
        command -v linuxdeploy
        return 0
    fi

    if [ -x "$LINUXDEPLOY_CANDIDATE" ]; then
        printf "%s\n" "$LINUXDEPLOY_CANDIDATE"
        return 0
    fi

    echo "linuxdeploy is required but was not found." >&2
    echo "Install linuxdeploy or place it at:" >&2
    echo "  $LINUXDEPLOY_CANDIDATE" >&2
    exit 1
}

has_appimage_builder() {
    command -v appimage-builder >/dev/null 2>&1
}

run_appimage_builder() {
    local recipe="$1"
    local appdir="$2"
    local build_dir="$3"
    if has_appimage_builder; then
        if APPIMAGE_EXTRACT_AND_RUN=1 appimage-builder \
            --recipe "$recipe" \
            --appdir "$appdir" \
            --build-dir "$build_dir" \
            --skip-build \
            --skip-tests; then
            return 0
        fi

        return 1
    fi
    return 1
}

run_linuxdeploy() {
    local linuxdeploy_bin="$1"
    shift
    if [[ "$linuxdeploy_bin" == *.AppImage ]]; then
        APPIMAGE_EXTRACT_AND_RUN=1 "$linuxdeploy_bin" "$@"
    else
        "$linuxdeploy_bin" "$@"
    fi
}

generate_icon() {
    local size="$1"
    local output_path="$2"

    if command -v ffmpeg >/dev/null 2>&1; then
        ffmpeg -y -loglevel error -i "$ICON_SOURCE" -vf "scale=${size}:${size}" "$output_path"
    else
        install -m 644 "$ICON_SOURCE" "$output_path"
    fi
}

echo "Building release binary..."
cargo build --release

if [ ! -x "$BIN_PATH" ]; then
    echo "Release binary not found at $BIN_PATH" >&2
    exit 1
fi
if [ ! -f "$ICON_SOURCE" ]; then
    echo "Icon source not found at $ICON_SOURCE" >&2
    exit 1
fi

echo "Preparing AppImage workspace at $APPIMAGE_WORK_DIR..."
rm -rf "$APPDIR"
rm -rf "$BUILD_DIR"
mkdir -p \
    "$APPDIR/usr/bin" \
    "$APPDIR/usr/share/sokoban/levels" \
    "$APPDIR/usr/share/applications"

install -m 755 "$BIN_PATH" "$APPDIR/usr/bin/sokoban"
cp -a "$ROOT_DIR/assets" "$APPDIR/usr/share/sokoban/"
install -m 644 "$ROOT_DIR/levels/default.txt" "$APPDIR/usr/share/sokoban/levels/default.txt"

install -m 755 "$APPIMAGE_DIR/AppRun" "$APPDIR/AppRun"
install -m 644 "$APPIMAGE_DIR/sokoban.desktop" "$APPDIR/usr/share/applications/sokoban.desktop"

for size in 64 128 256; do
    icon_dir="$APPDIR/usr/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$icon_dir"
    generate_icon "$size" "$icon_dir/sokoban.png"
done

install -m 644 "$APPDIR/usr/share/icons/hicolor/256x256/apps/sokoban.png" "$APPDIR/sokoban.png"
ln -sfn sokoban.png "$APPDIR/.DirIcon"
ln -sfn usr/share/applications/sokoban.desktop "$APPDIR/sokoban.desktop"

mkdir -p "$DIST_DIR"
export ARCH="${ARCH:-x86_64}"
pushd "$DIST_DIR" >/dev/null

echo "Packaging AppImage..."
if has_appimage_builder; then
    if ! run_appimage_builder "$APPIMAGE_BUILDER_RECIPE" "$APPDIR" "$BUILD_DIR"; then
        echo "appimage-builder failed." >&2
        exit 1
    fi
else
    LINUXDEPLOY_BIN="$(resolve_linuxdeploy)"
    run_linuxdeploy \
        "$LINUXDEPLOY_BIN" \
        --appdir "$APPDIR" \
        --executable "$APPDIR/usr/bin/sokoban" \
        --desktop-file "$APPDIR/usr/share/applications/sokoban.desktop" \
        --icon-file "$APPDIR/sokoban.png" \
        --custom-apprun "$APPIMAGE_DIR/AppRun" \
        --output appimage
fi
popd >/dev/null

shopt -s nullglob
artifacts=("$DIST_DIR"/*.AppImage)
shopt -u nullglob
if [ "${#artifacts[@]}" -eq 0 ]; then
    echo "No AppImage artifact was generated in $DIST_DIR" >&2
    exit 1
fi

echo "Done. Generated AppImage artifacts:"
printf '%s\n' "${artifacts[@]}"
