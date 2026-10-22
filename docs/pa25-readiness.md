# PA25 fixture readiness inventory

PA25 remains planned. The integrated compiler at `dc459f18` was checked against
all 122 registered or previously declared draft inputs without generating any
text oracles. Rust 1.98.0 object emission independently confirms 70 accepted and
52 rejected sources. All 70 positives pass the type and constants views; 62
also emit LowIR. All 52 negative sources reject LowIR.

Eight positive entry-free library sources cannot yet emit library artifacts
because their exported aggregates or generic functions exceed PA29's current
scalar metadata profile. Their PA25 constant inspection succeeds. These are
`noncopy-repeat`, `zero-repeat-fault`, `nested-zero`, `parenthesized-repeat`,
`inline-closure-pointer`, `generic-unused`, `nested-generic` and
`zero-with-inline-type-length` in `spec/inline/`.

Four negative generic inline sources pass lazy constants inspection and then
reject on demanded constant arithmetic during LowIR: `zero-repeat`, `division`,
`zero-erased` and `nonzero-erased` in `spec/inline-specialized/`. A deferred
inventory row is not a successful evaluated value. The other 48 negatives also
reject constants inspection. The released manifest must preserve this distinction
and explicitly classify mode differences.

Two scalar inline-owner fixtures had moved into PA25 but retained declarations
only in PA28's original hypothesis file. `tests/inline-owner-outcomes.json` now
registers them with their owning assignment. Their original bytes and independent
expectations are unchanged; both accept all three views. They are code-generation
controls with no runtime value assertion.

Evidence is retained in `obj/parallel/pa25/readiness-v2/`, including all source
hashes, original declarations, diagnostics and compiler identity. The first
inventory stopped because its legacy-schema adapter invented a program crate
kind for a library fixture. V2 follows the existing owning maintainer tests'
library qualification. The failed v1 evidence and explicit classification repair
are preserved; no acceptance expectation was changed.

This inventory does not cover every required constant-evaluation family or prove
assignment completion. Dynamic named-pattern/enum readiness, slice/text execution,
static/promotion allocation coverage and pointer/union validity still require
reconciliation against current implementation and independent controls. Final
handout/manifest/oracles, complete native qualification and release performance
checks remain necessary. Earlier historical checkpoint documents are evidence
for their stated snapshots, not the current completion checklist.

## Current release blockers (caller and projected-result work)

This is the single continuing blocker list for the caller/result lane. The
122-source inventory above is historical evidence, not a completion claim.
Accepted caller transport and bounded enclosing-instance observation preserve
original and instance proof, actual result identity, escape checks, and caller
debts. They do not close the following required families:

- Demanded associated results whose original declarations retain projections.
  Ordinary demanded reference/tuple returns and repeated distinct generic callers
  now execute through checked specialization. The projected tuple/record
  pattern and nested associated reference/tuple families now also execute through
  that path. A generic inline constant containing the same projected associated
  use now passes semantic normalization in the direct and nested controls. Its
  demanded direct, nested and sibling inline cases now execute through the sealed
  source initializer context and checked anonymous result interface described below.
  Inline children requiring a conditional parent output remain required followup.
- Projected named-pattern operands now classify their selected reference type
  before applying pattern ergonomics. Direct, paid, nested and declaration-time
  const-function controls pass source/native qualification; unpaid callers reach
  their actual lifetime proof. The two earlier owning fixture oracle additions
  below remain a publication prerequisite, rather than missing implementation.
- Selected type arguments containing open caller lifetimes now route through
  recursive, bounded Instance-keyed observation and actual caller ownership.
  Ordinary, pattern, inline, nested, projected and nested-callable controls execute.
  A paid trait-implementation premise still requires actual caller proof before
  specialization may assume it; a source handle alone grants no such authority.
  Unused rigid type parameters in generic constant patterns also remain a required
  positive family, separately from parameter-free open-Type routing.
- Source-associated uses inside conditional nested-output contracts. The original
  and instantiated conditional requirements must be discharged by the actual
  caller; no inherited summary is execution permission.
- The generic cleanup quadratic is repaired by integrator checkpoint `33d2ba12`:
  exact transition/work invariants and nine immutable inputs retain all outputs;
  width128 uses 64–73% fewer instructions. The combined checkpoint passes 2,588
  regular tests, 267 native tests and all 8,360 active text cases. Publication
  performance review remains integrator-owned; no worker cleanup work is pending.
- The released PA25 manifest, constants-view oracles, handout/counts, full owning
  source/native qualification and frozen performance review remain integrator
  gates. PA25 is still planned and no oracle was changed here.

The original projected-result sources, all 29 declared hypotheses, their pinned
Rust 1.98 full-object outcomes and the independently failing combined compiler
comparison are retained under `obj/parallel/pa25-projected-result-routing/`.
`restoration-first` shows the inference repair separately from the later
execution gaps. Unpaid caller/result, dangling and distinct-application negatives
retain their original bytes and outcomes. Upcoming checkpoints should update
this list as families become complete rather than treating these boundaries as
accepted language rejections.

### Selected opaque-application normalization prerequisite

A generic associated constant can supply an opaque result whose selected input
lifetime is a fresh alias of the caller's lifetime. Before publishing that opaque
application, normal-form completion now restores its exact application through
only its own completed selection proof. Every equality and formation debt remains
in that proof. A changed application returns through the existing replacement
normalizer; an unchanged application remains opaque. No inference equality rule
or execution permission changes.

The query retains the original requested family arguments separately from the
opened selection arguments. Comparing against the opened arguments incorrectly
redirected an unchanged GAT request to its own active normalization root. That
failed candidate and its 86 PA21 cycle failures remain recorded in
`obj/parallel/pa25-projected-result-routing/restoration-tail/`; the corrected
original-request control passes the full checked pipeline.

The final focused receipt is `restoration-gat/`: four private controls, two
existing restoration controls, strict release all-target/all-feature Clippy,
file audit (1,605 files, zero findings), 46 associated-declaration source tests,
and a nine-program native group pass. Controls cover missing/foreign equalities,
distinct applications, preserved nested public binders and unpaid lifetime debts.
Repeated normalization pairs at 8/32/128 uses charge 368/1,472/5,888 work units;
the existing restoration consumers keep their uncounted path. The ordinary512
and constcall64 outputs remain byte-identical.

The final immutable pin is recorded by `restoration-qualified/identity.json`
(SHA-256 `3d505d6484806f5ae0ecb24a3a892197544a2d13d06026b728606cd9044021bd`).
All 8,255 worker-base active text cases were compared, including the published
467-case PA24 archive: 8,254 are byte-identical. One independently rejected PA20
case, `recursion/canonical-queries/lowir/different-result-lifetime.rs`, still
rejects but changes from the trait-argument lifetime diagnostic at byte 92 to
the associated-type lifetime diagnostic at byte 115. That exact delta is pending
integrator review; no reference was changed. The text guard therefore exits 1.
This prerequisite advances original Rust-positive cases to the execution gaps
listed above; it does not complete demanded projected-result execution.

### Selected arguments and normalization allocation ownership

The next checkpoint composes selected constant arguments through only that
selection's retained equalities, then through the original caller's existing
alias transport. All selection obligations remain attached to the caller.
An empty equality set is an identity operation, including when the original
proof retains an unpaid type-outlives requirement. The opposing active PA20
`unpaid-type-bound` case must still reach its required-lifetime rejection.

Instance normalization can allocate additional identities while completing both
new proofs and existing suspended proof debts. A stack-only operation recorder
now observes those exact allocation intervals at their completion sites. Before
the existing reservation publishes them, it validates the full caller instance,
body/contract identity, proof position/source, contiguous retained coverage and
unchanged global frontier. It never fills gaps or infers ownership from a high
water mark. Ordinary normalization has no recorder. Temporary snapshot and row
capacities are checked before growth against the caller namespace's remaining
budget (at most 64 MiB), with one million validation steps/rows as upper bounds.

`owned-normalization-closing-second/` records six private ownership controls,
strict release all-target/all-feature Clippy, file audit (1,607 files, zero
findings), 46 associated source tests and two native groups executing 13 programs.
The controls include resumed existing debts, foreign gaps, stale retained proofs,
different enclosing instances and interleaved unrelated sources. For 8/32/128
resumed proofs, validation work is 46/166/646 and diagnostic payload storage is
180/660/2,580 bytes; this is a bounded normalization claim, not a cleanup scaling
claim. `selected-identity-closing/` additionally passes the four alias controls
and four focused source tests after the empty-equality correction.

The immutable final binary and production source-map hashes are recorded by
`selected-qualified/identity.json`; `selected-qualified/review.json` retains
source and expected/actual output hashes for every observed PA20 delta, including
the repaired unpaid-bound negative. Six semantics views now show the actual
selected caller/static arguments instead of their private selector aliases.
Together with the earlier rejected-result diagnostic change, these require
integrator review before oracle publication. No reference was changed here.

### Projected patterns and nested initializer dependencies

The next bounded checkpoint routes only the exact selected initializer root
through its retained caller when its arguments remain open. Other expressions
and the ordinary concrete/materialized path retain their strict result checks.
Seal preparation normalizes the declaration and request in the same dispatcher;
the source use retains its original projected operand separately from the
canonical requested result. A private observer rejects swapping those identities.

Original initializer preparation now recognizes its own rigid type/const
parameters without trying to select a concrete implementation. It validates the
actual declaration, body, original contract and retained source-local identities.
It preserves the complete associated operand for mandatory instance dispatch.
The original proof and eligibility pass still run before instantiation, and the
instance repeats ordinary dispatch, promotion and full proof. A lifetime-only
unresolved operand and foreign/missing source ownership cannot use this entry.
Bodies without associated operands do not allocate these temporary catalogs.

Dependencies published by an initializer must close their source-local aliases
before caller arguments are substituted. Publication records only the newly
appended constant/callable rows and exact main/late equation ranges for that body.
The existing equality closure supplies canonical identity anchors; it adds no
relations or proof authority. A bounded observer and fixed restore folder update
those rows transactionally. Source contracts and every original equality and
formation debt remain unchanged. Foreign/interleaved ranges fail. The private
external-ID collision control verifies canonicalize-original then substitute-caller
order; a missing equality preserves the old identity and strict admission failure.

`publication-tail/` retains the ten original/template/result/publication private
checks plus all five selected-cache authority checks. `publication-final/` passes
strict release all-target/all-feature Clippy, the architecture/file audit (1,612
files, zero findings), the affected result-identity private check, a 13-source
matrix (eight positives and five negatives), four retained-result test functions,
46 associated-declaration test functions and one native group executing eight
programs. Harness/layout and upstream verification also pass. The public compiler
and full combined release gates remain integrator-owned. No oracle was changed.

For 8/32/128 newly appended dependencies, measured publication work is
180/588/2,220. Temporary scratch is bounded to 64 MiB and one million work units;
the original owned-identity catalog is bounded separately. There is no new
Body/Target/Request/dependency representation or global proof graph. These are
changed-path bounds, not claims about the separate generic cleanup quadratic.

The four earlier projected/nested positives retain their original authored bytes.
A const-function caller also passes. The added unused generic non-const call and
unpaid source lifetime relation reject at their real permission/lifetime checks.
`publication-closing/owner-outcomes.json` preserves all four independent owner
hypotheses: its inline-projected positive accepts Rust object emission but still
fails the compiler's semantic associated-type lookup at byte 261. The declaration
and output hashes remain in that receipt; this boundary is required followup.

The first closing batch stopped at that independently valid inline boundary.
Subsequent batches corrected routine lint findings and a private test's unreviewed
`include_str!` by using normal runtime fixture loading. Their failures are retained;
none was counted as a passing source result or used to change a language outcome.

`publication-qualified/identity.json` freezes the candidate and production map.
All 8,255 worker-base active cases were compared, including the immutable
published 467-case PA24 archive. There are no additional text differences from
`395193a6`: the same seven PA20 source/output hash pairs in the prior review
bundle remain different from committed oracles (8,248 exact comparisons).
The text guard therefore exits 1 pending that existing central review. Both
ordinary512 and constcall64 outputs are byte-identical. No worker benchmark or
full Cargo publication run was repeated.

### Single-pass original operand validation

Review identified that per-operand subset checks rescanned all previously
collected parameter/lifetime uses. Original dispatch now retains its shared
occurrence caches and validates the accumulated sets once, before rewriting,
finishing the dispatcher or letting a template escape. Direct foreign lifetime
arguments still reject immediately; nested foreign identities and foreign
parameters reject at this mandatory boundary. Validation visits and failures are
charged, and no new owner, walker, global graph or ordinary representation is
introduced.

`original-validation-closing/private.log` records six passing ownership/scaling
checks through the actual original initializer entry. For 8/32/128 distinct
roots the measured routing work is 68/236/908, exactly `7*N+12`; repeated roots
cost 61/205/781, exactly `6*N+13`. These measure this dispatch/validation path,
not whole-body cleanup or compile time. `original-validation-tail/` passes strict
release Clippy, audit (1,612 files, zero findings), the 13-source matrix and all
eight native programs. The first private attempt retained an incorrect assertion
that a direct lifetime would wait for final validation; its existing immediate
rejection is preserved, with an additional nested foreign-lifetime boundary test.
Test-only lint corrections followed the passing six-test run.

`original-validation-qualified/identity.json` pins the exact candidate. All 8,255
active worker-base cases have the same outputs as the preceding checkpoint;
the same seven inherited PA20 differences remain pending central oracle review.
Both common output controls remain exact. Its separate `inline-hypotheses.json`
independently qualifies four Rust-positive inline/nested/closure/paid-lifetime
sources and one foreign-lifetime negative before the next implementation edit.
The unmodified compiler still fails their earlier inline normalization frontier;
that receipt does not claim their real lifetime checks have yet been reached.


### Inline query and original-template prerequisite

The inline child now opens its readonly query premises through the exact existing
nested `Interface.opening`. Its projection and trait queries share that opened
namespace, including synchronous, canonical and suspended selections. Original
method environments, imported terms, public binders and proof debts remain intact.
Mapped children build their own trait assumption cache instead of inheriting the
parent's alias cache. Registering the inline universals before local holes prevents
a selected equality from restoring an unrelated older implicit argument identity.

Provider construction of an original anonymous template now uses the same rigid
operand classification and single mandatory validation as original named
initializers. The entry checks the actual registered inline source, result,
universals, declaration, body key and contract, and rejects an instantiated body.
Concrete dispatch remains strict. No stored field, owner, frame or execution
permission was added; the ordinary query path can clone its existing environment
`Arc`, and nested queries reuse the existing immutable projection environment.

`tests/inline-query-outcomes.json` declares eleven unchanged sources. Three scalar
direct/nested/closure programs pass all checked views and execute natively. The
foreign-parent and unused-runtime negatives reach their actual checks. The original
projected reference direct/nested cases now pass types; execution still rejects
open initializer context. Closure/paid reference controls still require their
checked conditional parent, and the unpaid open-type control still reaches its
explicit implementation boundary. Those cases remain required work; their
unsupported diagnostics are not language rejections or completion evidence.

Evidence is under `obj/parallel/pa25-projected-result-routing/inline-*`.
The closing strict release Clippy and audit pass (1,613 files, zero findings).
Eight private ownership/linear-work checks, the two source functions, 36 existing
exact-place tests, 22 nested-evaluation tests, three new native programs, harness
and upstream checks pass. A stale file-stem filter and then a partial exact-case
filter were refused; both attempts are retained and excluded from the passing
counts. The corrected exact selections reuse the same unchanged source snapshot.

`inline-qualified/identity.json` pins the candidate and production source map.
All 8,255 active worker-base comparisons, including the immutable published
467-case PA24 archive, preserve the preceding checkpoint's outputs: 8,248 exact,
with only the same seven inherited PA20 hash pairs pending central review.
The text guard exits 1 for that existing review set; it introduces no new delta.
Both common output controls remain byte-identical. Full combined publication,
newer public-base fixtures and performance qualification remain integrator-owned.


### Checked source initializer inline contexts

A demanded inline constant inside an open-lifetime associated initializer now
retains the existing sealed `SourceComputation` handle in its enclosing context.
The handle is validated against the declaration catalog, selected instance,
original source, caller ownership and requested result before cold or warm
provider/cache use. Ordinary initializer contexts keep their concrete admission
rule. Inspection exposes source-only completion without inventing a typed value
root, and diagnostic ordering uses semantic identities rather than handle IDs.

Original anonymous eligibility and full proof remain mandatory. The specialized
anonymous body retains its actual result and proves a coercion to the separately
imported requested result in the same Bound universals. Lifetime-erased shape
comparison checks representation only; it supplies no lifetime relation. The
actual result is restored through the exact registered caller namespace before
selected arguments are substituted, matching the order used for the body itself.
Every instance obligation is then proved before code publication.

Only a fully proved immediate parent may transport its prepared inline request
results. One bounded folder consumes that parent's retained equality facts and
exact selected arguments. It preserves nested binders and rejects missing or
foreign anchors, sibling parents and shape-only/outlives-only substitutes. It
never walks an ancestor chain, changes original Body/Value types, or borrows a
conditional output summary as permission. The interpreter consumes the exact
prepared destination request rather than searching by a coincident type key.
Source initializer code has a lazy witness-keyed index, keeping distinct callers
with equal selected instances/results separate. Cold and warm requests validate
both the actual body result and separately published requested result.

The unchanged `required` and `nested` fixtures now pass all three views and native
execution. A Copy-bounded two-sibling fixture does likewise. The original
unconstrained sibling hypothesis is preserved separately: Rust rejects its generic
destructor, and demanded compiler evaluation rejects destructors as well. No
positive was rewritten to hide that outcome. Closure/paid-lifetime inline outputs
still require their checked conditional parent contract; the foreign-lifetime
negative currently reaches that earlier boundary rather than a claimed final
lifetime check. Open type arguments remain a required implementation family.

Evidence is under `obj/parallel/pa25-projected-result-routing/`.
`inline-source-tail/` passes strict all-target/all-feature release Clippy, the
architecture/file audit (1,615 files, zero findings), four immediate-parent
invariants, the source/frontier functions and six native programs, harness and
upstream checks. `inline-parent-transport-ninth/` additionally retains six context
and one source-cache authority test. Controls cover foreign catalogs, stale or
wrong result/caller ownership, equal selected/result handles with different
callers, nested binders and sibling parent identities. Earlier failed hypothesis,
test setup and lint attempts are retained and excluded from passing claims.

`inline-source-qualified/` passes the four existing storage/result-interface
invariants, both exact independent-selection/lookup-readiness smoke tests and all
22 nested-evaluation tests. Its immutable candidate SHA-256 is
`1c85414307da7cb26011a0ebba448656ec98104e155da5684f8ad638f921d1c2`;
`identity.json` also records the 928-file production map. All 8,255 worker-base
active comparisons, including the immutable published 467-case PA24 archive,
preserve the preceding checkpoint: 8,248 match oracles, with exactly the same
seven inherited PA20 source/output hash pairs in `review.json`. The text command
exits 1 for that existing central review set. Both ordinary512 and constcall64
outputs remain exact. This does not claim the newer 8,360-case public matrix;
combined release and performance qualification remain integrator-owned.

Measured layouts are Context 12 bytes, InlineInstance 20, BodyKey 20, Target 28,
Request 56 and Code 336. Code and prepared operands keep their existing layout;
the execution Catalog adds one lazy optional boxed map pointer (8 bytes on the
qualified host), with map/row retention charged only on source publication.
The parent folder bounds scratch at 64 MiB and visits at one million, including
failed traversals. Full witness validation is preserved; no sealed reuse cache or
ordinary-path proof shortcut was introduced. PA25 remains planned.


### Selected reference pattern classification

Pattern ergonomics previously inspected only the associated constant's raw
canonical declaration. An inherent constant declared as `T::Out` could therefore
peel a reference scrutinee before its selected output normalized to a reference.
The projected path now constructs its existing associated inference term once,
asks the existing projection machinery for its head, and passes that same term
into constant-pattern typing. Its original selection, normalization debts,
structural equality, value readiness and caller proof remain mandatory. The
ordinary nonprojected path is unchanged. No retained field, allocation owner,
execution permission or general inference equality rule is added.

The owning language rule is the pinned Reference
`doc/rust-reference/src/patterns.md:294`, section
`patterns.ident.binding.non-reference`: const patterns *of reference types* are
excluded from non-reference patterns. The adjacent automatic-dereference rules
therefore differ from pinned rustc's behavior for non-reference named constants.
Published `pa13/README.md:113` and the six `autoderef-*-implicit` manifest entries
explicitly retain this discrepancy: course acceptance is zero while Rust 1.98
rejects. Those six outcomes are preserved. The new projected scalar control
likewise declares course acceptance and pinned-Rust rejection separately; it is
not a same-source parity or native Rust claim.

The first shortcut incorrectly disabled dereferencing for every named constant.
Although its focused Rust controls passed, it broke those six published PA13
cases. That failed implementation and text evidence are retained in
`reference-pattern-closing/` and `reference-pattern-qualified/`; no oracle or
published expectation was changed. A later compile-only attempt also retained
an accidental extra helper parameter before the corrected projected-only path.

`reference-pattern-resolved-tail/` passes strict all-target/all-feature release
Clippy, architecture/file audit (1,615 files, zero findings), the expanded
24-source pattern matrix, seven existing constant-pattern tests, nine existing
reference-pattern tests and all 15 Rust-positive native programs. The source
matrix retains 15 Rust-positive cases, eight rejecting cases and the one explicit
Reference/rustc discrepancy. The new paid/unpaid caller pair reaches actual value
execution or the required lifetime rejection, respectively. A const root before
its trait, impl and callee declarations, a const-function caller, nested patterns,
and opposing referent/mutability cases are included. Original selected-reference,
paid and unpaid bytes are unchanged. Harness registration and all 89 Python
harness tests pass; upstream verification passes separately.

Two simple controls belong to earlier assignments. Their exact authored bytes
are retained under `pa13/tests/spec/named-pattern-rules/named-scalar-value.rs`
and `pa16/tests/spec/named-pattern-rules/literal-reference.rs`, with explicit
accepted LowIR/native entries in their owning manifests. No references were
generated. Consequently layout currently fails at the declared PA13 manifest
receipt change, and PA13/PA16 oracle generation/review plus receipt refresh remain
an explicit integrator prerequisite. This checkpoint does not claim layout or
complete harness success after those additions.

The cheap regression comparison uses an immutable git archive of the worker's
existing fixtures at `005bec4f`, avoiding fabricated references for the two new
entries. `reference-pattern-resolved-qualified/fixture-provenance.json` records
that archive and `identity.json` pins candidate SHA-256
`f1d4744b56ade4012a1da28a14e055b86b61a994114ff1309813852759f55fad` plus the
928-file production map. All 8,255 existing worker-base text cases preserve the
preceding output hashes: 8,248 exact and only the same seven inherited PA20
review pairs. All six deliberate PA13 discrepancies are byte-identical. Both
ordinary512 and constcall64 remain exact. The text command exits 1 for the
existing central review set. New oracles, the newer public case count, combined
full release and performance qualification remain integrator-owned.

The open-Type caller family below now implements parameter-free argument types
with owned caller lifetimes. Paid declared premises, unused generic parameters
and conditional nested-output contexts remain required. PA25 is still planned.


## Open type arguments in source initializers

The parameter-free type-argument family admits free caller lifetimes while
retaining the exact selected Instance, original initializer, result seal and
caller namespace. Nested callable-local binders stay local; escaping binders,
parameters and projections remain excluded from final argument admission.
Ordinary Associated requests retain their concrete gate. The actual raw operand
and independently normalized requested result remain distinct.

The existing immutable observed-argument catalog is reused for selected argument
DAGs. Complete rows share observation work across uses and sibling contexts;
every use still checks its actual Dispatcher reservation and every request checks
its actual body owner, cold and warm. Observation-only catalog creation records
storage and error work but creates no Caller, Publication or PreparedScope.
The private pre-publication test cannot construct a BodyCaller from that cache.
Closed and direct-lifetime paths retain their existing lazy behavior.

Selection restoration adds identity entries only for observed free inputs owned
by the actual caller, using the existing bounded restoration machinery. Existing
equality replacements win via entry.or_insert; foreign gaps, unobserved and
result-only identities stay unmapped. The original input is canonicalized through
its exact retained Contract equality folder before dispatch opens its own
universals. No equality, outlives fact, copied proof or execution permit is inferred
from observation or a coincident numeric identity.

The thirteen declared hypotheses live in
`pa25/tests/open-type-initializer-outcomes.json`. Nine positive programs pass all
three views, and unpaid/mismatched-result controls reach their real rejections.
Two unchanged Rust-positive cases remain required: `paid` needs its actual caller
premise proved before specialized initializer assumptions can be used;
`generic-pattern` disproved the original negative hypothesis because its unused
owner parameter does not affect the constant value. The failed hypothesis and
all source hashes remain in the fourth probe receipt. Neither frontier is a
language rejection or an assignment-completion exclusion.

The private selected-DAG matrix varies width and repeated/sibling use counts
independently at 8/32/128. Observation work is 37/109/397 at those widths; each
subsequent lookup costs one step and the single-region row retains 484 logical
bytes. A foreign sibling still rejects against the warm row. The current
`private-second` receipt passes all four observer controls, the restoration-input
control and four existing alias controls. The two pending PA13/PA16 oracle
receipts from the preceding pattern handoff remain unchanged.


The closing regression check caught two ordinary-route interactions. Applying
source input closing before direct-lifetime selection broke the existing
`qualified_lifetime.rs` static case. The fold is now limited to open Type
arguments. The later whole-text check caught
`partial-unique-reference-header.rs`: its declaration-inferred reference has an
exact ordinary AST-completed value but no body-source anchor. The closing folder
now exposes a charged read-only anchor-availability check. Original input closing
runs only when every observed input already has its actual mapping; otherwise
the ordinary current selection runs first. Only its exact completed value can
reuse that route. An uncached source request still must pass the original mapping,
ownership, result and proof gates. Unknown identities are not retained and no
error from a failed graph fold is swallowed. Both unchanged regression sources
are included in the focused private matrix; failed attempts and pins remain in
`closing/` and `qualified/` for diagnosis.

`routing-close/` passes strict all-target/all-feature release Clippy, the
architecture/file audit (1,616 files, zero findings), 16 computation tests, one
restoration-input test, four alias tests and four closing-folder tests. The
source matrix qualifies nine positives and two negatives with Rust 1.98 full
object emission and all three checked views; all nine positive native programs
execute. `legacy-text-tail/` passes the exact grouped independent-selection,
lookup-readiness and recursive-lookup work controls. The 89 Python harness tests
and registration of 336 roots in 13 binaries pass; the subsequent layout check
still stops at the already-declared PA13 manifest receipt. Upstream checks pass
separately. Common layouts remain SourceComputation 8, Target 28, Request 56,
Code 336 and Body 464 bytes. Catalog is 200 bytes: its preceding source-code index
from `005bec4f` already added eight bytes; this checkpoint adds no retained field.

`qualified-final/identity.json` pins candidate SHA-256
`104a15606ee1a75bd10fe6574a6ccd7ad917f3744c78e86f0527d1f0954c0d86`
and the 928-file production map SHA-256
`3fb0f1da4686abbe2f77b50ee0aba86c84904309c4dd110a2ec390d412c8fe55`.
The fixture provenance records the immutable `005bec4f` PA1–23 archive and the
previously exported 467-case PA24 tree. All 8,255 existing worker-base comparisons
preserve the preceding candidate: 8,248 exact plus the same seven inherited PA20
review hashes, with no new status or byte differences. Ordinary512 and constcall64
are exact. The raw PA20 stage still exits one for that retained review set; the
closing comparison explicitly verifies its complete identity. Current-public
case counts, the earlier oracle prerequisites, combined release and performance
qualification remain integrator-owned.
