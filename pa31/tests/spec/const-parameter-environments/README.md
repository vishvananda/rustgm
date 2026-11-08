# Several const parameters under an owner's lifetimes

Five staged-API source-core libraries with `min_adt_const_params`, checked
against rustc 1.98. They reduce census 12's lookahead record "invalid lifetime
constraint identity" at core's `Arguments::new<const N: usize, const M:
usize>` in `impl<'a> Arguments<'a>` (`fmt/mod.rs:727-737`), at its second
const parameter. Census batch 11's section 4 introduced it: each const
parameter type is proven under the owner's environment, and the second proof
of one owner reused the first one's lifetime numbering.

rustc's `check_param_wf` (`rustc_hir_analysis/src/check/wfcheck.rs`) proves
each const parameter's `Ty: ConstParamTy_` under its owner's param-env, on its
own, and reports E0741 for a type that does not implement it.

- core's shape, three const parameters under two lifetimes, a function with
  its own lifetime, and a trait method of `trait Template<'a>`: accepted;
- opponent: core's shape with `const M: Plain`, a unit struct without
  `ConstParamTy_`. The second parameter is still proven, and fails.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (5/5
agree). Exploration drafts with other bytes were run first, with rustc and
the census 12 pin; they are recorded in the declaration. The prefix is
`spec/const-parameter-derive/nominal-const.rs` through `SimdAlign`. It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/unstable_bound_parameters.rs` checks every case.
See `docs/parallel/census-batch11.md`.
