# Deep reject before overlap instantiation

Nineteen no_core source-core libraries for pairs of impls of one trait,
checked against rustc 1.98. They isolate census 11's crate record: core's
coherence pairs exhausted the one-million-step overlap budget, and 20,986 of
its 22,895 pairs instantiated both headers although they differ at a rigid
constructor.

rustc's `overlapping_impls` (`rustc_trait_selection/src/traits/coherence.rs`)
first asks `DeepRejectCtxt::relate_infer_infer(..).args_may_unify` whether the
two trait references could unify at all, and only then instantiates and
equates the headers. Parameters on both sides, aliases, projections,
inference, bound and placeholder types may unify with anything, and lifetimes
always may. Rigid constructors compare kind, mutability, array length, tuple
and function arity, function safety and ABI, principal trait and nominal
identity, to `STARTING_DEPTH` 8 (`rustc_type_ir/src/fast_reject.rs`).

- disjoint: four nested nominals under one simplified self type, trait
  arguments, array lengths, reference and pointer mutability, tuple and
  function arity, function safety and object principals;
- guards: a difference under ten nested nominals, past the depth, which the
  complete check decides, and a closed projection that normalizes to a
  different type. rustc's reject sees that projection as an alias and leaves
  it to `overlap`; RustGM forms the header already normalized, so its reject
  sees the rigid difference;
- opponents (E0119): a parameter, crossed arguments, a lifetime, a const
  parameter, a difference past the depth that does unify, a type alias, a
  normalizing projection and a projection of a parameter. The last three
  differ only behind the alias or under the projection, so the reject must not
  answer them.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (19/19 agree). The prefix
is `spec/function-pointers/foundation.rs` unchanged. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/rejected_overlap_pairs.rs` checks every case and how many
pairs the reject answers, then audits every PA31 coherence and specialization
control: no pair the reject answers has headers whose complete equation
succeeds. See `docs/parallel/census-batch11.md`.
