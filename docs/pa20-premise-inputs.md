# Premise-selected inputs

Pinned core's `Pin` comparison forwarding bodies 6456–6464 (`pin.rs:1108`,
`eq`, `ne`, `partial_cmp`, `lt`, `le`, `gt` and `ge`) failed with "associated
type projection requires normalization". Each forwards with
`Ptr::Target::eq(self, other)` under `Ptr::Target: PartialEq<Q::Target>`. The
path call leaves the comparison's `Rhs` to inference. `other: &Pin<Q>` then
deref-coerces to `&Q::Target`.

## Diagnosis

`obj/parallel/pin-forwarding/core/declared.json` records five declared
source-core batches on the `source-div` foundation prefix with a reduced
`Deref` and `Pin`. They show the following:

- rustc 1.98.0 reports an internal compiler error (`fully_perform`) for the
  accepted forms with a reduced `#[lang = "eq"]` trait, so those are not
  reductions. A non-lang trait reproduces the failure exactly.
- Naming `Rhs` (`<Ptr::Target as E<Q::Target>>::e`), annotating the coerced
  operand, or using the default `Rhs` passes. So does the same path call in a
  free function over annotated parameters, even beside competing impls.
- The call fails in any impl or free function whose `Rhs` argument has an
  overloaded-deref or coerced type, by path or by method call.

Ordinary mode reproduces it without `Deref`. `X::Item::e(a.get(), &b.get())`
under `X::Item: E<Y::Item>` fails, and so does the exact-reference form
(`obj/parallel/pin-forwarding/ordinary/`).

A temporary instrumented build located the two places that failed:

1. `prepare_call_argument` refines the call's goals for an early expectation
   when the parameter is `&?R`. Selecting the premise binds `?R` to its own
   projection `Q::Target`. The refinement loop's `head` check then turned that
   projection, which no owner had normalized, into a hard error.
2. Past that check, reference coercion (`reference_conversion`) read the same
   referent's head to choose dereferences, and failed the same way.

A projection in an instantiated signature reaches its normalizer when it is
instantiated. This one enters the call's inputs only through the premise's
binding, after instantiation.

## Change

- **Early expectation.** The refinement loop reads the frontier with the
  existing `contextual_head`. A projection that waits for normalization has no
  head yet, so the loop continues. The argument's ordinary check and coercion
  still run.
- **Coercion target.** `reference_conversion` normalizes its referent target on
  demand, through the existing `defer_normalization`, only when that head waits
  for normalization. Every other target keeps its owner.

Both changes apply only where the old code returned a hard error. No new
engine, model field, backend, storage policy or compiler limit is added.

Rejected designs (exploratory evidence under `obj/parallel/pin-forwarding/`):

- Normalizing every input of a premise-selected goal at selection time (wip3)
  cleared the forwarding bodies but regressed core's `array::from_fn`,
  `GenericShunt::try_fold` and `try_find`'s `check`.
- Restricting that to trait arguments whose head waits for normalization (wip4)
  still regressed `try_fold` and `check`. Their elaborated `Try` residual
  inputs relied on staying unnormalized.

Tradeoffs:

- Other consumers that read the head of a premise-bound projection still
  error. None is reached in core's observed prefix.
- The `missing` rejections keep RustGM's existing texts. rustc reports E0308
  (ordinary) and E0599 (source core).
- Source-core mode accepts copying a `&u8` field out of a shared reference with
  no `Copy for &T` impl in the prefix, where rustc reports E0507
  (`coerce-concrete`). That predates this change and is recorded, not fixed.

## Contracts

**PA20** (`pa20/tests/spec/premise-inputs/`, seven graded cases).

| Case | Form | Rust | 59a51ffa |
| --- | --- | --- | --- |
| `direct` | exact `&Y::Item` argument, premise-selected `Rhs` | accept, runs | reject |
| `double-reference` | a `&&Y::Item` argument that coerces | accept, runs | reject |
| `method` | member call on the receiver | accept, runs | reject |
| `impl-forwarding` | core's forwarding impl over `It` | accept, runs | reject |
| `qualified` | `<X::Item as E<Y::Item>>::e`, a control | accept, runs | accept |
| `annotated` | the coerced operand annotated first, a control | accept, runs | accept |
| `missing` | a premise naming another `Rhs` | E0308 | reject |

**PA31** (`pa31/tests/spec/premise-inputs/`, nine source-core reductions).
They pass rustc 1.98.0 9/9 (`outcomes.json`).

| Case | Form | Rust | 59a51ffa |
| --- | --- | --- | --- |
| `forwarding` | core's impl over a non-lang trait | accept | reject |
| `inherent` | the call from an inherent `Pin` method | accept | reject |
| `other-trait` | the call from an impl of another trait | accept | reject |
| `free-deref` | `&**b` as `Rhs` in a free function | accept | reject |
| `free-method` | the same by method call | accept | reject |
| `other-coerces` | only the `Rhs` argument coerces | accept | reject |
| `qualified` | named `Rhs`, a control | accept | accept |
| `coerce-return` | deref coercion to a signature projection, a control | accept | accept |
| `missing` | core's `PartialEq` form without the premise | E0599 | reject |

## Qualification

Focused receipt 1794498535115363418 passes in 88.4s. It covers formatting, the
architecture audit, strict Clippy for the projections, ownership and frontend
suites and the CLI, and eleven exact tests:

- the three new tests (`dev/tests/associated_declarations/forwarded_rhs.rs`):
  PA20 fixtures through rustc qualification and prepared lowering, the PA31
  packet through the CLI, and bounded work;
- rigid candidate headers, qualified-prefix argument expectations, and the
  source abstract callable input packet;
- three reborrow and reference-coercion contracts;
- two source `Deref` contract tests.

Two earlier attempts stopped at formatting and at the architecture audit. The
test's first name repeated the `premise` filename stem, so it was renamed.

Receipt 1794498808136641790 (53.3s) adds PA20's complete text oracle with the
dev-fast CLI to the three new tests. `make test-upstream` verifies the pinned
upstream fixtures.

Text qualification (`obj/parallel/pin-forwarding/text-review/passed.json`) uses
exploratory candidate b41f1792, built from this checkpoint's exact compiler
tree (wip6, byte-identical to wip5):

- Every PA20 positive crosses the independent Rust/CPPGM object bridge, and the
  rejection is independently rejected by Rust. Before generation, the locked
  CPPGM backend validated, compiled and ran every positive
  (`ordinary/native.json`).
- All 1,609 prior PA20 cases reproduce, as do PA1–PA19, PA21–PA24 and the
  released PA25 const-machine unit. All 28,431 prior reference hashes are
  unchanged.
- Normal scoped generation then adds seven reviewed triples, in batch
  `pa20/reference-batches/1794498755368398964-434c979db0e5.json`.

PA20 now has 1,616 cases (1,156 accept). No planned assignment is activated.

Bounded work (`obj/parallel/pin-forwarding/scaling-raw.txt`), for 8/32/128
functions whose premise selects the `Rhs` of `X::Item::e(a.get(), &b.get())`:

| Uses | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 6932 | 14 | 24 |
| 32 | 27716 | 38 | 96 |
| 128 | 110852 | 134 | 384 |

That is 866 inference steps, one canonical step and three trait selections per
use. Adding 64 unrelated `It` impls adds a constant 1,344 inference and 948
canonical steps at every width, and no trait selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/pin-forwarding/wip5-dev-fast`
(b41f1792) carries the change before the test and fixture work. The unchanged
120s observer ends at its deadline after 120.886s, with 20 partial records and
15,378 attempts. The last item is 10627, x86/avx2.rs:67. Against clean 59a51ffa
(24 records, 14,894 attempts, last item 9964) it clears all seven forwarding
bodies, 6456–6464. No new failure appears among previously passing reached
bodies.

The rejected designs' observations are kept for comparison. wip3 (all inputs at
selection) ended with 17 records and three regressions; wip4 (trait arguments
at selection) ended with 16 records and two.

The first remaining failure is `Result<&mut T, E>::cloned` 6721 at
result.rs:1795 ("invalid method trait selection"). Its `t.clone()` on `&mut T`
meets core's negative `impl !Clone for &mut T` before autoderef reaches
`T: Clone`.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
