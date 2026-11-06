# Actual Reborrow operations and original core advancement

Reborrow and CoerceShared now use their actual source declarations, ordinary
implementation/Copy obligations, feature authority and selected lifetimes.
Implementations validate the real struct, owner binder and data fields; only
genuine PhantomData is excluded. Unused alternate trait headers remain ordinary
Rust declarations. Coherence's permissive unused lifetime cases do not grant an
operation permission or erase the eventual caller's lifetime obligations.

A contextual coercion must know its nominal destination when checking the
expression. Later inference cannot retrospectively replace a move with a loan.
Transparent blocks and projected operands preserve the actual source place and
evaluate effects once. The selected destination itself supplies the loan type;
there is no extra lifetime, argument-vector rebuild or redundant IR conversion.
The independently retained Rust MIR confirms two exclusive and one shared loan
of the local wrapper, with no constructor-initializer reborrow.

Sparse semantic selections and cold CFG inputs bind actual source, body,
instance, type store, operation, place and endpoints. Template copying and type
remapping preserve those claims, with a constant-time context check before
mutation. Const execution owns a bounded immutable footer carrying the same
claims; native lowering copies checked physical storage. Neither path calls
Clone implicitly. Ordinary initialization, unsafe access, loan conflict, storage,
match-guard and promotion rules remain active. Pending goals are released at
each inference reset; this prevents stale indices crossing bodies and avoids
retaining an oversized rare table for subsequent bodies.

The [source-owner repair](reborrow-source-owner-repair.md) keeps this same
sparse semantic map under the existing optional source catalog, restoring the
ordinary Declarations7120 layout. It separately qualifies current source/native
behavior and compile-time width opponents after the full CI size failure.

Seven exact public tests across two targets pass in 115.23 seconds, including
ordinary loan and CStr controls. The Reborrow cohort has seven positive source
cases and twenty independently qualified ordinary rejections. Two real native
observers pass in 34.66 seconds and print `reborrow 23 23 47 47 0 99 31` and
`successor 17 23 19 31 23 59 1`. They verify actual writes, mixed generic fields,
implicit copies, explicit Clone effects, const evaluation and a projected source
evaluated once. All selected strict Clippy, formatting and architecture checks
pass. The subsequent harness/upstream checks also pass.

Immutable CLI `9aa812d6` advances unchanged original core past Reborrow to
**LIB-186, FieldRepresentingType**, `library/core/src/field.rs:11`, file79 byte252.
It exits1 after 24.067 seconds and 375 opens. All722 original input hashes, cfg
values, target/options, capAllow and finite budgets remain unchanged. Five
configuration-source pins stay identical; the sixth is explicitly rebound for
the already published typed slice repair and new authenticated value dispatch.
This is an observed declaration frontier, not a built core artifact.

Forty immutable ordinary A/A+ABBA observations preserve exact output. Median
candidate/baseline ratios are0.981 at512 functions and1.062 at2048; the A/A ranges
are0.126–0.166 and0.450–0.615 seconds. These noisy controls establish neither a
broad speedup nor absence of regressions. There is no common pre-feature baseline
for the new operations. Input validation avoids a statement scan for every claim;
field-declaration width per operation remains a named scaling dimension.

[Machine evidence](reborrow-operations-checkpoint.json) binds exact receipts,
inputs, CLI, profile rebind and every observation. The original and successor
reference packets retain setup failures and disproved predictions. Demanded
nested/zero no_core cases that ICE in pinned Rust remain unqualified and were
never executed or treated as semantic negatives. Full private/debug/release CI,
wider operations/Drop/const/closure/derive profiles and authenticated transport
remain required. No whole PA after24 or original core/alloc/std artifact is
completed. Keep the existing btrfs trees, two-heavy/16GiB limits,150GiB reserve
and five-minute local guards.
