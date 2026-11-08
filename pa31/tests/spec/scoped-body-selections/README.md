# Generic bodies that select a region-binding impl

Seventeen no_core source-core libraries whose generic function bodies select
an impl with a region-binding condition, as core's `Scan`
(`F: FnMut(&mut St, I::Item) -> Option<B>`) and `MapWindows`
(`F: FnMut(&[I::Item; N]) -> R`) have. The method signature's elided `&mut
self` lifetime puts the call's normalization in a callable universe, and the
selected impl proves its condition in a nested universe of that one. RustGM
rejected every such body with "invalid scoped proof evidence".

The positive controls call `next` by method, path and qualified path, match
or annotate its result, build the receiver in the body, vary the state and
the output over caller type parameters and `'static`, select an impl whose
condition determines nothing, and run core's `Scan` adapter body and a
`MapWindows` shape under a generic caller. The opponents return the item at
the wrong type (E0308, twice), give a premise whose output contradicts the
condition (E0599), and give a premise whose output names its own bound region
(E0599).

Two controls are recorded gaps of a separate cause: a body-selected impl
parameter that a caller premise determines to name a caller lifetime
(`caller-region`), with or without a region binder (`ordinary-region`, which
fails the same way before this change), stays an unresolved trait goal.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
rustc or RustGM run of these files; see `outcomes.json`. RustGM probes of
similar shapes ran first in scratch.

The lang-item prefix and the `Tuple`, `Fn`, `FnMut` and `FnOnce` declarations
are the exact pinned Rust core fragment of
`spec/fn-sugar-impl-parameters/map-adapter.rs`. The case bodies are reduced
from the original core `iter/adapters/scan.rs` and
`iter/adapters/map_windows.rs`. They keep Rust's MIT/Apache-2.0 attribution;
see the root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/nested_condition_universes.rs` checks every case.
