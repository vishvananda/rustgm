# Shared named-record wire foundation

This is a private implementation checkpoint for the full closed-record transport
family in `pa29-closed-nominal-plan.md`. It is not a source import/export feature,
an assignment release, or a standalone publication candidate. Resolver remapping,
semantic member/layout publication, export planning, source/native controls and
scaling qualification remain required before integration. PA29 remains planned.

Schema-one profile three stores one artifact-owned immutable type graph, followed
by sorted named-record declarations, function roots and the existing scalar value
table. Artifact-local `RecordRef` and `TypeRef` ordinals have no frontend identity.
Record fields retain declaration order, names, visibility and static shared type
roots. Private reachable declarations retain identities without becoming public.
A named record and a function may share a spelling in their separate namespaces;
a record cannot replace a required module prefix.

Graph construction computes bounded lifetime-slot summaries once. Each function
validates its own input/result roots and complete binder, retaining unused slots,
without traversing every unrelated record or copying the graph. Canonical graph
order is first-use postorder across sorted function roots and then record fields.
A separate linear by-value dependency check rejects infinite storage cycles;
reference cycles are not by-value edges. Validation rejects unknown ordinals,
noncanonical or unreachable storage, duplicate names, foreign owners, malformed
roots, escaping lifetime slots and combined artifact/dependency quotas.

The original signature-local formats remain unchanged. Three independently
Rust-object-qualified providers produce byte-identical profile zero, one and two
metadata and LowIR under the published compiler and this candidate. A fourth
Rust source confirms the separate named-record/function namespaces. These exact
controls do not establish whole-frontend performance or source record support.

The focused release metadata suite passes 18 tests, including roundtrips,
truncation, legacy encoding controls, storage/reference cycles, private reachability
and per-function binder ownership. All-target/all-feature release Clippy with
warnings denied passes; the syntax/architecture audit reports 1,600 files and
zero findings. Evidence is retained in
`obj/parallel/pa29-nominal-wire-v2/` in the PA29 checkout. The initial 17-test
foundation is retained under `pa29-nominal-wire-v1`.

Separately declared future source controls cover 14 outcomes and eight native
Rust executions using two separately compiled, same-name libraries. They test
returns, construction, nesting, private fields and patterns, static shared fields,
distinct nominal identities and moves. Their pinned-Rust evidence is in the main
checkout's `obj/parallel/pa29-source-controls-v1/`; no RustGM agreement is claimed.
