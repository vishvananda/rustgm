# Provided-method execution milestone

This is the required next execution milestone for PA29/PA31 transport, beyond
private template and Environment readiness. It is not implemented or qualified.
The [empty Contract checkpoint](../private-empty-contracts-checkpoint.md)
qualifies a prerequisite; it does not execute imported provided methods.

Keep the complete
[qualified semantic provider](../../pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs)
and add a genuine ordinary, non-role trait:

```rust
pub trait Holder<V> {
    fn carry<U>(value: U) -> U { value }
    fn borrow(value: &V) -> &V { value }
}
impl Holder<u8> for u8 {}
```

Produce an actual supported artifact, encode/decode its bytes, and create a
fresh SourceClient. Its main must invoke the provided `carry::<u8>` and `borrow`
bodies through the actual `Holder<u8>` implementation, carrying 7 through the
borrow and dereference. Verify the actual imported method instances, original
body proof and fresh substituted proof. A separate fresh `answer() -> u8`
consumer must link through the existing pinned backend and host C driver and
print exactly `7\n`. Successful header lookup or a separately emitted local
identity function is insufficient.

Wrong type/arity/trait arguments and a returned reference to a dead local must
fail at their actual semantic or proof boundary. Keep Clone, clone_from, their
ordinary/conditional const contexts and all original declarations in the source
family. Ordinary u8 Clone gains no const-call permission.

The [reviewed demand policy](../library-bootstrap-acceleration.md) separates this
first execution from unrelated imported body preparation. The defining provider
still checks all required source bodies and the artifact retains the complete
family. Decode all structural/header/visibility contracts. Unrequested imported
default bodies stay explicitly Pending; the first actual demand must construct
the genuine defining environment and prove the original before fresh instance
proof. Their complete demanded cases remain PA29/upstream requirements.

## Proposed codec boundary for the executable consumer

The disjoint codec task should reuse the existing row codec and add one explicit
`Source2024ProvidedContracts` profile (tag 11, row version 1). Existing tag 11
version 0 and all tag 10 versions remain byte-exact. This is an interface proposal,
not public admission or a completed transport claim.

| Cold record | Exact content and authority boundary |
| --- | --- |
| Tag 11 / version 1 envelope | Existing complete Provided columns plus namespace and modifier records below. Store advisory rows in the SINGLE existing `ValidatedRows.attributes`; do not copy them into another wrapper. Keep `SourceRows`, `FunctionHeader` and `TraitHeader` unchanged; retain the raw full family and numbering. |
| `NamespaceColumns` | Reuse the actual root/module/binding rows from the same `Stage::namespace_columns` / `ExportBindings` producer and existing pending-binding/module-remap validator. Consumer supplies its real validated `Dependency` alias. Missing columns grant no names. The first executable target requires only the genuine root-public Holder family; unsupported domain/reexport breadth stays explicit. |
| `HeaderAttributeColumns` | Reuse complete parallel function/trait rows, actual declaration/site, `must_use` presence and optional reason, and inline hint plus applicability. New complete rows give checked absence/presence; older omitted columns remain Unknown. Existing shared marker/defaultness/publication fences still run. |
| `DefaultModifierColumns` | Complete parallel function, trait and implementation rows. Each `ModifierRecord { declaration: DeclarationRef, site: u32, modifier: NoDefaultModifier }` binds the exact header; an implementation uses its existing actual `ImplHeader.self_definition` coordinate. The no-modifier value is explicit, never inferred from a missing row. It says ONLY that no `default` modifier was written; it neither supplies effective specialization finality nor forbids an ordinary impl from overriding a provided trait method. No specialization/default graph is admitted. |

Produce modifier records from the actual authenticated local owner and C's typed
`active_defaultness` query during the existing selected collector walk, including
real impl-owner scopes. Preserve the whole-package default impl/associated-value
gates. Const/type declarations outside this profile remain unsupported rather
than acquiring an empty/final record. Decoder validation must check complete
coverage, exact row kind/order/owner/site, bounds and aggregate bytes before
publishing the immutable columns; malformed, duplicate, missing or swapped rows
fail. No NodeKey, TypeId, RegionId, Environment or paid proof goes on wire.

The descriptor may expose a borrowed `provided_contracts()` view over these
validated data columns. Actual namespace installation, metadata-contract queries
and modifier consumers must reuse their current Catalog/owner/site validators.
Update `metadata_contracts::prepare`/Catalog profile routing and `column_storage`
explicitly; a new profile must not accidentally bypass their ordinary admission.
Include a genuine ordinary override of a provided trait method as a control.
Keep the four public Provided guards closed while this data codec is qualified.
A's installed owner, demanded original/instance and runtime checks must consume
the reloaded records before the explicit public constructor/driver gate opens.
Other untransported semantic policies, including UB binding, stay Unknown or
unsupported when needed; this profile must not imply their absence. Explicit
`final` trait methods are a separate policy: `NoDefaultModifier` says nothing
about a final modifier. Preserve their selected-header transport fence until the
genuine final-method producer and complete imported policy are transported.

## Required implementation path

| Boundary | Required result |
| --- | --- |
| Actual inherited inputs | Use the genuine non-role trait producer and one numbered source collector. Preserve real Self/V/U ownership, unused/inherited formals, canonical/declared/full clause boundaries and raw original numbering. No manufactured language role or patched rows. |
| Complete header contracts | Collect actual header columns once before finalizing sites. Retain them in a distinct unpaid owner and include their capacities and enclosing storage in validation peaks. Unknown imported policy cannot become KnownAbsent. |
| Selected nonzero domain | Seal one owned allocation domain to actual Catalog/inventory/arena/body, signature pair, binders and BodyUnit. Reuse the existing inverse and semantic issuance kernels; no cloned SourceRows, fake NodeKey or semantic IDs computed from wire slots. |
| Regional Environment | Consume actual allocation order and normalization intervals, complete bounds and defining assumptions. Preserve Bound/Variable/Parameter distinctions, nested units and caller-local proof suffixes. An empty alias map is not proof that aliases are unnecessary. |
| Budgeted shared kernel | Prepay selected region-equality/query/bounds work and capacities, observe actual attempted work including failures, and preserve ordinary construction order. No whole-Store bound or repeated whole-catalog replay on warm demand. |
| Complete family and body demand | Retain every actual default body and contract. When demanded, construct clone_from's genuine ConstTraitDefault environment and original proof, including MaybeConst Self/Destruct, or the implementation's actual Contract context. Unrequested bodies stay Pending and cannot grant permission. Do not remove bodies to reach Holder; complete default/const execution remains required. |
| Original and Required | Rebuild immutable original IR and raw Contract in the checked domain, retaining operation, formation, builtin, Copy, obligation, normalization, layout, region and loan debts. Preserve genuine ImportedBodyId origins. |
| Fresh implementation instance | Select actual trait/impl/member conformance and the provided body, substitute complete Self/V/U/lifetimes, and prove each demanded instance afresh. No caller-proof reuse or paid proof on wire. |
| Supported public contract | Review the actual advisory/defaultness/UB and other semantic contracts and an explicit supported profile/codec/driver extension. Old v0/v1 bytes remain exact; missing old fields grant no policy. |
| Acceptance | Actual artifact → fresh main/instances/proofs, the single native `7\n` transaction, negative controls and selected cold/warm/failure scaling all pass. |

The numbered ordinary-trait collector and shared mapping are qualified private
prerequisites. Current work joins complete checked declarations/member conformance,
genuine imported namespace installation and demanded original/instance execution.
The next acceptance remains actual encoded artifact and named source execution.
Its gate must include the actually qualified selected-header defaultness check
and whole-package default impl/value gate. An ordinary trait provided body has
no `default` keyword; these are distinct language facts.

The shared Stage constructor and export modes are coordinated with the
module/domain/advisory work. Nonzero allocation APIs and any public Provided
schema/admission changes require their own concrete representation review.
All four current public Provided guards remain closed in intermediate stages.

Measure actual enclosing owners, dense indexes, copied formal/clause vectors,
scratch coexistence and attempted work. Use real widths/unrelated bodies and
the actual Self → trait → member ancestry; do not fabricate deeper chains.
Private readiness and preserved bytes are progress toward this milestone,
not substitutes for its execution evidence.

Named/captured/quantified lifetimes, HRTBs, general alias and type-alias
transport remain additional assignment requirements. Passing this elided-borrow
milestone would not complete PA29 or build original core/alloc/std by itself.
