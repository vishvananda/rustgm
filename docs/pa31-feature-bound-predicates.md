# Required feature predicates

LIB-060 originated at the unchanged-core boundary
`mem/transmutability.rs:86`: `unstable_feature_bound(transmutability)` belongs
to the real unsafe trait and imposes a non-type feature requirement. Genuine
Trait/Fn/impl source capture and paid clauses now pass that loading boundary.
Accepting syntax without retaining and proving the requirement is insufficient.
The [joined source checkpoint](pa31-source-feature-integration.md) records
local/native proof and remaining wire requirements.

The unpublished contract in
`pa31/tests/spec/feature-bound-predicates/contract.json` freezes 26 local and
seven genuine external-client inputs. Its metadata qualification joins 78
source/profile outcomes and two external-provider builds from pinned Rust 1.98,
editions 2021/2024. The two unchanged local native fixtures independently return
`7` under pinned Rust. The joined candidate also returns `7` for both unchanged
whole-source fixtures through LowIR and the native backend. The initial concrete
Dispatcher gap and its qualified repair are retained in the joined checkpoint.
Cross-crate feature transport remains pending. This is neither an implemented
assignment nor a core artifact.

Observed semantics include matching function/impl/callee scope, closure
inheritance, nested-function isolation, reification and prior generic trait-bound
formation. Merely assuming `T: Trait` does not discharge the trait's feature
formation requirement. Staged global flags cannot replace an item's predicate
assumptions. Ordinary external clients can enable a predicate globally, but still
need separate regular API permission. Predicate assumptions do not supply that
API permission. A diagnostic cap silences neither obligation.

Two wrong hypotheses are preserved: the parser rejects word-only form despite
the broad template, and an unmarked generic forwarder is invalid even when its
caller has the marker. The first external-client harness also used an invalid
provider filename; every external row from that attempt, including coincidentally
matching negative statuses, is excluded. The corrected lib-prefixed provider
passes all 20 independent provider/client operations. Exact joins and raw-receipt
hashes are in `metadata-qualification.json`.

Implementation uses an actual non-type clause, Store-owned canonical feature
identity, paid shared cold assumptions and immutable enabling policy. Source
ownership and requirements must survive declaration formation, calls, impl
selection, reification and native instance proof. Old metadata profiles must
reject unsupported requirements. Keep dense body/node records unchanged and
avoid copying global feature sets per body or scanning source owners per use.

The semantic text view represents this non-type clause as
`predicate owner=... source=... feature="feature_bound_probe"`. It has no
`subject=` field. The quoted spelling is the canonical name in the actual Store;
an identity from another Store fails instead of exposing a nonce or name slot.

Root owns source production, actual caller/environment parenting, enabling-policy
publication and transport. The typed-core worker owns clauses, canonical IDs,
borrowed indexes and proof consumers. Whole assignment and original core/alloc/std
qualification remain open.
