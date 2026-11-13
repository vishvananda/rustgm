# Comparison body lifetimes (source core)

Twenty-one no_core source-core libraries compare operands whose types carry
lifetimes, checked against rustc 1.98. Pinned core's `marker/variance.rs:107`
derives `PartialEq` for `PhantomCovariantLifetime<'a>(PhantomCovariant<&'a
()>)`; `core-form` reduces it. Each case's prefix is the first 13,493 bytes of
`const-parameter-derive/actual-source.rs`, followed by an authored
`PhantomData` lang item and a `variance` module. The other cases are authored.

rustc's `check_overloaded_binop` gives `Rhs` a fresh variable and selects
`Self: PartialEq<Rhs>` before coercing the right operand. Borrow checking
renumbers each operand's regions, so covariant operands with unrelated
lifetimes compare, and invariant ones must have equal lifetimes.

- Positives:
  - `core-form`, `plain-field`, `manual-eq`, `contravariant`, `sized-param`,
    `deref-operands` and `local-values` compare operands with a function's or
    an impl's lifetime;
  - `unrelated-covariant`, `invariant-same`, `static-vs-local`, `ne-local`,
    `impl-lifetime-method` and `value-rhs` vary the lifetimes, the operator
    and the operand form;
  - `local-borrows` and `local-borrow-values` compare two local borrows, as
    places and as values;
  - `static-lifetime`, `path-call`, `scalar-field` and `unit-wrap-field` are
    controls.
- Negatives ("lifetime may not live long enough"): `invariant-unrelated` and
  `invariant-static-vs-local` compare invariant operands whose lifetimes
  differ.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 21
declarations agree. `obj/parallel/variance-eq/core/declared.json` records six
batches. Batch 1 was malformed. Batches 3 and 6 were declared as diagnostics
and are graded here. Batch 5 holds ungraded lifetime-proof diagnostics. The
cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/rhs_body_lifetimes.rs` checks every case's typing through the CLI
and runs prepared lowering. Every accepted case lowers. Each rejection types
and fails prepared lowering's lifetime proof. It also checks bounded work.
See `docs/pa31-comparison-body-lifetimes.md`.
