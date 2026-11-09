# PA41 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Required families: asm constraints; target features; unsupported capability rejection.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.

The authored SIMD native copy inputs in `spec/source-simd/` include the complete
provider `native-family-corrected.rs`, its byte-identical provider prefix plus
explicit entry in `native-family-probe.rs`, the unchanged C observer, and
`observer-bridge.c`/`observer-entry.c`. They are checkpoint inputs, not an active
grading suite. Their predeclared output, independent Rust observation, exact
entry adaptation and remaining library-mode rejection are recorded in
[source-simd-layout-checkpoint.md](../../docs/source-simd-layout-checkpoint.md).

`spec/asm-transport/` holds B2 slice A's module-body packet: a provider
whose first 69 lines are `spec/asm-native/native.rs` before `#[repr(simd)]`,
byte for byte, two accepted clients and two opponents.
`asm-transport-outcomes.json` lists every input with its sha256 and froze
their outcomes, the generated export controls and the forged rows before any
run. The comment fence's amendments, declared at 2026-11-09T08:12:41Z, are
recorded beside them. They are checkpoint inputs, not an active grading
suite. The evidence is in
[asm-transport.md](../../docs/parallel/asm-transport.md).

`spec/simd-transport/` holds B2 slice S's module-body packet: a provider
whose first 73 lines are `spec/asm-native/native.rs` through `__m128`'s
Clone and Copy impls, byte for byte, one accepted client and one opponent.
`simd-transport-outcomes.json` lists every input with its sha256 and froze
their outcomes, the generated export controls and the forged rows before any
run. Its one client amendment is recorded beside the outcome it changes.
They are checkpoint inputs, not an active grading suite. The evidence is in
[simd-header-rows.md](../../docs/parallel/simd-header-rows.md).

`spec/target-feature-transport/` holds B2 slice F's module-body packet: a
provider whose first 73 lines are `spec/asm-native/native.rs` through
`__m128`'s Clone and Copy impls, byte for byte, seven ordinary clients, and
the stream witness's provider and client. `target-feature-transport-outcomes.json`
lists every input with its sha256 and froze their outcomes, the generated
export controls, the local twins and the forged rows before any run. Its
five amendments are recorded beside the outcomes they change; the second
withdrew the declared inherent method and its client, and the fifth rebuilt
the stream witness's client with a scalar C surface. They are checkpoint
inputs, not an active grading suite. The evidence is in
[target-feature-rows.md](../../docs/parallel/target-feature-rows.md).
