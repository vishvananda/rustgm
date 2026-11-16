# Opaque capture premises (source core, bring-up profile)

Three no_core source-core libraries reduce pinned core's
`GenericShunt::try_fold` (`iter/adapters/mod.rs`) under the `fold` that
`impl_fold_via_try_fold!` writes, as `Sum for Result` reaches it through
`iter::try_process`. `fold` passes `NeverShortCircuit::wrap_mut_2(fold)`, an
opaque `impl FnMut`, to `try_fold`, whose closure captures it as `&mut F`. They
are checked against rustc 1.98. The prefix is the `source-call-members`
`owned` prefix with core's `const_precise_live_drops` feature.

- `wrapped` folds over a non-generic `Pair` whose closure has no region
  conditions;
- `generic` is core's shape: a generic `Shunt<'a, I>` whose closure also
  writes through its `&'a mut` field, so its declared `Output` premise is
  matched against the closure's actual execution;
- `rejected` moves the opaque callable before passing it, and it is not
  `Copy` (E0382).

Substituting the instance into the closure's checked execution interns the
opaque application its `&mut F` capture names. Like every other matcher of
closure executions, the declared `Output` premise prepares that application's
storage head before it compares the two.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
recorded in `outcomes.json` (3/3 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
