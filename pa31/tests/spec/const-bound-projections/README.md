# Const-bound projections

Fourteen no_core source-core libraries whose signatures name a projection of a
parameter bounded only by a `[const]` or `const` trait bound, checked against
rustc 1.98, which elaborates such a bound to its ordinary trait bound:

- core's `Option::as_deref` (`option.rs:1386`, census 9's crate record),
  `Option::as_deref_mut`, `Result::as_deref` and `Pin::as_ref` shapes, with
  an elided reference lifetime inside the nominal result;
- the free-function reduction with a named lifetime, the same type as a
  parameter, a `[const] SliceIndex<[T]>` output (core's `<[T]>::get` shape)
  and an always-`const` bound;
- guards that already passed: a `'static` reference, a raw pointer, a plain
  `T: Deref` bound and a reference without a nominal wrapper;
- opponents: `Option<T::Target>` of a `?Sized` target and a fully qualified
  projection with no bound on its subject (each E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json`. Its one amendment, made
after the first oracle and before any candidate run, adds core's `Freeze` lang
item, which rustc's const checking requires; no signature, body or outcome
changed. Census batch 8 had explored drafts of seven of the shapes. The prefix
is the census batch 8 drafts' exact pinned Rust core `PointeeSized`,
`MetaSized` and `Sized` fragment with its attributes and comments, followed by
minimal `LegacyReceiver`, `Clone`, `Copy`, `Freeze` and `const Deref`
declarations. It keeps Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/elaborated_effect_bounds.rs` checks every case.
See `docs/parallel/census-batch9.md`.
