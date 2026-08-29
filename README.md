# PersonaFlux

[English](README.md) | [简体中文](README.zh-CN.md)

PersonaFlux is an engine-independent social-affect simulation core written in Rust.

The current implementation provides a deterministic v1 core for normalized
values, factions, directed relationships, direct-witness Deeds, PAD state,
memory, logical time, and event processing. It also provides a versioned C ABI
v0 and thin C#, Swift, and Kotlin bindings for external applications and game
prototypes.

## Goals

- Model continuous PAD affect and configurable emotion regions.
- Represent factions, members, personality traits, and directed relationships.
- Evaluate v1 direct-witness Deeds using the fixed relationship and affect rules.
- Maintain short-term and long-term memory with deterministic expiration.
- Remain independent of rendering, physics, ECS, and game engines.
- Expose a versioned C ABI v0 for desktop, mobile, Unity, and other languages.

Future model versions may add personality, rumors, source trust, and other
evaluation factors without changing the v1 contract.

## Workspace

- `personaflux-core`: safe, engine-independent Rust domain logic.
- `personaflux-ffi`: versioned C ABI that builds static and dynamic libraries.
- `bindings/csharp`: `netstandard2.0` P/Invoke wrapper using `SafeHandle`.
- `bindings/swift`: Swift Package and iOS XCFramework build script.
- `bindings/kotlin`: Kotlin/Android wrapper with a small JNI bridge and AAR build script.
- `docs`: project charter, behavioral specification, architecture, FFI, and roadmap.

See [docs/README.md](docs/README.md) for the full design documentation. Release
and compatibility rules are documented in [docs/VERSIONING.md](docs/VERSIONING.md),
and user-visible changes are tracked in [CHANGELOG.md](CHANGELOG.md).

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo build -p personaflux-ffi --release
```

The FFI crate produces `personaflux.lib` and `personaflux.dll` on Windows, with corresponding `.a`, `.so`, or `.dylib` artifacts on other targets.

## Language bindings

ABI v0 wrappers are provided under `bindings/csharp`, `bindings/swift`, and
`bindings/kotlin`. See `docs/PLATFORMS_AND_BINDINGS.md` for platform build
requirements. Build the C# wrapper with
`dotnet build bindings/csharp/PersonaFlux.csproj`; Swift/iOS and Android
packaging require the respective Apple and Android toolchains. The canonical C
header and wire contract are documented in `docs/C_ABI_AND_FFI.md`.

## Status

The Rust v1 core and C ABI v0 are usable for external prototypes and small
games. C# has a local integration smoke test; Swift/iOS and Android/Kotlin
require their platform toolchains or CI for full validation. ABI v0 is not yet
the long-term compatibility freeze: ABI v1 requires a separate maintainer
review after cross-platform validation. Snapshots, rumors, personality, and
engine-specific integration remain future work, and the ADRs remain `Proposed`.

## Independence

PersonaFlux is an independent implementation based on public affective-computing concepts and original behavioral specifications. It is not affiliated with Pixel Crushers and must not copy or directly translate Love/Hate source code, documentation, assets, or branding.

## License

PersonaFlux is currently licensed under the MIT License.
