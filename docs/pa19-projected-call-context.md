# Projected call context: regression correction

Argument-prefix inference must not require a projection's normal form merely to
classify an optional callable hint or a two-phase receiver. The actual coercion
still retains its deferred shape equation, normalization and lifetime debts.

The correction uses the existing normalization-tolerant contextual head for
optional result, actual-item and mutable-reference classification. Prefix
FunctionItem inference is attempted only for a genuinely unknown expected head,
not an unresolved associated projection. It adds no storage, enum, conversion,
proof authority, delayed coercion or reborrow.

Two existing maintainer positives exposed the regression after the argument
worklist change: `call_operators`'s `multiple-call-inputs` and
`method_contracts`'s `ordinary-input-infers-self`. The immutable 89d3ec1b
executable rejects both with `associated type projection requires
normalization`. Pinned Rust 1.98.0 accepts both. The neighboring
`result-does-not-infer-self` rejection remains required.

The generic overloaded Fn case has an empty main and no exercised concrete
callable. Its owning PA24 proposal is codegen-only, not evidence that the
otherwise valid generic body executed. The ordinary-input case belongs to PA19;
a separate native control checks the selected result value.

## Opposing reference controls

The retained v3 consumption matrix distinguishes genuinely unknown projected
inputs from already normalized targets. Rust moves a bare mutable-reference
argument when its projected target is not yet determined. Real later reads in
the same call or after it reject, including group, block and if-tail forms. An
explicitly typed inner reference binding performs its own valid reborrow.
A known shared-reference target permits mutable-to-shared coercion; an unknown
projection that later becomes shared rejects the unequal inferred shape.
Explicit conflicting borrows and later writes remain invalid.

These observations rule out an initially considered delayed conversion or
identity reborrow. Neither was implemented. Preserving the existing source-time
shape/move distinction is essential to this correction.

Earlier v1/v2 hypotheses used `let _ = *r` as if it consumed the referent; Rust
accepts that initialization-only wildcard even after moving r. They are
superseded by v3's actual reads and remain in the evidence directory. The
frontend's rejection of those wildcard programs is a separate required Mention
boundary, not repaired or counted as passing by this checkpoint.

## Evidence and publication

Worker evidence is under `obj/parallel/pa19-prefix-projections/`, including the
immutable baseline hash, initial and corrected hypotheses, pinned Rust
observations, the intentionally interrupted first harness run and the failed
wildcard expectation. Source-only owning proposals retain exact original
maintainer bytes, explicit `--emit-lowir`, correct native metadata and no text
oracles. The integrator owns their final oracle review and publication.

The sequence-comparison draft is parked separately. Array/slice PartialEq/Eq,
PartialOrd/Ord, concrete str and the remaining PA26 hook families stay required.

The closing worker gates pass both complete affected maintainer roots and their
native groups, all 48 associated-declaration source functions (including the
41-case argument-prefix matrix), all eight call-inference source functions,
the 14 returned-item native programs, four new value-check native programs and
the argument-prefix native group. Five prefix and four notification invariants
pass, retaining the compact record assertions. Strict all-target/all-feature
release Clippy, architecture audit (1598 files, zero findings), assignment
layout and 337-root/13-binary registration pass. A duplicated native helper and
a seventh repeated filename stem were test-only closing failures; their
receipts are retained, and only the affected checks were repeated after repair.
Production changed in two files, with no record or enum changes. The integrator
still owns the combined active-text and full release qualification.
