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
  The generic `T::V -> T::Out` inference prerequisite now has an isolated repair;
  ordinary demanded reference/tuple calls still expose a normalized source-result
  seal disagreement, and specialized generic initializer bodies still expose
  missing projection normalization. These are independently Rust-positive cases.
- Projected named-pattern operands. The original selected reference and paid
  relation controls still fail shape inference before value execution; selected
  tuple/record controls reach the initializer projection boundary. Result seals
  must preserve the original operand and canonical requested identities separately.
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
