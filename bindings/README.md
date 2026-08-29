# PersonaFlux language bindings

The bindings in this directory are thin ABI v0 consumers. They do not link to
`personaflux-core` directly and they do not reimplement evaluation, relationship
resolution, memory, time, or event behavior.

- `csharp`: `netstandard2.0` P/Invoke wrapper with `SafeHandle`.
- `swift`: Swift Package using `OpaquePointer`; use the macOS XCFramework
  script for Apple device/simulator artifacts.
- `kotlin`: Android AAR with a small JNI bridge over direct ABI buffers.

The canonical wire contract remains
`crates/personaflux-ffi/include/personaflux.h`. Native libraries are generated
by the platform build scripts and are intentionally not committed.
