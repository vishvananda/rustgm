# PA26 operator-kernel component release

This releases the finite **operator-kernel** work unit. PA26 itself remains
planned. The graded contract of this unit is its published manifest plus the
deferred list below; nothing else in PA26 is claimed.

## Released contract

The canonical manifest is [operator-kernel.json](../pa26/tests/units/operator-kernel.json):
739 declared outcomes, 516 accept/223 reject, all `--emit-lowir` with file
input, under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and
aborting panics. 345 accepted programs execute (332 pass, 13 qualified panics)
and 171 are code-generation-only.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `assignment` | 35 | 25 | 10 | 0 |
| `comparison` | 51 | 33 | 18 | 32 |
| `constructor-promotion` | 12 | 7 | 5 | 0 |
| `dot-methods` | 26 | 17 | 9 | 0 |
| `environment` | 15 | 9 | 6 | 0 |
| `implicit-assignment-demand` | 17 | 12 | 5 | 11 |
| `index-capture` | 26 | 20 | 6 | 17 |
| `index-mut` | 44 | 28 | 16 | 1 |
| `library-types` | 17 | 11 | 6 | 0 |
| `operators` | 11 | 5 | 6 | 0 |
| `ord-helpers` | 28 | 21 | 7 | 21 |
| `ordering` | 42 | 29 | 13 | 28 |
| `ordering-impls` | 30 | 23 | 7 | 22 |
| `primitive-add` | 24 | 15 | 9 | 15 |
| `primitive-assignment` | 59 | 44 | 15 | 1 |
| `primitive-binary` | 44 | 32 | 12 | 32 |
| `primitive-constants` | 60 | 49 | 11 | 49 |
| `reference-comparison` | 60 | 44 | 16 | 26 |
| `sequence-equality` | 44 | 33 | 11 | 33 |
| `sequence-ordering` | 43 | 26 | 17 | 25 |
| `slice-length` | 51 | 33 | 18 | 32 |

Outcomes come from the 24 committed draft tables: the 20 `*-outcomes.json`
tables of the 2026-10-27 completion packet, the two implicit-assignment
prediction tables and the two `index-capture` tables. No fixture was authored
or edited. Each case's contract outcome is the table's independent pinned-Rust
outcome, never the draft candidate status. Conversions to the course schema:

- A draft `native: true` becomes an executed program (`traps`/`panic` rows
  expect the qualified abort). A dictionary is kept. Any other accepted row is
  code-generation-only, because no runtime outcome was declared for it.
- The 45 `sequence-equality` rows had no pinned-Rust row. Their outcomes were
  checked with pinned rustc before any reference was generated.
- The `environment` family declares the provider snapshot of
  `dev/tests/option_environment.rs`: `KEY` present, `MISSING` absent.
- `slice-length/mutable-static-lint.rs` is rejected only by the
  deny-by-default `static_mut_refs` lint, so it declares `default_lints`.

## Deferred list

These fixtures stay in the tree, unpublished, and are required later. The four
Rust-valid positives are not expected rejections.

| Fixture (`tests/spec/`) | Rust | Observed with the candidate |
| --- | --- | --- |
| `comparison/derive-boundary.rs` | accept | `ERROR: file 0: 116: trait obligation requires an unambiguous matching assumption or implementation` |
| `reference-comparison/contract-unsized-pointee.rs` | accept | `ERROR: file 0: 131: trait obligation requires an unambiguous matching assumption or implementation` |
| `sequence-equality/rest-metadata.rs` | accept | `ERROR: file 0: 138: incompatible types` |
| `primitive-constants/float-required.rs` | accept | `ERROR: file 0: 63: this primitive MAX constant is not implemented` |
| `primitive-constants/float-trait-priority.rs` | reject (E0308) | Rejected, but through the same unimplemented float `MAX` message, so its diagnostic is not a stable contract |

Other PA26 families are not owned by this unit and are unchanged: `iteration`
and `iteration-visibility` (iteration-kernel); `char-constants`,
`shared-value-operands`, `unsized-parameters`, `borrow-coercions`,
`deref-clone`, `deref-methods`, `explicit-const-dereference`, `attributes`
(hook-closure); `derives`, `derive-array-fields`, `derive-call-counts`
(format-derive); `option-niche` (const-memory); `impl-lifetimes` (a PA19
prerequisite control); `place-session` (a maintainer reduction contract).

## Qualification

The candidate is the release CLI built from clean `80229d28d` (3967 Rust
sources, empty patch), SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. All checks ran on 2026-11-15 through the resource and
storage guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit operator-kernel` | 739 cases valid |
| `check_syntax.py pa26 --unit operator-kernel` | 739 independent rustc checks agree (33.1 s) |
| `course.py pa26 --unit operator-kernel --bless` | 739/739 declared outcomes, then references written (22.0 s) |
| `check_layout.py --unit operator-kernel` | complete oracle provenance |
| `course.py pa26 --unit operator-kernel` | 739/739 against the committed text (20.6 s) |
| `check_native.py pa26 --unit operator-kernel` | 739 checks; 345 executable and 171 code-generation comparisons (36.4 s) |

Before the manifest was declared, a probe ran all 744 candidates through the
frontend, pinned rustc and the native backend with the same candidate. It found
the four rejected positives. A review of the rejection diagnostics found the
fifth entry. The first full generation was repeated after that case was
removed; no other reference changed. Receipts are under
`obj/publication/operator-kernel/`.

## Requirements

Requirements 26.2 and 26.4 are implemented components covered by this release.
Requirements 26.1 and 26.3 stay partial: their remaining source-method,
reference and coercion forms belong to the unreleased hook-closure unit. No
performance claim is made. Scheduled CI runs the unit through the
`--released-units` text, independent and native schedulers.
