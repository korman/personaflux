# PersonaFlux C# binding

This `netstandard2.0` wrapper is intentionally thin: all simulation behavior is
implemented by the PersonaFlux C ABI. `Simulation` owns a `SafeHandle`, copies
all returned data into managed values, and translates numeric result codes into
`PersonaFluxException` instances. The native library must be named
`personaflux` (`personaflux.dll`, `libpersonaflux.so`, or
`libpersonaflux.dylib`) and must be accessed serially per simulation handle.

The wrapper targets the ABI v0 header in
`crates/personaflux-ffi/include/personaflux.h`. It does not expose Rust types,
retain native buffers, or change model semantics.
