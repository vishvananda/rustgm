# PA42 fixture ownership

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
six amendments are recorded beside the outcomes they change; the second
withdrew the declared inherent method and its client, the fifth rebuilt
the stream witness's client with a scalar C surface, and the sixth, from
pa29's associated-unsafe item, returns `inherent.rs` as the client of the
now-exported `export_controls.inherent`. They are checkpoint
inputs, not an active grading suite. The evidence is in
[target-feature-rows.md](../../docs/parallel/target-feature-rows.md).

`spec/source-integer-assembly/outcomes.json` declares the v14 flags extension:
original pinned x86-64 flags functions, 64-bit register inputs and output
snapshots, AT&T templates and `nomem`. Fresh native clients check inout, lateout,
aliased output ordering and output through a reference; v13 and unsafe controls
retain their declared refusals. Run `scripts/check_source_artifact_clients.py`
with `--case source-integer-assembly`.
Original flags execution remains an unqualified expected-success case because
the pinned encoder does not implement `pushfq`/`popfq`; the outcome manifest
keeps that acceptance separate from the five qualified integer observers.

`spec/source-fixed-registers/outcomes.json` declares v14 CPUID transport and
native execution: 32-bit general outputs, exact fixed GPR aliases and the `r`
view. The original active body tail is byte-preserved; independent C queries
check all fields, a 64-bit RBX canary, signed inlateout and an ordinary control.
Run `check_source_artifact_clients.py --case source-fixed-registers`.

The fixed-register fixture also retains the original `_mm_stream_si32` and its
32-bit `e` register view. Five native integer inouts check zero extension after a 32-bit register write.
Original `movnti` runtime acceptance and sentinel checks remain pending on
pinned CPPGM opcode support.

`spec/source-memory-assembly/outcomes.json` declares pure and readonly source
assembly, full 128-bit output snapshots, aliased stores and memory dependencies.
Run `check_source_artifact_clients.py --case source-memory-assembly`. Original
SSE4.1 `movntdqa` runtime acceptance remains pending on pinned backend support.

`spec/source-wide-registers/outcomes.json` declares actual 128/256/512-bit
payloads independently of YMM/ZMM views. Original AVX stores and narrow YMM
controls execute on AVX2; AVX-512 clients are object-only on this host. Original
non-temporal load execution remains expected success pending backend support.
The same suite checks the original `_mm_mask_load_ss` through a fresh client
and an AVX-512F object. Its `u8` mask retains its source type and uses a
16-bit backend transfer.

`spec/source-byte-registers/outcomes.json` covers the eight original x86
bit-test functions. A fresh client observes the carry bit through a one-byte
output, with signed offsets across words. The observer checks both read-only
and modifying forms and the surrounding guard bytes. Run
`check_source_artifact_clients.py --case source-byte-registers`.
