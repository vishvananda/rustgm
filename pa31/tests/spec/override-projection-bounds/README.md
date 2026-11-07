# Override bounds that name `Self::Item`

Eight no_core source-core libraries for override conformance of a trait
method bound that names an associated type of `Self`. Core's
`impl_fold_via_try_fold!` overrides `fold` in `Scan` and other adapters with
`FFF: FnMut(AAA, Self::Item) -> AAA`, repeating the trait's own bound, and
`Flatten`'s adapters repeat a supertrait's `Self::Item`. Rustc substitutes the
impl into the trait method's bounds and normalizes them in the impl's
param-env before it compares the override's bounds with them, so both sides
state the impl's definition. RustGM compared the trait's bound unnormalized
with the override's normalized one and reported a stronger requirement.

The controls repeat the bound, write its normal form, name a projection of an
impl parameter and use callable sugar. The opponents name a different type
(two cases) or add a bound; rustc reports E0276 for each.

Outcomes were declared before the frozen rustc 1.98.0 oracle; see
`outcomes.json`. Seven reduced shapes had exploratory rustc runs in scratch
first, and `own-item-wrong` had none.

The lang-item prefix and the `Tuple`, `Fn`, `FnMut` and `FnOnce` declarations
are the exact pinned Rust core fragment of
`spec/fn-sugar-impl-parameters/map-adapter.rs`. The case bodies are reduced
from the original core `iter/traits/iterator.rs`, `iter/adapters/scan.rs` and
`iter/adapters/flatten.rs`. They keep Rust's MIT/Apache-2.0 attribution; see
the root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/override_projection_bounds.rs` checks every case.
