# Source type-reflection declarations

RustGM now retains the actual `type_info` and `type_id` struct declarations in
the existing source role catalog. This clears LIB-125's original source-admission
failure. Intrinsic reflection, general library publication and imported consumers
remain required; this is not a core artifact or a whole PA release.

## Declaration and operation boundary

Pinned Rust gives both roles `Target::Struct` and `GenericRequirement::None`.
Accept actual unit, tuple, generic and lifetime-bearing structures without
inventing field or arity restrictions. Duplicate/wrong-target/feature claims
still fail. Completion consumes the actual source owner and completed nominal
definition, using the existing paid cold ledger. Plain matching names and
cfg-removed claims confer no identity.

The two roles append at85 and86, preserving every earlier ordinal. Measured
Catalog1680 and Reservations2648 include their actual padding; provider1392 and
imported1464 keep the existing256-byte bound. Body456, syntax Node36 and NodeFacts80
remain unchanged. Existing exact record/count guards retain these measured sizes.

A completed role does not authorize `type_id`/`type_of` or reinterpret scalar
bits as a trusted type. The original TypeId contains pointer-sized storage with
CTFE type provenance and a runtime type hash. Genuine producers must preserve
that identity across canonicalization and imported remapping, reject forged
values and consume actual reflected field/kind/offset data. Old metadata profiles
explicitly reject reflection-role transport until its full contract exists.

## Evidence

The owning [packet](../pa31/tests/spec/type-reflection/README.md) freezes27 source
outcomes. Pinned Rust agrees with all v3 predictions and executes its C-linked
ordinary field observer with stdout `7\n`. Earlier setup failures remain retained:
the original minimal no_core setup lacked Copy support; its successor lacked
usize marker impls under min_adt_const_params. No prediction was changed to hide
either failure.

Private fastcheck1793744198137878706 passes14 exact tests in126.90s; production
fastcheck1793744367822938621 passes19 smoke tests in121.95s. Both include strict
Clippy, formatting and architecture checks. The private controls exercise all
source outcomes, actual field-body lowering, duplicate publication, exhausted
payments and feature/intrinsic/transport boundaries. Incorrect initial guessed
padding and test-context prerequisites remain recorded as failed checks.

The same immutable CLI passes29 source observations. The previous published
5d9a9ce5 compiler still rejects the declaration. The actual private lowerer's
ordinary field program executes through the pinned CPPGM object backend and C
observer with stdout `7\n`. CLI library output retains the existing metadata
profile rejection; this is an explicit remaining artifact gap. The failed
executable-without-main and library-publication attempts remain retained.
Upstream integrity and harness checks also pass.

Joined receipt: `obj/parallel/type-reflection-v1/qualification-v3/receipt.json`,
SHA-256 `4eeeb207e928e5af3d353395f877c36b0a838f3775e0f08bd2efc815ebb13799`.
CLI SHA-256:
`18c64d2603c988d75acae89a112e3b7fd7863c556a0e133452ee284b6ac0498f`.

## Current original-core frontier

That CLI clears type_info and reaches LIB-126, `#[lang = "compiler_move"]` in
`library/core/src/profiling.rs:12` (file62 byte518). The fresh run takes18.12s,
opens375 sources and verifies all722 input files and six configuration hashes.
Target, options, lint cap and finite source-contract limits remain unchanged.
This is a new blocker observation, not successful compilation of those files.

A static inventory of the opened raw sources finds100 more unrecognized role
names, all mapped to pinned target/generic contracts. This is not a count of
cfg-active blockers. Batch common declaration target/arity validation where the
primary rules agree; retain explicit typed semantics for every trait, intrinsic,
projection and operation. The profiling markers need actual function/arity,
ordinary-body and metadata/debug policy controls. PA29 nominal/value transport
and actual reflection consumers remain necessary for real library artifacts.

The previous17b954034 CI failed on a production wildcard import despite the
private kernel checks passing. Import repair8d4005d84 passes production smoke
and push CI41727937954. Full qualification of this successor is tracked separately.
