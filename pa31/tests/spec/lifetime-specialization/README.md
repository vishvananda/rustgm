# Specialization over header lifetimes

Nine no_core source-core libraries with `min_specialization` whose
specializing impl repeats the parent's header shape over a lifetime, checked
against rustc 1.98. They reduce the record that followed census 10's once its
`FnPtr` coherence was repaired: core's `impl<I: DoubleEndedIterator>
DoubleEndedIteratorRefSpec for &mut I` (`iter/traits/double_ended.rs:498`).

rustc's `specializes` equates the child's trait reference with the parent's
fresh instantiation, lifetimes included, so each parent header lifetime is the
child's corresponding one. Adding `Sized`, a specialization trait, to a
`?Sized` parent is permitted.

- core's double-ended and iterator `&mut I` shapes, a shared reference,
  differently named lifetimes, a concrete referent and a nominal with a
  lifetime parameter;
- a guard where both headers name `&'static I`;
- opponents: a `'static` argument where the parent is generic (E0119) and a
  child that repeats a lifetime among the parent's arguments.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (9/9 agree).
The prefix is `spec/function-pointers/foundation.rs` with
`min_specialization` added. It keeps Rust's MIT/Apache-2.0 attribution; see
root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/specialized_header_lifetimes.rs` checks every case.
See `docs/parallel/census-batch10.md`.
