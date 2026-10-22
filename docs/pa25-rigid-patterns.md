# Rigid caller parameters in constant patterns

The unchanged `open-type-initializer-context/generic-pattern.rs` now computes
`S::<T>::C` when the original caller owns `T` and the complete typed executable
footprint is independent of it. The new `rigid-pattern-context/` controls include
actual u8/u16 instantiations, Copy bounds, multiple parameters, and lifetime/type
combinations. This is a focused implementation checkpoint, not PA25 activation.

The existing source-computation caller row retains a lazy immutable parameter
index. Its constructor checks the actual original declaration/body/signature,
full parameter identities, and exact completed main/late publication ranges.
The index is shared on warm use and remains bound to that catalog epoch and full
caller key. Foreign parameters, sources, catalogs and stale publications reject.
Parameter membership grants no trait, well-formedness or outlives assumption.

Only direct owned rigid type arguments are admitted by this first extension.
Ordinary Associated admission remains concrete. The source instance retains its
actual parameters in the ordinary Contract, original and instance proof paths,
paid-premise checks, eligibility, escape checks and source-only publication.
Before execution, a bounded allocation-free pass checks every local, projection,
value and callee/dependency type, including skipped values. Parameter-dependent
storage, unresolved constants and generic inline dependencies reject. The
argument-shape pass retains its rigid classification rather than rescanning it;
its argument visits are charged. No new Target, Requirement, Rvalue or wire
permission is introduced.

## Debt and inspection boundary

The independently declared unpaid `T: 'a` control remains Rust E0309. Its original
all-view-rejection hypothesis and failed assertion are retained in the evidence.
PA19's CLI inspection contract and PA20's preservation of pending lifetime facts
instead require types inspection to retain the debt while constants inspection
and LowIR reject it. Existing selected-initializer lifetime controls use this
same stage distinction. No published reference is changed here.

The new paired private invariant checks the actual caller's TypeOutlives
obligation for its registered T and follows only retained Equality obligations
to the declared lifetime slot. The normal body region proof succeeds with the
real declared T:'a premise and fails without it. This goal is an ordinary
Contract obligation, not a root or quantified LifetimeProof type-vector row;
the earlier incorrect storage-location assertions remain in the failed logs.

## Qualification and retained costs

`obj/parallel/pa25-generic-pattern/final-fourth/` retains strict all-target,
all-feature release Clippy, the 1,630-file architecture audit with zero findings,
four rigid private controls plus the catalog/layout control, source outcomes and
five actual native programs. The unchanged original plus five new positives
accept in all three views. Six negatives retain their declared Rust and checked
outcomes; the unpaid lifetime control has the stage distinction above. Prior
paid-context and exact legacy independent-selection/lookup smoke tests also
passed in `close-third/` before allocation/accounting-only hardening.

At widths 1/8/32/128 the retained parameter context charges
116/256/736/2,656 bytes. Warm context preparation visits N+2 and selected-parameter
extraction visits N+1. Whole-footprint traversal has a one-million-visit limit
and no retained allocation. The cold caller row grows from 56 to 64 bytes;
Catalog 216, witness 8, Target 28, Request 56 and Code 336 bytes are unchanged.
Closed/direct-lifetime routes do not allocate a rigid parameter index.

The immutable debug-profile functional pin is
`obj/parallel/pa25-generic-pattern/final-fourth/pins/rustgm`, SHA-256
`c253f74047334afc60a4b1729296ff2309476935a705f6915f86bf2af9a4496e`.
The 934-file production source map hashes to
`1ff40d15166234f833d65ae0fc3a8b2c6cad9fe3445207ad0f5bff630a194f31`.
This pin is not performance evidence.

Current-public 907dcd0c fixtures were archived without rebasing this worker.
The all-active PA1–24 tail ran 8,362 comparisons: 8,266 exact reference matches,
8,334 matching declared statuses, and 96 differences byte-identical to the
accepted a873 baseline pin. All source/output hashes and status differences are
retained in `text-current-public/review.json`; they are not grandfathered as
passing current-public checks. Central integration/qualification remains
mandatory. Upstream verification passed. No broad worker Cargo or timing run was
added; the integrator owns combined release and immutable performance checks.

## Still required

`unused-const.rs` and `callee-independent.rs` remain independently Rust-positive
required frontiers. The former needs exact original const-parameter context;
the latter needs a checked source-only generic callee route, not an ordinary
callable permit or erased Instance key. Generic inline dependencies, composite
rigid argument shapes and conditional closure/inline parent outputs remain
required followups. The full assignment responsibility and release prerequisites
in `pa25-completion-gaps.md` remain unchanged.
