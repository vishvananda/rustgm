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
