# Always-const premises

Pinned core's two `core_arch` `simd.rs` `eq` bodies, 9611 and 9643
(`simd.rs:91` and `:304`), failed with "trait obligation requires an
unambiguous matching assumption or implementation":

```rust
pub(crate) unsafe trait SimdElement:
    Copy + const PartialEq + crate::fmt::Debug
// ...
const impl<T: SimdElement, const N: usize> crate::cmp::PartialEq for Simd<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_array() == other.as_array()
    }
}
```

The comparison needs `&[T; N]: [const] PartialEq`. The reference and array
impls reduce that to `T: [const] PartialEq`, and the only premise is
`T: const PartialEq`, elaborated from `SimdElement`'s supertrait.

## Diagnosis

Supertrait elaboration keeps an always-const supertrait's effect, so the body
environment does hold `T: const PartialEq`. Assumption matching
(`equate_trait_inputs` in `types/infer/predicates/equations.rs`) then required
the premise's effect to equal the goal's, unless the goal was ordinary. A
`const` premise therefore never proved a `[const]` goal.

`obj/parallel/simd-eq/core/declared.json` records five declared batches.
Batches 1 and 2 were malformed: stability attributes, then
`const_trait_impl` exposure. Batch 4 was three ungraded diagnostics that
separated the two candidate causes:

- one `T` element compared under `SimdElement` failed (`can't compare`);
- arrays under a direct `T: const PartialEq` bound failed (unambiguous
  matching);
- arrays under `T: [const] PartialEq` passed.

rustc's host effect matching (`BoundConstness::satisfies`) lets a `const`
clause satisfy both `const` and `[const]` goals. A `[const]` clause satisfies
only `[const]`.

## Change

- **Effect matching.** `Constness::proves` states which effects a premise
  proves. Every premise proves an ordinary requirement, a `[const]` premise
  also proves a `[const]` requirement, and a `const` premise proves all three.
  Assumption matching uses it. Impl selection already adopted the requested
  effect for an actual `const impl`, so this makes premises agree with impls.
- **Ambiguity.** A `[const]` goal with both a `const` and a `[const]` premise
  for the same inputs now matches both, and selection stays ambiguous, as in
  rustc (`both-premises`, E0284). The base accepted that case. Ordinary goals
  still share identical premises regardless of effect.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Only the premise effect order changes. A scoped equation between two goals
  (`Relation::Scoped`) still requires equal effects, because it relates two
  requirements rather than proving one from a premise.
- Two prepared lowering gaps remain. They reject accepted cases, and the base
  rejects the same cases there. Core's census checks typing, so these matter
  for the later core build:
  - A const body's trait member call whose `Self` an impl proves stops at
    "const function member lacks its current conditional trait premise". An
    example is `[T; N]: [const] PartialEq` through the array impl. The
    defining-call check accepts only a premise written in the environment,
    so `T: [const] PartialEq` with an array comparison (diagnostic
    `d-array-maybe-bound`) fails on the base too. Core's form now types and
    stops here.
  - `a == b` on two `&T` operands, even in an ordinary function, stops at
    "method borrow differs from selected receiver place": the operand and
    receiver references carry different regions.
- RustGM's rejection messages differ from rustc's E0277 and E0284 text.

## Contract

**PA31** (`pa31/tests/spec/always-const-premises/`, 18 source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`).
Seven come from batch 3 and eleven from batch 5. Batch 5 was declared after
this change was written and before it was built. The base is candidate
15c0f4ac (d901f562c).

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `core-form` | stdarch's `Simd` `eq`, `&[T; N] == &[T; N]` | accept | typing | lowering gap |
| `direct-call` | `PartialEq::eq` on the arrays | accept | typing | lowering gap |
| `element-fn` | a generic const fn compares the arrays | accept | typing | lowering gap |
| `concrete-use` | `Simd<u8, 4>` compared in a const fn | accept | typing | lowering gap |
| `const-impl-supertrait` | `W<T>`'s impl with premise `T: const PartialEq`, under `SimdElement` | accept | typing | lowering gap |
| `scalar-refs-op` | `a == b` on `&T` under `SimdElement` | accept | typing | lowering gap |
| `scalar-op` | `*a == *b` under `SimdElement` | accept | typing | accept |
| `transitive` | the supertrait through `trait Lane: SimdElement` | accept | typing | accept |
| `const-goal-const-premise` | `needs_const::<T>()` under `T: const PartialEq` | accept | accept | accept |
| `const-goal-supertrait` | `needs_const::<T>()` under `SimdElement` | accept | accept | accept |
| `ordinary-impl` | an ordinary impl | accept | accept | accept |
| `maybe-supertrait` | a `[const]` supertrait under `[const] SimdElement` | accept | accept | lowering gap |
| `ordinary-fn-supertrait` | `a == b` on `&T` in an ordinary function | accept | accept | lowering gap |
| `plain-supertrait` | an ordinary `PartialEq` supertrait | E0277 | typing | typing |
| `maybe-plain-premise` | a `[const]` supertrait under plain `SimdElement` | E0277 | typing | typing |
| `const-goal-maybe-premise` | `needs_const::<T>()` under `T: [const] PartialEq` | E0277 | typing | typing |
| `const-impl-maybe-premise` | `W<T>`'s impl under `T: [const] PartialEq` | E0277 | typing | typing |
| `both-premises` | `T: SimdElement + [const] PartialEq` | E0284 | accept | typing |

`maybe-supertrait` and `ordinary-fn-supertrait` already typed on the base, and
their lowering gaps are the base's. `const-impl-maybe-premise` now fails at
its caller. On the base it failed earlier, inside `W`'s impl body, whose
`const` premise could not prove the body's `[const]` comparison.

## Qualification

Three focused receipts pass with exact selections. The first covers assignment
metadata, formatting, the architecture audit and strict Clippy for the traits
suite and the CLI.

- 1794527575313737516 (45.1s), on the final tree: the two new tests in
  `dev/tests/supertrait_const_premises.rs`:
  - The first runs the packet's typing through the CLI and prepared lowering
    for every accepted case. It requires the exact reason for each rejection
    and each recorded lowering gap.
  - The second checks bounded work and lowers every width.
- 1794527406851305105 (77.7s): the `source_closures` module (48 tests) and 8
  library tests:
  - the new `always_const_premises_prove_conditional_goals`, which checks the
    full premise-by-goal effect table;
  - `ordinary_duplicate_effects_do_not_pay_const_obligations`;
  - the supertrait effect, elaborated effect bound, const capability (three)
    and host effect specialization invariants.
- 1794527154332758446 (86.4s): the `provider_traits`, `pointer_coercions` and
  `const_functions` modules, 62 tests.

Earlier attempts:

- 1794526197193064350 failed formatting.
- 1794526244341302855 and 1794526533485483326 exhausted the budget under load
  89, in private Clippy and then the build.
- 1794526817726739450 found the eight lowering failures. Its eight library
  tests passed. Base runs showed the gaps are pre-existing, and they are now
  recorded as exact reasons.
- 1794527093305490448 (51.1s) passed the packet tests before the
  documentation changes.
- 1794527247913034516 passed `source_closures`, but its `--pa pa25` selection
  was refused because PA25 is a planned assignment.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated. There is no
native test: stable ordinary-mode programs cannot write const trait bounds.

Bounded work (`obj/parallel/simd-eq/scaling-raw.txt`), for 8/32/128 const
functions, each comparing `*a == *b` under `T: SimdElement`:

| Functions | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 31680 | 6627 | 430 |
| 32 | 36672 | 7971 | 526 |
| 128 | 56640 | 13347 | 910 |

That is 208 inference steps, 56 canonical steps and 4 trait selections per
function, linear across widths.

## Core progress

Clean candidate2 (`obj/parallel/capture-match/candidate2-dev-fast`, 24b7ce91)
is built from e832ec039 with an empty patch. The unchanged 120s observer ran it
exclusively from load 11.8. It stopped at the deadline (121.07s) after 15,111
item attempts, at item 10246 (core_arch `x86/sse41.rs:83`), with one record. It
reached 9611 and 9643, and both pass the typing census. Both still stop at the
first lowering gap above, which blocks the later core build. No previously
passing reached body failed. The only remaining failure is `as_underlying` 7631
(`slice/cmp.rs:52`, "source transmute requires equal source and destination
sizes"). The census is incomplete, and no core artifact exists. The progress
SHA-256 is `4e581b3b30081f4fb55d93357da98575d378cac076d65df3e0a0c65f3a229424`,
and the comparison is
`obj/parallel/capture-match/original-candidate2-comparison.json`.
