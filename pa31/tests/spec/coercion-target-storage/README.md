# Coercion target storage (source core, bring-up profile)

Three no_core source-core libraries coerce a value whose nominal type has an
associated type projection field to another lifetime instance of that type,
as pinned core's `str::lines` passes `self.split_inclusive('\n').map(LinesMap)`
to the `Lines` constructor. They are checked against rustc 1.98. The prefix
is the `source-call-members` `owned` prefix with core's
`const_precise_live_drops` feature.

- `constructor` passes `split(..)`'s `Split<'_, u8>`, whose `matcher` field
  is `<u8 as Pattern>::Searcher<'a>`, to `Lines(..)`, whose field type is
  the instance at the function's own lifetime;
- `adapter` is core's shape: the argument is `split(..).map(LinesMap)`, a
  `Map` around that `Split`;
- `rejected` returns `Lines<'static>` built from a borrowed haystack, which
  the projected field makes invariant (rustc: lifetime may not live long
  enough).

The coercion evaluates its value again at the expected type. That lifetime
instance has no expression node of its own, so its fields need the normal
form that the body's other value types receive. Each library has a
destructor, so its concrete bodies classify destruction at that type. The
result lifetime of `split` is its second, so no other body names the field
instance at a function's first lifetime.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
recorded in `outcomes.json` (3/3 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
