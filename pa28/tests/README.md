# PA28 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Remaining families: atomic source operations and orderings; compare-exchange;
numeric and memory intrinsics; overlap rules.

`backend/` contains independent LowIR capability controls, run by the ignored
`dev/tests/intrinsic_backend.rs` maintainer test. They do not establish Rust
source support or activate a grading contract.

`spec/volatile/` contains the initial scalar source controls, native comparisons,
retained-effect controls and explicit aggregate/wide-pointer/zero-sized
unsupported boundaries. Every input is declared in `manifest.draft.json`; its
rustc outcome is independent of the frontend outcome. The draft is deliberately
outside the active manifest path until release qualification.

Run the two maintainer tests with `--include-ignored` to include the native
comparisons and pinned backend capability probes. Their generated outputs use
`obj/parallel/pa28/`, which must be included in scoped cleanup. Opcode evidence
is disassembled only from bounded executable ELF load-segment payloads, after
checking the pinned ELF64 little-endian x86-64 format and entry point.

Put required regressions here, in the earliest assignment owning all their
features. Register every input in `manifest.json`; no undiscovered fixtures.
