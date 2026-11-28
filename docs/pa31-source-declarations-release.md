# PA31 source-declarations component release

This releases the finite **source-declarations** work unit and closes PA31 at
its finite contract, deferring `source-runtime` to PA45.

## Released contract

The canonical manifest is [source-declarations.json](../pa31/tests/units/source-declarations.json):
41 declared outcomes (20 accept, 21 reject), all `--emit-lowir` with file input,
under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting
panics.

| Family (`tests/grading/`) | Cases | Accept | Reject | Executed (C driver) | Object |
| --- | --- | --- | --- | --- | --- |
| `local` | 9 | 3 | 6 | 0 | 3 |
| `transport` | 32 | 17 | 15 | 17 | 0 |

- Nine standalone `local/` cases cover `#![no_core]` source-core providers,
  duplicate and wrong-kind lang-item rejections, unwind-safety diagnostic-item
  validation, and `unreachable_pub` / `missing_docs` lint policies.
- 32 `transport/` cases compile separate `module-body-operations-v14` source
  providers and fresh `--source-client` consumers covering unwind markers,
  specialization, staged feature relationships, deprecated macros, final trait
  methods, pointer lints, slice length / `Pointee`, derived sizing, enum policy
  and `dyn` inherent methods, caller-site tracking, range and C-string roles,
  dynamic dispatch, declared intrinsics, `const trait`, `offset_of`, promotion,
  forged foreign roles, and client-identity mismatches.
- Three rejections (`invalid-unwind-not-auto.rs`, `forged-foreign-role.rs`,
  `client-identity-mismatch.rs`) document explicit RustGM source-profile /
  metadata-authentication rejections where pinned `rustc` accepts the source.
- Two accepted transport clients declare reviewed warning codes (`deprecated`,
  `unreachable_patterns`).

## Deferred work

`source-runtime` remains unpublished and is carried by PA45 under PA31's finite
contract in [assignments.json](../assignments.json).

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, linked with
the pinned CPPGM v0.12.0 backend ([backend.lock.json](../backend.lock.json)).
Qualified on 2026-11-28 through the storage and resource guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit source-declarations` | 41 cases valid |
| `check_syntax.py pa31 --unit source-declarations` | 41 independent rustc checks agree (3 documented differences) |
| `course.py pa31 --unit source-declarations --bless` | 41/41 declared outcomes, references written |
| `check_layout.py --unit source-declarations` | complete oracle provenance |
| `course.py pa31 --unit source-declarations` | 41/41 against committed text |
| `check_native.py pa31 --unit source-declarations` | 41 checks; 17 C-driver executable comparisons, 3 object comparisons, 32 against provider crates |
