# Closure parameter outlives requirements (source core, bring-up profile)

Six no_core source-core libraries return a reference into a type parameter's
value from a closure, as pinned alloc's `LinkedList::front` and
`Cursor::current` return `&'a T` from `map(|node| &node.as_ref().element)`.
They are checked against rustc 1.98. The prefix is the `source-call-members`
`owned` prefix with core's `const_precise_live_drops` feature and Copy impls
for `*const T`, as core's marker impls have.

- `cursor` maps a cursor's current node to `&'a T` for the cursor's lifetime;
- `front` maps a list's head to `&T` for the list borrow;
- `generic` is `cursor` as a free generic function;
- `direct` calls the closure directly;
- `rejected-local` returns a borrow of a closure local (rustc E0515);
- `rejected-captured` returns a borrow of a local captured by reference.

The closure's body needs `T: 'a` for the reference it returns. Its creator
names `T`, so rustc propagates that requirement to the creator, which proves
it from its own environment (here from `&'a` well-formedness). The closure's
output proof exports a rigid parameter's extent only as the longer side of
such a requirement, and the creator checks it as `T: 'a`.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
declared in `outcomes.json`; resumed independent and candidate checks are separate. It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
