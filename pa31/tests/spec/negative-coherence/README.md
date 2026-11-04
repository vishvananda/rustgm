# Explicit negative coherence

The active predictions are `contract-v6.json` and `cases-v6.tsv`: 41 authored
programs in Rust 2021 and 2024, pinned Rust 1.98.0, Linux x86-64, panic abort
and an explicit Allow lint cap. Every matrix source is checksum-bound. The native
program's independent C observer must print `7\n9\n`, selecting the actual
positive blanket and concrete implementations.

PA20 owns trait solving and coherence. PA31 owns the unstable source requests
and internal attribute; PA29 owns authenticated policy transport. This packet
does not extend PA20's published stable language profile or complete those PAs.

`with_negative_coherence` adds proven explicit negatives to ordinary structural
and compatible-world disjointness. An actual `rustc_strict_coherence` type or
trait instead requires explicit negative evidence: missing implementations are
insufficient. The marker needs both the genuine crate request and rustc_attrs
authority, has no arguments, and rejects wrong targets and duplicates even with
lint capping. Macro allowances cannot replace the crate's coherence request.

Unify the competing headers before examining their predicates. Still-unknown
type and const inputs are rigid representatives of the common intersection;
a negative implementation for one concrete type cannot specialize an unknown.
Preserve correlated trait arguments, reference mutability, Sized coverage,
supertrait predicates and the complete lifetime proof. Opposing static-lifetime
cases distinguish paid header premises from an unjustified static requirement.
Strict inherent method checks use the same typed solver.

Negative impls need negative_impls, deny the application and cannot provide
positive members, associated results or executable evidence. They may omit the
positive trait's members and supertraits. The pinned rustc wfcheck deliberately
does not run positive impl WF checks for a negative impl; it even accepts the
negative-argument-bound case. Ordinary non-auto negatives may be conditional or
specialized. Auto negatives retain their separate applicability restrictions.

Earlier prediction versions remain for review. Independent Rust rejected V1's
predictions that wrong targets and duplicates were capped warnings; V2 corrects
them to hard errors. V3 incorrectly required positive trait-argument WF for a
negative impl; V4 corrects that outcome. V5 and V6 add lifetime/Sized coverage and
isolate the ordinary stable versus negative-coherence modes. Original source
bytes and failed-oracle receipts are retained; no RustGM failure was reclassified
as an expected positive gap.

Conditional refutations remain distinct from absence and positive proof. The
declaration-only continuation must pay retained lifetime and Sized debts from
the actual intersection. The ordinary-use rejection guard remains in place.
Quantified conditional negatives, wider recursive interactions, authenticated
imported strict owners and whole original library artifacts remain required.
Existing metadata profiles explicitly reject strict policy they cannot encode.

`required-macro-contract.json` records a separate required positive gap. Independent
Rust accepts a macro-generated negative impl using negative_impls allowance;
RustGM currently requires the crate request and rejects that program in both
editions. Forwarding a captured impl is independently rejected by both compilers.
This syntax gate must eventually consume the actual `!` through trait-path span,
not the enclosing impl keyword or a general macro permission. These observations
are not included in the 41 matching cases and are not Rust source rejections.

Primary sources are pinned revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`:
`compiler/rustc_middle/src/traits/specialization_graph.rs`,
`compiler/rustc_trait_selection/src/traits/coherence.rs`,
`compiler/rustc_hir_analysis/src/check/{wfcheck,always_applicable,check}.rs`, and
`compiler/rustc_attr_parsing/src/attributes/rustc_internal.rs`.
