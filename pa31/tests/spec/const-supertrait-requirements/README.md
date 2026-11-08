# `[const]` supertrait requirements of const impls

Seven no_core libraries, checked against rustc 1.98. They isolate the `[const]`
supertrait gap that census batch 10 recorded: core's const callable reference
impls, such as `const impl<A: Tuple, F: ?Sized> Fn<A> for &F where F: [const]
Fn<A>`, did not prove the `[const] FnMut<A>` supertrait of `&F` from their
`[const]` premise.

rustc's wfcheck (`rustc_hir_analysis/src/check/wfcheck.rs`) proves each of the
trait's const conditions for a const impl as a `[const]` host-effect goal. The
impl's param-env (`rustc_ty_utils/src/ty.rs`) holds its own const conditions
as `[const]` assumptions, which elaborate to `[const]` supertraits. A plain
supertrait is an ordinary obligation. A `const Tr` where clause is `Tr` plus a
`const` host-effect clause, which a `[const]` assumption does not prove.

- accepted: `const trait Shared: [const] Once` with `const impl<F> Shared for
  W<F> where F: [const] Shared` beside a const `Once` impl over `F: [const]
  Shared`, or over `F: [const] Once` by elaboration; a plain `Shared: Once`
  with an ordinary `Once` impl; ordinary impls of both traits;
- rejected (E0277): an ordinary `Once` impl, an ordinary `F: Shared` premise,
  and a `Once` impl over `F: const Shared`.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (7/7
agree). Batch 10's exploration drafts (`repro/const-super-*.rs`) had other
bytes. The prefix is `spec/function-pointers/foundation.rs`. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/supertrait_effects.rs` checks every case.
See `docs/parallel/census-batch12.md`.
