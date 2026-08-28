# PersonaFlux Swift binding

`PersonaFlux` is a value-copying Swift wrapper over the ABI v0 header. The
package keeps the canonical header in `crates/personaflux-ffi/include`; the
local C target includes it by repository-relative path so ABI drift is visible
in review. Distribution scripts must stage that canonical header into the
package before publishing.

`Simulation` owns an `OpaquePointer`, releases it in `deinit`/`close()`, and
must be accessed serially. Native libraries are linked by the XCFramework
build script, not by the wrapper's domain code.
