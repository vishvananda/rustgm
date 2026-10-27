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
