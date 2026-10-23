# Generic body transport: next schema and consumer review

Proposal after the source-compatibility checkpoint `90629bdc` and source merge
`bcf3ef62`. The merged Copy/site adapters passed focused strict checks
and 75 private controls at `11199b03`. The subsequently approved envelope 9 /
row version 0 and first executable imported id family are documented in
[executable-generics](pa29-executable-generics.md); the broader design below
remains the required continuation, not an implementation-completion claim.
The complete executable source contract remains
[generic-contract-sources](pa29-generic-contract-sources.md). The full remaining
PA29 requirement is not limited to the first executable family.

## One artifact-owned envelope

Keep old profiles zero through seven, stored-array and variadic encodings
byte-exact. A reviewed later profile would carry the following distinct tables;
none of their references is a local DefId, TypeId, ParameterId, NodeKey, epoch or
proof handle.

| Table | Identity and contents |
| --- | --- |
| Crate profile | Existing defining CrateIdentity/dependencies, explicit defining Edition and supported semantic profile. Never infer edition from the importing crate. |
| Declarations | Complete owned declaration ordinals, including private helpers/trait members/impl owners, stable defining identity, visibility, kind and optional original BodyRef. Public names remain aliases of defining identities. |
| Parameters | `(owning declaration, ordinal, kind)` for every type/const/early-region slot, including unused slots. Const parameter types/default roots and source-level requirements remain explicit. |
| Binders | Actual owning declaration/body and parent binder, binder kind and full slot count, including empty binders. Public callable and original body binders retain their distinct relocation. |
| Types/arguments/constants | Shared canonical tables with per-header, field, clause, operation and result roots. Parameters reference their actual owners; applications retain complete ordered arguments. |
| Traits and impls | Actual trait/Self/member identities, full applications, conditions, qualified ordinary/const/maybe-const effects, impl constness/negative/safety and exact member/default relationships. |
| Body ownership | Existing canonical BodyCatalog, separate region-numbering unit and real operation-site partition. Named declarations do not fabricate bodies for clauses. |
| Originals | One typed operation/CFG record and complete original contract input for each original body, associated with its actual header and environment. No published proof result. |
| Locations | Separate declaration sites and body sites, with bounded optional provider-file/span labels. These labels never establish ownership or proof. |

A cross-artifact reference retains both its defining dependency identity and its
owned declaration/body ordinal. Dense remaps are installed only after all real
resolver declarations exist. The actual importer authenticates every complete
external-to-local mapping against its metadata definition table before it can
install the origin catalog. Merely seeing `Definition.node == None` is not enough.

Type nodes must distinguish parameters, nominal applications, projections,
callables and their binders, references/pointers, arrays with const roots,
products, slices and supported opaque identities. A parameter reference's owner
and kind are checked before interning it. Bound references name actual binder
frames and slots; root validation must preserve genuine captures while rejecting
same-arity sibling frames. Callable root validation retains unused binders and
complete early arguments. Validate by-value nominal cycles separately from
reference edges. Do not duplicate the shared graph per signature.

Const nodes reference typed scalar values, actual const parameters or exact
unevaluated bodies/applications supported by the compiler. Importing an
unevaluated const root grants neither evaluation nor a concrete array bound.
Unsupported expressions/contracts reject explicitly; they do not become zero,
static or an erased argument.

## Typed original operations

Use a versioned, explicit operation encoding, not Rust enum discriminants or
rendered CFG text. Preserve local/temporary/guard/scope/projection/value/argument/
statement/block tables with checked indices, exact value types and actual body
sites. Every statement range, place parent, variant/member, list and successor
must validate before body publication. Operation/type relationships must be
validated, not just index bounds.

Source-only identities need declarative translations, never integer casts:

| Existing frontend record | Transport boundary |
| --- | --- |
| Function/constant/static/variant/member IDs | Actual defining declaration/member references, then checked canonical remaps. |
| Callable Instance | Function/closure declaration plus complete arguments and exact requested signature. Import constructs its local canonical Instance. |
| ConstId/TypeId | Roots in the artifact-owned canonical tables. |
| InlineConstant/InlineInstance | Original BodyRef plus complete enclosing argument/binder binding description. No NodeKey or serialized live inline context. |
| SourceAssociated | Exact selected initializer declaration/application and requested result, owned by its original caller operation. Never serialize SourceComputation or CallerId. |
| Conditional output link | Exact parent/child originals, numbering units, binder bridges and raw goals. Never serialize a paid-parent row or Proved. |
| Intrinsic/adapter | A declared operation request checked against the actual imported/source registry and header; no unchecked factory permission. |

The first executable family must not silently export unsupported instances of
these records. The later inline/source-associated/conditional consumers remain
required when not exercised by that first family.

Retain `Unary.literal`, per-side `BinaryOperands` and `Cast.direct_variant` as
bounded structural source data. Validate their supported encodings against the
actual operation/operands. They are not promotion, const, lifetime or execution
permission. Source and transported division/remainder/minimum-overflow, grouped
literals, blocks, locals and calls must keep their declared parity. Promotions,
resolved-callable maps, owned/storage facts, checked closure entries and all
specialized bodies are recomputed by the importer rather than accepted as
transported authority.

## Contract reconstruction and proof

The imported header supplies the exact declared/input assumptions, universal
schema, parameter identities and immutable Environment. Validate every retained
contract input against that header. Derive mandatory well-formedness, operation
relations, actual selection/normalization requests, borrow/formation and caller
debts from all typed operations, including skipped branches. An omitted goal or
a claimed implementation cannot qualify an ill-typed body. Retained selection
requests replay ordinary selection; any stored target must agree with its actual
result and residual obligations.

Raw contract goals and binder relationships are data requiring proof. Artifacts
carry no Checked, Permit, Proved, SourceComputation, source caller/reservation,
paid scope, Copy decision or success flag. The source Copy carrier preserves its
neutral requirement and exact environment; import prepares/proves a new local
carrier. Actual const/conditional effects remain qualified and are paid at the
existing call/proof boundaries.

Original grouping preserves exact parent/sibling/unit membership and immutable
Contract lineage. Context-only originals cannot enter executable lookup. The
ordinary joint region/loan proof must succeed for the complete original group;
then every demanded instance still pays its own contract, eligibility, promotion,
escape and caller obligations. C's defining-head publication consumes only its
exact validated original result. Specialize/prove_body/paid-child remain proof
consumers, not independent head publishers.

## Cold runtime collection and consumers

Keep the existing local Template Vec without wrapping each source template in
another Arc. Add a cold indexed imported sidecar owning one immutable
Template+Contract pair per imported row. Both early const providers and runtime
instantiation borrow that same imported payload. Collection access must cover
local and imported originals consistently; imported rows cannot be missed by a
source-only `.bodies` iteration. Measure collection/Checked owner layouts before
adopting the concrete access representation.

A borrowed local/imported function-header facade returns the actual declaration
origin, optional real original origin, complete signature/parameters/universals,
const/effect contract and environment. Imported declarations never get invented
Function.node/body entries. The imported Original validator checks current
catalog epoch/kind/remap, actual key, result/header, numbering unit, complete
binders, parent/link and environment before ordinary proof. Matching counts or
shapes are insufficient.

Required execution changes:

1. Install complete typed headers/trait/impl conditions and canonical roots using
   checked deterministic remaps; keep source AST facts source-only.
2. Decode/validate each imported original once into its cold row. Preflight all
   retained/scratch growth and publish only a complete transaction.
3. Supply those same rows to ordinary original proof and early const providers.
   Original success never pays specialized caller debt.
4. Prefer an actual imported original over the current foreign-function runtime
   shortcut. A bodyless closed foreign declaration keeps its existing path.
5. Demand actual instantiation and stable symbols from defining identities and
   complete arguments; never render catalog epochs or local IDs.
6. CTFE resolves the exact original and full Instance, proves it, then publishes
   ordinary Code. A cached scalar/source-only value is not execution authority.
7. Route imported errors/views through the checked frozen origin catalog. Missing
   lookup stays explicit; no source-file-zero compatibility fallback.

Preserve original-to-owned graft before applying external actual arguments, so
colliding numeric local/external region IDs cannot enter a range-only fold.
Complete canonical graph observations may be shared, while namespace ownership,
reservations and actual caller proofs remain context-specific on cold/warm use.

## First executable family and qualification

Drive the unchanged 18-scenario/17-source matrix through actual candidate crate
emission/import, LowIR and seven native clients. It already independently proves
id<T:Copy>, private helper use, unused slots, early reborrow, const/const-generic
arrays, real user trait calls, a transitive relay and equal-path different-crate
identity. Keep its Copy, trait, const-call, escape, array, foreign-trait and invalid
unused source opposites. Declare additional provider-owned impl, private-helper
access and unused-parameter-bound opposites before qualification; do not alter
the already-qualified originals.

Malformed metadata must separately cover foreign/retired owner handles, missing
remaps, same-arity wrong headers, missing operation debts, wrong constness,
foreign binder/unit/member references, duplicate bodies and transactional failure.
Vary one artifact graph width, number of imported declarations and actual use/
caller count independently. Canonical graph installation is once per artifact;
full original/instance/caller proof duties cannot be erased to meet a work bound.
Keep no-import allocation/layout and old-profile byte controls.
