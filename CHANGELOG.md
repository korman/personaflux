# Changelog

All notable changes to PersonaFlux are documented in this file.

The release version follows Semantic Versioning. Release, C ABI, and model
versions are independent; see [docs/VERSIONING.md](docs/VERSIONING.md).

## [0.1.1] - 2026-08-30

Cross-platform development release.

- Fixed Swift error-code matching and batch buffer access for Apple runners.
- Fixed iOS XCFramework simulator library packaging.
- Fixed Android Gradle/CMake configuration, ABI selection, NDK discovery, and
  Kotlin `AutoCloseable` lifecycle compilation.
- Added GitHub Actions packaging for desktop, iOS, Android, and C headers.

Compatibility status: ABI v0 and model version 1 are unchanged. ABI v0 remains
intended for prototypes and is not yet the long-term compatibility freeze.

## [0.1.0] - 2026-08-30

Initial development release.

- Added the engine-independent Rust social-affect simulation core.
- Added deterministic PAD state, factions, directed relationships, direct-witness
  deed evaluation, memory, logical time, and event processing.
- Added the versioned C ABI v0 with static and dynamic library targets.
- Added C#, Swift, and Kotlin/Android wrapper foundations for ABI v0.
- Fixed the evaluation model at model version 1.

Compatibility status: ABI v0 is suitable for prototypes and is not yet the
long-term compatibility freeze. ABI v1 requires a separate maintainer review
after cross-platform validation.
