# Defaulted impl arguments

Eight no_core libraries, checked against rustc 1.98. They reduce census 14's
crate record, "trait implementation does not satisfy the trait's requirements",
at core's `const impl<T> Try for NeverShortCircuit<T>` (`ops/try_trait.rs:433`).
Its `const impl<T> FromResidual for NeverShortCircuit<T>` (line 448) leaves
`FromResidual`'s argument to its default, `<Self as Try>::Residual`.

rustc fills an omitted type argument with its default instantiated by the
preceding arguments, `Self` included (`rustc_hir_analysis/src/hir_ty_lowering/mod.rs`).
`check_impl` (`check/wfcheck.rs`) deeply normalizes the impl's trait reference
and proves it well-formed, including the trait parameter's bounds (E0277), and
`match_impl` (`rustc_trait_selection/src/traits/select/mod.rs`) normalizes the
instantiated reference before equating it with a goal. A default that names a
parameter is not checked at the trait's own declaration.

- accepted: the omitted default (`impl FromResidual for Short`), the same
  default written out, a default naming the trait's other associated type, the
  omitted default under a `where Self: Sized` bound, core's generic const
  `NeverShortCircuit<T>` shape, a default naming another trait's associated
  type that satisfies the parameter's bound, and a scaling control of three
  impls each selected through two traits;
- rejected: the same default normalizing to a type that does not satisfy the
  parameter's bound (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (8/8
agree). Exploration drafts with other bytes were run first; the declaration
records them. The prefix is `#![feature(const_precise_live_drops)]`, which core
enables, followed by `spec/function-pointers/foundation.rs`. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`default-bound-satisfied` was a recorded gap until census batch 15: the
default's retained alias requirement was instantiated without the trait's
`Self`. It is now instantiated with `Self` as the path's subject, and the
control is accepted as rustc accepts it.

`dev/tests/invariants/defaulted_impl_arguments.rs` checks every case and builds
`default-scaling.rs`'s shape at larger sizes: each omitted default is normalized
once per impl, however many traits select it. See
`docs/parallel/census-batch14.md`.
