# Conditional const fn items (source core)

Eleven no_core source-core libraries reduce pinned core's `Iterator::last`
(`iter/traits/iterator.rs:271`), `Option::as_deref` and
`Option::get_or_insert_default`, checked against rustc 1.98. They use the
`default-const-closures` prefix, with `Default` and `Deref` const traits added
where a case needs them.

A const fn item whose own predicates include `[const]` bounds, or a member of a
const trait, meets a `[const]` callable bound when the caller proves those
conditions: the item's `[const]` predicates and a member's
`Self: [const] Trait`, substituted with the item's arguments.

- `last` (core's form) and `last-call` pass the nested
  `const fn some<T>(..) where T: [const] Destruct` to `fold`, and
  `nested-item` passes a nested conditional const fn in a free const fn;
- `default-item` passes `T::default` under `T: [const] Default`, `deref-item`
  passes `Deref::deref` as `as_deref` does, and `default-impl` passes
  `<u8 as Default>::default` through a `const impl`;
- `ordinary-item` passes an unconditional const fn item, a control;
- `last-missing`, `default-missing`, `default-nonconst-impl` and
  `nonconst-item` leave a condition unmet (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run; both are recorded in `outcomes.json` (11/11 agree). It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/source_closures/conditioned_items.rs` checks every case.
See `docs/pa25-conditional-const-items.md`.
