# Closure entry roots (source core)

Four no_core source-core libraries call core's `Option::and_then` with a
closure, as pinned core's `option.rs` does when a generic caller passes
`|slot| slot.as_ref()`. They are checked against rustc 1.98. The prefix is the
`source-call-members` `owned` prefix with core's `const_precise_live_drops`
feature, which `and_then`'s `match self` needs.

- `concrete` creates the closure in a non-generic function;
- `generic-fn` creates it in a generic function `first<T>`, and
  `generic-method` in a generic method that borrows the first of two slots;
- `rejected` returns the closure's `&Option<T>` input where `and_then` expects
  an `Option<U>` (E0308).

A closure created in a generic body is instantiated with its creator, and
`and_then`'s call to it reads the closure's public callable and its
specialized execution signature, whose captured lifetimes stay bound beneath
the public binder. Those signature types are not types of any body, so their
nominal roots (`Option<&u8>`) are prepared with the instantiated closure body,
as a function's public roots are.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
The strict executable graph proves both generic creators' output lifetimes.
The bring-up library profile also proves them. It reports a source metadata
export restriction for these declarations.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
recorded in `outcomes.json` (4/4 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
