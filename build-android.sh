#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export JAVA_HOME="${JAVA_HOME:-$HOME/.local/jdk}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export NDK_HOME="${NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
export PATH="$HOME/.cargo/bin:$HOME/.local/node/bin:$JAVA_HOME/bin:$ANDROID_HOME/build-tools/35.0.0:$ANDROID_HOME/platform-tools:$PATH"

cd "$ROOT_DIR/gui"
RUSTFLAGS="--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}/= --remap-path-prefix=$HOME/=/" \
  npx tauri android build --apk --target aarch64

OUT_DIR="$ROOT_DIR/Release/android"
mkdir -p "$OUT_DIR"
FINAL_APK="$OUT_DIR/DJI-Mic-Control-1.1.0-android-arm64.apk"

SIGNED_CANDIDATE="$ROOT_DIR/gui/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk"
UNSIGNED_CANDIDATE="$ROOT_DIR/gui/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
KEYSTORE="$ROOT_DIR/gui/src-tauri/gen/android/app/release-key.jks"

if [ -f "$SIGNED_CANDIDATE" ]; then
  cp "$SIGNED_CANDIDATE" "$FINAL_APK"
elif [ -f "$UNSIGNED_CANDIDATE" ]; then
  ALIGNED_TMP="$(mktemp --suffix=.apk)"
  zipalign -f -p 4 "$UNSIGNED_CANDIDATE" "$ALIGNED_TMP"
  apksigner sign \
    --ks "$KEYSTORE" \
    --ks-key-alias djimic \
    --ks-pass pass:djimic123 \
    --key-pass pass:djimic123 \
    --out "$FINAL_APK" \
    "$ALIGNED_TMP"
  rm -f "$ALIGNED_TMP"
fi

echo "Built and signed Android APK: $FINAL_APK"
