# Const parameter types under the owner's feature bounds

Five staged-API source-core libraries with `min_adt_const_params`, checked
against rustc 1.98. They reduce census 11's record after its overlap budget
was repaired: "const parameter type must implement actual ConstParamTy" at
core's `TransmuteFrom<Src, const ASSUME: Assume>`
(`mem/transmutability.rs:85-95`). Its `impl ConstParamTy_ for Assume`
(`mem/transmutability.rs:291-293`) holds only under
`#[unstable_feature_bound(transmutability)]`.

rustc's `check_param_wf` (`rustc_hir_analysis/src/check/wfcheck.rs`) proves
`Ty: ConstParamTy_` under the param-env of the parameter's owner, and
`predicates_of` adds each `#[unstable_feature_bound]` of an item to that
param-env. `may_use_unstable_feature` (`rustc_type_ir/src/infer_ctxt.rs`)
holds a feature bound when the param-env names it. Otherwise it holds only
outside a staged-API crate that enables the feature.

- core's shape: the trait carries the matching bound, and so does a function
  with the same const parameter;
- opponents: the trait without a bound, the trait whose bound names another
  feature, and the unbound trait with `where Assume: ConstParamTy_`. rustc
  proves that global where clause in an empty param-env first
  (`check_false_global_bounds`), so it cannot stand in for the feature.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (5/5
agree). The first oracle rejected every case for the fixture itself, which
is not an outcome. The recorded amendment makes `Assume` a fieldless enum and
leaves every declared outcome unchanged. The prefix is
`spec/const-parameter-derive/nominal-const.rs` through its `fmt` module. It
keeps Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/unstable_bound_parameters.rs` checks every case.
See `docs/parallel/census-batch11.md`.
