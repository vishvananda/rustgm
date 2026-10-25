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

## Required implementation path

| Boundary | Required result |
| --- | --- |
| Actual inherited inputs | Use the genuine non-role trait producer and one numbered source collector. Preserve real Self/V/U ownership, unused/inherited formals, canonical/declared/full clause boundaries and raw original numbering. No manufactured language role or patched rows. |
| Complete header contracts | Collect actual header columns once before finalizing sites. Retain them in a distinct unpaid owner and include their capacities and enclosing storage in validation peaks. Unknown imported policy cannot become KnownAbsent. |
| Selected nonzero domain | Seal one owned allocation domain to actual Catalog/inventory/arena/body, signature pair, binders and BodyUnit. Reuse the existing inverse and semantic issuance kernels; no cloned SourceRows, fake NodeKey or semantic IDs computed from wire slots. |
| Regional Environment | Consume actual allocation order and normalization intervals, complete bounds and defining assumptions. Preserve Bound/Variable/Parameter distinctions, nested units and caller-local proof suffixes. An empty alias map is not proof that aliases are unnecessary. |
| Budgeted shared kernel | Prepay selected region-equality/query/bounds work and capacities, observe actual attempted work including failures, and preserve ordinary construction order. No whole-Store bound or repeated whole-catalog replay on warm demand. |
| Complete default family | Construct the actual ConstTraitDefault environment and original proof for clone_from, including its MaybeConst Self/Destruct context. Do not skip an otherwise required body merely to reach Holder. A demand-only policy would need separate review. |
| Original and Required | Rebuild immutable original IR and raw Contract in the checked domain, retaining operation, formation, builtin, Copy, obligation, normalization, layout, region and loan debts. Preserve genuine ImportedBodyId origins. |
| Fresh implementation instance | Select actual trait/impl/member conformance and the provided body, substitute complete Self/V/U/lifetimes, and prove each demanded instance afresh. No caller-proof reuse or paid proof on wire. |
| Supported public contract | Review the actual advisory/defaultness/UB and other semantic contracts and an explicit supported profile/codec/driver extension. Old v0/v1 bytes remain exact; missing old fields grant no policy. |
| Acceptance | Actual artifact → fresh main/instances/proofs, the single native `7\n` transaction, negative controls and selected cold/warm/failure scaling all pass. |

The immediate approved step is the genuine numbered ordinary-trait collector
and its owned positive header sidecar. It may land as a private prerequisite.
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
