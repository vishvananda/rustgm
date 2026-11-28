# PA36 std bootstrap release

This releases the **std-artifact** work unit and completes PA36.

## Released contract

The grading manifest is [suite.json](../pa36/tests/suite.json) (`suite_kind:
"consumers"`, strategy `whole-graph-std`), driven by
[grade_consumers.py](../scripts/grade_consumers.py):

- Compiles 14 independent `std` consumers declared in
  [std-consumer-outcomes.json](../pa36/tests/std-consumer-outcomes.json)
  (`println`, `hashmap`, `env-args`, `fs`, `io-stdin`, `process-exit`,
  `main-result`, `exit-code`, `sync-local`, `threads`, `path-ffi-time`,
  `float-math`, `panic-default`, `panic-hook`) against the complete unmodified
  pinned Rust 1.98.0 Linux x86-64 abort-profile standard library graph (`core`,
  `alloc`, `cfg_if 1.0.4`, `libc 0.2.185`, `unwind`, `std_detect`,
  `hashbrown 0.17.1`, `rustc_demangle 0.1.27`, `panic_abort`, `std`).
- Strict zero-bringup mode, linked through pinned CPPGM v0.12.0, executed 3
  times each in fresh directories against pinned `rustc 1.98.0` stdout, stderr,
  and exit-status oracles (including thread spawning, `std::thread::scope`,
  TLS, filesystem I/O, `libm` float operations, exit codes, and panic hooks).
- Recompiles `println` a second time and verifies byte-identical LowIR and
  native link manifests.

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, with pinned
CPPGM v0.12.0 ([backend.lock.json](../backend.lock.json)). Qualified on
2026-11-28:

| Check | Result |
| --- | --- |
| 14 `std` consumers (3 repeats each) | 14/14 PASS, zero bring-up records |
| Reproducibility (`println`) | PASS, byte-identical LowIR `8996bc9ad04ee0e2a0742ce91a97b450e3fe2918b237c24d110e1035b1f74303` |
| Total wall time (`--jobs 2`) | 244.1s |
