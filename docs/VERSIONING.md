# PersonaFlux Versioning

PersonaFlux uses separate version numbers for separate compatibility
contracts. The numbers must not be combined into one project-wide value.

## Version dimensions

| Dimension | Current value | Source of truth | Change it when |
| --- | ---: | --- | --- |
| Release version | `0.1.1` | Workspace `Cargo.toml` | The packaged Rust crates, native libraries, headers, or bindings are released |
| C ABI version | `0` | `PF_ABI_VERSION` in Rust and `personaflux.h` | A public C layout, symbol contract, wire type, or compatibility rule breaks |
| Model version | `1` | `PF_MODEL_VERSION` | Simulation behavior or replay meaning changes |
| Snapshot schema version | Not implemented yet | Reserved for the snapshot format | The serialized snapshot representation changes |

The release version identifies an artifact. The ABI version identifies whether
an existing host can continue calling the exported C interface. The model
version identifies whether the same inputs and configuration retain the same
simulation meaning. A release may change one dimension without changing the
others.

## Release version policy

Release versions follow Semantic Versioning (`MAJOR.MINOR.PATCH`):

- Increment `PATCH` for backward-compatible fixes, test improvements, and
  documentation changes.
- Increment `MINOR` for backward-compatible functionality.
- Increment `MAJOR` for a stable-release compatibility break.

Before `1.0.0`, the leading zero communicates that the public Rust API and
artifact surface are still evolving. The C ABI and model version rules remain
independent and explicit during the `0.x` period.

The workspace `version` in `Cargo.toml` is the single source of truth for the
release version. Do not add a second hand-maintained `VERSION` file.

## ABI policy

`PF_ABI_VERSION` is a monotonically increasing integer. ABI v0 is an
experimental, versioned interface and is not yet the long-term compatibility
freeze. Additive changes must preserve existing layouts and symbols. Breaking
changes require a new ABI version and a separately versioned header/artifact;
they must not silently change the meaning of ABI v0.

Every ABI release must run the C header layout checks and the compatibility
tests for all supported bindings. Existing ABI headers and symbols remain
available when a new ABI version is published.

## Model policy

`PF_MODEL_VERSION` is a monotonically increasing integer. Increment it for any
change that can alter evaluation, PAD, relationship, memory, event ordering,
logical-time, or replay semantics. A model-version change must include
behavioral test vectors and release notes. Hosts that require deterministic
replay must persist and validate the model version.

## Snapshot policy

When snapshot import/export is implemented, each snapshot must carry a schema
version and the model version used to produce it. Snapshot schema migrations
must be explicit and tested; Rust internal memory layout is never a snapshot
contract.

## Release checklist

1. Update the workspace `version` and `CHANGELOG.md`.
2. Confirm the ABI and model versions, changing them only when their policies
   require it.
3. Run `cargo fmt --check` and `cargo test --workspace`.
4. Run the C header smoke test and available binding checks.
5. Create an annotated Git tag matching the release version, for example
   `v0.1.0`.
6. Publish release artifacts and notes that list all three applicable version
   values.
