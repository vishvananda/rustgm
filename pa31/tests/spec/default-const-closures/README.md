# Const closures in const trait defaults (source core)

Ten no_core source-core libraries reduce pinned core's `Iterator::count`
(`iter/traits/iterator.rs:229`), checked against rustc 1.98. They use the
`source-copy-captures` prefix with `const_closures`, `stmt_expr_attributes` and
`const_precise_live_drops`, and core's `Add` with its integer and float impls.
A `const trait Iterator` has core's `fold`, which requires
`F: [const] FnMut(B, Self::Item) -> B + [const] Destruct`.

A const closure defined in the trait's default `count` meets that bound through
the default body's `Self: [const] Iterator` premise.

- `count` (core's form), `count-typed`, `count-suffix` and
  `count-no-attribute` vary the closure and the initial value;
- `count-plain` is an ordinary trait, closure and `FnMut` bound, a control;
- `count-call` calls a concrete iterator's `count` from a library function,
  with core's `panic_const_add_overflow` lang item for object generation, and
  `call-count` calls it from a root `main`;
- `count-clone-premise` clones the element in a const closure under
  `[const] Clone`;
- `count-plain-const-bounds` passes a non-const closure, and
  `count-clone-missing` clones under only `Clone` (both E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file; both are recorded in `outcomes.json` (10/10 agree). It
keeps Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.
`dev/tests/source_closures/defaulted_const_closures.rs` checks every case.
See `docs/pa25-default-const-closures.md`.
