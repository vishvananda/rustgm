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

## Associated selection during ordinary completion

Ordinary initializer evaluation now supplies an explicit typed initializer scope
to the existing availability-aware constant dispatcher. Selection and its proof
use the real initializer body; an initializer is not registered as an anonymous
constant. Anonymous const arguments and discriminants keep their existing scope
and type. The named evaluator restores its previous scope after either success
or failure. The initializer index, shared dependency cache and declaration order
remain unchanged.

This repairs the eager-validation regression in PA20's global constant and
members/mixed inspection fixtures. All six original sources compile again, and
their three LowIR outputs are byte-identical to committed references. Semantic
inspection now includes completed dispatch facts: one default selection for
global, a default and an override for members, and one lifetime/type/const-argument
selection for mixed. Only those rows and their counts differ; they are retained
for integrator review rather than hidden to reproduce an incomplete inventory.
PA22's raw temporary retains its existing precise diagnostic.

At widths 8/32/128, unused ordinary roots reading one associated default keep
selection and dependency-expansion counts linear. Three native controls execute
default/override constants and a static, lifetime/type/const arguments, and nested
named dependencies against independent pinned Rust executables. The initializer
policy, indexing and scalar crate-linking regressions also pass. Formatting,
check, strict all-target/all-feature Clippy and the file audit pass. Evidence and
the exact six view deltas are under
`obj/parallel/pa25-initializer-selection/`. An initial byte comparison failed on
the newly completed semantic facts; no references were changed. A merge-only
duplicate standalone import test was removed in a separate commit before the
final zero-finding audit.

## Initializer-local borrow regions

Explicit borrows, receiver autoref and callable autoref now use the existing
owning-body `ExpressionBorrow` region in initializer and anonymous constant
bodies as well as functions. Being outside a function does not establish static
backing. The existing checked CFG place/promotion proof determines whether the
referent is local, promoted, extended or static; `in_function` keeps its existing
control and eligibility meaning.

Twelve independent Rust object controls cover nine valid transient, mutable,
projected, reborrowed, receiver, literal/static/const-backed cases and three
invalid local escapes. Both semantic and LowIR outcomes agree. Two native groups
execute local mutations, reborrows, const methods and valid static reference
results. The PA20 selection and ordinary initializer policy regressions also
pass, along with formatting, strict Clippy and the zero-finding file audit.

Against immutable `50f6f63e`, the three already-valid literal/static/const-backed
controls retain identical LowIR. Their type views gain the actual borrow region,
reference type, coercion and adjustment; the declared static result is unchanged.
Six previously rejected local-borrow controls now succeed. No references were
regenerated. Evidence and exact view deltas are under
`obj/parallel/pa25-initializer-lifetimes/`; the dev-source manifest SHA-256 is
`fbf77d2b01d5c1a997613e2539a4c0cc609a375228ad223ebf5274339932270c`.
General inline-const expressions inside runtime bodies remain an explicit
unsupported boundary, including PA28's inline-const zero-array repeat control.

## Remaining PA25 work

- Resume dynamic cross-body pattern dependencies and forward unready enum tags;
  direct named-pattern discriminant roots still have an explicit boundary.
- Define and implement deterministic `--emit-constants`/count contracts, stable
  value/allocation graph identities, and entry-free checked inspection.
- Complete slice/text projection and compressed literal reference execution.
- Qualify and implement raw pointer and union const validity with PA27, retaining
  allocation provenance, initialization and overlap rules.
- Complete promotion/static allocation graph coverage and deterministic export
  representation; associated initializer dispatch now retains its actual owner.
- Finish owning fixtures, manifest/handout contracts and integrated release
  qualification. PA25 remains planned and incomplete.

The combined `7249dce9` validation in `rustgm-validation-work` passes 32 focused
release groups and six native groups. PA9, PA19 and PA22 retain their existing
bytes after their separately reviewed boundary/diagnostic repairs. PA20's three
inspection fixtures now retain the dispatches proved during required initializer
evaluation: the default global, default/override pair, and mixed lifetime/type/
const application add one, two and one records respectively. Every prior row and
identity remains unchanged; the matching three LowIR outputs remain byte-identical.
All six source copies independently compile with pinned rustc before regeneration.
The generator changes only those three stdout sidecars and its source provenance
receipt; outcomes, fixtures and the other 1,378 PA20 oracles are unchanged.

Evidence is retained in `obj/parallel/integration-7249/` and
`obj/parallel/integration-7249-recovered/`. The first native attempt stopped when
the shared CPPGM executable no longer matched the locked hash. Exact original
bytes were recovered into checkout-owned immutable pins; the backend smoke test
and all three interrupted native groups then passed. The failed attempt remains
recorded as infrastructure drift. No backend lock was changed. Combined full
publication checks and PA24–PA30 assignment completion remain outstanding.
