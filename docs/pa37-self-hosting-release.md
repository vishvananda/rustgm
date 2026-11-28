# PA37 reproducible self-hosting release

This releases the **self-hosting** work unit and completes PA37.

## Released contract

The grading manifest is [suite.json](../pa37/tests/suite.json) (`suite_kind:
"self-hosting"`), driven by [pa37_self_host.py](../scripts/pa37_self_host.py)
and [pa37/Makefile](../pa37/Makefile) (modeled on CPPGM's self-hosting
inception assignment):

- Freezes the ordered 16-crate Linux x86-64 abort-profile source graph
  ([configuration.json](../pa37/configuration.json)): unmodified pinned Rust
  1.98.0 `core`, `alloc`, `cfg_if 1.0.4`, `libc 0.2.185`, `unwind`,
  `std_detect`, `hashbrown 0.17.1`, `rustc_demangle 0.1.27`, `panic_abort`,
  `std`, RustGM's locked dependencies (`unicode_ident 1.0.24`,
  `tinyvec_macros 0.1.1`, `tinyvec 1.10.0`, `unicode_normalization 0.1.25`),
  `rustgm` (`dev/src/lib.rs`), and `app` (`dev/bin/rustgm.rs`).
- **Stage 1:** Stage 0 compiles the frozen whole-source graph in strict mode
  with zero bring-up records and links `stage1/native/rustgm` through pinned
  CPPGM v0.12.0 (`-O1`) and the pinned host linker.
- **Oracle ladder:** Stage 1 runs all 24 committed PA1–PA24 text oracle suites
  (9,462 cases) through `RUSTGM_TEST_APP` with zero regenerated or relaxed
  oracles.
- **Stage 2 (inception):** Stage 1 compiles the exact same frozen source graph
  in strict mode with zero bring-up records and links `stage2/native/rustgm`
  through the exact same CPPGM and linker pins.
- **Byte-for-byte identity:** Stage 2's `program.lowir`,
  `program.native-links.json`, and linked `rustgm` executable must match Stage 1
  byte for byte (`MATCH rustgm`).
- **Negative controls:** Verifies that perturbed Stage 2 output and nonzero
  bring-up records are rejected by the strict receipt and comparison checks.

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, with pinned
CPPGM v0.12.0 ([backend.lock.json](../backend.lock.json), `cppgm` SHA-256
`3e464fc6b13f6cb64a15ef907294642e5ff4ede2e582fb94d40fa48bbb42749c`) and host
linker `/usr/bin/x86_64-linux-gnu-gcc-15` (SHA-256
`b5f1b773a7c733738352000c92a077dc5852a1a2fc6d836b1e411be1e9ec5f88`). Qualified
on 2026-11-28 (`obj/pa37/summary.json`):

| Check | Result |
| --- | --- |
| Frozen source inventory SHA-256 | `d132742d094dc24b364ecc868d38d35a7ca6755f0867b720b44283ae33dab2d4` |
| Stage 1 frontend (`stage0`) | 381.9s, peak 6,336.4 MiB, `bringup_records=0` |
| Stage 1 CPPGM `-O1` + link | 245.8s + 6.1s, peak 2,725.2 MiB |
| Stage 1 PA1–PA24 oracle ladder | 24/24 suites, 9,462/9,462 cases PASS |
| Stage 2 frontend (`stage1`) | 962.6s, peak 9,137.5 MiB, `bringup_records=0` |
| `program.lowir` (Stage 1 == Stage 2) | 754,461,643 bytes, SHA-256 `3fe5d9c5e6186bd3a0f8956b7f531ece4021495dde9f281b62fa8dbf496c649a` |
| `program.native-links.json` (Stage 1 == Stage 2) | 202 bytes, SHA-256 `86d16af06fddc7e88b641f76e43d6a57c3cd01b2cb94a35204ef9247a9bdc60c` |
| Linked `rustgm` (Stage 1 == Stage 2) | 164,171,592 bytes, SHA-256 `2ca128398857a06a54cb8ba4fdcf695dbe66e2c2ca3c7bdc8ce6b0ffe3838f29` |
| Negative controls | PASS (`perturbed_stage2_rejected=true`, `bringup_record_rejected=true`) |
| Total wall time (`--optimization 1 --jobs 4`) | 1,906.3s |
