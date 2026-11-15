# PA27 unsafe-places component release

This releases the finite **unsafe-places** work unit. PA27 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below.

## Released contract

The canonical manifest is [unsafe-places.json](../pa27/tests/units/unsafe-places.json):
253 declared outcomes, 140 accept/113 reject, all `--emit-lowir` with file
input, under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and
aborting panics. Seventeen accepted programs execute and 123 are
code-generation-only.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `callable-safety` | 4 | 2 | 2 | 0 |
| `capture-integration-native` | 2 | 2 | 0 | 2 |
| `capture-precision` | 45 | 31 | 14 | 0 |
| `contexts` | 46 | 25 | 21 | 0 |
| `native` | 5 | 5 | 0 | 5 |
| `policies` | 39 | 18 | 21 | 0 |
| `statics` | 24 | 11 | 13 | 0 |
| `statics-native` | 5 | 5 | 0 | 5 |
| `union-borrows` | 20 | 11 | 9 | 0 |
| `unions` | 58 | 25 | 33 | 0 |
| `unions-native` | 5 | 5 | 0 | 5 |

Outcomes come from committed draft tables: the `manifest.draft.json` rows that
originate in the suites named by requirements 27.1 to 27.3, and the
`capture-precision`, `capture-integration`, `capture-native` and `union-borrow`
tables. No fixture was authored or edited. Each contract outcome is the
table's pinned-Rust outcome. Two schema conversions apply:

- The draft tables were qualified with default lints, and the course checker
  caps lints unless told. The 29 cases whose rejection is a lint level
  (`static_mut_refs`, `unsafe_op_in_unsafe_fn` policies) declare `default_lints`.
- Six accepted cases emit required unsafe-policy warnings. Each declares its
  reviewed warning code, taken only where pinned rustc reports the same lint:
  `contexts/raw-fn-body.rs`, `contexts/unsafe-fn-caller.rs`,
  `contexts/nested-unsafe-item.rs`, `statics/unsafe-function-store.rs`,
  `policies/operation-warn.rs` (`unsafe_op_in_unsafe_fn`) and
  `policies/reference-warn.rs` (`static_mut_refs`).

Two `union-borrows` rows that the draft recorded as candidate boundaries
(`pointer-family-boundary.rs`, `unit-family-boundary.rs`) are accepted by the
current compiler and are published as accepted, code-generation-only.

## Deferred list

| Fixture (`tests/spec/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `union-borrows/partial-write-only-boundary.rs` | accept | `ERROR: file 0: 38: constant union scalar read uses uninitialized bytes` |
| `union-borrows/repair-bool-boundary.rs` | accept | `ERROR: file 0: 41: constant union scalar read has invalid bool representation` |
| `capture-native/{packed,packed-move,raw-write,raw-move,raw-cancellation,union,nested,shared-before-boundary,pointer-slot}.rs` (9) | accept, as declared runs | Not standalone programs: each needs the maintainer suite's `check` prelude, so pinned rustc reports `E0425: cannot find function check`. They stay maintainer observers in `dev/tests/unsafe_precision.rs` |

Not owned by this unit and unchanged: the draft-manifest rows from
`image_storage` and `string_words` (`scalar-union`, `string-byte-words` and six
`unions` constants) and the byte-image, enum-byte, option-niche and capture
storage families (const-memory); raw reference, store, slice, validity and
union reference/address families (raw-validity and const-memory);
`union-borrow-native`, which has no path-level declared table.

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. Checks ran on 2026-11-15 through the resource and
storage guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit unsafe-places` | 253 cases valid |
| `check_syntax.py pa27 --unit unsafe-places` | 253 independent rustc checks agree (21.5 s) |
| `course.py pa27 --unit unsafe-places --bless` | 253/253 declared outcomes, then references written (18.4 s) |
| `check_layout.py --unit unsafe-places` | complete oracle provenance |
| `course.py pa27 --unit unsafe-places` | 253/253 against the committed text (17.8 s) |
| `check_native.py pa27 --unit unsafe-places` | 253 checks; 17 executable and 123 code-generation comparisons (23.8 s) |

A probe ran all 264 candidates through the frontend, pinned rustc and the
native backend before the manifest was declared. A review of the 37 distinct
rejection diagnostics found no implementation-gap wording. Receipts are under
`obj/publication/unsafe-places/`.

## Requirements

Requirements 27.1, 27.2 and 27.3 are implemented components covered by this
release. Requirements 27.4 to 27.7 are unchanged and belong to const-memory and
raw-validity. No performance claim is made.
