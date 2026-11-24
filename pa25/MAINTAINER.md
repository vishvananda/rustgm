# RustGM Programming Assignment 25: Constant evaluation

The [student handout](README.md) gives the course instructions.

The [const closure member checkpoint](../docs/pa25-closure-members.md) accepts
actual closure-owned trait calls and clears six older prepared-lowering gaps.
Concrete runtime execution qualifies; default premises, CTFE, ordinary imports
and original-core artifacts remain required. Whole assignments remain planned.

The [captured const-call checkpoint](../docs/pa25-captured-const-calls.md)
authenticates actual closure ownership and complete enclosing arguments.
Returned adapters and runtime execution qualify; cleanup, CTFE, ordinary imports
and original-core artifacts remain required. Whole assignments remain planned.

The [unsafe const-member checkpoint](../docs/pa25-unsafe-const-members.md) consumes
exact discharged safety rows and complete source calls.34 ordinary/source outcomes
and twelve native programs pass, including the new simple CTFE consumer. Original
core, earlier payment/bootstrap contracts and whole assignments remain required.


The [const Try protocol checkpoint](../docs/pa25-const-try-calls.md) authenticates
implicit calls through their existing source plans and preserves residual effect,
conversion and destruction duties.18 source outcomes, one ordinary opponent and
ten native programs pass. Original core and unfinished CTFE/bootstrap contracts
remain required; whole assignments retain their prior planned status.


The [borrowed const-callable checkpoint](../docs/pa25-borrowed-callables.md)
authenticates the existing source reference adjustment and final call-trait borrow.
Seventeen source outcomes, one ordinary opponent and nine native programs pass.
Source CTFE payment, ordinary declaration preparation and original core artifacts
remain required; whole assignments retain their planned status.


The [explicit const call-member checkpoint](../docs/pa25-explicit-call-members.md)
retains the original Drain adapter, actual Fn member identity, tuple ABI and
mutable receiver proof. Full assignment, bootstrap and original artifacts remain
required; source CTFE payment is still an accepting boundary.


The [projected const promise checkpoint](../docs/pa25-projected-const-members.md)
retains conditional associated-result parent proof, actual source signatures,
ordinary/source opponents and native observations. Whole-PA and original-library
artifact requirements remain open.


The [generic-subject default checkpoint](../docs/pa25-default-generic-subjects.md)
qualifies ordinary custom const-trait premises here and actual source reference
requests in PA31. Bound imports and the separate CTFE signature-payment consumer
remain required; no whole assignment is released.

The [generic const-call checkpoint](../docs/pa25-generic-const-calls.md) preserves
ordinary callable predictions here and actual source-role counterparts in PA31.
All15 source outcomes and three native observers qualify; ordinary Destruct
imports and whole-assignment requirements remain open.

The [default-supertrait checkpoint](../docs/pa25-default-supertraits.md) qualifies
ordinary const-trait default minima while retaining two Destruct import boundaries.
Actual source roles and unchanged Ord::max belong to PA31; full bootstrap and
whole-assignment requirements remain.

The [generic conditional-member checkpoint](../docs/pa25-conditional-member-calls.md)
retains ordinary custom const-trait minima here and actual language-role source
cases in PA31. Defining legality accepts checked normalized generic applications;
actual const dispatch and all full library/whole-assignment requirements remain.

Status: **closed at its finite contract.** What is graded is the contract under
[Graded contract](#graded-contract). The rest of this brief is buildout guidance
for the work that contract defers, which
[PA44 carries](../pa44/MAINTAINER.md#carried-requirements). Checkpoint notes below
that say the assignment remains planned predate the closure and describe that
deferred work.

The [wide constant product checkpoint](../docs/pa25-wide-constant-products.md)
qualifies98 new exact/classified results and genuine source Mul calls, with
native byte observers and bounded scaling. It addresses release21's original
midpoint HI/MAX constant. Original-library verification and all other required
constant, runtime and transport boundaries remain open.

The [wide constant division checkpoint](../docs/pa25-wide-constant-division.md)
qualifies exact f16/f128 division with74 observed results, actual source calls,
native byte images and bounded scaling. Ordinary CTFE belongs to PA25 and source
operator authority to PA31. Original-core verification and the separate runtime,
metadata and ABI requirements remain open.

The [inferred anonymous-array reductions](../docs/pa25-anonymous-array-outputs.md)
retain ordinary local-output conditions and lifetime opponents. Actual source
wrapper counterparts belong to PA31. Their graph repair passes16 complete
outcomes, real output-port scaling and previous native controls.

The [anonymous implied-bound continuation](../docs/pa25-anonymous-implied-bounds.md)
retains receiver/input well-formedness in original inline constants. Ordinary
reductions live in PA25 and actual source-wrapper counterparts in PA31; all
remaining assignment and original-library artifact requirements stay open.

The [captured closure integration](../docs/parallel/captured-pointer-main-join.md)
executes real checked shared/mutable/owned captures and reuses their originals
through existing source components. Reached pointer guarantees use actual CTFE
provenance and native fallback bodies. Generic/conditional caller obligations,
receiver adapters, bootstrap calls, complete capture destruction and imported
transport remain required; this finite checkpoint does not release PA25.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.

## Graded contract

PA25 is graded as a finite contract: the published manifest of each released
component, less the fixtures on that component's deferred list. A fixture that
no released manifest claims is not graded, whatever this brief says about it.

| Component | Manifest | Cases | Deferred list |
| --- | --- | --- | --- |
| `const-machine` | [const-machine.json](tests/units/const-machine.json) | 113 | [2 maintainer-level limit contracts](../docs/pa25-const-machine-release.md) |

From the repository root, `python3 scripts/course.py pa25 --app BIN` runs the whole
contract and `--unit const-machine` selects one component. `make test`,
`make test-rustc` and `make test-native` include PA25.

Not released: `const-memory` and `const-callables` are partial.

PA25 is closed at this contract. [tests/manifest.json](tests/manifest.json) is
exactly the union of the released manifests above and runs against their
references; `python3 scripts/finite_contracts.py compose pa25` rewrites it when a
component is released. Closing waives nothing. Each unreleased unit and each
[requirement row](../docs/assignment-requirements.json) a released unit does not
cover is deferred to [PA44](../pa44/MAINTAINER.md#carried-requirements), which cannot
be complete while one of them is unreleased. Everything the self-hosting corpus
needs stays required through PA32–PA36, PA44 and PA45.

| Deferred to PA44 | Still owed |
| --- | --- |
| `const-memory` | Symbolic constant storage and provenance (partial) |
| `const-callables` | Advanced const queries and callables (partial) |
| `source-runtime` (PA31) | Compiler runtime and source callable integration (partial) |
| Row 25.3 | Named/generic/associated initializer demand and readiness |
| Row 25.4 | Array lengths, discriminants, constant patterns and const arguments |
| Row 25.5 | Inline/anonymous and generic closure-owned const bodies |
| Row 25.6 | Const closures |
| Row 25.7 | Statics, promotions and symbolic allocation graph |
| Row 25.8 | Array/slice/rest/DST constant operations |
| Row 25.9 | Actual source const calls, selectors and cleanup |

## Required implementation

Const functions, statics, promotions and symbolic allocation provenance.

The [const inherent checkpoint](../docs/pa25-inherent-block-checkpoint.md)
implements actual implicit members, exact source authority, generic conditional
bounds and distinct runtime/CTFE materializations. Its51 independently checked
language/library outcomes and real mixed-call native consumer preserve unused
body, ordinary-impl and missing-bound opponents. Authenticated imports and the
rest of this assignment remain required.

The [canonical aggregate const-value boundary](../docs/pa25-aggregate-const-values.md)
provides typed products, nominal variant identities, compressed repeats and
evaluator/reader integration for the original core `Assume` dependency. Suspended
queries publish canonical identities, and readers/VM allocations share immutable
values while preserving typed mutation and authority checks. The local source
profile now admits structs, enums, tuples and fixed arrays after actual feature,
marker and field proofs, including forward defaults and const-function consumers.
Its explicit fourteen-case corpus belongs to the existing const-machine component.
Wider value/feature contexts and PA29 transport remain required; the whole
assignment remains planned.

The [source allowance checkpoint](../docs/pa31-const-fn-allowances-checkpoint.md)
implements exact rustc_allow_const_fn_unstable body effects, keeping recursive
const API checking separate from feature requests, constness and unsafe
permission. Genuine imported const evaluation consumes transported body policy
and its ordinary defining-environment producer. This finite component does not
complete the remaining constant machine or whole assignment requirements.

Const closures extend PA24's actual anonymous identities and capture/call-trait
machinery. Enforce const creation contexts separately from runtime invocation:
the pinned profile accepts a const-fn-produced closure invoked at runtime, and
rejects creating a const closure directly in an ordinary runtime context. Cover
real CTFE invocation, captured storage and instance/ownership proofs; grammar
and the const_closures feature request alone do not establish these semantics.

The [const adapter reductions](../docs/pa25-const-adapter-reductions.md) retain
ordinary conditional-callable requirements in PA25 and actual source-role
contrasts in PA31. The finite scoped defining-proof and local-child completion
checkpoint now passes; ordinary bootstrap and three concrete source factory
runtime/CTFE positives remain required. No whole assignment is released.

The [conditional callback packet](tests/spec/callback-formation/README.md)
separates function-value formation from actual invocation effects. Its paired
ordinary/source controls, bounded creation/invocation work and three native
observers pass; two paid alias/reference LowIR artifacts remain explicit positives.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

The released [typed constant machine component](../docs/pa25-const-machine-release.md)
now declares its own source/inspection/native corpus independently of whole-PA
activation. Its explicit manifest is tests/units/const-machine.json; the parent
assignment and all remaining requirements retain their planned status.

`dev/rustgm --emit-constants -o OUTPUT SOURCE` and `--count-constants` inspect
completed values through the entry-free checked pipeline. Earlier stage views
remain available. The implemented draft vocabulary, graph identities, ordering,
resource/error contract and independently checked example are documented in
[the constants view draft](../docs/pa25-constants-view.md). Review its complete
fixtures and generated oracles before declaring this assignment active.
The [sequence execution checkpoint](../docs/pa25-sequences.md) records implemented
array-backed slice operations, scaling invariants and remaining sequence gaps.
The [DST field-metadata checkpoint](../docs/pa25-dst-tail-metadata.md) forwards
already-checked metadata to final unsized fields while retaining canonical
physical backing types, reference permissions and separate logical bounds.

## Tests to develop with the reference

const restrictions; pointers; recursive calls; limits; const generics.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.

The [source wrapper checkpoint](../docs/pa31-source-wrappers-checkpoint.md) qualifies
real source wrapper constants through typed byte/result traversal, compressed
arrays, bounded shared graphs and failed-budget rollback. Its local/source/native
component leaves general imported nominal transport and broader pointer metadata
open. The [MaybeUninit native checkpoint](../docs/pa31-maybe-uninit-native-checkpoint.md)
qualifies the local source constant/native profile; general validity/runtime and
imported requirements remain. Original core currently reaches genuine
TransmuteFrom and its structured Assume const argument.

The [anonymous associated-borrow checkpoint](../docs/pa25-anonymous-associated-borrows.md)
qualifies five ordinary outcomes and three real native programs; actual source
layout counterparts stay in PA31. Original generic inline values retain their
caller and selected instances, while two complete source consumers remain
explicit requirements. This does not extend the released const-machine component
or complete the whole assignment.

The [initializer-storage checkpoint](../docs/pa25-initializer-storage.md) retains
computed temporary backing using existing lexical extension facts, with safe
PA25, unsafe PA27, C ABI PA28 and source-role PA31 reductions. Local/callee expiry
and mutable temporary restrictions remain checked. Qualified native programs do
not replace the required named mutable-static and library metadata surfaces.
Whole-assignment status and the separate const-machine component are unchanged.

The [associated Copy root checkpoint](../docs/pa31-root-copy-qualification.md) reuses
scoped qualification for projected roots and filters only actual NoDrop answers.
Thirteen outcomes, eighteen native programs and bounded work pass; three unused
negative definitions remain required. Original-core retry/full CI are pending;
whole assignments and original library artifact gates remain unchanged.
