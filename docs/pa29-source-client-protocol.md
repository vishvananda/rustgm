# Source-role header and client protocol review

This is the next protocol proposal after the focused borrowed-header checkpoint
`6049ded5`, based on the e3 compiler now published in `234d94b3`. No wire version,
source-client mode, imported role or new execution authority is installed by
this document. The [original source inventory](pa29-source-header-interface.md)
and [validator evidence](pa29-neutral-role-headers.md) remain authoritative.

The first executable target is the actual `identity<T>(x:T)->T { x }` with its
absent Copy bound. Its provider must retain the complete actual sizing trio,
Copy/const Clone/clone_fn/clone_from, LegacyReceiver and reference impls,
Destruct/Freeze, and genuine user u8 impls from the declared source matrix.
Required methods remain bodyless; provided methods retain their real checked
originals, including unused ones. Runtime, const and non-Copy Token clients and
the unsized opposites remain the end-to-end qualification target. Header-only
artifacts or a synthetic replacement Clone body cannot satisfy it.

## Version and index namespaces

Propose a new row descriptor under existing envelope 9: `[2, 0, 1]`, meaning row
version two, defining Rust 2024, actual source-role policy. This is a proposal,
not a tag reservation. Keep envelope 9 versions zero and one, and all old base
profiles 0–8, byte-identical. They must reject these new rows. Never reinterpret
their legacy Copy/Sized contracts as actual source roles.

The defining edition is explicit per artifact, checked against its language
header and actual defining physical unit. Reserve descriptor edition value 1
for Rust 2021, but reject it until that complete source/import path is qualified.
Neither imported macro provenance nor the consumer edition substitutes for the
defining edition. Root owns edition/build integration.

Use existing LE-u32 counts, ordinals and half-open ranges; booleans and finite
tags are explicit u8, and semantic attribute flags are LE-u32. Optional indices
encode a 0/1 presence byte followed by the u32 when present. Reject unknown tags,
bits, reordered partitions and overflow before allocation/publication.

`DeclarationRef`, `BodyRef`, `BinderRef`, `TypeRef` and `ApplicationRef` retain
distinct table namespaces. Add distinct `ScopeRef`, `TraitRef`, `ImplRef` and
`TraitApplicationRef`, `ClauseRef` and `MemberLinkRef` wrappers. A cross-artifact declaration address contains
the exact existing encoded `CrateIdentity` and its artifact-local
`DeclarationRef`; it is never a local DefId, path guess, epoch or body ID.
Every address must authenticate against that exact completed dependency artifact.

Proposed version-two table order is fixed: source policy; declarations and
site partitions; binders; input scopes; type rows and type-root lists; parameter
rows; clauses; written-supertrait clause-index list; member declaration-index
list; trait headers; impl headers; callable headers/formal-binding lists; role
bindings; complete applications/argument lists; originals and their value lists.
Every list has its own count and typed range domain. Member lists contain real
declaration indices, so complete members need not be adjacent to their type
parameters/locals in the declaration table. Role bindings are ordered by their
explicit wire role tag; owner/member ordinals retain actual declaration order.

The source policy's provider is always explicit. If it equals this artifact's
identity, its role range covers all actual provider claims. A source-client
artifact names its actual dependency provider and carries no replacement role
claims; it must use that exact provider artifact's validated role table.

## Concrete header rows

Version two uses its own declaration/callable row decoding. Existing version-zero
and version-one row layouts are not extended in place.

| Row | Fields |
| --- | --- |
| Source policy | exact provider CrateIdentity; range of role bindings |
| Role binding | explicit LangItem wire tag; exact declaration address |
| Trait declaration | actual ExternalDef; visibility; lexical parent; ordinal |
| Trait Self declaration | owning trait DeclarationRef; ordinal zero |
| Impl Self declaration | real lexical parent; owner ordinal; no invented name/body |
| Member declaration | actual owner DeclarationRef; member ordinal; kind; name/visibility; optional exact implemented trait-member address |
| Type parameter | real owning declaration; full ordinal/name; source sizedness requirement identity; optional default argument |
| Formal binding | owning callable declaration; ordinal/name; distinct from a body local |
| Input scope | owning declaration; optional parent ScopeRef; own parameter range; own clause range; declared-prefix count |
| Trait header | trait declaration; exact Self declaration/type; input ScopeRef; captures; unsafe/const/auto flags; complete member range; written-supertrait range; semantic attribute bits |
| Canonical impl header | actual Impl Self declaration; target TypeRef; optional complete trait application; input ScopeRef; unsafe/const/negative flags; complete member range |
| Callable header | exact function/member declaration; optional actual BodyRef; original/public signature roots and binder frames; input ScopeRef; complete formal-binding range; declared Ordinary/Const/Conditional effect |
| Clause | owning declaration; declaration-site ordinal; subject TypeRef; optional quantifier BinderRef; complete predicate/effect; written subject identity |

The actual source Impl Self definition is already its unnamed owner; do not
reserve a second fictitious impl declaration. Imported canonical impls use
target types and trait applications, independently of source OwnerKind's syntax
keys. Trait Self is a separate real declaration from the trait. Required
bodyless method bindings belong to the callable, not a fabricated BodyRef.
Originals with bodies explicitly map formal bindings to their actual locals;
`definition=None` remains absence.

Member ranges cover the declaration inventory exactly once, including private,
required and provided members. Validate owner/kind/ordinal and complete coverage,
not counts or same-shaped signatures. Lexical ownership is an indexed DAG;
recursive callable dependencies remain legal and separate.

Input scopes retain direct source-owned clauses and actual parents rather than
flattening each inherited environment into every use. Preserve original,
declared-prefix and inherited boundaries, including implicit Self predicates.
Borrowed source vectors stay owned where they are today. Imported canonical
rows are cold, immutable and indexed once; they do not add an Arc around each
local source Header or Template.

The initial written-subject domain is an exact parameter DeclarationRef, whose
mapped parameter and scope must match the canonical subject. Classify inheritance
as Self only when this is the actual Self declaration. An alias reducing to
Self must not acquire that classification. Wider written type/alias subjects
are explicit unsupported rows until their full retained representation exists;
do not manufacture classification from TypeId equality.

Ordinary, Const and Conditional trait goals retain their exact trait application
and full type/const/lifetime arguments. A language sizing requirement names its
actual source declaration; it is not the old unqualified Sized bit. No source
Copy identity becomes a spelling, legacy builtin or supplied implementation.
Original assumptions remain assumptions only for original proof and do not pay
substituted instance goals.

Semantic attribute bits must name every admitted effect, initially the actual
fundamental, specialization-trait, deny-explicit-impl, dyn-incompatible,
coinductive and trivial-field-read facts of the selected source declarations.
Unknown or unimplemented active effects reject. Recording a bit is not an
implementation of its semantics. API stability/trivia is separate from const
effect and permission; preserve its source/configuration provenance honestly.

## Type and body requirements before codec freeze

The header graph needs references with exact mutability/referent and static or
bound lifetimes. A bound lifetime is a BinderRef plus slot, with exact root and
ancestor validation; serialize no free-region number. Retain complete original,
public, trait and quantified binders, including unused slots. Trait applications
carry exact addressed declaration and a complete argument range. No arbitrary
argument from another original's scope is valid merely because the type row is
present in the same artifact.

Version-two callables explicitly distinguish `body=None` from a proved original
candidate. Provided bodies retain real Owner/BodyOrigin/site pairing and complete
contracts. The actual checked clone_from requires retained reference places,
dereference/borrow/assignment, selected trait-method *requests*, and cleanup.
Import must derive their mandatory formation, coercion, trait, lifetime and
cleanup obligations from operations; an omitted serialized goal cannot erase
them. Conditional goals and every child output contract stay goals through
substitution and proof. No selected proof, Checked, Permit, paid const witness,
SourceComputation or caller handle is serialized.

These body extensions are not assigned guessed opcodes here. First admit the
genuine provider's still-unsupported source feature/attribute prerequisites,
then inspect its actual retained checked bodies and freeze the complete typed
operation/contract encoding before implementing the version-two codec. Do not
emit a header-only version two while dropping clone_from, its Destruct debt, or
an unsupported body. Existing promotion operand provenance remains structural
data with original/instance eligibility proof mandatory.

## Requested and authenticated client policy

Add a distinct requested `Mode::SourceClient { provider: CrateIdentity }`.
`Mode::Source` remains the current own-source-provider policy with
`current == provider`. SourceClient requires a distinct current crate and an
exact dependency artifact supplying the actual source policy. A bare mode value
or direct --extern alias is not authentication.

A private borrowed `PreparedSourceClient` is minted from the validated dependency
graph, exact provider artifact and policy/header inventory. It contains no paid
obligation or execution permission. Reserve the client's language mode before
bootstrap construction: source-client selection leaves bootstrap Copy/Sized,
prelude identities and ordinary library factories absent. There is no transition
that first installs them and later tries to remove or reinterpret them.

Failed/missing/malformed source import is an error, never Bootstrap fallback.
The provider's metadata inventory is distinct from visible aliases; use the
existing transitive inventory boundary, not artificial names. A real core root
or prelude binding must refer to actual exported metadata. The first no_core
clients need no implicit prelude; ordinary clients still require the actual
configured source prelude rather than synthetic declarations.

Active source policy with a missing Copy declaration remains missing. It does
not become ordinary mode or authorize a demanded read. Metadata header validity
does not prove any Copy implementation. First source clients reject v0/v1 generic
originals checked under the legacy-role policy; mixed language-contract semantics
are not inferred from an apparently simple body. Any later compatibility for
role-free ordinary/FFI contracts must be explicit and retain actual obligations.

## Installation order and owning consumers

1. Decode and validate whole artifact tables, source descriptor, counts,
   ownership/lexical DAG, ranges, root/binder scopes, role addresses and complete
   member/body coverage. Validate the dependency graph before local remapping.
2. Stage real resolver declarations, scopes and exact external-to-local remaps.
   Reserve all trait/Self/impl/member/formal identities before canonical type
   construction. No syntax node or fake function is allocated.
3. Install the current catalog's actual typed declaration/body/site rows with
   their original diagnostic provenance. Validate every mapped kind, owner and
   provider coordinate; same-kind foreign IDs are insufficient.
4. Construct all canonical headers, input scopes and exact role bindings once.
   The neutral trait/function adapters borrow these cold rows and check the
   current catalog epoch, bounds and exact remap. The shared validators then
   validate actual role headers; a successful header grants no body permission.
5. Publish the declaration registry only to the staged compilation. Reconstruct
   ordinary environments and operation-derived goals; prepare promotion and
   eligibility, then prove all executable originals and required original proof
   components using their actual contracts. Invalid unused originals reject.
6. Publish the completed immutable ImportedOriginals allocation and completed
   catalog only after full success. Demand actual instances through the existing
   imported-first function provider, original/instance reproof, checked CTFE and
   ordinary lowering. Preserve exact public/owned signatures, execution=None on
   originals, environment/contract lineage and cold/warm caller identity.

On failure no new alias, origin map, role row or executable row escapes. Staged
resolver reservation or unreachable canonical interning is not executable
publication and cannot trigger a fallback. Preserve earlier successfully
installed artifacts and their identities.

Concrete owning seams: `support::{bindings,crates::generics,crates::codec}` for
requested policy and validated wire data; `resolve::{bindings::policy,crates,
body_origin}` for no-bootstrap staging and authenticated real declarations;
`sema::{associated,function_headers,declaration_inputs,compiler_items,traits,
crates::typed_headers}` for borrowed validated canonical input; and existing
pipeline ImportedOriginals/original-group/provider paths for actual proof and
execution. B owns const-call continuation/effect payment and C owns physical
codec/storage. Coordinate their exact consumer seams when adding these fields.

## Budgets and qualification

All new tables, strings, owner/member indexes, type/application summaries,
canonical header rows, environments and scratch share existing total metadata
bytes/components limits. Charge failure work and reserve aggregate growth before
allocation. Index once; share graphs and completed structure, never caller proof.
Do not scan a whole artifact per header/use or expand shared diamond types into
flattened closures. Keep no-import allocation cold and measure containing
layouts before central enum changes.

Required controls: all old byte goldens; descriptor/edition/role-policy mismatch;
missing/duplicate/foreign role addresses; wrong Self/member kind/parent/slot;
same-shaped foreign headers; declaration-versus-body site misuse; omitted unused
parameters/goals/provided bodies; qualifier and binder mismatch; malformed/failed
transaction rollback; no Bootstrap coexistence/fallback; cold/warm paid/unpaid
actual callers; the original six native clients and unsized negatives; independent
header width, unrelated declarations, repeated use and shared graph axes. Existing
full PA25/29 requirements, eighteen metadata scenarios and whole core→alloc→std
acceptance remain required beyond this first source family.
