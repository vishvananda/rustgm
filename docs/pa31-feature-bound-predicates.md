# Required feature predicates

LIB-060 is the actual unchanged-core boundary at
`mem/transmutability.rs:86`: `unstable_feature_bound(transmutability)` belongs
to the real unsafe trait, while the current consumer supports only impl lint
suppression. This attribute also imposes a non-type feature requirement. Accepting
the trait's syntax without retaining and proving that requirement is insufficient.

The unpublished contract in
`pa31/tests/spec/feature-bound-predicates/contract.json` freezes 26 local and
seven genuine external-client inputs. Its metadata qualification joins 78
source/profile outcomes and two external-provider builds from pinned Rust 1.98,
editions 2021/2024. Native `7` predictions remain unqualified until actual
execution. This is neither an implemented assignment nor a core artifact.

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

Root owns source production, actual caller/environment parenting, enabling-policy
publication and transport. The typed-core worker owns clauses, canonical IDs,
borrowed indexes and proof consumers. Whole assignment and original core/alloc/std
qualification remain open.
