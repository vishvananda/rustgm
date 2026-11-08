# Projection sizedness

Eight no_core source-core libraries whose trait arguments are rigid
projections, checked against rustc 1.98's sizedness hierarchy:

- core's `PartialEq<Rhs: PointeeSized = Self>` over two `?Sized` Deref
  targets in a where-clause, and core's exact `Pin` PartialEq impl header
  (`pin.rs:1105`, census 8's crate record);
- a `Takes<Rhs: MetaSized>` argument `Q::Target`, in a function and in a tuple
  struct where-clause: `Deref::Target: ?Sized` is implicitly `MetaSized`;
- an associated type bounded only by `PointeeSized` as a `PointeeSized`
  argument, and a default (`Sized`) associated type as a `MetaSized` argument;
- opponents: `Q::Target` as an implicitly `Sized` argument, and a
  `PointeeSized`-only associated type as a `MetaSized` argument (each E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json`. The predecessor's drafts of
the first three shapes had been explored with rustc. The prefix is the exact
pinned Rust core `PointeeSized`, `MetaSized` and `Sized` fragment of the census
batch 7 drafts, with its attributes and comments, followed by minimal
`LegacyReceiver`, `Clone`, `Copy` and `Deref` declarations. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/sizedness_hierarchy.rs` checks every case.
See `docs/parallel/census-batch8.md`.
