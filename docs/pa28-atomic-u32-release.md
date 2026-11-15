# PA28 atomic-u32 component release

This releases the finite **atomic-u32** work unit. PA28 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below.

## Released contract

The canonical manifest is [atomic-u32.json](../pa28/tests/units/atomic-u32.json):
246 declared outcomes, 139 accept/107 reject, all `--emit-lowir` with file
input, under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and
aborting panics. Twelve accepted programs execute, each a declared
invalid-ordering panic. Ninety-nine are code-generation-only and 28 are
entry-free libraries compared as objects.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `atomic-preparation` | 11 | 11 | 0 | 0 |
| `atomic-cas` | 17 | 7 | 10 | 0 |
| `atomic-cas-orders` | 50 | 30 | 20 | 0 |
| `atomic-cas-native` | 4 | 4 | 0 | 4 |
| `atomic-rmw` | 21 | 10 | 11 | 0 |
| `atomic-subtract` | 21 | 10 | 11 | 0 |
| `atomic-bitwise` | 27 | 13 | 14 | 0 |
| `atomic-extrema` | 28 | 18 | 10 | 0 |
| `atomic-methods` | 33 | 15 | 18 | 0 |
| `atomic-method-native` | 8 | 8 | 0 | 8 |
| `atomic-storage` | 26 | 13 | 13 | 0 |

Outcomes come from the seven committed draft tables for requirement 28.3:
`atomic-cas`, `atomic-rmw`, `atomic-subtract`, `atomic-bitwise`,
`atomic-extrema`, `atomic-method` and `atomic-storage`. No fixture was authored
or edited. Each contract outcome is the table's pinned-Rust outcome. Three
schema conversions apply:

- Fifty-nine `atomic-methods` and `atomic-storage` sources have no root `main`.
  They are declared as libraries with the explicit identity
  `atomic_fixture@pa28`, the shape PA24 uses for entry-free sources. As
  programs, both compilers would reject them for the missing `main`, which
  would hide the declared reason. The 28 accepted ones emit an empty LowIR unit
  and declare `lowir: empty`; the native checker confirms that the pinned
  rustc object and the backend object both define nothing.
- Twenty-nine cases whose rejection is the deny-by-default
  `invalid_atomic_ordering` or `unreachable_patterns` lint level declare
  `default_lints`.
- The twelve executed programs are the tables' `panic: true` rows. Rows that
  expect a normal exit are on the deferred list.

## Deferred list

| Fixtures (`tests/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `spec/atomic-cas-native/`: strong-matrix, weak-matrix, reified, shared-storage; `spec/atomic-rmw-native/`: orders, wrapping, reified, effects-storage; `atomic-extrema-native/`: all-orders-unsigned, equal-and-unused, operands-once, reified, static-nested, aliased-order; `spec/atomic-method-native/`: legal-orders, variable-orders, reified-methods, nested-shared-static, local-roundtrip, ordering-tags; `spec/atomic-storage-native/`: shared-static, nested-static, readonly-slot, projected-constant, transient-const (25) | accept, as declared runs | Not standalone programs: each needs the maintainer `check` prelude, so pinned rustc reports `E0425: cannot find function check`. They stay maintainer observers |
| `spec/atomic-methods/ordering-tags.rs` (1) | accept | `ERROR: file 0: 57: unresolved macro is not in textual scope or path scope`. The draft table already records this boundary |
| `spec/atomic-methods/warn-invalid-order.rs` (1) | accept | Accepted with `warning[invalid_atomic_ordering]`. That code is not in the course's reviewed warning vocabulary (`REVIEWED_WARNING_CODES`), so the successful-warning contract cannot be declared |
| `spec/atomic-storage/const-reference-static.rs` (1) | accept | The candidate emits an empty library unit, but the pinned rustc object defines the private static `A`. The native checker rejects the empty declaration |

The first row is a harness limit, not a compiler result. No Rust-valid program
is published as an expected rejection.

Not owned by this unit and unchanged:
`spec/atomic-storage/shared-derived-raw-store.rs`, which no atomic table
declares (it is a `provenance_writes.rs` control); the backend atomics packet
and wider atomic families (28.4, intrinsic-closure).

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. Checks ran on 2026-11-15 through the resource and
storage guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit atomic-u32` | 246 cases valid |
| `check_syntax.py pa28 --unit atomic-u32` | 246 independent rustc checks agree (24.4 s) |
| `course.py pa28 --unit atomic-u32 --bless` | 246/246 declared outcomes, then references written (23.2 s) |
| `check_layout.py --unit atomic-u32` | complete oracle provenance |
| `course.py pa28 --unit atomic-u32` | 246/246 against the committed text (30.9 s) |
| `check_native.py pa28 --unit atomic-u32` | 246 checks; 12 executable, 99 code-generation and 28 object comparisons, all 28 declared empty units (25.6 s) |

A probe ran all 274 candidates through the frontend, pinned rustc and the
native backend before the manifest was declared. A review of the 21 distinct
rejection diagnostics found no implementation-gap wording and no missing-`main`
rejection. The first native run failed on `const-reference-static.rs`; that
case was moved to the deferred list, its sidecars were removed and every check
was rerun. Receipts are under `obj/publication/atomic-u32/`.

## Requirements

Requirement 28.3 is an implemented component covered by this release.
Requirements 28.4 to 28.7 are unchanged. This is not the full U32 accessor API
or a concurrency proof, and no performance claim is made.
