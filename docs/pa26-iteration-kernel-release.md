# PA26 iteration-kernel component release

This releases the finite **iteration-kernel** work unit. PA26 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below; nothing else in PA26 is claimed.

## Released contract

The canonical manifest is [iteration-kernel.json](../pa26/tests/units/iteration-kernel.json):
196 declared outcomes, 135 accept/61 reject, all `--emit-lowir` with file
input, under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and
aborting panics. 134 accepted programs execute and pass; one is
code-generation-only.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `iteration` | 44 | 25 | 19 | 25 |
| `iteration-visibility` | 14 | 10 | 4 | 10 |
| `iteration/clone` | 30 | 21 | 9 | 20 |
| `iteration/generic-storage` | 10 | 8 | 2 | 8 |
| `iteration/integer-ranges` | 45 | 34 | 11 | 34 |
| `iteration/mutable-collections` | 27 | 19 | 8 | 19 |
| `iteration/shared-collections` | 26 | 18 | 8 | 18 |

Outcomes come from six committed draft tables: `iteration`,
`mutable-iteration`, `iterator-clone`, `integer-range`, `shared-iteration` and
`iteration-visibility` (`pa26/tests/*-outcomes.json`), 204 candidates in all.
No fixture was authored or edited. Each case's contract outcome is the table's
independent pinned-Rust outcome, never the draft candidate status. A draft
`native: true` becomes an executed program; any other accepted row is
code-generation-only, because no runtime outcome was declared for it.

The unit depends on `operator-kernel` only. Its earlier `source-declarations`
edge is dropped for this finite contract: every published case passes without
that unit, and the cases that need actual source range declarations are on the
deferred list.

## Deferred list

These eight fixtures stay in the tree, unpublished, and are required later. The
six Rust-valid positives are not expected rejections.

| Fixture (`tests/spec/iteration/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `clone/empty-zst.rs` | accept | Accepted, but with `warning[unreachable_patterns]: file 0: 128: unreachable pattern`, which pinned rustc does not report |
| `integer-ranges/range-inclusive-max.rs` | accept | `ERROR: file 0: 76: this range form requires actual source range declarations` |
| `integer-ranges/range-char.rs` | accept | `ERROR: file 0: 76: this range form requires actual source range declarations` |
| `integer-ranges/range-from-break.rs` | accept | `ERROR: file 0: 103: expected punctuation` |
| `integer-ranges/range-clone-required.rs` | accept | `ERROR: file 0: 52: concrete dispatch cannot use a symbolic assumption` |
| `integer-ranges/range-char-half-open-required.rs` | accept | `ERROR: file 0: 67: associated type requires an unambiguous validated implementation` |
| `integer-ranges/range-full-reject.rs` | reject | Rejected, but for the wrong reason: `ERROR: expected punctuation` (the parser does not accept the `..` form, as in `range-from-break.rs`) |
| `integer-ranges/range-to-reject.rs` | reject | Rejected, but with implementation-gap wording: `this range form requires actual source range declarations` |

The question-mark and residual fixtures of requirement 26.7 live under
`pa31/tests/spec` (`nominal-residuals`, `source-try-blocks`, `try-captures`,
`const-try-calls`) and are not claimed by this unit. Other PA26 families are
unchanged; see the [operator-kernel record](pa26-operator-kernel-release.md)
for their owners.

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. All checks ran on 2026-11-15 through the resource and
storage guards, using the [release recipe](../TESTING_AND_REFERENCES.md#component-release-recipe):

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit iteration-kernel` | 196 cases valid |
| `check_syntax.py pa26 --unit iteration-kernel` | 196 independent rustc checks agree (4.1 s) |
| `course.py pa26 --unit iteration-kernel --bless` | 196/196 declared outcomes, then references written (2.6 s) |
| `check_layout.py --unit iteration-kernel` | complete oracle provenance |
| `course.py pa26 --unit iteration-kernel` | 196/196 against the committed text (1.6 s) |
| `check_native.py pa26 --unit iteration-kernel` | 196 checks; 134 executable and 1 code-generation comparisons (7.1 s) |

Before the references were generated, the probe ran all 204 candidates through
pinned rustc, the frontend and the native backend with the same candidate: 198
passed. A review of the rejection diagnostics then deferred the two
wrong-reason rejections. Receipts are under `obj/publication/iteration-kernel/`.

## Requirements

Requirement 26.5 is an implemented component covered by this release.
Requirement 26.6 stays partial: this release covers half-open integer ranges
and the selected clone families, and owned collections, the other range forms
and helpers remain required. Requirement 26.7 stays partial and unpublished.
No performance claim is made. Scheduled CI runs the unit through the
`--released-units` text, independent and native schedulers.
