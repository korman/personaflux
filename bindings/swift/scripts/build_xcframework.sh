#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="${ROOT}/dist/PersonaFlux.xcframework"
HEADER="${ROOT}/crates/personaflux-ffi/include/personaflux.h"
BUILD="${ROOT}/target/personaflux-ios"

rm -rf "${OUT}" "${BUILD}"
mkdir -p "${BUILD}/device" "${BUILD}/simulator"
mkdir -p "${BUILD}/headers"
cp "${HEADER}" "${BUILD}/headers/personaflux.h"

rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cargo build --manifest-path "${ROOT}/Cargo.toml" -p personaflux-ffi --release --target aarch64-apple-ios
cargo build --manifest-path "${ROOT}/Cargo.toml" -p personaflux-ffi --release --target aarch64-apple-ios-sim
cargo build --manifest-path "${ROOT}/Cargo.toml" -p personaflux-ffi --release --target x86_64-apple-ios

cp "${ROOT}/target/aarch64-apple-ios/release/libpersonaflux.a" "${BUILD}/device/"
cp "${ROOT}/target/aarch64-apple-ios-sim/release/libpersonaflux.a" "${BUILD}/simulator/libpersonaflux-arm64-sim.a"
cp "${ROOT}/target/x86_64-apple-ios/release/libpersonaflux.a" "${BUILD}/simulator/libpersonaflux-x86_64-sim.a"

xcodebuild -create-xcframework \
  -library "${BUILD}/device/libpersonaflux.a" -headers "${BUILD}/headers" \
  -library "${BUILD}/simulator/libpersonaflux-arm64-sim.a" -headers "${BUILD}/headers" \
  -library "${BUILD}/simulator/libpersonaflux-x86_64-sim.a" -headers "${BUILD}/headers" \
  -output "${OUT}"
