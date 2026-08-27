# PersonaFlux

[English](README.md) | [简体中文](README.zh-CN.md)

PersonaFlux is an engine-independent social-affect simulation core written in Rust.

The project is at the initial specification and scaffolding stage. Its intended scope includes PAD affect, factions, directed relationships, personality traits, deeds, memory, rumors, and deterministic event processing.

## Goals

- Model continuous PAD affect and configurable emotion regions.
- Represent factions, members, personality traits, and directed relationships.
- Evaluate deeds and rumors using trust, relationships, traits, and context.
- Maintain short-term and long-term memory with deterministic expiration.
- Remain independent of rendering, physics, ECS, and game engines.
- Expose a stable C ABI for desktop, mobile, Unity, and other languages.

## Workspace

- `personaflux-core`: safe, engine-independent Rust domain logic.
- `personaflux-ffi`: versioned C ABI that builds static and dynamic libraries.
- `docs`: project charter, behavioral specification, architecture, FFI, and roadmap.

See [docs/README.md](docs/README.md) for the full design documentation.

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo build -p personaflux-ffi --release
```

The FFI crate produces `personaflux.lib` and `personaflux.dll` on Windows, with corresponding `.a`, `.so`, or `.dylib` artifacts on other targets.

## Status

The public behavior and C ABI are not stable yet. Resolve the open decisions in `docs/BEHAVIOR_SPEC.md` before implementing the full simulation model or freezing ABI v1.

## Independence

PersonaFlux is an independent implementation based on public affective-computing concepts and original behavioral specifications. It is not affiliated with Pixel Crushers and must not copy or directly translate Love/Hate source code, documentation, assets, or branding.

## License

PersonaFlux is currently licensed under the MIT License.
