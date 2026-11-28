# PA33 alloc bootstrap release

This releases the **alloc-artifact** work unit and completes PA33.

## Released contract

The grading manifest is [suite.json](../pa33/tests/suite.json) (`suite_kind:
"consumers"`, strategy `whole-graph-alloc`), driven by
[grade_consumers.py](../scripts/grade_consumers.py):

- Compiles 12 independent `no_std` + `alloc` consumers declared in
  [alloc-consumer-outcomes.json](../pa33/tests/alloc-consumer-outcomes.json)
  (`vec`, `string`, `box-dyn`, `btree`, `deque-heap-list`, `rc`, `arc`,
  `format`, `algorithms`, `allocator`, `alloc-failure`, `panic-borrow`) against
  unmodified pinned Rust 1.98.0 `core` and `alloc` in strict zero-bringup mode.
- Links each consumer through pinned CPPGM v0.12.0 and executes each binary 3
  times in fresh directories against pinned `rustc 1.98.0` stdout, stderr, and
  exit-status oracles (including OOM abort and `RefCell` panic checks).
- Recompiles `vec` a second time and verifies byte-identical LowIR and native
  link manifests.

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, with pinned
CPPGM v0.12.0 ([backend.lock.json](../backend.lock.json)). Qualified on
2026-11-28:

| Check | Result |
| --- | --- |
| 12 `alloc` consumers (3 repeats each) | 12/12 PASS, zero bring-up records |
| Reproducibility (`vec`) | PASS, byte-identical LowIR `3793929ad41908649dbc343e1bf6e60168bba8351683b557baf16e73fc11fc3b` |
| Total wall time (`--jobs 2`) | 168.8s |
