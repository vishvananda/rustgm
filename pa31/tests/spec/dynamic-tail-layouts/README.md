# Dynamic tail layouts

Eight no_core source-core libraries whose declarations end in a trait object,
checked against rustc 1.98, where a trait object, like a slice or str, has no
fixed layout:

- core's `Request` (`error.rs:588`, census 10's layout lookahead record): a
  `#[repr(transparent)]` struct over `Tagged<dyn Erased<'a> + 'a>`, and the same
  struct without `repr(transparent)`;
- a tuple struct, a record struct with a lifetime and a `repr(transparent)`
  struct whose last field is a trait object;
- a guard that already passed: a struct ending in a slice;
- opponents: a trait object before another struct field and as an enum variant
  field (each E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json`. Its two amendments, each
made after an oracle run and before any candidate run, complete the prefix with
the minimal `LegacyReceiver`, `Clone` and `Copy` lang items rustc requires; no
declaration or outcome changed. The prefix is the const-bound projection
controls' prefix up to `Copy`: the exact pinned Rust core `PointeeSized`,
`MetaSized` and `Sized` fragment with its attributes and comments, then those
minimal declarations. It keeps Rust's MIT/Apache-2.0 attribution; see root
NOTICE and `third_party/sources.json`.

`dev/tests/invariants/trailing_objects.rs` checks every case.
See `docs/parallel/census-batch9.md`.
