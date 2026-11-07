# Generic non-null pattern types

Nine no_core source-core libraries for `pattern_type!(T is !null)` over a type
parameter base:

- core's exact pattern-type `DispatchFromDyn` impl, the same impl without its
  recursive bound and a generic struct field;
- core's `NonNull` shape, a `repr(transparent)` struct over
  `pattern_type!(*const T is !null)` that implements `DispatchFromDyn` through
  the generic pattern impl, with an opponent that lacks it (E0277) and an
  opponent from a raw pointer to a pattern type (E0377);
- non-null patterns over `u32`, over a reference and instantiated at `u32`.
  rustc 1.98.0 accepts all three; this analysis rejects each by name, since
  only a raw pointer instance has a defined non-null representation here.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. The lang-item prefix is the exact pinned Rust core fragment
from the census batch 2 controls, with its attributes and comments; the
`DispatchFromDyn` impls are core's. They keep Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/nonnull_pattern_bases.rs` checks every case.
See `docs/parallel/census-batch6.md`.
