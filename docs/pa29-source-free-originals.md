# Source-free originals for executable generic crate transport

Design only, refreshed against frontend `46d8b82a`, the focused actual-parent port
`4eb1a20f`, and C's current `defining_heads` proposal. No production API, artifact
profile tag or edition encoding is changed by this document. The broad required
scope remains in [the generic transport proposal](pa29-generic-transport-proposal.md).
This prerequisite must lead to actual generic/const cross-crate execution, not
stop at another neutral vocabulary. Root owns edition/build integration.

## Body identity and location records

Keep the existing real local NodeKey/NodeId representation and source builder.
Introduce these private types at the body ownership boundary:

```rust
struct ImportedBodyId { catalog: NonZeroU32, row: u32 }
enum BodyOrigin { Source(NodeKey), Imported(ImportedBodyId) }
struct BodySite(NonZeroU32);
enum ProofSite {
    Source(NodeKey),
    Imported { body: ImportedBodyId, site: BodySite },
}
```

Only the compiler-local imported catalog constructs ImportedBodyId. `catalog`
comes from a checked, process-local unique catalog epoch; `row` is dense in that
catalog. It is not an artifact ordinal, environment identifier or emitted value.
Every lookup checks the epoch, row, actual BodyKey and remapped declaration.
Simultaneously live catalogs and copied bodies cannot exchange same-number rows.
BodyOrigin is semantic ownership; rendered file names/spans never qualify it.

Body and Contract replace their root NodeKey with the same exact BodyOrigin.
The named function BodyKey remains Declaration(remapped DefId). Imported nested
anonymous originals require an additive Imported(ImportedBodyId) BodyKey, while
closure rows use their actual remapped ClosureId only after closure-header
transport exists. No fake Function.body, AST node, source DefId or ClosureId is
created to make a lookup succeed. Existing source-only anonymous/caller APIs
reject an imported origin until their actual owner protocol is implemented.

BodySite replaces a body-relative NodeId only in records used by both body
origins. A local constructor retains the actual NodeId's checked nonzero value;
its source-node accessor requires BodyOrigin::Source. An imported constructor
checks a dense operation/site ordinal against that body's imported site table.
It never constructs NodeId or calls Package::unit. Local/Value/Statement/Block
locations keep one four-byte site field: no field is added to hot Value, View,
Constant, Place or scalar payloads.

ProofSite replaces NodeKey where an obligation/proof/error can identify an
operation outside its enclosing contract, including child goals retained at a
parent. Imported site identity is (actual imported body, checked operation slot),
not a diagnostic label. Parent debt matching keeps that complete identity and
mapped relation. It must not deduplicate different bodies merely because their
provider locations or region numbers coincide. Source ProofSites preserve real
source keys exactly. This explicitly audits the current dual ownership/diagnostic
uses of Obligation.source rather than changing only an error formatter.

Measured port baseline is Body=448, Value=24, Code=304, Target=28, Request=56,
full Proved=472, paired Proved=496 bytes. NodeKey is two four-byte IDs, and NodeId
is NonZeroU32. The proposed ImportedBodyId payload is eight bytes and BodySite
four; the straightforward enums are expected to require 12 and 16 bytes for
BodyOrigin and ProofSite. Those new enum/containing-record sizes are **not yet
measured**: no production migration or standalone layout build has run. The
first approved implementation gate measures Body/Contract/Obligation/Proof/error,
Function/header, Prepared/Checked and Code after field placement. Value must
remain 24 bytes. Do not claim a Body size increase simply by summing fields;
actual compiler layout/padding and cold vector capacities must be recorded.

## One cold imported catalog

Add a lazy imported-original sidecar to the existing crate linkage owner. Its
concrete contents are:

- epoch, a vector of artifact descriptors and dense imported body rows;
- (artifact slot, BodyRef) -> row and BodyKey -> row Fx indexes;
- each row's actual remapped declaration/key, optional exact parent handle,
  numbering-unit handle, BodyKind, original-data slot and site range;
- one shared immutable decoded original pair per row: typed Template plus Contract,
  with the complete header environment; this pair contains no Checked/Permit;
- site rows mapping operation slots to optional provider-file/range labels;
  file strings/profile data are shared per artifact, never copied per operation;
- incremental retained-byte and work totals, including failure paths.

The existing support::crates::owner_catalog BodyCatalog supplies the validated
artifact owner forest (100,000 declaration/body cap, canonical breadth-first
rows, no forward/cyclic/foreign parent). It does not supply runtime availability
or proof. The new catalog binds that forest to actual remapped headers and body
contents. Numbering-unit rows remain separate from diagnostic sites: import
validates the declared shared inference family against body kinds, parent links
and actual opening/closing maps. It never treats a numeric region frontier as
ownership. Each body-variable/quantifier reference is validated in its exact
unit, retaining empty and unused binder/parameter slots.

Decode reserves IDs and preflights all capacities before population. Keep the
existing 16,000,000-byte artifact limit and body/declaration quotas. Charge decoded
originals and catalog-owned maps/site vectors through the retained original
preparation budget; shared immutable pointees are charged once by their actual
owner, not rescanned per request. Imported original/template and constant Code
budgets remain mandatory. O(rows + edges + sites) validation/index construction
is separate from per-demand proof work; repeated ports cannot rescan all imports.

To avoid a full body/Contract clone per consumer, keep the decoded pair immutable
and shared in this cold sidecar. Add an indexed imported sidecar/iterator to the
existing template collection, retaining its ordinary local Vec and empty fast
path. Prepared and Engine lend the same imported original data. Checked indexes
address local or imported rows in O(1); they still publish only actual local
verification results. This small collection adaptation must be coordinated with
borrow::templates and measured; it must not wrap every local Template in Rc/Arc
or retain duplicate proof graphs for const and runtime lookup.

## Concrete consumers and proof ordering

| Consumer | Required change before enabling generic imports |
| --- | --- |
| sema::crates + generics/traits | Reserve complete defining declaration/parameter/binder identities, then validate/remap generic headers, trait/impl/member/default conditions. Register const eligibility as a declaration property only. Existing closed foreign-import behavior remains intact. |
| Function/body header lookup | Add a borrowed local/imported header facade carrying definition, exact original origin, signature, full parameter order and universal schema. Imported functions do not enter local AST-only Function.body/node tables with fabricated entries. |
| Prepared/region Collector | Local builders and collectors remain source-only. Append imported typed originals/contracts through the exact keyed sidecar; independently validate/derive operation requirements before ordinary original proof. |
| Original::validate_parts | Source branch keeps its exact current checks. Imported branch checks epoch/row/key, remapped header/result/universals, environment, unit and parent/link identity against that catalog. Equal arity or shape is insufficient. |
| borrow original/component proof | Group by BodyOrigin/BodyKey, preserving actual immutable Contract lineage and complete parents/siblings. Context-only originals remain unavailable to executable lookup. Storage/CFG cache sharing remains structural only. |
| Checked::function_proof(ItemId) | Return the same exact Template/Contract/full region-result tuple for imported rows after actual region and loan verification; no artifact success token enters this index. |
| Engine::prove_defining_original | Use validate_parts and the real ordinary original proof. C's opaque producer can consume only matching full original success; specialize/prove_body/paid-child remain proof-only. Imported hidden declarations must themselves be validated before publishing choices. |
| lower::instances::Ordinary::select | A foreign function with an imported checked template must demand instantiation; it cannot take today's unconditional externally-implemented fast return. A closed external declaration without transported body retains the old route. |
| const Engine/provider | Resolve a demanded imported const function by exact original row and full Instance; run original and instance eligibility/region/loan/escape proof before Code. Cold/warm Code keys include complete owner/arguments/result context; scalar value presence is not execution permission. |
| diagnostics/views | Resolve ProofSite through the real local source or cold imported location table, with the actual importing use as fallback and provider provenance as a note. Never invent a FileId/NodeId to render a source-free body. |

Typed import validation must derive required constraints from all supported
operations and retained selection requests. It cannot trust an omitted Contract
goal or a claimed impl witness. Declared assumptions must be reconstructed from
the exact imported header/input environment; alias equalities, WF/outlives,
coercions, borrow provenance, formation and selected-impl debts remain goals.
Unsupported operations reject export/demand explicitly. Original proof success
still cannot pay a specialized caller debt: ordinary from_checked_caller/instance
proof remains mandatory. SourceComputation/CallerId/reservation/paid-parent scope
handles and all Checked/Permit/Proved payloads are forbidden in artifacts.

C's current defining-head seam is used without weakening it:
`Original::validate_parts`, `Checked::function_proof(ItemId)` and
`Engine::prove_defining_original`. Parent-component publication validates every
exact pairing before publishing any head. The imported branch changes ownership
lookup, not that ordering, joint region+loan checks or choice producer.

## First executable multi-crate checkpoint

Use one complete generic envelope with traits/impls/associated conditions in its
schema from the outset; preserve exact profile 0–7 and stored-array bytes. Do not
assign a new tag in this proposal. Import/export must jointly support:

1. A provider exporting `id<T:Copy>`, a private generic helper, early-lifetime
   shared reborrow and `const fn array<const N:usize>() -> [u8;N]`, plus a consumer
   that actually instantiates/runs these and calls the imported const body in an
   array/constant initializer. Keep unused parameter slots and equal-signature
   distinct type/const/lifetime applications.
2. A provider user trait and impl, with a generic function calling the constrained
   method; the consumer's paid implementation executes, and a missing/unsatisfied
   impl rejects. Transport real method selection requests/conditions and ordinary
   impl validation, not a function-pointer substitute or a trusted proof bit.
3. A transitive provider/helper graph and two independent artifacts with colliding
   local ordinals/names, imported in both orders. All outputs and defining
   identities must agree with independently compiled original Rust programs.

The source-free owner migration may land first as a behavior-preserving
prerequisite, but the family is not complete until these paths execute through
ordinary Templates/instantiation/CTFE/LowIR/native linking. Generic trait defaults,
associated type/const/GAT bodies, generic nominals/destruction, imported closures/
anonymous conditional outputs, opaque/dynamic interfaces, macro contracts and
allocation/relocation graphs remain required subsequent consumers of this same
owner/parameter format. No source-only caller handle is synthesized to skip them.

Each artifact descriptor retains its defining crate's semantic profile, including
Rust edition. Old language ID0 stays exactly Rust1.98/2024 under the existing
encoding. Rust2021 serde_core/serde_json and Rust2024 consumers must not exchange
implicit consumer-edition semantics: body/macro definition edition comes from
the provider profile. Conflicting profiles for the same defining artifact are
rejected; profile participates in import/cache identity. Root owns profile/tag
and build integration; no new edition tag is assigned here.

Before enablement, qualify foreign/stale/epoch/site/unit/header/result/binder
negatives, omitted-goal and forged-impl artifacts, unpaid and escaping references,
invalid unused generic bodies and const eligibility, cold/warm cross-caller
requests, reverse import order and deterministic repeated output. Independently
vary declarations, shared type-DAG width, demanded instances and repeated ports;
record exact visits/capacities and ordinary no-import allocations. Do not repeat
an entire graph validation per signature or pay instance proof from a cache bit.
