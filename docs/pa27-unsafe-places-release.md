# PA27 unsafe-places component release

This releases the finite **unsafe-places** work unit. PA27 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below.

## Released contract

The canonical manifest is [unsafe-places.json](../pa27/tests/units/unsafe-places.json):
262 declared outcomes, 149 accept/113 reject, all `--emit-lowir`, under Rust
1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting panics. 26
accepted programs execute and pass, and 123 are code-generation-only. 253
cases use file input; the nine `capture-native` programs use standard input
behind the declared check prelude.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `callable-safety` | 4 | 2 | 2 | 0 |
| `capture-integration-native` | 2 | 2 | 0 | 2 |
| `capture-native` | 9 | 9 | 0 | 9 |
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
table's pinned-Rust outcome. Three schema conversions apply:

- The draft tables were qualified with default lints, and the course checker
  caps lints unless told. The 29 cases whose rejection is a lint level
  (`static_mut_refs`, `unsafe_op_in_unsafe_fn` policies) declare `default_lints`.
- Six accepted cases emit required unsafe-policy warnings. Each declares its
  reviewed warning code, taken only where pinned rustc reports the same lint:
  `contexts/raw-fn-body.rs`, `contexts/unsafe-fn-caller.rs`,
  `contexts/nested-unsafe-item.rs`, `statics/unsafe-function-store.rs`,
  `policies/operation-warn.rs` (`unsafe_op_in_unsafe_fn`) and
  `policies/reference-warn.rs` (`static_mut_refs`).
- The nine `capture-native` bodies call the maintainer native harness's
  `check`, which that harness supplies as a fixed prelude. Each case declares
  `prelude: tests/prelude/check.rs`, a byte copy of that prelude, and both
  compilers read the prelude followed by the fixture as one translation unit.
  The joined bytes reproduce the draft table's `qualified_input_sha256`.

Two `union-borrows` rows that the draft recorded as candidate boundaries
(`pointer-family-boundary.rs`, `unit-family-boundary.rs`) are accepted by the
current compiler and are published as accepted, code-generation-only.

## Deferred list

| Fixture (`tests/spec/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `union-borrows/partial-write-only-boundary.rs` | accept | `ERROR: file 0: 38: constant union scalar read uses uninitialized bytes` |
| `union-borrows/repair-bool-boundary.rs` | accept | `ERROR: file 0: 41: constant union scalar read has invalid bool representation` |

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
| `assignment_metadata.py --unit unsafe-places` | 262 cases valid |
| `check_syntax.py pa27 --unit unsafe-places` | 262 independent rustc checks agree (10.5 s) |
| `course.py pa27 --unit unsafe-places --bless` | 262/262 declared outcomes, then references written (2.3 s) |
| `check_layout.py --unit unsafe-places` | complete oracle provenance |
| `course.py pa27 --unit unsafe-places` | 262/262 against the committed text (1.1 s) |
| `check_native.py pa27 --unit unsafe-places` | 262 checks; 26 executable and 123 code-generation comparisons (7.5 s) |

The first release published 253 cases and deferred 11, of which nine needed
the check prelude. The prelude input was added to the course runner and the
checkers the same day and the unit was probed, declared and checked again:
all 264 candidates ran, the nine passed, and no declaration or sidecar of the
253 earlier cases changed. A review of the rejection diagnostics found no
implementation-gap wording. Receipts are under `obj/publication/unsafe-places/`.

## Requirements

Requirements 27.1, 27.2 and 27.3 are implemented components covered by this
release. Requirements 27.4 to 27.7 are unchanged and belong to const-memory and
raw-validity. No performance claim is made.
