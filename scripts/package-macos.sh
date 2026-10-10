#!/usr/bin/env bash
# Build a universal (Apple Silicon + Intel) Ragdoll Cricket.app and zip it.
# Usage: scripts/package-macos.sh [out_dir]   (default: dist/)
set -euo pipefail
cd "$(dirname "$0")/.."

out="${1:-dist}"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
app="$out/Ragdoll Cricket.app"

for target in aarch64-apple-darwin x86_64-apple-darwin; do
    rustup target add "$target" >/dev/null
    cargo build --release --target "$target"
done

rm -rf "$app"
mkdir -p "$app/Contents/MacOS"
lipo -create -output "$app/Contents/MacOS/ragdoll-cricket" \
    target/aarch64-apple-darwin/release/ragdoll-cricket \
    target/x86_64-apple-darwin/release/ragdoll-cricket

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Ragdoll Cricket</string>
    <key>CFBundleDisplayName</key><string>Ragdoll Cricket</string>
    <key>CFBundleIdentifier</key><string>com.atulpatildbz.ragdoll-cricket</string>
    <key>CFBundleExecutable</key><string>ragdoll-cricket</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc signature (no Apple developer account needed).
codesign --force --deep --sign - "$app"

zip="$out/RagdollCricket-$version-macos-universal.zip"
rm -f "$zip"
ditto -c -k --keepParent "$app" "$zip"
shasum -a 256 "$zip" | tee "$zip.sha256"
