# PersonaFlux Engineering Instructions

Read `docs/PROJECT_CHARTER.md` before architectural or behavioral changes.

## Project Direction

PersonaFlux is an independent Rust social-affect simulation engine with a stable C ABI. Preserve behavioral concepts, not the Unity architecture of existing products.

## Hard Boundaries

- Keep `personaflux-core` independent of engines, platforms, and FFI.
- Keep all C ABI translation inside `personaflux-ffi`.
- Do not copy or directly translate Pixel Crushers Love/Hate code.
- Do not add Unity, Bevy, Godot, rendering, physics, or ECS dependencies to the core.
- Use host-provided logical time, witnesses, and perception results.
- Do not expose Rust-owned types or addresses across the C ABI.
- Do not allow Rust panics to cross the FFI boundary.
- Prefer opaque handles, stable IDs, explicit errors, and batch APIs.
- Preserve deterministic processing order.
- Add behavioral tests for every evaluation-rule change.
- Record major architectural decisions under `docs/ADR/`.

## Sources Of Truth

- Product scope: `docs/PROJECT_CHARTER.md`
- Behavioral rules: `docs/BEHAVIOR_SPEC.md`
- Architecture: `docs/ARCHITECTURE.md`
- Public ABI: `docs/C_ABI_AND_FFI.md`
- Legal boundary: `docs/CLEAN_ROOM_AND_LEGAL.md`
