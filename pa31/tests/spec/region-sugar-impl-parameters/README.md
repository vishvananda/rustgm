# Impl parameters determined under a region binder

Eight no_core source-core libraries for impl parameters that only a callable
bound with an elided input lifetime determines, as in core's `Scan`
(`F: FnMut(&mut St, I::Item) -> Option<B>`) and `MapWindows`
(`F: FnMut(&[I::Item; N]) -> R`). The sugar quantifies the condition over the
elided region. Rustc proves it with that region instantiated, so the `Output`
equality determines `B` or `R` while the condition stays universal. RustGM
proved such conditions only after the parent resolved, which left `B`
undetermined.

The controls cover both core shapes, a caller premise that normalizes the item
in a signature, and an output naming a caller lifetime. The opponents return
the normalized item at the wrong type (E0308) and give a premise whose output
names its own bound region, which no single `B` can name (rustc: one type is
more general than the other).

Two controls select such an impl from a generic body (`body-caller`) or an
impl whose condition determines nothing (`body-fixed-output`). They were
recorded gaps until the selection's own binders were attached below the
call's universe; both now pass, and `spec/scoped-body-selections/` adds the
body controls and opponents.

Outcomes were declared before the frozen rustc 1.98.0 oracle; see
`outcomes.json`. The reduced shapes had exploratory rustc runs in scratch
first.

The lang-item prefix and the `Tuple`, `Fn`, `FnMut` and `FnOnce` declarations
are the exact pinned Rust core fragment of
`spec/fn-sugar-impl-parameters/map-adapter.rs`. The case bodies are reduced
from the original core `iter/adapters/scan.rs` and
`iter/adapters/map_windows.rs`. They keep Rust's MIT/Apache-2.0 attribution;
see the root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/elided_sugar_parameters.rs` checks every case.
