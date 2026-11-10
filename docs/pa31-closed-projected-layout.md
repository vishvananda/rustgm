# Closed projected-field layout

Original core stopped while checking AtomicPtr::from_mut's alignment assertion:
AtomicPtr<T> aliases Atomic<*mut T>, whose stored field contains a projection.
The layout kernel prepared ordinary field templates without running the existing
projected-storage producer. The independently accepted PA31 reductions in
`pa31/tests/spec/closed-projected-layout` keep an aligned projected field and its
direct-field control. The old f5b5db49 CLI rejects only the projected form.

The kernel now uses shared stored-field recipes with a fresh caller preparation
and concrete projection session. It independently discharges retained lifetime
evidence before layout readiness or cache use. Structural recipes preserve raw
inputs and never authorize a new caller. Disposable proof identities come from
BodyRegions::probe_frontier, beyond source-body identities. Missing field
templates keep their existing declaration wait. Unsupported normalization
requirements remain errors; this does not promise a new general continuation API.

Graph, caller-fold and nested work are charged between producer steps, including
failed steps, so later children cannot run after earlier work spends the parent
allowance. A cold exhausted-budget control stops before publishing normalized
projected fields or a layout, and repeated attempts remain budget errors. The
empty-environment proof checker is shared with the existing vtable consumer;
its diagnostic staging and actual lifetime checks are retained.

Private fastcheck 1794276982567407752 passes all 19 focused cases, including
shared/replayed fields, waits, unowned real lifetime debts, vtable consumers and
the new cold-budget control. Production 1794277185114392564 passes three actual
object-lowering controls, strict Clippy, formatting and architecture. Final
source-bound CLI 734c14fea58694f877a7bbd941bf9d4fe93ad91562b1632aa8da89c7367a9806
accepts both frozen alignment assertions. Harness 313/465 roots/44 stages and
128 upstream fixtures/126 Reference files pass. Twenty immutable development
A/A+ABBA observations preserve complete control outputs; cold-start calibration
is noisy, so no timing or optimized-performance conclusion is claimed.

The final unchanged 722-input census opens 375 sources, checks 42,310 declarations
with none failed/blocked and bodies 0, and clears the atomic stop. It reaches
fmt/num.rs byte 2783 (radix_integers expansion), "symbolic slice reference lacks
its normalized range". Census2 exactly matches the first candidate's new TSV,
SHA-256 da6e9262ea7683ae3b03d84d0d4d9d615b8d9c84bd82539f2038b49685705423.
This is discovery, not an original core artifact. Inputs/profile/limits and the
backend are unchanged. Raw evidence remains in pa26-work/obj/parallel/atomic-layout.

Full predecessor CI 42736874497 separately found a hot Evidence/Selection size
regression and three depth 128 receiver-budget regressions in the earlier nested
reuse checkpoint. This atomic correction is kept local while those are repaired;
no full release/native/debug/independent pass is claimed for it. PA31 stays planned.
