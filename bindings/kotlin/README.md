# PersonaFlux Kotlin / Android binding

The Kotlin API in `personaflux` is an `AutoCloseable` value-copying wrapper over
the ABI v0 header. The JNI layer only bridges direct buffers, primitive values,
and the opaque handle; evaluation, relationships, memory, time, and event
semantics remain in the Rust core.

Build the native Rust library for `arm64-v8a`, `armeabi-v7a`, and `x86_64`, set
`PERSONAFLUX_NATIVE_DIR` for the CMake import path, and assemble the AAR with
the Android Gradle Plugin. A simulation handle must be accessed serially.
