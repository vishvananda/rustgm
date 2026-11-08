# `[const]` bounds in `min_specialization`

Seven source-core `min_specialization` libraries, checked against rustc 1.98.
They reduce census 12's crate record, "minimal specialization introduces a
non-marker predicate", at core's `SpecNextChunk` pair (`array/mod.rs:1044` and
`1062`). The parent bounds `I: [const] Iterator<Item = T>`. The child adds
`TrustedLen`, a `const unsafe trait TrustedLen: [const] Iterator` marker, and
its ordinary bound elaborates to the ordinary `I: Iterator`.

rustc lowers a `[const] Tr` bound (`rustc_hir_analysis/src/hir_ty_lowering/mod.rs`)
to the ordinary `Tr` clause of `predicates_of`; its constness goes only to
`const_conditions`. A `const Tr` bound adds a host-effect clause as well.
`check_predicates` (`impl_wf_check/min_specialization.rs`) compares only
`predicates_of`, and `check_specialization_on` allows only a trait clause on a
marker trait. `specializes` (`rustc_trait_selection/src/traits/specialize/mod.rs`)
checks constness instead: a conditionally const parent needs a conditionally
const child, and the parent's const conditions must hold as `[const]` goals
under the child's param-env.

- accepted: core's pair, the pair with a plain `TrustedLen: Iterator`
  supertrait, a `[const]` child bound over an ordinary parent bound, and the
  ordinary pair;
- rejected: a non-const child of core's const parent and a const child with
  only ordinary bounds, which do not specialize (E0119), and a child bound
  `const Exact` on a marker, which is a host-effect clause.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (7/7
agree). Exploration drafts with other bytes were run first, with rustc and the
census 12 pin; the declaration records them. The prefix is
`spec/function-pointers/foundation.rs` with `min_specialization`. It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/host_effect_specialization.rs` checks every case.
See `docs/parallel/census-batch12.md`.
