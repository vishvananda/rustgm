# PA32 core bootstrap release

This releases the **core-artifact** work unit and completes PA32.

## Released contract

The grading manifest is [suite.json](../pa32/tests/suite.json) (`suite_kind:
"consumers"`, strategy `standalone-core`), driven by
[grade_consumers.py](../scripts/grade_consumers.py):

- Compiles the unmodified pinned Rust 1.98.0 `library/core/src/lib.rs` (commit
  `88d9e12ae178fab0fb5cc050a94da85685d449ea`, 718 verified files in
  [original_core_manifest.json](../scripts/original_core_manifest.json)) with
  `--source-profile module-body-operations-v14` and strict zero-bringup checks,
  twice in independent directories to verify byte-identical `core.rgcrate`,
  `core.lowir`, and `core.native-links.json`.
- Emits `core.o` through pinned CPPGM v0.12.0 and builds a minimal
  `#![compiler_builtins]` loader artifact.
- Compiles and links 18 independent `no_std` consumers declared in
  [core-consumer-outcomes.json](../pa32/tests/core-consumer-outcomes.json)
  against the standalone `core.rgcrate` and `core.o`, executing each binary 3
  times in fresh directories against pinned `rustc 1.98.0` stdout, stderr, and
  exit-status oracles (including aborting panic checks).
- Verifies the 2 panic-abort FFI unwind lint controls in
  [cases.json](../pa32/tests/spec/ffi-unwind-lint/cases.json).

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, with pinned
CPPGM v0.12.0 ([backend.lock.json](../backend.lock.json)). Qualified on
2026-11-28:

| Check | Result |
| --- | --- |
| Standalone `core` build (run-a / run-b) | 20.92s / 21.42s, zero bring-up records, byte-identical |
| `core.rgcrate` SHA-256 | `e91bbce2c3227dab277ad1849bd7e9c5d440c4b09324be94fbd0872edb373dca` |
| `core.lowir` SHA-256 | `4f3f5f1b9949c25806713683a9397fcbc9b1eb7a4a12b4f6b692dbc29dffa776` |
| 18 `no_std` consumers (3 repeats each) | 18/18 PASS |
| `ffi-unwind-lint` | 2/2 PASS |
| Total wall time (`--jobs 2`) | 78.8s |
