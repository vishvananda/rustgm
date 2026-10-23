# Source-language header transport: interface review

This is a source-only proposal on `a6aa69628aca2b256dd785cf72bf856e62d048fe`,
not an implementation, new wire version or upstream-core acceptance claim.
The completed helper branch and `22f9d758` receipts remain preserved. No
compiler source has changed in this successor branch.
The later [borrowed validator checkpoint](pa29-neutral-role-headers.md) records
its actual implementation and focused retained/fresh evidence separately.

## Independently established closure

Pinned source: Rust 1.98.0, commit `88d9e12ae178fab0fb5cc050a94da85685d449ea`,
under the shared immutable source tree. Exact source paths, line ranges, original
file/slice SHA-256s and transformations are in
`pa31/tests/spec/source-header-transport/contract.json`.

The original `convert::identity<T>` body is exactly `x`, with no Copy bound.
Three sizing headers alone are insufficient: rustc explicitly requires the
Copy language item, and RustGM's prepared Copy policy also errors on a demanded
read without that role (`body/copy_requirements.rs::Prepared::decision`).
Adding actual Copy requires actual const Clone and clone_fn. The original
Clone includes **both** its required clone method and its provided clone_from
body, whose `Self: [const] Destruct` condition and `*self = source.clone()` must
not be discarded. Rustc additionally requires Freeze for this const family.

Two corrected providers now pass full independent Rust metadata/code emission:

- `semantic-provider-qualified.rs` retains complete original declarations,
  signatures, semantic attributes and provided bodies; only API stability
  attributes are explicitly removed.
- `original-attributes-provider-qualified.rs` retains the original contiguous
  declaration/body slices with their original attribute attachment. Its explicit
  no_core/staged_api wrapper and small user primitive impls are recorded.

Both include Sized, MetaSized, PointeeSized, LegacyReceiver and its actual shared/
unique-reference impl declarations, Destruct, unsafe auto Freeze, complete const
Clone/clone_fn, Copy, and genuine user Clone/Copy impls for u8. Full original core
has more roles and impls; this reduced provider does not claim to replace it.
Each provider supports unchanged runtime, const and non-Copy Token consumers;
all six C-linked native observers return 7. Both unsized consumers reject with
E0277. Thus the transport must preserve ordinary moves and must **not** add
`T: Copy` to identity as a shortcut.

`obj/parallel/pa29-source-headers/independent-third/rust-receipt.json` records
these ten matching outcomes. Initial and second hypotheses remain unchanged,
with their mismatches recorded separately: missing fundamental feature requests,
missing Copy/Destruct/Freeze, missing const_destruct, and a corrected test assembly
that had attached Clone's stability attribute to Freeze. The signature-only
Clone control is a reduced source success, expressly not original Clone coverage.
The accepted immutable 22f CLI rejects the corrected providers at source feature
admission. Those observations are pin comparisons, not fresh a6 qualification.

## Neutral borrowed validation input

Add a semantic-layer borrowed view, following `function_headers::FunctionHeader`
and `declaration_inputs`, rather than constructing source Function/Trait nodes.
Proposed names below are private until the actual migration is reviewed:

```
RoleDeclaration {
    definition: DefId,
    parent: Option<DefId>,       // declaration owner; never a fabricated body
    origin: DeclarationOrigin,
}
TraitHeader<'a> {
    declaration: RoleDeclaration,
    self_definition: DefId,      // actual separate Self binding
    self_type: TypeId,
    flags: TraitFlags,           // unsafe, auto, const
    parameters: &'a [Parameter], // own slots; complete inherited view is separate
    captures: u32,
    members: &'a [DefId],
    inputs: DeclarationInputs<'a>,
    inheritance: SelfBounds<'a>,
    written: WrittenHeader,
    attributes: SemanticAttributes,
}
```

All fields are private. `source(report, facts, actual_reserved_declaration)`
checks the actual resolver declaration, Source origin, retained associated
owner/schema and separate Self owner/index. It reads source item/attribute
facts once. `imported(current_catalog, authenticated_remap, typed_header)` checks
exact artifact/epoch/row/kind, real DefIds, parent and Self mappings before
constructing a view. Missing state differs from a completed empty list.
Neither constructor nor a successful header check pays any body/caller debt.

`WrittenHeader` preserves the actual written supertrait/where inventory; matching
counts alone never validate a header. `DeclarationInputs` borrows existing source
Located predicates and imported ProofSite inputs without allocating a converted
source predicate table. Own/declared-prefix/inherited boundaries and qualified
ordinary/const/maybe-const clauses remain exact.

A concrete migration issue is `sema::traits::Header::self_bounds`, currently a
HashSet<NodeKey>. `implies_self` intentionally distinguishes written Self from
an alias reducing to the same type. Imported clauses need a retained structural
subject classification and declaration sites, not a canonical-type guess.
A borrowed source/imported SelfBounds/inputs adapter must serve the common trait
contract algorithms; source vectors remain directly owned. Do not widen every
canonical Predicate or duplicate local headers behind another Arc.

Extract pure shared validators from `compiler_items::{sizing,roles}` to consume
these views plus the canonical Store and exact reserved role map. Source wrappers
retain current entry-point bytes and checks. Imported wrappers use checked
DeclarationOrigin diagnostics, with no NodeKey/FileId fallback. Preserve actual
supertrait identities, Self identity, full member sets, captures, qualifiers,
Rust signatures, reference binders and provided-vs-required body state.

SemanticAttributes must preserve relevant fundamental, specialization,
coinductive, deny-explicit-impl, dyn incompatibility and trivial-field-read facts;
unsupported active semantics reject. Actual fundamental/auto_traits/
const_destruct/staged_api feature boundaries remain explicit. Recognizing a
feature request or an attribute does not import a solver permission.

## Actual trait/impl/member ownership

Imported traits and impls require genuine canonical semantic owners. The current
associated Owner/source and source Impl target syntax are source-specific; no
import may populate them with a fake NodeKey. Proposed minimum coherent change:

- Owner origin becomes DeclarationOrigin, with explicit source-only extraction
  in source elaboration. Trait flags and exact separate Self identity stay real.
- Keep source Impl syntax distinct from an imported canonical Impl header.
  An additive ImportedImpl form can carry the actual self type/application and
  flags without pretending an artifact type is an AST target. Measure the enum
  before adoption; use its existing maximum payload where possible.
- Reuse ordinary canonical trait contracts/implementation conformance. Preserve
  actual impl constant/negative/unsafe flags, supertrait obligations and exact
  trait-member to impl-member/default relationships.
- The primitive impls and receiver impls are real declarations, not compiler
  factories or imported evidence. A source impl's genuine self_definition is
  its semantic owner; there is no invented resolver DefKind::Impl.

Clone's required clone member has an honest absent body. Extend the imported
function-header sidecar to represent that declaration without creating an
Original. The provided clone_from, identity and user clone implementation have
real body records. The existing FunctionHeader body Option fits this distinction;
its original/public signature namespaces and exact binder transport still apply.

The smallest faithful executable closure therefore needs references, actual
trait invocation and dereference/store operations in addition to v1's direct
calls. It must carry clone_from's checked typed body and reconstruct operation
requirements under its real original trait/member environment. Imported originals
reprove normally; demanded instances pay actual caller obligations. Do not mark
the provided method bodyless, drop its qualified Destruct goal, serialize Checked,
or replace it with a guessed ordinary library implementation. Coordinate B's
per-call const payment/CheckedInput seam; it retains an actual prepared Contract,
not an Instance-only success token.

## Authenticated source-sysroot client policy

Current `Mode::Source` and both policy preparation and Catalog::reserve require
current crate == provider. That is a provider declaration boundary, not a client
installation protocol. Keep it strict for source providers. Add a distinct
explicit source-client/sysroot selection, for example:

```
Mode::SourceClient { provider: CrateIdentity }
AuthenticatedProvider<'catalog> // checked catalog + exact dependency artifact
```

The borrowed authentication object is minted only by the importer after the
selected dependency's source semantic profile, defining edition and complete
catalog/remap validate. It is not serialized. An ordinary extern alias alone
cannot select a sysroot or grant roles. Client current crate remains distinct
from the provider; defining edition never comes from the client.

Selection happens **before** resolver bootstrap factories/prelude installation.
In source-client mode install no synthetic Copy/Sized/core/Option/iterator/macro
identities alongside actual ones. Primitive syntax remains language syntax.
Use only authenticated actual imported declarations and real prelude exports;
missing/unsupported source imports fail, with no retry in Bootstrap mode. The
first no_core consumers make the explicit dependency boundary observable; full
ordinary core/std prelude/macros remain required beyond those controls.

Provider header validation and client role installation are separate steps:

1. Decode bounded structural rows, select exact source provider and authenticate
   the whole real declaration/owner remap in a transaction.
2. Reserve all actual Self/parameter/member identities; build the shared canonical
   graph and complete headers without fabricating syntax or body entries.
3. Run common header/role validation locally. Registry Reserved/Validated state
   is derived here and never transported; absence stays distinct from failure.
4. Ingest and validate ordinary trait/impl contracts; reprove complete originals,
   including the provided default and primitive method body, with all debts.
5. Publish the complete imported payload/role installation transaction. During
   checking, staged private facts can identify roles but confer no Checked/Code
   authority. Actual caller/instance proof, scoped Copy decisions, eligibility,
   promotion and loan gates remain mandatory.

Reject wrong/same-kind remaps, foreign catalog/epoch, duplicate core selections,
missing body/default/member, false Self/supertrait provenance, absent-vs-reserved
roles and missing qualified debts. Test cold/warm caller/instance separation,
consumer rename/transitive imports, no-bootstrap coexistence and failure without
partial publication.

## Representation and next executable checkpoint

All new imported header/body records belong to one cold immutable sidecar, with
indexed DefId/owner/member lookup and shared type/binder/argument graphs. Source
Vec ownership stays direct. Observe each producer/header once; repeated consumers
reuse structure, never caller proof. Charge successful/error work and aggregate
scratch/retention before growth; vary traits/impls, body uses and argument width
independently.

Before production migration, measure RoleDeclaration, associated Owner/OwnerKind,
source Catalog, Contracts and containing models with the proposed origins.
Preserve canonical Predicate12, Value24 and no-import allocation behavior. A
DeclarationOrigin replacement is expected to affect cold location records, but
no unmeasured exact size is claimed here. Avoid changing dense source predicates
or adding a catalog pointer to every Body/Value.

The current v0/v1 descriptors explicitly mean legacy roles and cannot be reused.
An explicit later row version/policy needs complete reviewed typed row fields,
index namespaces, effects/attributes and transaction order before assignment.
Preserve all profiles0–8 and v0/v1 bytes. The first acceptance target is the two
faithful source providers through actual artifact import and the six runtime/
const/noncopy native consumers with unsized/unpaid opposites. The entire original
18-case metadata matrix and full core/alloc/std goal remain required.
