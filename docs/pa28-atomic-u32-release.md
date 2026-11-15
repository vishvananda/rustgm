# PA28 atomic-u32 component release

This releases the finite **atomic-u32** work unit. PA28 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below.

## Released contract

The canonical manifest is [atomic-u32.json](../pa28/tests/units/atomic-u32.json):
272 declared outcomes, 165 accept/107 reject, all `--emit-lowir`, under Rust
1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting panics. 37
accepted programs execute: 25 pass and 12 are declared invalid-ordering
panics. Ninety-nine are code-generation-only and 29 are entry-free libraries
compared as objects. 247 cases use file input; the 25 passing native
observers use standard input behind the declared check prelude.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `atomic-preparation` | 11 | 11 | 0 | 0 |
| `atomic-cas` | 17 | 7 | 10 | 0 |
| `atomic-cas-orders` | 50 | 30 | 20 | 0 |
| `atomic-cas-native` | 8 | 8 | 0 | 8 |
| `atomic-rmw` | 21 | 10 | 11 | 0 |
| `atomic-rmw-native` | 4 | 4 | 0 | 4 |
| `atomic-subtract` | 21 | 10 | 11 | 0 |
| `atomic-bitwise` | 27 | 13 | 14 | 0 |
| `atomic-extrema` | 28 | 18 | 10 | 0 |
| `atomic-extrema-native` (under `tests/`) | 6 | 6 | 0 | 6 |
| `atomic-methods` | 34 | 16 | 18 | 0 |
| `atomic-method-native` | 14 | 14 | 0 | 14 |
| `atomic-storage` | 26 | 13 | 13 | 0 |
| `atomic-storage-native` | 5 | 5 | 0 | 5 |

Outcomes come from the seven committed draft tables for requirement 28.3:
`atomic-cas`, `atomic-rmw`, `atomic-subtract`, `atomic-bitwise`,
`atomic-extrema`, `atomic-method` and `atomic-storage`. No fixture was authored
or edited. Each contract outcome is the table's pinned-Rust outcome. Five
schema conversions apply:

- Sixty `atomic-methods` and `atomic-storage` sources have no root `main`.
  They are declared as libraries with the explicit identity
  `atomic_fixture@pa28`, the shape PA24 uses for entry-free sources. As
  programs, both compilers would reject them for the missing `main`, which
  would hide the declared reason. The 29 accepted ones emit an empty LowIR unit
  and declare `lowir: empty`; the native checker confirms that the pinned
  rustc object and the backend object both define nothing.
- Twenty-nine cases whose rejection is the deny-by-default
  `invalid_atomic_ordering` or `unreachable_patterns` lint level declare
  `default_lints`.
- `atomic-methods/warn-invalid-order.rs` is accepted with
  `warning[invalid_atomic_ordering]`. That code joined the reviewed warning
  vocabulary on 2026-11-15, after a run over all 274 atomic candidates showed
  pinned rustc and the candidate report the lint for the same 28 sources: 27 at
  deny level and this one as a warning.
- Twelve executed programs are the tables' `panic: true` rows.
- The 25 native observers that expect a normal exit call the maintainer
  native harness's `check`, which that harness supplies as a fixed prelude.
  Each case declares `prelude: tests/prelude/check.rs`, a byte copy of that
  prelude, and both compilers read the prelude followed by the fixture as one
  translation unit.

## Deferred list

| Fixtures (`tests/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `spec/atomic-methods/ordering-tags.rs` (1) | accept | `ERROR: file 0: 57: unresolved macro is not in textual scope or path scope`. The draft table already records this boundary |
| `spec/atomic-storage/const-reference-static.rs` (1) | accept | The candidate emits an empty library unit, but the pinned rustc object defines the private static `A`. The native checker rejects the empty declaration |

No Rust-valid program is published as an expected rejection.

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
| `assignment_metadata.py --unit atomic-u32` | 272 cases valid |
| `check_syntax.py pa28 --unit atomic-u32` | 272 independent rustc checks agree (4.2 s) |
| `course.py pa28 --unit atomic-u32 --bless` | 272/272 declared outcomes, then references written (1.8 s) |
| `check_layout.py --unit atomic-u32` | complete oracle provenance |
| `course.py pa28 --unit atomic-u32` | 272/272 against the committed text (1.3 s) |
| `check_native.py pa28 --unit atomic-u32` | 272 checks; 37 executable, 99 code-generation and 29 object comparisons, all 29 declared empty units (10.5 s) |

The first release published 246 cases and deferred 28, of which 25 needed the
check prelude. The prelude input was added to the course runner and the
checkers the same day and the unit was probed, declared and checked again:
all 274 candidates ran, the 25 passed, and no declaration or sidecar of the
246 earlier cases changed. The warning case was added in a third pass, which
changed no other case. A review of the rejection diagnostics found no
implementation-gap wording and no missing-`main` rejection. The probe now runs
the native checker's own empty-object test, so `const-reference-static.rs` is
deferred before any reference is written. Receipts are under
`obj/publication/atomic-u32/`.

## Requirements

Requirement 28.3 is an implemented component covered by this release.
Requirements 28.4 to 28.7 are unchanged. This is not the full U32 accessor API
or a concurrency proof, and no performance claim is made.
