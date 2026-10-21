# Remaining PA25 work and dependency protocols

This is an implementation proposal, not an accepted assignment contract. PA25
and PA29 remain planned until their full required behavior and release gates
are completed. Explicit compiler boundaries below are outstanding work.

## PA25 versus closure prerequisites

The source-owned nested-output checkpoint establishes full child proof and
parent discharge for closed output/formation interfaces. Its capture receiver
consumers belong to PA24; the anonymous body/provider integration belongs to
PA25. Hidden receiver escapes and private storage upper bounds remain ordinary
proof failures. Generic specialization of participating output links is still
outstanding, rather than an accepted language restriction.

The independent PA25 correction here is local annotation ownership. The
declaration annotation traversal now recognizes typed Const blocks alongside
function, initializer and closure boundaries, setting Local(block node),
while preserving the inherited generic binder. This gives elided written
lifetimes the same owner as the inline body's semantic checker. The original
`inline-local-annotation.rs` fixture must pass unchanged. Nested blocks, mutable
references, reborrows, method calls, promotions and static addresses must remain
distinct from dangling local results, forced static-local borrows, captured
outer locals and non-const operations. This does not alter `in_function`, const
eligibility or the final storage/borrow proof.

Other PA25 implementation obligations remain: generic anonymous specialization;
dynamic named-pattern and enum-value readiness continuations; associated/generic
initializer execution coverage; slice/text projections; complete symbolic static
and promotion allocation coverage; and the remaining pointer/union validity
families coordinated with PA27. The implemented constants view still needs its
complete released manifest and independently reviewed references. No earlier
explicit boundary is treated as completed assignment scope.

## Generic anonymous identity proposal

Use a phase-neutral `AnonymousInstance { source, enclosing }`. `source` is the
exact local anonymous template owner. `enclosing` retains the complete canonical
callable Instance, including closure instances, or the selected initializer
instance with all canonical generic arguments. It is not just a DefId, a result
type or an erased signature. Every nested anonymous child inherits the same
complete enclosing context, while retaining its own source identity.

Original templates keep BodyKey::Anonymous(source). Specialized anonymous bodies
need a distinct key containing this complete instance; they must not pretend to
be FunctionItem instances or ClosureAdapter bodies. The integrator owns the
central key decision alongside receiver adapter identities.

Template preparation, materialized body/contract cache, interpreter catalog,
suspended request and completed value cache each include the complete anonymous
instance and checked result type. An original registry row's deferred-template
state never becomes a value shared across instances. Inspection orders template
declarations by source and completed instance roots by canonical full argument
identity. No local cache key becomes an external metadata identity.

Prepare each immutable anonymous CFG/environment once from completed typed
facts, using the enclosing template's checked generic environment. Specialize
the retained CFG, contract, cleanup and promotions through the existing typed
substitution pipeline; do not parse or infer the body again. Rvalue inline
demands retain or derive the selected anonymous instance during substitution,
so recursive nesting and const-call suspension cannot lose context. The provider
remains iterative and resumable, with work and retained-storage charges before
copying or publishing materialized bodies.

Const permission, template proof and concrete proof remain separate. Every
materialized body must satisfy const eligibility, full region/ownership/loan and
cleanup checks before its Code enters the execution catalog. Conditional output
summaries cannot authorize specialized execution. Reference results require the
same complete child/parent proof under the specialized interface, not reuse of
a source-only positive result. A first scalar implementation is a coherent
development slice, but is not completion of generic reference behavior.

Required controls include equal arguments reusing one body/value, distinct
arguments with equal signatures retaining separate values, type/const arguments
used only in the anonymous body, recursive nested anonymous contexts, const
functions and generic closures, suppressed zero repeats, unused invalid bodies,
and failure isolation across instances. Scaling must count template preparation,
materializations, cache hits/misses, CFG execution and retained payload.

## PA29 extension from retained typed facts

The current wire profiles retain scalar function signatures and closed scalar
values with explicit external identity/remapping. Full nominal/reference/generic
exports require a bounded typed type/definition graph, not source reparsing or
serialization of local arena IDs. Preserve existing schema/profile bytes and
introduce a new explicitly versioned profile.

A wire type table should represent primitive types, bound lifetime slots,
references/pointers, arrays/tuples, callable conventions and nominal applications.
Nominal references use ExternalDef plus typed argument slots. Nominal declaration
records retain kind, fields/variants, visibility, representation/layout contract,
generic parameter kinds and required predicates. Import first validates all
counts/edges/identities and reserves canonical definitions; a second pass interns
types and publishes completed shapes. Recursive nominal references must not
require cyclic raw type-table evaluation. A type name alone is never sufficient
to synthesize a layout or proof contract.

Generic exports additionally need retained typed CFG templates, environments,
predicates, const permissions and symbolic constant/allocation graphs. Every
DefId, TypeId, field/variant, callable, intrinsic and allocation edge is encoded
through an explicit wire identity map and validated/remapped on import. Anonymous
bodies and closures need stable owner-relative body identities, independent of
NodeKey and local allocation order. Optional source provenance may support
diagnostics, but cannot be required to fabricate local source nodes.

Imported templates must enter the same ordinary specialization and full body
proof pipeline before executable permission. A serialized const bit never
authorizes execution without an available, revalidated checked body. Foreign
ABI/safety, exact naming and linkage reuse the canonical PA30 registry; Rust
symbols use the crate identity context. Reference lifetime contracts retain all
formation/outlives predicates, and allocation records retain symbolic origin,
initialized bytes and relocation edges instead of host addresses.

The next source-to-native dependency slices should be a public scalar-field
nominal passed by value, a function borrowing that imported nominal through an
explicit lifetime contract, then a generic identity body instantiated in a
downstream crate. Each needs independent Rust acceptance, deterministic artifacts,
reordered dependency inputs, equal private names, malformed-wire controls and
ordinary proof/ABI validation. Supporting the first slice does not silently
drop unsupported exposed declarations in the others.

## Local annotation checkpoint qualification

The original local annotation source is unchanged and now accepts through the
checked constants pipeline and native comparison. Four focused tests cover 13
independently qualified source outcomes, nine native values and a declaration-only
enclosing named-binder control. Dangling and forced-static local borrows still
report an unentailed lifetime bound; a non-const call still reports const
eligibility failure. Outer capture remains rejected. The enclosing generic
binder is preserved without claiming generic anonymous execution support.

Strict release all-target/all-feature Clippy, the 1,344-file architecture audit,
23 initializer/closure/nested regression tests (five existing backend tests
ignored there), and upstream24/126 verification passed. The nine new natives
were explicitly enabled in the focused stage. The first lint attempt caught
only duplicate test-helper loading, corrected by sharing one qualification
module in the parent test. Failed and final evidence remains under
`obj/parallel/pa25-inline-written`, including exact negative diagnostics and
`source.json`. This checkpoint still inherits the pending PA16 reference/receipt
publication described in `pa25-nested-output.md`; layout is not claimed passed.

Source-set SHA256: `9eba256c72c1c68d54175c697518443660efd537d400eb800ef117a905ab663e`.
Qualified debug frontend: `a8cc0e3d9ad4082e3650f6c9ca3f9ba951f83f9f13cbbcab405a22d33cfae5df`.
