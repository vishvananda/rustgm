# PA35 unicode-crates component release

This releases the **unicode-crates** work unit and closes PA35 at its finite
contract, deferring `core-crates` (`itoa` / `serde_core`) and `alloc-crates`
(`serde_json`) to PA45.

## Released contract

The grading manifest is [suite.json](../pa35/tests/suite.json) (`suite_kind:
"consumers"`, strategy `whole-graph-unicode`), driven by
[grade_consumers.py](../scripts/grade_consumers.py):

- Compiles 4 independent consumers declared in
  [unicode-consumer-outcomes.json](../pa35/tests/unicode-consumer-outcomes.json)
  (`xid`, `normalization`, `identifiers`, `small-vectors`) over unmodified
  pinned Rust 1.98.0 `core`/`alloc`/`std` and RustGM's locked upstream crates
  verified by SHA-256 in [setup_rust_src.py](../scripts/setup_rust_src.py):
  `unicode-ident 1.0.24`, `tinyvec_macros 0.1.1`, `tinyvec 1.10.0`, and
  `unicode-normalization 0.1.25`.
- Strict zero-bringup mode, linked through pinned CPPGM v0.12.0, executed 3
  times each in fresh directories against pinned `rustc 1.98.0` oracles.
- Recompiles `xid` a second time and verifies byte-identical LowIR and native
  link manifests.

## Deferred work

`core-crates` and `alloc-crates` remain unpublished and are carried by PA45
under PA35's finite contract in [assignments.json](../assignments.json).

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, with pinned
CPPGM v0.12.0 ([backend.lock.json](../backend.lock.json)). Qualified on
2026-11-28:

| Check | Result |
| --- | --- |
| 4 upstream crate consumers (3 repeats each) | 4/4 PASS, zero bring-up records |
| Reproducibility (`xid`) | PASS, byte-identical LowIR `1fe6ee688c1143642c4d3e38b9a48c2d4cba148ea46a6763ce8591e61207e670` |
| Total wall time (`--jobs 2`) | 98.9s |
