# Declaration-phase constant execution

Twelve no_core source-core libraries for constants evaluated while
declarations are still being checked:

- source `ctpop`, `ctlz`, `cttz` and their `_nonzero` forms in array lengths at
  four widths, a wrong-length opponent (E0308) and `ctlz_nonzero(0)` (E0080);
- enums with only implicit discriminants, matched and cast, and their layouts
  read through `size_of` before the enum is declared (core's `const _` shape),
  with wrong-length opponents;
- a const fn that stores a struct declared later in source order, also inside
  an enum around a struct whose field is a projection through an impl's
  associated type, with a wrong-length opponent;
- a scalar impl whose associated type value is a struct declared later, with a
  non-`Copy` opponent (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`, with the amendment that added the layout cases before
oracle v2. The lang-item prefix is the exact pinned Rust core fragment from the
census batch 2 controls, with its attributes and comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/early_constant_execution.rs` checks every case.
See `docs/parallel/census-batch5.md`.
