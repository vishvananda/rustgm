# Generic crate transport for source-built core

Proposal only. No shared compiler interface or wire bytes change here. The
inspection base is validation `6a547f96` (`907dcd0c` + array/C-record metadata
`eac3885e`, IterMut `0600db40`, Option `6a547f96`). The worker remains at the
caller-premise checkpoint; no rebase is implied. Root's stored-array worktree
at `bc640d44` was inspected read-only while dirty: its Repeat/Bytes values and
Field/Element reference paths remain an independent data-transport family.
The public roadmap keeps PA1–30 responsibilities intact and prioritizes source
core, then alloc/std. This proposal does not call existing PA25 gaps complete.

## Existing boundaries that must change

`resolve::crates::Imported` already reserves stable external named definitions
and retains artifact coordinates. `sema::crates::import` currently installs only
closed FunctionItem(EMPTY) types and foreign Registry entries. Imported functions
are deliberately absent from local Function/body/template and const registries.
`lower::instances::demand::Ordinary::select` treats foreign Import as externally
implemented and never instantiates its body. Setting a generic/const bit on that
route cannot provide a generic original or const execution authority.

The shared TypeGraph has only scalar/closed-record/array/reference nodes and
Static/one-level Bound lifetimes. Signature validates one late binder, including
unused slots. Generic bodies additionally require owner-qualified type/const
parameters, early lifetimes, nested binders, trait applications/projections,
function-item identities and full argument lists. Extending the closed graph's
Bound slot meaning in place would corrupt existing profile contracts.

Actual reuse seams are `types::Store::intern_parameter`, callable Templates'
`insert_scoped`, Generics' original declared predicate prefix/environments,
traits' header/impl/member catalogs, `body::templates::Template`,
`body::regions::Contract`, `borrow::templates::verify`, and
`body::instantiate::from_checked_caller` followed by ordinary instance proof.
They are retained typed data, not instructions to repeat parsing or inference.

## First end-to-end family and full later scope

The smallest useful body checkpoint exports real generic and const function
originals with type/const/early+late lifetime parameters, private generic helper
calls, tuples/arrays/references, ordinary branches/returns and declared
Copy/Sized/user-trait/outlives conditions. Every parameter remains present even
when absent from the public signature/result. This already allows meaningful
core-like identity/reborrow/array/helper/const routines; it is not a scalar ABI
wrapper around a guessed body. A user-trait paid/unpaid pair exercises actual
imported selection and implementation conditions.

The declaration format must also describe traits, impls, associated type/const
members, defaults and projections from the outset. Their body consumers can be
qualified in successive coherent checkpoints using the same owner/type/clause
format: associated projection and method/default demand, generic nominal
construction, then imported closure/anonymous output contracts. Unsupported
operations must stop export/demand explicitly; do not omit a dependency or
pretend a declared contract is implemented. The independently declared matrix
includes these followups so required work is visible before implementation.

Complete PA29 still requires generic nominals/enums/destructors/variance and
well-formedness, all generic/const body kinds and nested owners, full trait/impl
and associated/GAT conditions, opaque/dynamic/function interfaces, allocation
and relocation graphs, macro/language identities and cross-crate linking.
Source-built core will exercise several of these immediately. Neither the first
family nor the existing closed profiles can substitute for that whole contract.

## Proposed neutral artifact tables

Names below are proposed; ordinals are checked artifact-local references, never
frontend arena IDs. Tables are immutable and shared by all declaration/body roots.

- `DeclRef(u32)`, `BodyRef(u32)`, `ParameterRef(u32)`, `BinderRef(u32)`,
  `TypeRef32(u32)`, `ConstRef(u32)`, `ClauseRef(u32)`, `ProofRef(u32)`.
- `Declaration { identity, owner: Option<DeclRef>, kind, access, parameters,
  header, body: Option<BodyRef> }`. The stable identity is defining CrateIdentity
  plus typed owner path/component. Named top-level items reuse ExternalDef.
  Unnamed impl owners use their defining declaration ordinal; members use their
  real trait/impl owner plus member name/kind/ordinal. A lexical ordinal is
  stable within that exact defining artifact, not a hash of rendered source or a
  promise of stability when crate contents change under the same identity.
  Exported aliases remain the separate PublicNames binding graph.
- `BodyOwner { declaration: DeclRef, parent: Option<BodyRef>, kind,
  ordinal }`. Private helpers have private declaration identities. Anonymous and
  closure bodies have distinct parent-owned ordinals even when source spans or
  types coincide. Their owner chain is acyclic and validated before allocation.
- `Parameter { owner: DeclRef, ordinal, kind, default }`, where kind distinguishes
  type with declared Sized policy, const with its exact declared type, and early
  lifetime. Inherited parameters retain the original owner and complete order;
  an impl method cannot relabel an impl parameter as its own. Late regions belong
  to an explicit Binder and do not become type/const ParameterIds.
- `Region { Static | Early(ParameterRef) | Bound{depth,slot} |
  BodyVariable{body:BodyRef,slot} }`. BodyVariable is scoped to its exact original
  proof family, not an allocation frontier. Scope tables retain private owned
  intervals/gaps, binder parent/depth, empty binders and universal-alias mode.
- `GenericType` mirrors supported canonical type constructors using table roots:
  Parameter, nominal application, projection with exact trait/member/arguments,
  tuple, array with ConstRef length, slice/reference/pointer, function signature
  with its own binder, and selected function item. Closure/opaque/dynamic forms
  need their real owner descriptors, never a primitive/function substitute.
  Scalar leaves reuse the established scalar vocabulary.
- `Argument { Type(root) | Const(root) | Lifetime(region) }` and immutable
  argument ranges retain every slot. An instance is its exact declaration,
  complete arguments and selected callable/result roots; lifetime erasure for
  symbol emission is a later operation and cannot key proof/value caches.
- `Clause { Predicate | ForAll{binder,predicate} }` retains the actual typed
  subject and Copy/Clone/Sized/auto/user trait, type-outlives and projection
  equality bounds. Trait headers retain Self parameter, supertraits, associated
  member bounds/defaults and the distinction between declared requirements and
  elaborated consequences. Preserve the existing typed written-Self implication
  classification; do not reconstruct it from canonical type equality.
- `ImplHeader` retains subject, exact trait application or inherent owner,
  original parameters, declared conditions, associated replacements/default
  choices and method mappings. Import recomputes validated availability and
  coherence; an artifact boolean cannot mark a header validated.
- `BodyTemplate { owner, signature/result, CFG, storage/temporary scopes,
  symbolic cleanup description, contract }`. CFG tables use body-local checked
  indices and typed references, scalar/text literal payloads and canonical
  operation tags. They do not contain NodeKey, raw enum memory layouts or local
  DefId/TypeId/ConstId bytes. Only existing operations with qualified consumers
  can decode successfully.
- `Contract` retains universals/parameters, declaration inputs, actual result,
  original goals, quantified LifetimeProof blocks and source labels. Assumptions
  must match the declaration/input-header producer; body equalities, local storage
  bounds, coercions, formation and selected implementation debts remain goals.
  No serialized Checked token, Permit, solved-positive cache, SourceComputation,
  CallerId/epoch/reservation or executable proof is permitted.

Use a new explicitly versioned generic-body envelope; preserve exact old profile
0–7 bytes and root's separately qualified stored-array profile. Do not assign its
numeric tag until coordinated. Generic graph storage is one artifact-owned table,
not a copy of the entire graph per function/Contract. Old closed payload objects
can remain absent from the new generic catalog or share referenced stable named
identities; duplicate ownership of the same declaration is rejected.

## Concrete source-free owner prerequisite

Do not reuse diagnostic NodeKeys as imported body keys. Add a cold catalog of
remapped imported body rows keyed by `(artifact identity, BodyRef)` and a separate
origin table of optional provider file/span/line information. All such locations
are diagnostic; no span, path string or importing use grants semantic authority.
Dense `ImportedBodyId` is compiler-local and never emitted. The row binds exact
remapped declaration, parent BodyId and typed body kind. Named bodies continue to
use BodyKey::Declaration(remapped DefId); nested bodies get real local closure/
anonymous identities tied to their imported owner row, not invented DefIds.

Proposed minimum common representation is a body-level origin discriminator:
`BodyOrigin::Source(NodeKey) | Imported(ImportedBodyId)`. Body and Contract retain
that same origin plus their real BodyKey. Error and proof source labels use a
neutral diagnostic site; body-local statement/local/block locations use a u32
`LocationId` resolved by that origin. For local bodies this wraps the real AST
NodeId without a new per-node allocation; for imported bodies it indexes the
cold provider-origin labels. It is never passed to `Package::unit` as a NodeId.
No Value, View, Constant, Place or scalar payload grows for metadata provenance.
Measure Body/Contract/diagnostic-owner layouts before accepting the exact tag
representation. The source builder keeps its known local NodeKey and does not
consult an imported origin table on ordinary construction.

Consumers audited in validation:

| Semantic ownership use | Required migration |
| --- | --- |
| instantiate `with_premises`, initializers/anonymous/parent_results; borrow region/check/loans template matching | Match BodyKey + exact origin row + full selected instance, not a diagnostic source. Local source matches remain additional local validation. |
| borrow/regions/output and lower/instances/proof group by source/parent source | Group by typed owner/body identity; retain exact template/instance and parent link. Equal source labels never combine two bodies. |
| templates Prepared maps and region Collector completed/pending maps | Local source collection remains local; imported originals enter an explicit keyed original-template/Contract append seam. No fake Function.body or fabricated NodeKey. |
| instantiate caller_transport/allocated_owners, SourceAssociated, InlineInstance | Existing source-only registries remain source-only. Imported ordinary generic functions need no source CallerId. Imported anonymous/associated contexts require their real imported owner protocol before enabling that family. |
| body constants eligibility, const code catalog/provider | Resolve declared const status and original typed body by owner. Importing a foreign symbol or const bit never supplies Code. Const Code is published only after local original + instance proof and existing escape/eligibility gates. |

Diagnostic-only consumers include borrow error sites, loan/ownership/views,
lower function/alignment errors and per-statement locations. They resolve a
neutral site through the cold origin table or actual importing demand site.
`body/build/*`, `Declarations::node`, `Package::unit` and AST scope plans remain
strict local-source consumers. The importer never calls a builder that expects
syntax. Existing local text views must stay byte-exact unless independently
reviewed behavior changes. This migration is one coherent prerequisite; simply
making source optional at the last error formatter is insufficient.

### Original components and executable ownership

An original parent can be required as proof context even when ordinary template
partitioning excludes it as a runtime specialization candidate. Retain its real
BodyKey, origin, child-parent links and paired Contract without reclassifying it
as an executable generic template. The imported component and the runtime
template index serve different purposes.

Derive these roles from validated declarations and body kinds; an artifact role
bit cannot grant executable authority. Original proof success cannot pay a
specialized caller's debt. Reprove the complete imported original component with
its exact contracts, then require the actual instance's ordinary proof before
execution. Source-local caller handles and serialized Checked/Permit tokens
remain forbidden.

Validate duplicate identities and every parent edge before transactional
publication. Equal shapes or diagnostic locations cannot reconcile two owners.
Cache completed components by their real identities so repeated child ports do
not rescan or reprove every ancestor. Proof-context-only rows must remain absent
from executable-template lookup. This is an import design constraint; generic
body transport remains unimplemented.

## Export, import and demanded execution

1. Export only from an exact, immutable original template/Contract that passed
   the existing original semantic/eligibility/promotion/region/loan/destruction
   gates. Add a narrow borrowed export visitor to the checked original owner;
   do not clone a full proof graph per export root. Reachability includes private
   body helpers, header/nominal/trait/impl/member contracts and const bodies.
   Hidden layout/helper declarations never acquire public bindings.
2. Decode into bounded neutral tables; validate all lengths/tags/owner chains,
   type/const/binder edges, slot kinds/unused slots, scope/owned intervals, CFG
   indices, root reachability and canonical ordering before semantic publication.
   Typed-body validation checks operand/place/signature/result agreement, return
   sites, storage endpoints, canonical intrinsic identity and eligibility. It
   must not accept an omitted obligation merely because a body claims checked.
3. Reserve every named/unnamed defining identity, parameter and body once in
   canonical dependency/owner order. Register parameters using their remapped
   owner/ordinal/kind; build explicit type/const/region/clause remaps bottom-up.
   Recursion through callable/body references is allowed after reservation;
   by-value nominal cycles, binder/owner cycles and malformed type edges are not.
4. Publish declaration headers/trait impl inventories only after remapping and
   their ordinary formation/conformance/coherence checks. Rebuild implied
   environments and availability from those real declarations. Do not import
   per-query witnesses, active search states or assumed local proof facts.
5. At pipeline preparation, load immutable imported original CFG + Contract
   through a typed `ImportedOriginals` seam alongside local prepared templates.
   Recompute original promotion/destruction facts from the imported typed plans
   as required and run `borrow::templates::verify`; store the resulting local
   Checked borrowing that exact catalog. Import success itself is not proof.
   All bodies actually exported as originals must have been checked by provider;
   no invalid unused generic source may escape via lazy client demand.
6. On actual runtime/const demand, select the full remapped Instance through the
   same callable/impl selection machinery. Ordinary caller debt must complete
   before `from_checked_caller`; unsealed/manual requests retain Required.
   Substitute the retained original body and Contract, normalize projections
   with their real debts, reprove the concrete instance and then lower/execute.
   Do not register body-owning generics as foreign Import leaves: the current
   demand skip must distinguish body templates from externally linked functions.
   Nongeneric foreign ABI imports keep their current path.
7. Source-only and materialized constants stay distinct. Imported const-function
   and initializer bodies require checked local execution service; no AST or
   SourceComputation handle crosses the artifact. Final value/escape/provenance
   validation and the existing static/relocation import route remain mandatory.
   Root's stored-array recipes are data, not evidence that a body is const-safe.

Shared edit consumers therefore span support/crates codec/validation; resolver
reservation and source-free provenance; semantic parameter/callable/trait/impl/
member header registration; body provenance/template/Contract import; pipeline
construction/const provider; demand/proof and symbol naming. Rvalue/Requirement
changes are not needed for the first named-function family, but new body-origin
and imported owner APIs require coordination before implementation. Nested
closure/inline consumers must not be enabled by weakening existing source APIs.

## Bounds, determinism and qualification axes

Retain existing closed-profile quotas. The new family needs explicit combined
limits for decoded bytes, declarations/parameters/body rows, shared graph nodes,
arguments/clauses/proof scopes/edges, CFG cells and diagnostic strings. Preflight
capacity before allocation, account scratch plus retained tables and charge work
on error. Publish complete rows/catalog phases only; malformed imports leave no
partially usable body or permission. Work is proportional to retained graph and
body edges plus actual demands. One shared remap per artifact and one immutable
original per owner must avoid width × number-of-functions graph rescans.

Canonical identity order, owner order and first-use graph order are independent
of alias spelling, input artifact order, local arena indices, HashMap traversal
and demand order. Symbol identity uses the defining crate/owner and complete
code-relevant type/const arguments; lifetime erasure cannot erase proof context.
Repeated const N values with equal public signatures must not share values.
Two crates with identical paths and declaration ordinals remain distinct.

Declared independent sources are frozen under
`obj/parallel/pa29-generic-design/{declared.json,inputs/}` before observation:
identity/private helper, distinct const arguments, const generic identity,
array generic, paid early lifetime, nested callable binder, user-trait conditions,
generic record, projected input, associated method/default, transitive helper,
and opposing unpaid/mismatched/private/nonconst/cross-crate identity cases.
Invalid unused move/escape/const providers are separate original-source negatives.
The bounded Rust runner passed all 23 declared commands: three accepted rlib
builds (two distinct same-path crate identities plus the bridge), three invalid
provider rejections, eleven accepted client objects and six rejected client
objects. Sources and expected outcomes were frozen before observation; exact
commands/statuses/source hashes are in `observed-second.json`. The first run
retains a harness E0463 at the transitive client because the rlib filenames did
not match rustc's dependency search prefix; only those output filenames changed
for the successful retry. These cases do not prove compiler support or
source-built core.

Future focused private controls must alter owner/parameter kind/ordinal, swap
late slots, remove a raw premise/debt, forge a same-shaped Contract, use another
artifact's BodyRef, corrupt proof scope/gap ownership, mismatch body/result and
change const eligibility. Include reordered dependency and demand order, private
helper non-exposure, recursive demand versus expanding instance growth, actual
native values, codec backward bytes and error/limit cases. Scaling varies shared
type graph width, number of declarations, calls per caller and distinct actual
instances independently (8/32/128), with ordinary512/constcall64 controls. Root
owns combined full release, frozen performance, profile assignment and activation.
