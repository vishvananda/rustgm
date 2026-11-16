# Projected captures (source core)

Five no_core source-core programs return a closure whose captured environment
names an associated type of a generic parameter, as pinned core's
`iter::adapters::flatten::flatten` captures
`frontiter: &'a mut Option<T::IntoIter>` and `Fuse::try_fold` passes it on.
They are checked against rustc 1.98. The prefix is the `source-call-members`
prefix through its `&mut F` call impls.

A caller stores the revealed closure with that environment normalized, as
rustc does, so its checked execution and the call's selected signature agree.

- `front` returns the closure and calls it once;
- `written` then reads the `Option` the closure assigned through its capture;
- `flatten` passes the closure to a trait's default `fold`, as core does;
- `plain` captures no projected type and is a control for the same call;
- `borrowed` reads the captured `Option` while the closure still holds it
  (E0503).

Each accepted program exports `entry` and `probe`; `observe.c` prints
`41 59`. Outcomes were declared before the frozen rustc 1.98.0 oracle; both
are recorded in `outcomes.json` (5/5 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/explicit_call_members.rs` checks every case and runs the accepted
ones natively.
