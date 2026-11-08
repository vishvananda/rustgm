# Copy impl targets

Thirteen no_core libraries, checked against rustc 1.98. They reduce census 13's
crate record, "source Copy target is outside the supported implementation
profile", at core's `impl Copy for ! {}` (`marker.rs:484`), and check the whole
set of targets rustc accepts.

rustc's `type_allowed_to_implement_copy` (`rustc_trait_selection/src/traits/misc.rs`)
accepts integer, float, `bool`, `char`, raw pointer, `!`, shared reference and
array targets without fields, and an ADT whose fields are Copy. It does not
check an array's element. Every other target is "not an ADT", which
`visit_implementation_of_copy` (`rustc_hir_analysis/src/coherence/builtin.rs`)
reports as E0206. Tuples and fn pointers have builtin Copy and Clone
(`copy_clone_conditions`), so a written Clone impl for one can also be
ambiguous (E0283).

- accepted: core's `!` impls, scalars, raw pointers, core's array impls, an
  array Copy impl whose element is only Clone, and a fixed-length array of a
  non-Copy struct;
- rejected: `&mut T`, `str` and `[T]` targets (E0206), `!` without a Clone impl
  (E0277), and tuple and fn pointer targets (E0206; E0283 where the builtin
  Clone also applies).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (13/13
agree). Exploration drafts with other bytes were run first; the declaration
records them. `tuple-copy-elements.rs` is a recorded amendment: the census 13
pin rejected `tuple-target.rs` only because its `bool` element has no Copy impl
here, so a tuple of Copy elements was declared afterwards, before its own
oracle. The prefix is `#![feature(never_type)]` followed by
`spec/function-pointers/foundation.rs`. It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`fn-pointer-target` and `tuple-copy-elements` are recorded gaps: this compiler
still accepts a written Copy impl for a fn pointer or a tuple of Copy elements.
`dev/tests/invariants/allowed_copy_targets.rs` checks every case. See
`docs/parallel/census-batch13.md`.
