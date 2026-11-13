# Branch unsizing

Pinned core's `Split::as_slice` 7874 (`slice/iter.rs:427`) and
`SplitInclusive::next_back` 7900 (`slice/iter.rs:646`) failed with
"incompatible types". Both choose between an empty array and a slice:

- `as_slice` returns `&'a [T]` from
  `if self.finished { &[] } else { &self.v }`. `&self.v` is a `&&'a [T]`.
- `next_back` binds `let remainder = if self.finished { &[] } else { ... }`
  with no annotation, and the other branch is a `&'a [T]`.

## Diagnosis

rustc's `CoerceMany` gives an `if` or `match` one type:

- With a known expectation, each branch coerces to it.
- Without one, the first branch sets the merged type. A later branch first
  coerces to the merged type. If that fails, rustc coerces the earlier branches
  to the later branch's type, and then tries a least upper bound.

RustGM used an expected reference as the common type only for `dyn` and
nominal storage-tail referents. Other expected references, including `&[T]`,
fell through to an ordinary join. That join coerced the later reference to the
earlier one unless depth or mutability chose the other way. `&[T]` cannot
coerce to `&[_; 0]`, so both core forms failed. So did any annotated
`let r: &[u8] = if c { &[1] } else { &[1, 2] }`.

`obj/parallel/branch-unsize/core/declared.json` records three declared
batches. 13 of the 14 declarations agree with rustc. `adjusted-earlier` was
declared rejected, on the reading that rustc refuses to coerce earlier
branches that already took a `&mut`-to-`&` adjustment. rustc accepts it, and
its outcome is the oracle. Batch 1 ran rustc and the clean destructuring
candidate (2ddb3a67) before any implementation. That candidate accepts only
`lub-earlier`, where the later branch already unsizes to the merged type.

## Change

- **Expected slices.** An `if` or `match` with an expected reference to a
  slice uses that expectation as its common type, as it already did for `dyn`
  and nominal tails. Each branch coerces to it, so `&[]` unsizes and `&self.v`
  dereferences, as in rustc.
- **Joins without an expectation.** When two references have the same depth
  and mutability, an earlier array referent and a later slice referent join
  at the slice. The earlier branch unsizes to it. Every other join keeps its
  existing direction.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- The join keeps one fold over the branch types. It does not track each
  earlier branch's adjustment, as `CoerceMany` does before it coerces earlier
  branches to a later type. Batch 3's `adjusted-earlier` shows rustc
  accepting the one disagreement the declaration predicted, and no other
  disagreement is known.
- Only array-to-slice referents take the earlier-to-later direction. An earlier
  `&S<[T; N]>` with a later `&S<[T]>`, or an earlier `&T` with a later
  `&dyn Trait`, without an expectation, still fails closed with "incompatible
  types". rustc unsizes both. Core's reached failures use neither form.
- Each branch now relates directly to the expectation's regions, instead of
  first joining at fresh regions. That is rustc's relation.

## Contract

**PA31** (`pa31/tests/spec/branch-unsizing/`, fourteen source-core
reductions). RustGM agrees with rustc 1.98.0 on all 14 (`outcomes.json`). The base,
clean candidate 2ddb3a67, rejects every case except `lub-earlier` and the
three negatives with "incompatible types".

| Case | Form | Rust | Expected type |
| --- | --- | --- | --- |
| `core-as-slice` | core's `as_slice` | accept | `&'a [T]` |
| `ret-expected` | `if c { &[] } else { v }` | accept | `&'a [T]` |
| `let-annotated` | `&[1]` and `&[1, 2]` | accept | `&[u8]` |
| `core-next-back` | core's `next_back` remainder | accept | none |
| `lub-later` | earlier `&[]`, later `v` | accept | none |
| `lub-earlier` | earlier `v`, later `&[]` | accept | none |
| `match-lub` | two arms | accept | none |
| `three-arms` | `&[]`, `v`, `&[]` | accept | none |
| `else-if` | `&[]`, `v`, `&[]` as an else-if chain | accept | none |
| `noop-earlier` | two arrays, then `v` | accept | none |
| `adjusted-earlier` | `&mut [u8; 2]`, `&[u8; 2]`, then `v` | accept | none |
| `lub-arrays` | `&[1u8]` and `&[1u8, 2]` | E0308 | none |
| `mut-slice-later` | `&[]` and a `&mut [T]` | E0308 | none |
| `lub-deref` | `&[]` and a `&&[T]` | E0308 | none |

## Qualification

Focused receipt 1794514585136884276 passes in 46.1s. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the
declarations and lowering suites and the CLI, plus three exact tests:

- the two new tests in `dev/tests/branch_unsizing.rs`. The first runs the
  packet's typing through the CLI, and runs prepared lowering for every case
  that types. The second checks bounded work.
- `body_types::control_flow_and_assignment_types_match_pinned_rustc`, for
  ordinary branch types.

Native receipt 1794514249352767503 passes in 40.0s with the CPPGM backend:

- the new ignored test, with four ordinary-mode programs that rustc also
  runs. Empty branches have length 0, and selected slices keep their own
  elements: an expected result, a later slice, three match arms, and core's
  `as_slice` form.
- `tail_coercions::structural_unsizing_preserves_fields_metadata_and_reborrows`,
  for the existing nominal and structural unsizing.

Receipt 1794514201726839124 passed the same selection before batch 3 added two
cases. Earlier attempts 1794514034907377185 and 1794514058232163099 used
unknown selection keys, and 1794514087444823250 selected an ignored test
without `--ignored`. `make test-upstream` is not rerun: no upstream fixture changed.
No graded case or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/branch-unsize/scaling-raw.txt`), for 8/32/128
generic functions, each joining `&[]` with a slice:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 6369 | 2096 | 55 |
| 32 | 18561 | 2240 | 103 |
| 128 | 67329 | 2816 | 295 |

That is 508 inference steps, 6 canonical steps and 2 trait selections per
function, linear across widths.

## Core progress

Clean candidate2 (`obj/parallel/capture-match/candidate2-dev-fast`, 24b7ce91)
is built from e832ec039 with an empty patch. The unchanged 120s observer ran it
exclusively from load 11.8. It stopped at the deadline (121.07s) after 15,111
item attempts, at item 10246 (core_arch `x86/sse41.rs:83`), with one record. It
reached 7874 and 7900, and both pass. No previously passing reached body
failed. The only remaining failure is `as_underlying` 7631 (`slice/cmp.rs:52`,
"source transmute requires equal source and destination sizes"). The census is
incomplete, and no core artifact exists. The progress SHA-256 is
`4e581b3b30081f4fb55d93357da98575d378cac076d65df3e0a0c65f3a229424`, and the
comparison is `obj/parallel/capture-match/original-candidate2-comparison.json`.
