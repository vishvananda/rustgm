# Captures through an overloaded dereference (source core, bring-up profile)

Six no_core source-core libraries read or write through a source `Deref` or
`DerefMut` inside a closure, as pinned alloc's `BTreeMap::replace` and
`get_or_insert_with` read `*map.alloc` (a `ManuallyDrop<A>`) in a closure
passed to `map.root.get_or_insert_with(..)` while `map.root` is mutably
borrowed. They are checked against rustc 1.98. The prefix is the
`source-call-members` `owned` prefix with core's `const_precise_live_drops`
feature.

- `shared` reads `*map.alloc` through `Deref` in the closure;
- `generic` is `shared` with a generic element, as `BTreeMap<K, V, A>` has;
- `method` calls `(*map.alloc).clone()`, alloc's exact shape;
- `mutable` assigns `*map.alloc = value` through `DerefMut` while
  `map.root` is mutably borrowed;
- `rejected` reads `map.root` in the closure while `get_or_insert_with`
  holds it mutably (rustc E0502);
- `rejected-alloc` reads `*map.alloc` in a closure while `&mut map.alloc`
  is live (rustc E0502).

An overloaded dereference calls `Deref::deref(&place)` or
`DerefMut::deref_mut(&mut place)`, so, as in rustc, the closure's captured
place stops before it: the closure captures `map.alloc` by shared or unique
borrow, which is disjoint from `map.root`.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
declared in `outcomes.json`; independent and candidate checks run separately. It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
