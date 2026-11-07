# Impl headers naming a projection of their parameter

Ten no_core source-core libraries for an impl header whose type names a
projection of the impl's own parameter, as in core's
`impl<R: ops::Try> ControlFlow<R, R::Output>` (`ops/control_flow.rs`). Rustc
checks that header under the impl's predicates, where `<R as Try>::Output`
is a rigid projection. RustGM formed the header without any environment, so
`R: Try` was unprovable there and the projection was ambiguous.

The controls cover the core shape with and without core's `Try`,
`FromResidual` and `Residual` traits, inline, qualified and where-clause
bounds, a trait impl, a method returning `Self` and a `where Self: Sized`
clause. The opponents name the projection without an `R: Try` bound (E0277)
and use `R` only inside a projection, which leaves it unconstrained (E0207).

`self-where` stays a recorded gap: its `where Self: Sized` bound names the
self type that the header projection's environment would need before that
type is formed. The same bound beside a header without a projection passes.

Outcomes were declared before the frozen rustc 1.98.0 oracle; see
`outcomes.json`. Seven reduced shapes had exploratory rustc runs in scratch
first; `where-clause`, `self-where` and `unbounded` had none.

The lang-item prefix and the `Tuple`, `Fn`, `FnMut` and `FnOnce` declarations
are the exact pinned Rust core fragment of
`spec/fn-sugar-impl-parameters/map-adapter.rs`. The case bodies are reduced
from the original core `ops/control_flow.rs` and `ops/try_trait.rs`. They
keep Rust's MIT/Apache-2.0 attribution; see the root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/normalized_impl_headers.rs` checks every case.
