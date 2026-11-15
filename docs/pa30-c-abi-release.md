# PA30 c-abi component release

This releases the finite **c-abi** work unit. PA30 itself remains planned. The
graded contract of this unit is its published manifest plus the deferred list
below. The deferred list is long: every program that needs a host C provider
at link time is outside what the course native checker can run today.

## Released contract

The canonical manifest is [c-abi.json](../pa30/tests/units/c-abi.json): 85
declared outcomes, 26 accept/59 reject, all `--emit-lowir` with file input,
under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting
panics. Two accepted programs execute and 24 are code-generation-only.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `aggregate-abi` | 5 | 2 | 3 | 2 |
| `aggregate-abi/composites` | 3 | 0 | 3 | 0 |
| `conventions` | 13 | 6 | 7 | 0 |
| `imports` | 10 | 0 | 10 | 0 |
| `linkage` | 13 | 5 | 8 | 0 |
| `statics` | 22 | 12 | 10 | 0 |
| `variadic-tails` | 4 | 0 | 4 | 0 |
| `variadics` | 15 | 1 | 14 | 0 |

Outcomes come from the eight committed draft tables for requirements 30.1 to
30.4 and 30.7: `convention`, `import`, `linkage`, `static`, `aggregate`,
`composite`, `variadic` and `variadic-tail`. No fixture was authored or
edited. Each contract outcome is the table's pinned-Rust outcome. Five cases
whose rejection is a lint level declare `default_lints`.

## Deferred list

All 73 deferred fixtures are Rust-valid programs. None is published as an
expected rejection.

| Fixtures (`tests/spec/`) | Observed with the candidate |
| --- | --- |
| `imports/`: default, safe, unsafe-pointer, rust-abi, expanded-item, expanded-block, scalars, callback, pointer, generic-caller, constant-pointer; `linkage/`: alias, exports; `statics/`: mutable, imports, exports, callback, pointer-constant; `aggregate-abi/`: interop, aligned, indirect; `aggregate-abi/composites/`: nested-integer, nested-sse, nested-integer-sse, nested-sse-integer, array-integer, array-sse, array-bytes, array-nested-merge, nested-memory, zero-array-field, shifted-nested-mixed, shifted-array-mixed; `variadics/`: integers, zero-tail, float-register-spill, integer-register-spill, indirect, constant-pointer, safe-ignores-tail, named-tail, explicit-promotions, callback, evaluation-order, mixed-scalars, record-tail, record-prefix-result, nested-record-tail, generic-caller; `variadic-tails/`: multiple-records, multiple-records-indirect, inferred-pointer, shared-tail, generic-small-tail, zero-sized-tail, moved-tail (56) | The frontend accepts each one and pinned rustc compiles it to an object. It needs a host C provider at link, and the course native checker links standalone programs only, so `check_native.py` cannot qualify it. These stay executed by the maintainer suites through the object bridge |
| `statics/const-address.rs` (1) | Same cause on the backend side: `backend failed (1): ERROR: undefined symbol: X` |
| `linkage/import-alias.rs` (1) | Its provider is `imports/provider.c`, outside the fixture's directory, which the course loader rejects as an auxiliary |
| `aggregate-abi/`: rust-repr, packed, union, enum, empty (5) | `ERROR: C aggregate ABI profile requires a nonempty aligned repr(C) struct` |
| `aggregate-abi/composites/`: packed-inner, rust-layout-inner, union-inner (3) | `ERROR: C aggregate ABI profile requires nonempty aligned repr(C) fields` |
| `linkage/`: unicode, unsupported-byte (2) | `ERROR: exact symbol spelling is outside the pinned LowIR vocabulary` |
| `linkage/`: helper-first, export-first (2) | `ERROR: multiple function definitions for one LowIR symbol` |
| `statics/`: generated-collision, generated-import-collision (2) | `ERROR: LowIR symbol kind conflicts with prior declaration` |
| `statics/interior-storage.rs` (1) | Passes text, rustc and native, but declares `pa28.atomic-storage`. c-abi has no atomic-u32 prerequisite edge, so the unit validator rejects it as a future-stage feature |

Not owned by this unit and unchanged: the four `*-metadata` tables
(value-metadata, 30.5), `option-niche` (const-memory), `call-argument-context`
(a PA19 prerequisite control) and the remaining PA30 packets without a draft
outcome table.

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. Checks ran on 2026-11-15 through the resource and
storage guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit c-abi` | 85 cases valid |
| `check_syntax.py pa30 --unit c-abi` | 85 independent rustc checks agree (20.4 s) |
| `course.py pa30 --unit c-abi --bless` | 85/85 declared outcomes, then references written (19.6 s) |
| `check_layout.py --unit c-abi` | complete oracle provenance |
| `course.py pa30 --unit c-abi` | 85/85 against the committed text (18.6 s) |
| `check_native.py pa30 --unit c-abi` | 85 checks; 2 executable and 24 code-generation comparisons (21.4 s) |

A probe ran all 158 candidates before the manifest was declared. A review of
the 28 distinct rejection diagnostics found no implementation-gap wording.
Receipts are under `obj/publication/c-abi/`.

## Requirements

Requirements 30.1 to 30.4 and 30.7 stay implemented components. This release
publishes their text contract and standalone programs. Their C-provider native
matrix is not part of the published contract until the course native checker
can link a provider. Requirements 30.5 and 30.6 are unchanged.
