# Canonical trait implementation members

This component replaces spelling-only impl matching with the resolver's actual
trait member table. The impl identifier is normalized and adjusted against the
actual trait declaration's expansion, then checked in its exact namespace and
context. The selected binding must be a direct member of that trait with the
same declaration kind. A separate impl identifier cannot overwrite or duplicate
an existing canonical trait-member mapping.

Methods retain all actual declaration IDs and use indexed source/library function
access. Their signature, receiver, default-body and lifetime conformance checks
remain mandatory. The index is compatible with the separate imported-header
work, whose full authenticated transaction is not enabled by these changes.

Associated types establish canonical links before their prepared impl header
becomes selectable. Both early projection normalization and final result proofs
consume those links. An actual member also retains its own owner identity for
generic-family substitution. This replaces the old name map and does not cache
any type or lifetime proof. Constants retain one identity map per actual impl,
shared by early readiness and final conformance. Ordinary contract normalization,
value evaluation and lifetime discharge remain separate requirements.

The [eight independent contracts](../pa31/tests/spec/declarative-macros/member-conformance/README.md)
pass sixteen metadata/object observations and four native programs. They cover
generated literals, captured identifiers, inaccessible opaque members and
duplicate method/type/constant mappings. The private consumer controls use actual
resolved legacy invocations with a supplied opaque context; general scoped source
invocation remains disabled.

The implementation passes formatting, the 2,330-file architecture audit and
all-target/all-feature strict Clippy. Runtime source `9b6665a6` passes all 1,464
nonignored private tests, 405 affected integration cases and 109 native test
functions. Measurements confirm a 240-byte associated inventory and a 7,032-byte
declaration record, down from 7,064 bytes after removing the redundant name map.

The initial native batch exceeded its 8 GiB worktree cap while retaining completed
test outputs. The diagnosed continuation preserves eighteen completed stages,
reruns the interrupted native target and cleans outputs between native groups.
PA20 then exposed two unchanged-reference diagnostic-location failures: the new
canonical constant matcher discarded the offending member ID. Recovery
`51acf093` retains that identity through early and final conformance. It passes
fresh formatting/audit/strict checks and all 8,382 active text cases with a fresh
CLI; no reference changed. The earlier private/integration/native results remain
bound to `9b6665a6`, and the combined release will validate the final compiler.

Evidence remains in the integrator implementation worktree under
`obj/parallel/member-conformance-runtime-v1/`, `runtime-v2/` (with the same
`member-conformance-` prefix), `member-conformance-static-v6/` and
`member-conformance-text-v3/`. Earlier audit/style and test failures remain
recorded; no lint allowances or audit exceptions were added. Combined release
and performance qualification, a core artifact and assignment activation remain
open.
