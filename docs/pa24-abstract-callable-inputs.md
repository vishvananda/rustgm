# Abstract callable inputs

Pinned core's `Iterator::max` 5777 (`iter/traits/iterator.rs:3255`) failed
with "trait obligation requires an unambiguous matching assumption or
implementation; diagnostic contract: diagnostic nested trait path formatting is
not implemented". So did `min`, `max_by_key` and `min_by_key`. `max` is
`self.max_by(Ord::cmp)`, where `max_by` requires
`F: FnMut(&Self::Item, &Self::Item) -> Ordering`. `max_by_key` passes a local
`fn compare<T, B: Ord>(..: &(B, T), ..: &(B, T)) -> Ordering` through `Map`.

## Diagnosis

Probes with crate-root traits (`obj/parallel/max-items/probes/README.md`) show
that the failure needs three things together:

- a function item whose generic input (`Ord::cmp`'s `Self`, or a free fn's `S`)
  is inferred only through a callable bound;
- a reference input in the item's signature;
- an abstract projection (`I::Item`, `Self::Item`) in the bound's input.

By-value inputs, type parameters, qualified paths such as
`<I::Item as Ord>::cmp`, and direct calls pass. Ordinary mode fails the same
way, without any source core.

Callable signature matching (`traits/callables.rs`) substitutes the item's
late-bound regions with fresh signature regions. It then equates the
signature's input tuple with the bound's through the scoped input worklist
(`equate_scoped_inputs`). The worklist binds the item's `S` to `I::Item`, then
blocks on two identical projection constructors, since projections are not
injective. Its fallback then required the complete input tuples to be
identical. `(&'r S)` and the bound's `(&'a I::Item)` differ only in the
signature's fresh region, so every such goal stayed deferred. The value case
passed because its complete tuples are identical.

## Change

- **Per-comparison identity.** `equate_scoped_inputs` now checks exact
  resolved identity for each blocked comparison, as the caller-premise path
  (`predicates/equations.rs`) already does. Equal complete applications are
  congruent. A blocked pair that still differs, or still has an unresolved
  input, waits as before. Nothing is inferred from a projection's result.
- **Regions.** The independent region equations recorded during the same walk
  stay the caller's lifetime obligations, exactly as on the old fallback path.
- **Single owner.** The whole-tuple fallback in `match_signature_as` is
  removed; the input equation owns the rule.

No new engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- Rejections keep the generic unambiguous-obligation text at the item path.
  rustc reports E0631 for a mismatched item and E0277 for a missing bound.
- The check compares resolved types. Two alpha-equivalent projections under
  different binder depths would still wait, as before.

## Reductions

`obj/parallel/max-items/core/declared.json` records two declared batches.
Rust outcomes were declared before rustc 1.98.0 and before any RustGM run of
each file; the RustGM outcomes are observations.

1. Eleven source-core reductions on the `default-const-closures` prefix, and
   seven inline ordinary forms.
2. Nine PA24 fixtures. They are the ordinary forms with executable checks, so
   their bytes differ from batch 1.

## Contracts

**PA24** (`pa24/tests/spec/abstract-callable-inputs/`, nine graded cases).

| Case | Form | Rust | 716cbe11 |
| --- | --- | --- | --- |
| `generic-item` | generic fn item, one reference input | accept, runs | reject |
| `unbounded-item` | unbounded generic fn item | accept, runs | reject |
| `two-reference-item` | two reference inputs | accept, runs | reject |
| `trait-member-item` | member path `Num::num`, as core passes `Ord::cmp` | accept, runs | reject |
| `default-method-item` | member path in a default method over `Self::Item` | accept, runs | reject |
| `ord-cmp` | std `Ord::cmp` | accept, runs | reject |
| `parameter-item` | type parameter input, a control | accept, runs | accept |
| `mismatch` | concrete `u16` item for `I::Item` | E0631 | reject |
| `distinct` | one item for `I::Item` and `J::Item` | E0631 | reject |

**PA31** (`pa31/tests/spec/abstract-callable-inputs/`, eleven source-core
reductions). They pass rustc 1.98.0 11/11 (`outcomes.json`).

| Case | Form | Rust | 716cbe11 |
| --- | --- | --- | --- |
| `max` | core's form | accept | reject |
| `max-call` | the same with a concrete iterator from a root `main` | accept | reject |
| `max-by-key` | core's local `compare` through `Map` and an opaque `key` | accept | reject |
| `projection-item` | free `Ord::cmp` for `I::Item` | accept | reject |
| `generic-item` | generic local fn item | accept | reject |
| `unbounded-item` | unbounded local fn item, one reference | accept | reject |
| `parameter-item` | type parameter input, a control | accept | accept |
| `qualified-item` | `<I::Item as Ord>::cmp`, a control | accept | accept |
| `max-missing` | `max` without `Self::Item: Ord` | E0277 | reject |
| `projection-mismatch` | concrete item for `I::Item` | E0631 | reject |
| `projection-distinct` | one item for two projections | E0631 | reject |

## Qualification

Focused receipt 1794491986552426026 passes in 136.3s under external load near
55. It covers formatting, the architecture audit, strict Clippy for the
projections and closures suites and the CLI, and thirteen exact tests:

- the three new tests: PA24 fixtures through rustc qualification and LowIR
  instance building, the PA31 packet through the CLI, and bounded work;
- the projected callable argument outcomes;
- six projected callback-result tests, which use the same signature equations;
- callable alias normalization;
- both source const callable packet tests.

Text qualification (`obj/parallel/max-items/text-review/passed.json`) uses
exploratory candidate ead8009c, built from this checkpoint's exact compiler
tree:

- Every PA24 positive crosses the independent Rust/CPPGM object bridge, and
  both rejections are independently rejected by Rust. Before generation, the
  locked CPPGM backend validated, compiled and ran every positive
  (`core/pa24/native.json`).
- All 700 prior PA24 cases reproduce, as do PA1–PA23 and the released PA25
  const-machine unit. All 28,381 prior reference hashes are unchanged.
- Normal scoped generation then adds nine reviewed triples, in batch
  `pa24/reference-batches/1794492319042947748-d8bde8227435.json`.

PA24 now has 709 cases (509 accept). No planned assignment is activated.

Bounded work (`obj/parallel/max-items/scaling-work.txt`), for 8/32/128
functions passing `Num::num` (member) or a generic `get` (generic) for an
`I::Item` input:

| Uses | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 18204 | 46 | 74 |
| 32 | 72060 | 46 | 290 |
| 128 | 287484 | 46 | 1154 |

Both forms do the same work, apart from two more canonical steps for the
generic form. That is about 2244 inference steps and 9 trait selections per
use. Adding 64 unrelated `Num` impls adds a constant 320 inference and 944
canonical steps at every width, and no trait selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/max-items/wip2-dev-fast`
(ead8009c) is built from this checkpoint's exact compiler tree; wip1 was
byte-identical. The unchanged 120s observer ends at its deadline after 121.155s,
with 22 partial records and 14,277 attempts. The last item is 8794,
str/pattern.rs:501. Against clean 716cbe11 (32 records, 15,395 attempts, last
item 10644) it clears `max` 5777, `min` 5778, `max_by_key` 5779 and
`min_by_key` 5784. No new failure appears among previously passing reached
bodies.

The first remaining failure is `read_radix_max_digits` 6024 at
net/parser.rs:117 ("this typed capture use is not implemented"). Next come
`Pin`'s `PartialEq` and `PartialOrd` forwarding bodies 6456–6464 at
pin.rs:1108, which stop at "associated type projection requires
normalization".

**Clean pin.** Pin ead8009c is built from clean source a7ce2784a, with 3915
Rust hashes and an empty patch (`obj/parallel/max-items/candidate-dev-fast`).
Only test and documentation files changed after the exploratory build, so it is
byte-identical to wip2. Its unchanged observation ends at its deadline after
121.104s, with 20 partial records and 14,011 attempts. The last item is 8344,
slice/mod.rs:2768. External load stayed near a load average of 22–27. Against
clean 716cbe11 it clears the four bodies above, with no new failure among
previously passing reached bodies, and the first remaining failure is
`read_radix_max_digits` 6024. The postcheck verifies all 722 source hashes, 375
opens, six configuration bindings, unchanged immutable bindings and the 3915
head blobs. The progress SHA is
`5612831617c31158a5bbe531c5ad63b6b8d613bb2ff417279eeac52a1988ab91`.

**Measurement.** Ninety exclusive A/A plus ABBA accepting observations against
716cbe11 preserve complete outputs. The controls are the previous checkpoint's
seven, plus `parameter-item` from each new packet. Paired wall ratios are
1.011/1.075/0.962/0.990/1.002/1.071/1.021/1.061/0.997, against A/A
0.843/1.138/1.050/0.922/0.857/1.011/0.948/0.887/1.025, at 11–810 ms.

User-space instruction counts (`instructions.txt`, three runs per pin) differ
by −0.007% for `trait-blanket` and +0.004% for the PA24 `parameter-item`. Both
use callable signature matching. `count-plain` runs 0.11% fewer instructions.
The two earlier attempts stopped at the script's frozen-input assertions before
timing anything: both new controls are named `parameter-item.rs`, in
same-named PA24 and PA31 directories (`measure-aborted-1`, `measure-aborted-2`).
No optimized, neutrality or original-core speed is claimed; the bounded-work
test above qualifies the work.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
