# Required ordinary initializer completion

After declaration and body readiness completes, the outer executable pipeline
requires every ordinary closed const/static initializer before handing checked
bodies to lowering. One evaluator visits the existing initializer vector once in
declaration order. Associated owners and generic owners remain lazy. Unused const
functions are checked normally but are executed only when a required value calls
them. Mutable statics demand their initializer without reading mutable storage.

The pass uses the same shared index, solved value arenas, checked-body provider,
execution sessions and CFG handoff as earlier demands. No parsing, rebuilt index
or per-initializer whole-table search is introduced. The initializer-free path
does not construct an evaluator. `initializer_visits` counts visited rows and
`initializer_validations` counts required ordinary demands, including cache hits.

The pre-policy control run independently qualified seven invalid unused
initializers, then reproduced all fourteen incorrect acceptances across semantic
and LowIR modes: const/static/mutable-static division by zero, overflow, a closed
const call dividing by zero, and the two PA27 invalid union-read fixtures. Both
modes now reject all seven. Arithmetic failures are ordinary constant evaluation
diagnostics. Union byte initialization/projection still has an explicit execution
boundary: rejecting these invalid programs does not establish valid union const
execution or byte validity support.

Four independently accepted controls retain unused-function, lazy-branch and
associated/generic laziness. Four native controls cover ordinary const/static,
mutable static, skipped failing branch and forward value dependencies. At
8/32/128 widths, three ordinary initializer kinds plus one associated row produce
exactly `3*N+1` visits and `3*N` validations/expression executions, bounded shared
index lookups, and one constructed callee CFG before handoff. Ordinary lowering
then has three function CFGs (callee, main, deliberately unused function), with no
new expression execution or validation pass.

Check, sixteen focused integration tests (including native controls), formatting,
strict all-target/all-feature Clippy and the architecture/file audit passed.
Evidence is under `obj/pa25-initializer-validation/`: `pre-policy.log`,
`initial-cfg-expectation-failure.log` (corrected test expectation only),
`focused-regressions.log`, final numbered logs and `source-sha256.json`.
The source manifest SHA-256 is
`70a4eb1060206ca07e32ba85effec4e8113cbd7327c25da3478079190fc4b5e6`.
The reborrow lint and reviewed-vocabulary test inclusion audit failures are
retained separately; their repairs change neither evaluated source nor policy.

## Remaining PA25 work

- Resume dynamic cross-body pattern dependencies and forward unready enum tags;
  direct named-pattern discriminant roots still have an explicit boundary.
- Define and implement deterministic `--emit-constants`/count contracts, stable
  value/allocation graph identities, and entry-free checked inspection.
- Complete slice/text projection and compressed literal reference execution.
- Qualify and implement raw pointer and union const validity with PA27, retaining
  allocation provenance, initialization and overlap rules.
- Complete promotion/static allocation graph coverage, including associated
  initializer contexts and deterministic export representation.
- Finish owning fixtures, manifest/handout contracts and integrated release
  qualification. PA25 remains planned and incomplete.
