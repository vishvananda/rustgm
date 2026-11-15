# PA28 fixture ownership

The released `runtime-memory` component publishes `units/runtime-memory.json`
and the reference sidecars of its 265 cases; see the
[release record](../../docs/pa28-runtime-memory-release.md) for its deferred
list. The released `atomic-u32` component publishes `units/atomic-u32.json` and
the sidecars of its 246 cases; see its
[release record](../../docs/pa28-atomic-u32-release.md). Every other family
here is unpublished.

Whole PA28 is planned: no graded whole-assignment suite yet.

Remaining families: atomic source operations and orderings; compare-exchange;
numeric and memory intrinsics; overlap rules.

`unaligned-memory-contract.json` declares the Sized unaligned transfer controls
under `spec/unaligned-memory/` and the native ownership comparisons under
`unaligned-memory-native/`. `unaligned-memory-outcomes.json` binds the checked
inputs and outputs. These development receipts do not activate the assignment.

`backend/` contains independent LowIR capability controls, run by the ignored
`dev/tests/intrinsic_backend.rs` maintainer test. They do not establish Rust
source support or activate a grading contract.

`backend/atomics/contract.json` registers the frozen multi-width, pointer,
boolean and fence matrix, including independent pinned-Rust and validator-only
outcomes. Its [contract notes](backend/atomics/README.md) state the exact
coverage and concurrency limits. All its native tests remain ignored by default.

`spec/volatile/` contains the initial scalar source controls, native comparisons,
retained-effect controls and explicit aggregate/wide-pointer/zero-sized
unsupported boundaries. Every input is declared in `manifest.draft.json`; its
rustc outcome is independent of the frontend outcome. The draft is deliberately
outside the active manifest path until release qualification.

Run the maintainer tests with `--include-ignored` to include the native
comparisons and pinned backend capability probes. Their generated outputs use
`obj/parallel/pa28/`, which must be included in scoped cleanup. Opcode evidence
is disassembled only from bounded executable ELF load-segment payloads, after
checking the pinned ELF64 little-endian x86-64 format and entry point.

Put required regressions here, in the earliest assignment owning all their
features. Register every input in its declared contract manifest; active grading
cases use `manifest.json`, while these planned capability controls retain their
explicit companion manifest. No undiscovered fixtures.

`spec/multiply-overflow/outcomes.json` freezes six source outcomes plus a
156-pair native observer across all12 integer types, with declared mathematical
result bits/flags independently checked against a pinned Rust object. This numeric
intrinsic execution packet follows the source-subtraction ownership in PA28.
