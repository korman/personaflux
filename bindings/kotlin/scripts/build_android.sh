#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
NATIVE="${ROOT}/target/personaflux-android"
JNI_LIBS="${ROOT}/bindings/kotlin/personaflux/src/main/jniLibs"
ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION:-26.3.11579264}"
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
if [[ -n "${ANDROID_SDK_ROOT}" && -d "${ANDROID_SDK_ROOT}/ndk/${ANDROID_NDK_VERSION}" ]]; then
  export ANDROID_NDK_HOME="${ANDROID_SDK_ROOT}/ndk/${ANDROID_NDK_VERSION}"
  export ANDROID_NDK_ROOT="${ANDROID_NDK_HOME}"
fi

rm -rf "${NATIVE}"
rm -rf "${JNI_LIBS}"
mkdir -p "${JNI_LIBS}/arm64-v8a" "${JNI_LIBS}/armeabi-v7a" "${JNI_LIBS}/x86_64"
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o "${NATIVE}" build -p personaflux-ffi --release
for abi in arm64-v8a armeabi-v7a x86_64; do
  if [[ -s "${NATIVE}/${abi}/libpersonaflux.so" ]]; then
    :
  elif [[ -s "${NATIVE}/jni/${abi}/libpersonaflux.so" ]]; then
    mkdir -p "${NATIVE}/${abi}"
    cp "${NATIVE}/jni/${abi}/libpersonaflux.so" "${NATIVE}/${abi}/libpersonaflux.so"
  else
    echo "cargo-ndk did not produce a library for ${abi}" >&2
    find "${NATIVE}" -maxdepth 3 -type f -print >&2 || true
    exit 1
  fi
done
cp "${NATIVE}/arm64-v8a/libpersonaflux.so" "${JNI_LIBS}/arm64-v8a/"
cp "${NATIVE}/armeabi-v7a/libpersonaflux.so" "${JNI_LIBS}/armeabi-v7a/"
cp "${NATIVE}/x86_64/libpersonaflux.so" "${JNI_LIBS}/x86_64/"
PERSONAFLUX_NATIVE_DIR="${NATIVE}" gradle --no-daemon --stacktrace -p "${ROOT}/bindings/kotlin" :personaflux:assembleRelease
