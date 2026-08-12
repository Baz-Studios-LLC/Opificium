#!/usr/bin/env bash
# Assemble Opificium.app around an already-built release binary.
# Usage: packaging/macos-app.sh <path-to-binary> <version> [out-dir]
#
# The bench loads real files from assets/ (its fonts), so the bundle carries
# that folder BESIDE the binary - Bevy resolves the asset root from the
# executable's own directory.
#
# It carries no game's data. A project is a folder somewhere else entirely,
# and the bench asks for one the first time it is run.
set -euo pipefail
BIN="${1:?usage: macos-app.sh <binary> <version> [out-dir]}"
VERSION="${2:?need a version}"
OUT="${3:-dist}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"

APP="$OUT/Opificium.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

# The executable's name must match CFBundleExecutable in Info.plist, or macOS
# refuses to launch the bundle.
cp "$BIN" "$APP/Contents/MacOS/opificium"
chmod +x "$APP/Contents/MacOS/opificium"
strip "$APP/Contents/MacOS/opificium" 2>/dev/null || true

cp -R "$ROOT/assets" "$APP/Contents/MacOS/assets"

# The icon, which Info.plist names as CFBundleIconFile. It goes in Resources,
# which is where macOS looks for it and nowhere else - a .icns beside the binary
# is a .icns nobody sees.
cp "$HERE/Opificium.icns" "$APP/Contents/Resources/Opificium.icns"

sed "s/__VERSION__/$VERSION/g" "$HERE/Info.plist" > "$APP/Contents/Info.plist"

# Ad-hoc sign so macOS runs it without a "damaged" error; the launcher also
# strips the download quarantine on install.
codesign --force --deep --sign - "$APP" 2>/dev/null || true

echo "built $APP ($(du -sh "$APP" | cut -f1))"
