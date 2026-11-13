# GAT region premises

Pinned core's `SplitInclusive` `clone` 8624 (`str/iter.rs:1438`) failed with
"trait obligation requires an unambiguous matching assumption or
implementation" at `self.0.clone()`:

```rust
impl<'a, P> Clone for SplitInternal<'a, P>
where
    P: Pattern<Searcher<'a>: Clone>,
// ...
impl<'a, P: Pattern<Searcher<'a>: Clone>> Clone for SplitInclusive<'a, P> {
    fn clone(&self) -> Self {
        SplitInclusive(self.0.clone())
    }
}
```

Selecting `SplitInternal`'s impl gives its `'a` a fresh instance region `'r`.
The nested goal `<P as Pattern>::Searcher<'r>: Clone` differs from the body's
premise `<P as Pattern>::Searcher<'a>: Clone` only in that region.

## Diagnosis

`obj/parallel/pattern-clone/core/declared.json` records four declared batches.
Batch 1 was malformed: the prefix has no `Copy` for `bool`. Batch 3 was three
ungraded diagnostics. A non-GAT associated type and a lifetime-generic struct
with a non-GAT bound passed. So did the GAT instantiated only at `'static`
(`d-gat-static`).

Two debug pins (`wipdbg3`, `wipdbg4`, never committed) traced the cause:

- Selection relates the two projections by their exact type identity unless
  the premise is certified normal. Certification is census batch 17's
  [certified premise](parallel/census-batch17.md) mechanism (044455065). Only a
  certified premise relates a rigid goal's projection inputs invariantly,
  leaving their lifetimes to the region proof.
- The impl owner's environment does certify `Searcher<'a>: Clone`. The method
  body then opens the impl's `'a` at a universal alias, and
  `Environment::opened_certificates` carries certificates through that
  lifetime-only substitution only when every opened source is one of the
  declaration's own trait premises.
- One opened source is `<P as Pattern>::Searcher<'a>: Sized`, elaborated from
  `Clone: Sized`. The declaration holds it as a shape assumption, not a trait
  premise, so no certificate was carried. With `'static` nothing is opened,
  which is why `d-gat-static` passed.

The uncertified premise then failed the exact compare. The goal was deferred
for normalization, and no wakeup came, since its type inputs were complete.

## Change

- **Assumed `Sized` sources.** `opened_certificates` also accepts a `Sized`
  source whose subject the declaration assumes. Such a source carries no
  certificate itself, so only the declaration's certified trait premises are
  carried. A source the declaration does not hold still withholds every
  certificate.
- The opened premise is then certified, and the existing invariant relation
  records `'r == 'a` as a region equality. Prepared lowering's lifetime proof
  checks it. A premise for another lifetime types the same way, and the proof
  rejects it, as rustc's borrow checker does.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- An exploratory review proposed relating any uncertified rigid pair that
  differs only in regions. That would cover environments certification never
  reaches, but it would bypass the certificate's normal-form guarantee. This
  change keeps the certificate gate and repairs only the lost carry.
- Other builtin sources, such as auto traits, still withhold certificates. No
  reduction needed them.
- rustc rejects `other-lifetime`, `static-premise` and `outlives-only` in its
  borrow checker ("lifetime may not live long enough"). RustGM rejects them in
  prepared lowering ("required lifetime does not outlive its target"). The
  base already rejected `static-premise` there.

## Contract

**PA31** (`pa31/tests/spec/gat-region-premises/`, 15 source-core reductions).
All declarations agree with rustc 1.98.0 (`outcomes.json`). Six come from
batch 2 and nine from batch 4. Batch 4 was declared after this change was
built and run on batches 2 and 3. The base is candidate 15c0f4ac (d901f562c).

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `core-form` | core's `SplitInclusive` `Clone` | accept | typing | accept |
| `path-call` | `Clone::clone(&self.0)` | accept | typing | accept |
| `where-projection` | the bound as `P::Searcher<'a>: Clone` | accept | typing | accept |
| `local-value` | a free function clones a `SplitInternal` | accept | typing | accept |
| `copy-premise` | a `Searcher<'a>: Copy` bound | accept | typing | accept |
| `wrapper-chain` | a second wrapper's `Clone` | accept | typing | accept |
| `mutual-outlives` | the premise for `'b` with `'a: 'b, 'b: 'a` | accept | typing | accept |
| `static-use` | a `'static` premise for a `'static` split | accept | accept | accept |
| `searcher-method` | the GAT's `Searcher<'a>` item bound | accept | accept | accept |
| `direct-field` | `s.matcher.clone()` under its own premise | accept | accept | accept |
| `missing-bound` | no associated type bound | E0599 | typing | typing |
| `wrapper-missing-bound` | the wrapper lacks the bound | E0599 | typing | typing |
| `other-lifetime` | the premise is for an unrelated `'b` | borrowck | typing | lifetime proof |
| `static-premise` | a `'static` premise for a generic `'a` | borrowck | lifetime proof | lifetime proof |
| `outlives-only` | the premise for `'b` with only `'b: 'a` | borrowck | typing | lifetime proof |

The base rejected `wrapper-missing-bound` inside `SplitInclusive`'s impl body,
where core's form fails. RustGM now rejects it at the wrapper ("no applicable
method for this receiver"), as rustc does.

## Qualification

Four focused receipts pass with exact selections. The first and the last also
cover assignment metadata, formatting, the architecture audit and strict
Clippy for the projections suite and the CLI.

- 1794528536476105671 (58.5s), fourteen tests and PA21's text oracle:
  - the two new tests in `dev/tests/gat_region_premises.rs`. The first runs
    the packet's typing through the CLI and prepared lowering for every case
    that types. It requires the exact reason and phase of each rejection.
    The second checks bounded work and lowers every width.
  - the new `opened_clauses_admit_assumed_sized_sources`, beside the existing
    `opened_clauses_carry_only_held_certified_sources` and
    `certified_premise_relates_rigid_goal_inputs_as_region_equalities`;
  - seven certified premise invariants, including the declared outcome,
    elaboration, copy and member contract tests;
  - two defining environment tests;
  - `--pa pa21`, whose certified premise text oracle passes unchanged.
- 1794528601774567218 (47.3s): eight projections-suite modules, 28 tests.
- 1794528654079067290 (135.9s): five traits-suite modules and the
  `source_closures` and `closure_lifetimes` modules, 192 tests.
- 1794528883759076585 (52.6s): the two packet tests again, on the final tree.

An earlier attempt, 1794528290016061242, named a wrong private module for two
library tests and stopped before running.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated. There is no
native test, because stable ordinary-mode programs cannot write the
associated type bound.

Bounded work (`obj/parallel/pattern-clone/scaling-raw.txt`), for 8/32/128 free
functions, each cloning a `SplitInternal` under its own premise:

| Functions | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 34285 | 7872 | 402 |
| 32 | 48733 | 12120 | 474 |
| 128 | 106525 | 29676 | 762 |

That is 602 inference steps, 182 canonical steps and 3 trait selections per
function, linear across widths.

## Core progress

Clean candidate2 (`obj/parallel/capture-match/candidate2-dev-fast`, 24b7ce91)
is built from e832ec039 with an empty patch. The unchanged 120s observer ran it
exclusively from load 11.8. It stopped at the deadline (121.07s) after 15,111
item attempts, at item 10246 (core_arch `x86/sse41.rs:83`), with one record. It
reached 8624, which passes. No previously passing reached body failed. The only
remaining failure is `as_underlying` 7631 (`slice/cmp.rs:52`, "source transmute
requires equal source and destination sizes"). The census is incomplete, and no
core artifact exists. The progress SHA-256 is
`4e581b3b30081f4fb55d93357da98575d378cac076d65df3e0a0c65f3a229424`, and the
comparison is `obj/parallel/capture-match/original-candidate2-comparison.json`.
