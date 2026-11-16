# Ordered raw pointer comparisons (source core)

Six no_core source-core programs order raw pointers, as pinned core's
`slice::sort::unstable::quicksort` partition loops while
`state.right < right_end`. They are checked against rustc 1.98. The prefix is
`source-comparisons/native/actual-profile.rs` through its integer `PartialOrd`
probe, followed by core's raw pointer `Clone`/`Copy` (in the prefix's non-const
form), `PartialEq` and `PartialOrd` impls. `partial_cmp` inlines core's
`Ord::cmp` body for raw pointers.

rustc's builtin binary rule covers every comparison of two raw pointers, not
only `==` and `!=`. Core's `fn lt(&self, other: &*const T) -> bool { *self <
*other }` is therefore an address comparison. If the operator were a call, it
would select the same impl and recurse until the stack overflows.

- `operators` uses `<`, `<=`, `>` and `>=` on generic `*const T` and `*mut T`;
- `explicit` calls `lt`, `le`, `gt` and `ge` by path and by method, so each
  impl body runs;
- `generic` compares through `T: PartialOrd` with `T = *const u8`;
- `nominal` is a control: operators on a struct call its deliberately reversed
  impls;
- `const-operator` uses the operator in a `const fn`, where the non-const impl
  does not satisfy `[const] PartialOrd` (E0277);
- `mismatch` compares `*const u8` with `*const u16` (E0308).

Each accepted program exports `entry` and `probe`. `observe.c` prints
`entry() probe(0) probe(59)`: `3 10 12`, or `12 10 3` for `nominal`.
Outcomes were declared before the frozen rustc 1.98.0 oracle. Both are recorded
in `outcomes.json` (6/6 agree). The packet keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/guaranteed_pointers.rs` checks every case and runs the accepted ones
natively.
