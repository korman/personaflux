#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
NATIVE="${ROOT}/target/personaflux-android"
JNI_LIBS="${ROOT}/bindings/kotlin/personaflux/src/main/jniLibs"

rm -rf "${NATIVE}"
rm -rf "${JNI_LIBS}"
mkdir -p "${JNI_LIBS}/arm64-v8a" "${JNI_LIBS}/armeabi-v7a" "${JNI_LIBS}/x86_64"
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o "${NATIVE}" build -p personaflux-ffi --release
cp "${NATIVE}/arm64-v8a/libpersonaflux.so" "${JNI_LIBS}/arm64-v8a/"
cp "${NATIVE}/armeabi-v7a/libpersonaflux.so" "${JNI_LIBS}/armeabi-v7a/"
cp "${NATIVE}/x86_64/libpersonaflux.so" "${JNI_LIBS}/x86_64/"
PERSONAFLUX_NATIVE_DIR="${NATIVE}" gradle -p "${ROOT}/bindings/kotlin" :personaflux:assembleRelease
