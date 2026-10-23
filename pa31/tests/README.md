# PA31 fixture ownership

Planned: no graded manifest or generated reference outputs yet.

`spec/needs-drop/` retains independently declared source-provider inputs for the
actual `needs_drop<T: ?Sized>` intrinsic and its ordinary const-block wrapper.
The grouped maintainer `needs_drop_provider` tests discover these files directly;
this does not activate PA31. `contract.json` records the fifteen original
positive/negative source hashes and independent expected outcomes. `work/`
contains thirteen separately declared width/repetition/noise controls.

The small provider uses actual attributed declarations, not the complete pinned
Drop/Pin implementation. Full default-method, Freeze/drop_glue and conditional
destruction support remain required. Public generic/trait metadata and executable
entry readiness are distinct from checked LowIR/native kernel qualification.
See [the source-query review](../../docs/pa28-source-needs-drop.md).

Required families: lang-item identity and signatures; intrinsic declarations and const eligibility; builtin expansion and capability answers; duplicate and forged bindings; panic-abort runtime boundary.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.
