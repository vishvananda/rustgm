# Source role/body graph after actual original proof

Review input, not a frozen wire descriptor or implementation. This refines
[the accepted source-client protocol](pa29-source-client-protocol.md) using the
actual bodies retained by [the capability checkpoint](pa29-provider-capability-checkpoint.md).
The provisional descriptor must not enable an artifact until the complete graph
below is representable and validated. Existing profiles remain unchanged.

## Actual ownership closure

Use authenticated artifact-local declaration/body/site coordinates. Local DefId,
TypeId, ParameterId, RegionId, NodeKey and catalog epochs are not wire identities.
The selected dependency/current crate and defining edition remain explicit.

The unchanged semantic provider contains these genuine declarations:

| Owner family | Owned declarations and dependencies |
| --- | --- |
| PointeeSized, MetaSized, Sized | Distinct trait declarations and actual Self declarations/parameters; the exact sizing hierarchy, deny-impl and retained semantic attribute claims. |
| LegacyReceiver | Trait/Self declaration, plus the two actual generic impl owners for `&T` and `&mut T`, their own Self/type parameters and PointeeSized input. |
| Destruct | Actual const trait/Self declaration, PointeeSized input, deny-impl and forced dyn restriction. No const-destruction permission. |
| Freeze | Actual unsafe auto trait/Self declaration. Existing Freeze proof rules remain authoritative. |
| Clone | Actual const trait/Self declaration and Sized input; distinct clone_fn and clone_from member declarations. clone_fn genuinely has no provided body; clone_from genuinely has the provided body observed below. |
| Copy | Actual ordinary trait/Self declaration with the actual Clone supertrait. |
| `impl Clone for u8`, `impl Copy for u8` | Separate real impl owners, exact target/trait applications and member/conformance data. The Clone impl owns its actual clone method; its body cannot be replaced with a library factory. |
| identity | Actual module-level const function and its own T parameter, formal binding, signature, declaration premises and provided body. |

Every trait/impl Self declaration is reserved from the typed inventory, with its
real kind and owner. A declaration absent from that inventory cannot be invented
to fill an index. Receiver/local definitions are separate from formal generic
parameters. Formal-to-local associations remain explicit; `definition: None`
must continue to mean absence. Trait/member ownership, lexical body ancestry and
recursive callable dependencies are distinct relations.

Source-owned vectors remain owned by their producer. The neutral borrowed
header adapter reads their complete parameters, inherited scope, written-prefix
boundary, binder/effect data and member inventory. Imported adapters borrow
equivalent authenticated rows; they never ask for a fabricated AST owner.

## Exact observed bodies

The recorded clone_from original has two late-bound reference parameters,
`&mut Self` and `&Self`, and two typed reference dereference projections. Its
ordered values and statements are:

1. Make both parameter locals live; form the shared source dereference.
2. Borrow that exact place, then Constrain the borrowed value to the checked
   call argument type.
3. Form the actual clone_fn FunctionItem with its retained generic arguments.
4. Call it through its actual opened call signature and ordered argument list.
5. Form the unique destination dereference, Store the call result there, form
   unit, end the parameter storage suffix, then return unit.

The original has one block, eleven statements, fourteen region obligations and
four possible cleanup sites: destination replacement and three recorded exits.
It retains the exact conditional `Self: Destruct` premise in its actual template
Environment. Its region Contract has two universals, the exact rigid Self
parameter and a signature matching the original callable; trait premises must
not be reconstructed from the region-only type-outlives list.

The actual identity original has one T formal/local, StorageLive, Read,
StorageDead and a return terminator. It has zero universals, three region
obligations, one retained source proof and four possible cleanup sites. The
read is governed by the actual source Copy policy and move proof; it is not
automatically a Copy operation merely because the body returns its input.

The full dump contains every observed local, projection, value, statement,
terminator and cleanup trigger. A future encoder must also preserve/rederive
every actual operation-derived requirement; recording the counts alone would
not qualify the encoder. The original's Proved/Checked objects are never sent.

## Concrete row and consumer changes to review

The proposed new source-language descriptor needs these additional neutral
tables/fields; it must not reinterpret the existing scalar v0/v1 forms:

- **Declaration headers:** actual kind, defining crate/edition, parent owner,
  complete owned parameter slots and captures, full qualified input clauses,
  explicit written-prefix boundary, signature scheme and complete member list.
  Trait flags retain unsafe/const/auto state and supported semantic attributes.
  Impl rows retain the actual Self declaration/type, optional trait application,
  unsafe/const/negative state and full conformance/member mapping. Missing and
  empty inputs/bodies remain distinct.
- **Role bindings:** exact provider declaration coordinates for every requested
  role/member, supported attribute claims and separate diagnostic metadata.
  Copy/Sized cannot become name matches or LegacyBuiltin bits. Header validity
  grants no caller evidence.
- **Types and arguments:** retain actual parameter owners, reference mutability
  and lifetime, callable signature binder/convention/safety, FunctionItem target
  declaration and complete actual arguments. Static, nested-bound and body-owned
  variable lifetimes need distinct checked forms; numeric free region IDs from
  the producer are never imported as ownership. Complete unused slots survive.
- **Bodies:** retain genuine declaration/body origin, body-local sites, lexical
  scopes, formal-local associations and ordered flat tables. The observed forms
  require Dereference projections, Borrow, Constrain, FunctionItem, Call, Unit,
  Read, Store, storage actions and Return. Call signature and actual argument
  roots must match the caller's scope and selected target; the public callable
  signature remains separate from an actual materialized execution signature.
- **Proof inputs:** preserve complete original Environment clauses, lifetime
  parameter/variable declarations, signature and formation/coercion/trait goals.
  Import derives/checks required goals from every typed operation and does not
  trust a payload that omits one. Original environments are removed/replaced at
  actual instance selection in the existing manner; caller debts are not paid
  by metadata validity or successful original proof.
- **Provided and concrete bodies:** every provided generic/const body in the
  closure must enter the checked original sidecar. An ordinary concrete native
  method may use an actual compiled external callable contract where supported;
  this is explicit compiled-body provenance, not body absence or CTFE permission.
  Any body requiring instantiation/CTFE must carry its actual typed original.
  Additional concrete/helper operation forms must be inspected from that real
  completed producer before the descriptor is frozen.

Resolver installation reserves real declarations once and authenticates the
complete actual remap, including owner/ordinal/kind, before catalog publication.
Semantic header installation must publish completed type cells/headers before
ordinary readiness queries, without marking any body proved. Role validation
uses the common complete header validators. The cold immutable imported original
sidecar owns Template/Contract rows once; no extra Arc layer is added to source
templates. Full original/group proof precedes ordinary instance and CTFE proof.

The source-client policy is a separate transaction before bootstrap identities
exist. It selects one authenticated source provider and installs its actual
roles; legacy Copy/Sized/prelude identities cannot coexist or become fallback
after a failed source import. Defining edition and later API/const-stability
availability are preserved independently of the consumer's edition/features.

## Required rejection and work controls

Reject foreign Self/formal/argument owners even inside the same artifact,
incorrect trait-member parents, changed effects/qualifiers, omitted or weakened
operation-derived goals, incomplete provided bodies, unknown lifetime ownership,
wrong source-provider selection, mixed bootstrap roles and invalid unused bodies.
Retain paid/unpaid caller, escaping reborrow and cold/warm cross-instance
opposition. A library stability inventory is not a trait proof or const-call
payment. Unsupported active contracts reject before any public mapping/root.

Charge combined old/new tables, maps, scratch and failed work before growth.
Index each exact declaration/body once, share the actual argument/type DAG and
structural signature preparation, and keep proof per actual caller/instance.
Vary declarations, shared roots, uses and independent callers separately.
Rollback must leave no origin/role mapping, executable original or public alias.
All eighteen prior transport scenarios and full upstream bootstrap remain
required; this review does not narrow their completion boundary.
