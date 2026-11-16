# Runtime const closures (source core)

Six no_core source-core programs build a `const` closure that calls an
ordinary closure, as pinned core's `NeverShortCircuit::wrap_mut_1` and
`wrap_mut_2` (`ops/try_trait.rs`) do when `Iterator::fold` and friends wrap
the caller's closure. They are checked against rustc 1.98. The prefix is the
`captured-const-calls` `returned-shared` prefix through `Drop`.

A `[const]` bound outside a const context is an ordinary bound, so a const
closure executed natively may call an ordinary closure. Const evaluation
keeps the const requirement.

- `ordinary` and `pair` wrap a one- and two-argument ordinary closure and
  call the wrapper through a generic `FnMut` parameter;
- `capturing` wraps a closure that assigns a captured local;
- `direct` calls the returned opaque wrapper itself;
- `ctfe-callee` passes an ordinary closure to a `[const]` bound in a const
  item (E0277);
- `plain-callee` calls an ordinary function inside a const closure (E0015).

Each accepted program exports `entry` and `probe`; `observe.c` prints
`41 59`. Outcomes were declared before the frozen rustc 1.98.0 oracle; both
are recorded in `outcomes.json` (6/6 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/explicit_call_members.rs` checks every case and runs the accepted
ones natively.
