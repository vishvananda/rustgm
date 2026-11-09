# Negative coherence with MetaSized premises

This is the Lane B batch of 2026-11-09, on origin/main `68c26f42a`, which
contains C3b (`aa6c36f65`). It repairs the recorded gap "negative coherence
over a `?Sized` negative impl" of
[library build blockers](../library-build-blockers.md):

```rust
pub trait Seal {} impl<T: ?Sized> !Seal for &T {}
pub trait Holds<X: ?Sized> {} impl<T: Seal + ?Sized, S> Holds<T> for S {}
pub struct Local; impl<U: ?Sized> Holds<&U> for Local {}
```

rustc 1.98.0 accepts it. This compiler rejected it with "overlapping trait
implementation headers require coherence proof".

## 1. Rule

rustc's `impl_intersection_has_negative_obligation`
(`rustc_trait_selection/src/traits/coherence.rs`) runs in both orders. Each
order assumes `param_env(impl1)` and tries to prove the negation of each of
impl2's elaborated predicates. Here it negates the blanket's `&U: Seal` under
the client's environment. The client's `U: ?Sized` gives `U: MetaSized`, so
`impl<T: ?Sized> !Seal for &T` applies and the impls are disjoint.

`pay_negative` (`overlap/inputs.rs`) paid only Sized debts, so `U: MetaSized`
on the rigid intersection parameter was never proved: the source sizing
fallback answers `MetaSized` on a parameter with absence.

## 2. Change

- **Sides.** Each header of a pair records its side. Each trait condition
  gets the side of the header it came from (`trait_sides`, reserved and
  charged with `reserve_traits`).
- **Premises.** Each side keeps the premises that imply `MetaSized`: its
  Sized parameters, its Sized, Copy and Clone subjects, and its `MetaSized`
  clauses, written or the default of a `?Sized` parameter. They are read by
  the same `declaration_inputs::all` reader as the conditions, with the
  perspective argument kept. After the pair is frozen, each side's set is
  resolved once, sorted by identity and shared.
- **Search.** A negative coherence query for a condition from one side
  assumes only the other side's set. The source sizing fallback proves
  `MetaSized` for a rigid parameter in that set. The proof node keeps the
  parameter as a debt, as a Sized debt is kept, so the node is never debt-free,
  never cached and never pays an equality-only negative. The query records
  the node. The premises and recorded nodes are one optional box on the
  query, so an ordinary query grows by one word; only negative coherence
  queries, which are cold, allocate it.
- **Payment.** The refutation moves the debts of recorded nodes into a
  separate `meta_sized` list (`collect_graph`), and `pay_negative` pays them
  only from the same set. Sized debts keep both headers' premises, as before.
- **Elaborated premises.** `Generics::elaborate_trait_bounds`
  (`generics/traits.rs`) already appends each owner's supertrait-implied
  predicates after its declared ones, and `declaration_inputs::all` returns
  them. So `U: PointeeSized + Foo` gives the premise `U: MetaSized` through
  Foo's implicit supertrait, as rustc's elaborated `param_env` does. An
  elaborated predicate is read under its owner's header, so it keeps that
  header's side.
- **Unchanged.** Stable-mode `may_overlap` and the two single-header
  comparisons (`reference_comparison`, the iterator blanket) assume no
  `MetaSized` premise. Sized debts are still paid from both headers' Sized
  premises (section 7).

Shared files touched in `sema::traits`: `selection/search/coherence.rs`,
`selection/search/sizing_fallback.rs`, `selection/search/continuation.rs`
(one query field), `selection/search.rs` (`collect_graph`),
`selection/search/conjunction.rs` (one argument),
`selection/search/refutations.rs`, `selection/refutation.rs`, and
`implementations/overlap.rs`, `overlap/inputs.rs` and `overlap/iteration.rs`.

## 3. Controls

`pa31/tests/spec/negative-metasized/` has thirteen no_core libraries. Each
is the map-adapter core prefix plus one case line. Outcomes for rustc 1.98.0
and for this compiler, before and after the change, were declared before each
twin's oracle and before any RustGM run of it (`outcomes.json`).

| Twin | Shape | rustc 1.98.0 | Before | After |
| --- | --- | --- | --- | --- |
| `metasized-param` | client `U: ?Sized` | accepted | overlap | accepted |
| `metasized-written` | client `U: MetaSized` | accepted | overlap | accepted |
| `metasized-strict` | `#[rustc_strict_coherence]` trait | accepted | overlap | accepted |
| `sized-param` | client `U` Sized | accepted | accepted | accepted |
| `pointee-param` | client `U: PointeeSized` | E0119 | overlap | overlap |
| `sized-negative` | `impl<T> !Seal for &T` | E0119 | overlap | overlap |
| `metasized-stable` | no `with_negative_coherence` | E0119 | overlap | overlap |
| `supertrait-implied` | client `U: PointeeSized + Foo` | accepted | overlap | accepted |
| `supertrait-premise-side` | blanket `V: PointeeSized + Foo` | E0119 | not run | overlap |
| `edge-pointee` | specializing `Cond`, client `PointeeSized` | accepted | accepted | accepted |
| `edge-specialized` | specializing `Cond`, client `?Sized` | accepted | accepted | accepted |
| `premise-side` | blanket's own `V: ?Sized` | E0119 | overlap | overlap |
| `sized-premise-side` | blanket's own Sized `V` | E0119 | accepted | accepted |

"Overlap" is "overlapping trait implementation headers require coherence
proof". `edge-specialized` was a declared observation: its acceptance must
not flip, and it did not. `sized-premise-side` is the recorded accepts-invalid
of section 7.

`supertrait-premise-side` was declared, with rustc's predicted E0119, after
candidate run 1 and before its oracle. The base CLI had been retired by then,
so it has no "before" run. It checks that an elaborated premise keeps its
owner's side: the blanket's own `V: PointeeSized + Foo` must not pay its own
clause's debt.

## 4. Amendments

- **Fixture corrections** (before any RustGM run). rustc rejected four twins
  for the fixture itself: E0277 and min_specialization's item rule in the
  edge twins, E0637 in the side twins. They were rewritten, and no outcome
  changed.
- **A wrong prediction** (after candidate run 1, CLI `c87ef088…`).
  `supertrait-implied` was declared fenced and was accepted, as rustc accepts
  it. This compiler already elaborates supertrait bounds (section 2), so no
  open item is needed. The declared fields are kept beside the observation.
- **pa29.** `negative-metasized-param` in `knowable-absence-outcomes.json`
  and `negative-coherence` in `unsafe-negative-impl-outcomes.json` were
  declared rejected as recorded gaps. Both are amended to accepted, as rustc
  accepts them; the amendments were declared before the candidate run. C3 now
  has six native clients and ten opponents.

## 5. Work

Each side's premises are resolved, sorted and deduplicated once per pair,
after the pair is frozen, and charged for growth, the shared copy and the
sort. A query assumes a shared set at constant cost. A recorded node costs
one step, and each `MetaSized` debt pays a binary search.

`negative_metasized_premises.rs` measures a pair whose blanket has N
conditions against a client with N `?Sized` parameters, for N = 8, 16 and 32,
beside a `PointeeSized` control of the same shape. The premises' added work
must grow by at most a constant per premise. The first candidate charged the
premise count on every negative query that assumed it, so the pair paid N
steps for each of its N conditions: premise steps were 1798, 3609 and 7660
against 1472, 2818 and 5540. Each query now shares the side's set and pays
one step, and the invariant passes.

## 6. Checks

All on the uncommitted batch over `68c26f42a`, by `make fastcheck`
(evidence under `obj/qualification/fast/`) unless noted:

| Check | Result | Receipt |
| --- | --- | --- |
| 18 private cases: this batch's two invariants, vacuous callable coherence, imported same-head scaling, rejected overlap pairs, sizing sharing, coherence policy (PA31 `negative-coherence`), the coherence cycle, fn-pointer coherence and refutation layouts | 18/18 | `1794190317760557286` |
| `crate_linking`: knowable absence and modifier transport | 5/5 | `1794190486907434855` |
| Their native clients, `--ignored` (C3's six, `negative-coherence` included) | 2/2 | `1794190698009134191` |
| `suite_traits`, first 27 roots | 116/116 | `1794190751701200620` |
| `suite_traits`, other 26 roots | 240/240 | `1794190826933100786` |
| PA31 `negative-coherence` native, `--ignored` | 1/1 | `1794191107264546407` |
| PA1 to PA12 text | 1885/1885 | `1794191157658535097` |
| PA13 to PA24 text, except PA20 | 5440/5440 | `1794191331445307848` |
| PA20 text | 1482/1482 | `1794191731022055469` |
| `make test-upstream` | 128 fixtures, 126 Reference files | `obj/lane-b7/checks/upstream.log` |
| `make test-harness` | 313 tests, suite registration, layout | `obj/lane-b7/checks/harness.log` |

The first two runs of the private selection (`1794189049970464605`,
`1794189240103261802`) failed only the scaling invariant, before the
constant-cost premise sharing of section 5.

The join then found the `Query` layout pin of `sharing_sizing.rs` (728
bytes against 688): the premises and debts were 40 inline bytes on every
query. They are now one optional box, and the pin is amended to one word
more (696). On that tree:

| Check | Result | Receipt |
| --- | --- | --- |
| The 18 private cases and the layout pin | 19/19 | `1794193727450664959` |
| C2b's two lib sets (Lane C's receipts `1794187556880685380` and `1794187620664290658`), less 4 C2b-only cases | 408/408, 352/352 | `1794194005537349081`, `1794194224432979295` |
| `suite_traits`, both halves | 116/116, 240/240 | `1794194307185979891`, `1794194496255981043` |
| `crate_linking` transport and natives | 5/5, 2/2 | `1794194659364677512`, `1794194717712086942` |

The PA text, upstream, harness and census runs above are from the tree
before the box, which changes no behaviour.

## 7. Open: Sized premises of both sides

Repaired by Lane B's [premise sides](negative-premise-sides.md) batch, which
also found that a rigid parameter's sizedness was its representative's
declaration. The text below is the record as it stood.

Negative coherence still pays a Sized debt from both headers' Sized
premises. `overlap/inputs.rs` gathers both headers' Sized parameters and
Sized, Copy and Clone subjects into one `sized` set, and `freeze` makes the
first header's binding the rigid representative. So the negated clause's own
impl can prove its Sized condition. rustc assumes only `param_env(impl1)`
while negating impl2's clauses. `sized-premise-side`, which rustc rejects
with E0119, is accepted here. `prove` also assumes both headers' regions,
outlives and formed types. This is Lane B's next item, with a lifetime-premise
twin.

## 8. Flip survey

An accepted case that became rejected would stop the batch, as would the
scoped-debt error ("conditional negative implementation requires scoped debt
payment") on a new path. Neither occurs:

- PA31: the 26 `negative-coherence` libraries and their native program, the
  14 `fn-pointer-coherence`, 9 `callable-sugar-coherence` and 19
  `deep-reject` controls keep their outcomes.
- PA20's coherence packets and the traits suite's custom auto sources pass
  (PA20 text and `suite_traits` above).
- PA29: `knowable-absence` and `unsafe-negative-impls` keep every outcome
  except the two amended ones (section 4).

The only changed outcomes are rejected-to-accepted: four twins
(`metasized-param`, `metasized-written`, `metasized-strict` and
`supertrait-implied`) and the two PA29 amendments.

## 9. Census lookahead

An analysis-only census of the batch (not a qualification), with census 19's
manifest rebound only to the candidate's hash (`326eecfd…`, dev-fast, over
`68c26f42a` plus this batch) and the same original-core inputs, checks 42,310
declarations with none failed or blocked. Its `census.tsv` is byte-identical
to census 19's (`0e6b96ed…`): the same crate record (`str/iter.rs:569`,
"trait implementation does not satisfy the trait's requirements") and the
same 7 lookahead records. No record was added or removed; core had no
overlap record for this batch to remove. Receipt:
`obj/lane-b7/census/look1/receipt.json`.
