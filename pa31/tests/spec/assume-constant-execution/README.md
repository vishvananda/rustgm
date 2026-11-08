# Assume in constant execution

Seven no_core source-core libraries whose constants, evaluated while
declarations are still being checked, reach `intrinsics::assume`:

- core's exact `assume` declaration with its fallback body, assuming a true
  condition before returning an array length;
- core's `hint::assert_unchecked` shape over a source `ctpop` result (core's
  `ilog10` path) and core's `NumBufferTrait` shape, whose associated type and
  associated constant share that length;
- a bodiless `assume` declaration in an anonymous length block;
- a wrong-length opponent (E0308), a false condition (E0080) and a false
  condition whose authored fallback body is empty and would accept (E0080).
  Constant evaluation executes `assume` itself and never its fallback body.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. The lang-item prefix is the exact pinned Rust core fragment
from the census batch 2 controls, with its attributes and comments, plus a
`not` lang item and `u32` Copy; the `unreachable` and `assume` declarations are
core's. They keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/assume_constants.rs` checks every case.
See `docs/parallel/census-batch7.md`.
