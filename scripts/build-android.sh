#!/usr/bin/env bash
# Build an Android APK with macroquad's cargo-quad-apk Docker image (plus a
# newer Rust, see scripts/android/Dockerfile), and
# install it if a device is connected.
# Usage: scripts/build-android.sh [--no-install]
# With several adb devices, pick one: ANDROID_SERIAL=<serial> scripts/build-android.sh
set -euo pipefail
cd "$(dirname "$0")/.."

# cargo-quad-apk's own lock-file parser predates format v4; v3 says the same
# thing for this project (no git dependencies).
perl -pi -e 's/^version = 4$/version = 3/' Cargo.lock

image=ragdoll-cricket-apk-builder
docker build --platform linux/amd64 -q -t "$image" scripts/android >/dev/null

docker run --rm --platform linux/amd64 \
    -v "$PWD":/root/src -w /root/src \
    -v ragdoll-cricket-cargo-registry:/usr/local/cargo/registry \
    -v ragdoll-cricket-android-keys:/root/.android \
    "$image" cargo quad-apk build --release

apk=target/android-artifacts/release/apk/ragdoll_cricket.apk
echo "Built $apk"

if [[ "${1:-}" != "--no-install" ]] && adb get-state >/dev/null 2>&1; then
    adb install -r "$apk"
    adb shell monkey -p com.atulpatildbz.ragdoll_cricket 1 >/dev/null
fi
