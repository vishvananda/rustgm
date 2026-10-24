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

The current implementation passes formatting, the 2,330-file architecture audit
and all-target/all-feature strict Clippy in
`obj/parallel/member-conformance-static-v5/`. Earlier filename-stem and function
shape findings remain recorded; no lint allowances or audit exceptions were
added. Candidate runtime/native/text and measured-layout qualification are next.
The expected declaration-record size drops by the removed name map; runtime
measurement must confirm the updated layout assertions. Combined release and
performance qualification, a core artifact and assignment activation remain open.
