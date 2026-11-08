# Rigid projection lifetimes

Lane B batch of 2026-11-08, branch `implement/nested-projection-20261108` on
main `6cd9f9e82`. It repairs the projection of a projection that
[census batch 9](census-batch9.md) left to Lane B (section 1, "Not repaired"):

```rust
pub trait Holder { type Inner: Deref; }
pub fn g<'a, H: Holder>(x: &'a H) -> Option<&'a <H::Inner as Deref>::Target> {
    Option::None
}
```

rustc 1.98.0 accepts it. This compiler rejected it with "invalid canonical
trait selection". It has no `[const]` bound, so batch 9's `assumes_trait`
repair does not reach it.

## 1. Reduction

The [`[const]` repair](census-batch9.md) shows that the failure needs a
reference lifetime and a nominal argument. The reduction drops the `deref`
lang item: `View { type Target: ?Sized; }`, `Holder { type Inner: View; }`
and `Option<&'a <H::Inner as View>::Target>`. Seventeen exploration drafts
(`obj/lane-b3/t2/explore`, not declared controls) separate the cases. rustc
1.98.0 ran ten of them; the declared controls of section 4 settle the others.

- Rejected by this compiler before the repair: the reduction, an elided
  lifetime, the lang trait, a `Sized` `Target`, two levels of item bounds, a
  supertrait item bound, an inherent `&'a self` method and batch 9's draft.
  rustc accepts the first four drafts and the declared `nested-holder`,
  `supertrait-bound` and `method` controls.
- Accepted by this compiler before the repair: an explicit `H::Inner: View`
  premise, a raw pointer (`Option<*const …>`), `&'a …` as a parameter or as a
  result without `Option`, a one-element tuple, a struct field, and concrete
  projections that normalize through an impl or an equality. rustc accepts the
  premise, raw pointer, parameter and tuple drafts and the declared
  `concrete-result` control.
- Rejected by both: an associated type without the `View` bound (E0277).

So the failure needs a projection that only an item bound forms, a free
caller lifetime and a sizing goal for a nominal argument.

## 2. Cause

An analysis-only build with a backtrace at the failing restoration
(`obj/lane-b3/t2/dev/dbg1`, instrumentation not committed) gives one path for
every rejected draft:

1. `validate_signatures` proves the formation of `Option<&'a …>` through the
   nominal conjunction (`Proof::check_inside`, `State::prove_conjunction`).
   In a source core, `Option`'s implicit bound is the source `MetaSized`
   trait, so its goal is a trait root of the conjunction.
2. The selection query normalizes the goal's inputs (`advance_normalization`).
   The projection stays rigid: `Holder`'s item bound proves
   `<H as Holder>::Inner: View`. The goal keeps the caller's free `'a`
   (`RegionId(1)`).
3. `source_sizing_result` asks the query's builtin proof for the goal's
   physical shape. That proof has the caller's environment but no region
   scope (`nominals::Proof::new`). Its shape walk finds no environment premise
   for the projection's subject, so `Proof::shape_inside` normalized the root
   again in that unscoped proof.
4. That proof's restoration maps only its own equality components, so the
   caller's `'a` has no mapping: "selected type retains an unmapped lifetime"
   (`types/substitute/region_mapping.rs`). The conjunction folds the cause
   into "invalid canonical trait selection".

The trace recorded `normalized_inputs = true` at step 3 for every rejected
draft. The raw pointer reaches step 3 with normalized inputs but no free
lifetime, so the unscoped restoration has nothing to map. The explicit
premise lets the shape walk's fast path keep the projection rigid without
normalizing.

## 3. Repair

- `Proof::normalized_physical_shape` (`sema/nominals.rs`) is the shape of a
  root that a selection query has already normalized under the same
  environment. Its remaining projections are rigid, as they are after the
  proof's own normalization. `Proof::shape_inside` now prepares its shape
  caches and walks the normalized root through the same two helpers
  (`prepare_shapes`, `normalized_shape`), so the existing path is unchanged.
- `source_sizing_result` (`sema/traits/selection/search/sizing_fallback.rs`)
  uses it when the frame's inputs are normalized and keeps `physical_shape`
  otherwise. This is the batch's only `sema::traits` edit (6 lines). The
  root's nominal formation is still a child goal (`require_nominal_formed`),
  and the fallback still rejects a malformed or unsized root.

The same restoration fails at two more kinds of site, which section 3a
repairs the same way.

## 3a. Freeze, Destruct and the builtin `Sized` proof

`destructibility.rs` and `freezing.rs` call `physical_shape` the same way for
source `Destruct` and `Freeze` goals. Four drafts put the nested shape and its
explicit premise behind `Frozen<T: Freeze>` and `Gone<T: Destruct>`. Their
outcomes were declared before any run (section 4). rustc 1.98.0 accepts all
four, and this compiler rejected all four, the premises included, with
"invalid canonical trait selection". The first version of the drafts failed
in rustc on construction errors only, E0658 for an explicit `Freeze` impl
without `freeze_impls` and a const trait without its const stability
attribute. Version 2 adds that feature and core's own `Destruct` line and
keeps the shapes and the declared outcomes.

An instrumented build, not committed, traced two failing sites:

1. `Search::complete_builtin` (`search/builtins.rs`), reached from
   `advance_inputs` once the query has normalized the goal's inputs. It asks
   `Proof::prove` for the internal builtin `Sized` bound on the wrapper's
   argument (`&'a P` or `Option<&'a P>`). `prove` calls `normalize_builtin`,
   which normalizes the type again in the unscoped builtin proof, and that
   proof's restoration cannot map the caller's `'a`. This site fails first,
   for the premise drafts as well.
2. With that site repaired, `Search::source_auto_result` (`freezing.rs`, its
   `physical_shape` call) and `Search::source_destruct_result`
   (`destructibility.rs`, line 44) are reached for the nested drafts. They are
   reached with the query's own inputs normalized, a free caller region and a
   projection in the subject. Their `physical_shape` call fails the same way
   as the sizing site did. The premise drafts reach them too, but their shape
   walk keeps the projection rigid through the premise and succeeds.

The repair:

- `Proof::prove_normalized` (`sema/nominals.rs`) is `Proof::prove` without
  the redundant `normalize_builtin`. Every arm runs exactly the code `prove`
  runs, except the `Sized` arm, which takes the shape of the normalized root
  through the same `normalized_shape` helper as `normalized_physical_shape`
  (`rigid_shape`). No other arm gains a shortcut.
- `Search::complete_builtin` chooses it only when the frame's input
  normalization is ready (`original_inputs()` is `Some`). `advance_inputs`
  records that state after replacing every projected input with its
  normalized value under the query's environment. The choice reads that
  recorded state, never the term.
- `Search::builtin_shape` (`search/sizing_fallback.rs`) is the one shared
  helper for the three `physical_shape` sites: the sizing fallback, the
  `Freeze` call in `Search::source_auto_result` and the `Destruct` call in
  `Search::source_destruct_result`. Each site's edit is that one call line.

Skipping the second normalization cannot hide a legitimate rejection. The
unmapped-lifetime check guards a term the builtin proof would map back to the
caller. Here the normalized term exists only for the shape walk. The builtin
answer is a `bool` or a pair of shape flags, and no term from the builtin
proof returns to the frame. The node's goal and its retained `sized` entries
are the search's own terms, which the search's own restoration maps. The
query's normalization of the inputs still reports its own failures. The
normalized shape walk still decides sizedness and well-formedness, and an
unresolved projection still fails through the walk.

The opponent `destruct-unsized-pair` is `Gone<(&'a P, P)>` with `P: ?Sized`.
Its caller's `'a` would have to cross the builtin `Sized` answer. rustc
rejects it with E0277. This compiler rejects it before and after the repair
with the same text, "invalid canonical trait selection". Before the repair its
cause was "selected type retains an unmapped lifetime". After the repair it is
"source nominal formation prerequisite is unsatisfied", which is the
legitimate failure.

"invalid canonical trait selection" folds several causes. In this batch it is
the unmapped caller lifetime of a builtin proof's second normalization. Lane A
owns the follow-up that makes the label carry its cause category (section 6).

## 4. Controls

Twenty-one no_core controls in `pa31/tests/spec/rigid-projection-lifetimes/`.

- The first thirteen were declared before any rustc or RustGM run of their
  bytes, in `obj/lane-b3/declarations/rigid-projection-lifetimes.declared.json`
  (sha256 `f9394c2ab4bb…`). Their rustc 1.98.0 oracle receipt is
  `obj/lane-b3/oracle-rigid-projection/receipt.json`. Their "before" column is
  the analysis-only build of `e02716c40` with trace output only.
- The eight additions are declared in
  `obj/lane-b3/declarations/rigid-projection-additions.declared.json` (sha256
  `5fab7f9e1d11…`). Of these:
  - the four sibling drafts keep version 2's declared outcomes
    (`rigid-projection-siblings-v2.declared.json`, sha256 `3a8ed5e3fad5…`);
  - the three ordinary controls are exploration probes that ran before that
    declaration;
  - the opponent was declared before any run.

  Their oracle receipt is
  `obj/lane-b3/oracle-rigid-projection-additions/receipt.json`. Their
  "before" column is a CLI built from this batch's sizing repair alone
  (`220b753bb`, sha256 `7e2358104…`).
- All twenty-one agree with rustc.

| control | declared | rustc 1.98.0 | before | after |
| --- | --- | --- | --- | --- |
| `alias-bound.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `elided-lifetime.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `lang-deref.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `sized-target.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `nested-holder.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `supertrait-bound.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `method.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `sized-tuple.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `where-premise.rs` | accept | accepts | accepts | accepts |
| `raw-pointer.rs` | accept | accepts | accepts | accepts |
| `concrete-result.rs` | accept | accepts | accepts | accepts |
| `unsized-tuple.rs` | reject | E0277 (`Target` is not sized) | invalid canonical trait selection | "nominal type argument must be sized" |
| `missing-bound.rs` | reject | E0277 (`Inner: View` unsatisfied) | "associated type requires an unambiguous implementation" | the same |
| `freeze-nested.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `freeze-premise.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `destruct-nested.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `destruct-premise.rs` | accept | accepts | invalid canonical trait selection | accepts |
| `freeze-plain-reference.rs` | accept | accepts | accepts | accepts |
| `destruct-generic-reference.rs` | accept | accepts | accepts | accepts |
| `destruct-raw-projection.rs` | accept | accepts | accepts | accepts |
| `destruct-unsized-pair.rs` | reject | E0277 (the pair is unsized) | invalid canonical trait selection (unmapped lifetime) | invalid canonical trait selection (unsatisfied formation prerequisite) |

Batch 9's draft itself (`obj/parallel/census-batch9/repro/nested-item.rs`, a
`const trait Deref` with no `[const]` bound) is accepted after the repair, as
are all sixteen other exploration drafts except the missing-bound one.
`sema::nominals::tests::normalized_builtin_goals_keep_rigid_projections_and_caller_lifetimes`
checks every control.

The undeclared exploration of the sibling sites (`obj/lane-b3/t2/siblings-explore`)
also ran these shapes. All are accepted after the repair except
`freeze-raw-projection`, a raw pointer to the projection under `Freeze`.
rustc rejects that shape with E0277, because no `Freeze` impl covers
`*const P` in this core. This compiler rejects it before and after the repair
with "invalid canonical trait selection", whose cause is "source nominal
formation prerequisite is unsatisfied". That text is recorded as observed.
The repair does not change it.

## 5. Files

- `dev/src/sema/nominals.rs`: `normalized_physical_shape`,
  `prove_normalized` and the shared shape helpers (`prepare_shapes`,
  `normalized_shape`, `rigid_shape`, `prove_builtin`).
- `sema::traits` edits, listed:
  - `search/sizing_fallback.rs`: `Search::builtin_shape`, and its call in
    `Search::source_sizing_result`;
  - `search/builtins.rs`: one branch in `Search::complete_builtin`;
  - `search/freezing.rs`: the `physical_shape` call line in
    `Search::source_auto_result`;
  - `search/destructibility.rs`: the `physical_shape` call line in
    `Search::source_destruct_result`.

  The `freezing.rs` edit leaves Lane C's `bound` and `imported_application`
  untouched.
- `dev/tests/invariants/normalized_sizing_lifetimes.rs` (new).
- `pa31/tests/spec/rigid-projection-lifetimes/` (new) and
  `pa31/tests/README.md`.

No wire bytes, layouts or size pins change. The repaired paths normalize
nothing, so no scaling or once-only control applies.

## 6. Open

- Closed: the `Destruct` and `Freeze` call sites are reached and repaired
  (section 3a).
- "invalid canonical trait selection" folds distinct causes:
  - this batch's unmapped caller lifetime;
  - Lane A's `Flatten` case, an unresolved rigid-projection result;
  - the legitimate formation failure behind `destruct-unsized-pair` and
    `freeze-raw-projection`.

  Lane A owns the follow-up that labels the cause category.

## 7. Checks

Receipts are under `obj/qualification/fast/`. Each receipt's source fingerprint
equals the final tree.

| Selection | Receipt | Result |
| --- | --- | --- |
| Private: the 21-control test, 17 sizing, projection and selection tests, and 15 formation, refutation, sizing-sharing, auto-override, destruct-pattern and Copy-target tests | `1794157952343972580` | 33 passed |
| Integration: `size_conditions`, `size_declarations`, `tail_rules`, `nominal_declarations`, `shape_constructors`, `census_mode`, `deref_contracts` | `1794158711598503022` | 40 passed |
| Native `--ignored`: `unsized_storage` | `1794158774832018154` | 2 passed |
| Integration: `projection_shapes`, `projection_normalization`, `deferred_alias_projections`, `inferred_projections`, `associated_type_declarations`, `projection_lifetimes`, `projection_region_mapping`, `derive_contracts` | `1794158419416385015` | 40 passed |
| Integration: `principal_auto_sources`, `provider_traits` | `1794158478910125606` | 59 passed |
| PA1–PA12 | `1794158843072564842` | passed |
| PA13–PA19, PA21–PA24 | `1794158898818120403` | passed |
| PA20 | `1794158984257054002` | passed |
| `make test-upstream` | `obj/lane-b3/t2/final/upstream.log` | 128 pinned fixtures and 126 Reference files verified |

The first integration run, `1794158296359614422`, selected `unsized_storage`
without `--ignored`. It failed with "no runnable tests match", because both of
that module's tests need the native backend. They pass in the native run above.
