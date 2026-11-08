# Rigid projection lifetimes

Twenty-one no_core source-core libraries whose signatures pass projections to
`Option` or to a nominal with a `Freeze` or `Destruct` bound, most of them
rigid projections behind a reference with the caller's lifetime. Census
batch 9's draft `Option<&'a <H::Inner as Deref>::Target>` has no `[const]`
bound: only `Holder`'s `type Inner: Deref` forms the inner projection, so the
sizing goal of `Option`'s argument is normalized by the selection query.

- accepted: that shape with a plain `View` trait, an elided lifetime, the
  `deref` lang trait, a `Sized` `Target`, two levels of item bounds, a
  supertrait item bound, an inherent `&'a self` method, and a sized rigid
  projection beside the reference in a tuple;
- controls, accepted before and after the repair: an explicit `H::Inner: View`
  premise, a raw pointer without a lifetime, and a concrete projection that
  normalizes through an impl;
- opponents (E0277): the same tuple with a `?Sized` `Target`, and an
  associated type without the `View` bound;
- `freeze-*` and `destruct-*`: the nested shape and its explicit premise as
  the argument of `Frozen<T: Freeze>` and `Gone<T: Destruct>`, whose source
  auto-trait, ordinary `Destruct` and builtin `Sized` goals are normalized by
  the selection query too; the caller's lifetime alone, a type parameter
  behind it, and a raw pointer to the projection as ordinary controls; and an
  opponent (E0277) whose `Gone` argument `(&'a P, P)` is unsized.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. Drafts of eleven shapes were run through this compiler
first, seven of them also through rustc, so those are a regression record of
that exploration. The eight `freeze-*` and `destruct-*` additions have their
own declaration: the four sibling drafts were declared before any run, the
three ordinary controls are exploration probes run before it, and the
opponent was declared before any run. The prefix is the `projection-sizedness` prefix: the exact
pinned Rust core `PointeeSized`, `MetaSized` and `Sized` fragment with its
attributes and comments. It keeps Rust's MIT/Apache-2.0 attribution; see root
NOTICE and `third_party/sources.json`.

`dev/tests/invariants/normalized_sizing_lifetimes.rs` checks every case.
See `docs/parallel/rigid-projection-lifetimes.md`.
