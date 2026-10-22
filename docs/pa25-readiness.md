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
  use still fails earlier semantic normalization and remains Rust-positive.
- Projected named-pattern operands. The original selected reference and paid
  relation controls still fail shape inference before value execution. Selected
  tuple/record controls now pass their actual projected operand and a separately
  normalized requested result through the same checked seal. The remaining
  reference-shape controls are required work, not language rejections.
- Selected type arguments containing open caller lifetimes. Their exact owning
  caller proof must supply authority without relaxing the ordinary concrete path.
- Source-associated uses inside conditional nested-output contracts. The original
  and instantiated conditional requirements must be discharged by the actual
  caller; no inherited summary is execution permission.
- The known generic cleanup quadratic remains a required changed-path performance
  followup. The observation cache only bounds shared argument observation and
  makes no whole-specialization or cleanup scaling claim.
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
