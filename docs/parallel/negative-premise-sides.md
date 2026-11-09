# Negative coherence under the other header's premises

This is the Lane B batch of 2026-11-09, on origin/main `28e2b19fb`
(`dcb956cf8` plus Lane D's M and K and Lane C's R1). It
repairs the open item that the [MetaSized premises](negative-metasized.md)
batch recorded (its section 7): negative coherence paid a Sized debt from
both headers' premises, and `prove` assumed both headers' regions, outlives
and formed types. The repro was `sized-premise-side`, which rustc 1.98
rejects with E0119 and this compiler accepted:

```rust
pub trait Seal {} impl<T> !Seal for &T {}
pub trait Holds<X: ?Sized> {} impl<'a, V, S> Holds<&'a V> for S where &'a V: Seal {}
pub struct Local; impl<U: ?Sized> Holds<&U> for Local {}
```

## 1. Rule

`impl_intersection_has_negative_obligation`
(`rustc_trait_selection/src/traits/coherence.rs`) runs in both orders. Each
order assumes `param_env(impl1)` and tries to prove the negation of each of
impl2's elaborated predicates. In rustc 1.98.0 (`88d9e12ae`), under
`compiler/`:

- `coherence.rs:541-546` builds that `param_env` from impl1's arguments
  after equating the headers and plugging the unified variables with
  placeholders. A placeholder is Sized or `MetaSized` only through that
  `param_env`.
- `coherence.rs:698` resolves the negated obligation's regions with
  `resolve_regions(CRATE_DEF_ID, param_env, [])`, with no WF types.
- `regions.rs:33-48` keeps each type-outlives caller bound.
- `outlives/mod.rs:25-34` and `outlives/env.rs:76-84` add a region-region
  caller bound to the free-region relation only between `'static` or
  parameter regions. The impl's regions are still inference variables
  (`ReVar`), so the bound is skipped (FIXME #109628).

So a negated clause's own header pays none of its debts. Only the other
header's Sized and `MetaSized` premises and its written type-outlives
premises are assumed. Implied WF bounds and region-region premises never
are. The verdict does not depend on which header comes first.

## 2. What this compiler did

The before-runs (the `dcb956cf8` base CLI `05626ce0…`) found more than the
declared repro. Three declared "before" outcomes were wrong, and their cause
is a second leak:

- `freeze` (`overlap/inputs.rs`) makes the first header's binding the rigid
  representative of each unified parameter. A rigid parameter's sizedness
  is its representative's declaration (`types/parameters/shape.rs:285-290`,
  `types.is_sized`), which `builtins.prove` and the source sizing fallback
  read. So the first header's own `Sized` or `?Sized` decided whether a
  negated clause's Sized condition held. The union of both headers' Sized
  premises only paid the resulting debt.
- `sized-premise-side` was accepted while `sized-premise-side-swapped`, the
  same pair with the client first, was rejected. `sized-client-side`, which
  rustc accepts, was rejected: a rejects-valid on main.
- `prove` assumed every header's schema region bounds, written outlives and
  formed types, and an unpaid obligation was a hard error ("required
  lifetime does not outlive its target").

## 3. Change

- **Per-side Sized premises.** Each side keeps its own Sized premises
  (`sized_premises`): its Sized parameters, its Sized, Copy and Clone
  subjects, and the subjects of its source trait clauses whose sizing kind
  is `Sized`, written or supertrait-elaborated. So `U: Copy` gives `U:
  Sized` through `Clone: Sized`. The last part is new: source core lowers
  `Copy` and `Sized` as trait applications, which the union never
  collected. The `MetaSized` set of each side also takes them. After the
  pair is frozen, both sets of both sides are resolved once, sorted by
  identity and shared (`Premises`).
- **(f) Placeholder sizing.** A negative coherence query with premises
  (`NegativeSizing`, inside the existing query box) decides a rigid
  parameter's `Sized` and `MetaSized` only from the other side's sets,
  never from the representative's declaration. `PointeeSized` holds with no
  debt. A `Sized` decision stays an ordinary Sized debt; a `MetaSized`
  decision stays a `MetaSized` debt. There are three decision sites in
  `sema::traits::selection::search`:
  - the source sizing fallback (`sizing_fallback.rs`, `source_sizing_result`),
    which replaces `assumes_meta_sized`;
  - a `Bound::Sized` goal (`builtins.rs`, before `builtins.prove`);
  - an impl's `Bound::Sized` requirement (`frames.rs`, through
    `builtin_requirement`).
- **Payment.** `pay_negative` with the other side's `Premises` pays
  `MetaSized` debts from its `MetaSized` set and Sized debts from its Sized
  set. A rigid parameter's Sized debt is never paid by a shape proof; any
  other type keeps its shape proof. Then `prove(Some(side))` assumes only
  that side's written type-outlives premises: no formed types (decision
  (d)) and no schema region-region bounds (decision (e)). A region graph
  `Unsatisfied` or `Verification` result is "not disjoint" (decision (b)),
  matched before any text is made. Every other error and its text are
  unchanged, and `prove` keeps its `String` error type.
- **Unchanged.** `has_unsized` and the two single-header comparisons
  (`reference_comparison`, the iterator blanket) keep the union of Sized
  premises, formed types, schema regions and hard region errors. They pass
  no premises, so (f) does not apply to them. Stable-mode `may_overlap` is
  unchanged.
- **Other `retain_sized` sites.** `bound_frames.rs:44`, `conditions.rs:76`
  and `input_universe.rs:30` are not affected. Each returns an error for a
  quantified clause, quantified impl conditions or a universal-input query
  in a `retain_sized` query. Each does so before any frame, fold or scope
  exists, decides no sizedness and records no debt.

Shared files touched in `sema::traits`:

- `selection/search/sizing_fallback.rs`, `selection/search/builtins.rs` (one
  branch), `selection/search/frames.rs` (one call),
  `selection/search/coherence.rs` (`NegativePremises`),
  `selection/search.rs` and `selection.rs` (one re-export each);
- `implementations/overlap/inputs.rs`, `overlap.rs`, `overlap/iteration.rs`
  and `overlap/disjointness.rs` (one argument each in the last three).

## 4. Controls

`pa31/tests/spec/negative-premise-sides/` has twenty-one no_core libraries;
its README has the table. With `sized-premise-side` and `premise-side` of
`spec/negative-metasized/`, the twins cover:

- Sized, Copy and `MetaSized` premises on either side, with either header
  first;
- region-region, implied, `'static` and type-outlives obligations under
  `#[rustc_strict_coherence]`. Without that attribute, knowability alone
  makes those pairs disjoint in rustc.

Outcomes were declared before each twin's oracle and any RustGM run of it,
in five frozen declarations (`outcomes.json` gives their hashes). Every
after-outcome, the residual twins and the amendments were declared in the
fifth, before any code change.

| Kind | Twins |
| --- | --- |
| Accepted to rejected, as rustc rejects | `sized-premise-side`, `region-client-side`, `region-premise-side`, `region-implied`, `type-outlives`, `type-outlives-premise-side` |
| Rejected to accepted, as rustc accepts | `sized-client-side`, `copy-client-side` |
| Region text to overlap text | `region-absent`, `region-head-unified`, `strict-static-unheld` |
| Unchanged | the other twins, and the other twelve `negative-metasized` cases |

Candidate 1 (`9088d4b3…`, on `dcb956cf8`) gave every declared after-outcome.
After the rebase, the `28e2b19fb` base CLI (`d4cc6c92…`) gives the same
line for every twin as the `dcb956cf8` base did. Candidate 2 (`db7f1965…`,
on `28e2b19fb`) gives the same line as candidate 1. Between the two
`28e2b19fb` CLIs, only the declared rows of the table change.
Twenty-two of the twenty-three twins match rustc. The residual (section 6)
does not.

`dev/tests/invariants/coherence_premise_sides.rs` checks every case. Its
symmetry check covers the whole packet:

- every `X-swapped` twin has `X`'s outcome (four pairs, besides the
  residual pair; the test counts all five);
- every accepted case is disjoint with its `Holds` pair passed to
  `overlap::possible` in both orders.

The residual pair is exempt by name. The invariant's `RESIDUAL` constant is
`nested-premise-side`, and its comment cites the residual's text (section 6,
and `residual` in `outcomes.json`): `Wrap<V>: Sized` is still decided by the
shape walk, which reads the declaration of the representative that `freeze`
chose. The exempt pair's two orders must still differ. `nested-premise-side`
is disjoint with the blanket first and not with the client first, and the
twins are accepted and overlap. So a fix must update the record. The
[nested-sizedness item](nested-sizedness.md) did so and removed `RESIDUAL`.

## 5. Amendments

- **Strict correction** (before any RustGM run). The first five region
  twins never reached negative coherence in rustc. They were rewritten with
  `#[rustc_strict_coherence]`, and `region-strict-control` was added.
- **Wrong rustc predictions** (after oracle 2). `region-client-side` and
  `region-head-unified` were predicted accepted, and rustc rejects them. A
  region-region premise between two inference regions is never assumed
  (section 1), so decision (c), head-unified region equalities, is moot.
- **Wrong before predictions** (after the base runs). `copy-premise-side`,
  `sized-client-side` and `sized-premise-side-swapped` were declared
  accepted and were rejected with the overlap text. The cause is the
  representative's declaration (section 2). Each amendment is dated and
  sits beside its declaration in `outcomes.json`, with the observed text
  and rustc's code.
- **negative-metasized.** `sized-premise-side` is amended from accepted to
  the overlap text, and its invariant row with it.

## 6. Recorded gap: nested sizedness (fixed)

When this batch landed, a non-parameter type whose sizedness depends on a
rigid parameter was still decided by the shape walk, which read the
representative's declaration (`types/parameters/shape.rs:285-290` at
`7e8c5cbbf`). An example is a nominal's sizedness argument (`Wrap<V>: Sized`
for `struct Wrap<X: ?Sized>(X)`). `pay_negative`'s shape proof did the same.
So `nested-premise-side` (the blanket's Sized `V` first) was accepted, while
rustc rejects it with E0119, and `nested-premise-side-swapped` was rejected.

The [nested-sizedness item](nested-sizedness.md) fixes it. The negative
query's shape walk, size query and payment now follow `sized_conditions`
under the other header's Sized premises. Both twins are rejected, and the
symmetry invariant has no exemption. A projected-size nominal stays behind a
named fence, a known rejects-valid, until a later item threads the premises
into the nested trait proof.

## 7. Work

Each side's two sets are resolved, sorted and deduplicated once per pair.
They are charged for growth, the shared copy and the sort, as the
`MetaSized` set was. Every negative query with premises shares them by
`Arc` at one step, now even when both sets are empty, so that a rigid
parameter is never sized by declaration. Each (f) decision and each debt
payment is a binary search, charged `ilog2(n) + 1`. There is no per-goal
scan. `NegativeSizing` stays inside the existing query box, so the
`sharing_sizing` pin stays at `672 + size_of::<Vec<usize>>()` (696). The
disjointness scaling control and `negative_metasized_premises_cost_linear_pair_work`
pass unchanged (section 8).

## 8. Checks

These checks ran on `28e2b19fb` plus this batch, with
`make fastcheck`-equivalent runs. Every receipt passed with the working
tree's source fingerprint.

| Check | Selection | Receipt |
| --- | --- | --- |
| Private | 21 exact library cases: this invariant, the `negative-metasized` invariant and its linear-work control, the disjointness scaling control, `head_tests`, rejection, vacuous, coherence policy, pointer and cycle controls, and the sizing-sharing and layout pins (`sharing_sizing` at 696) | `1794204533945861336` |
| C2b, part 1 | 408 library cases | `1794204779549309272` |
| C2b, part 2 | 352 library cases | `1794204849741143111` |
| Traits roots, part 1 | 116 `suite_traits` cases | `1794204908551118044` |
| Traits roots, part 2 | 240 `suite_traits` cases | `1794204975553512037` |
| Crate linking, wire | 5 absence and modifier transport cases | `1794205133079178662` |
| Crate linking, native | 2 native client cases | `1794205180795840419` |
| Negative coherence, native | 1 runtime-selection case | `1794205224929645732` |
| PA20 | `--pa pa20`, 10 cases | `1794205268197278130` |

`make test-upstream` verified 128 pinned upstream fixtures and 126 Reference
files. `make test-harness` ran 313 script tests, the suite registration
check and the layout check (44 stages, no orphan fixtures). Both passed.

Before the rebase, the private selection also passed on `dcb956cf8`
(`1794202209837828990`). The first private run on `28e2b19fb` ran out of
budget in its cold strict-Clippy build. After the cache was rewarmed, one
attempt was refused admission while this lane's own harness run held the
shared lease. Neither attempt ran any test.

## 9. Flip survey

The change is active only in a two-header negative coherence pair. Every
other path passes no premises and keeps the union, formed types, schema
regions and hard errors. The survey ran the `28e2b19fb` base CLI and
candidate 2 over every fixture under `pa*/tests` that enables
`with_negative_coherence`, `negative_impls` or `rustc_strict_coherence` (336
files, besides the two packets). Each gave the same line for every file as
its `dcb956cf8` counterpart. There is no accepted-to-rejected flip. One
rejected case changes text, from the region text to the overlap text:
`pa31/tests/spec/negative-coherence/reference-static-unpaid.rs`. Its
negative impl needs `'a == 'static`, its contract pins only status 1, and
rustc rejects it with E0119.

## 10. Census lookahead

This is an analysis-only lookahead and not a qualification. It used census
19's manifest over the original core inputs (`88d9e12ae`), rebound only to
each pinned CLI. Both the `28e2b19fb` base CLI and candidate 2 exited 1 with
no infrastructure failure, unchanged inputs and 375 opened sources. Their
`census.tsv` files are byte-identical (sha256 `0e6b96ed…`). The first record
is still `validate_trait_implementations` at file 236:17456 ("trait
implementation does not satisfy the trait's requirements"), followed by the
same seven lookahead records. So the batch moves no census record.
