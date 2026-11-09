# Generic bodies that select a region-binding impl

Thirty-nine no_core source-core libraries. The first seventeen are generic
function bodies that select
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

Two of them select an impl parameter that a caller premise determines to name
a caller lifetime, with a region binder (`caller-region`) and without one
(`ordinary-region`). The body's method environment keeps such a premise in its
declaration form beside the alias copy that body goals use, and both copies
competed as distinct written results. Body trait queries now see the same
alias view as body projection queries.

Fifteen additions cover that change:
- the same selection under a let annotation, a qualified impl path, an
  annotated reference, a by-value receiver, a bound-only use, an unsized impl
  parameter and a goal input that also names the lifetime;
- premises that a let annotation's well-formedness, a turbofish, a qualified
  path and a closure signature name at the caller lifetime;
- `'static` and type-parameter premise outputs;
- ten repeated selections in two bodies, whose private check counts one alias
  view per body;
- an opponent with two premises whose results differ in a type (E0284), which
  must stay ambiguous.

Seven more select through such a premise in a closure body (audit a08). A
closure body inherits the caller's premises mapped to its own lifetime
binders, and its first selection folded them before any body step had aliased
a caller lifetime, so it was rejected with "missing lifetime argument". The
closure body now aliases them on entry, as an inline const body does:
- the a08 shape, and the same selection placed before a turbofish that
  aliases the lifetime;
- the closure return annotation without a selection, an explicit-lifetime
  closure without a premise, a `'static` premise and a lifetime-free premise;
- an opponent whose closure return names an undeclared lifetime (E0261).

The first seventeen outcomes were declared before the frozen rustc 1.98.0
oracle and before any rustc or RustGM run of these files; RustGM probes of
similar shapes ran first in scratch. Fourteen of the additions were
exploration probes that rustc 1.98.0 and RustGM ran before their
declaration, so they are a regression record; `repeated-selections` was
declared before any run. The seven closure additions are exploration probes
whose outcomes were declared before rustc 1.98.0 or RustGM ran them, so they
are a regression record too. See `outcomes.json`.

The lang-item prefix and the `Tuple`, `Fn`, `FnMut` and `FnOnce` declarations
are the exact pinned Rust core fragment of
`spec/fn-sugar-impl-parameters/map-adapter.rs`. The case bodies are reduced
from the original core `iter/adapters/scan.rs` and
`iter/adapters/map_windows.rs`. They keep Rust's MIT/Apache-2.0 attribution;
see the root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/nested_condition_universes.rs` checks every case.
