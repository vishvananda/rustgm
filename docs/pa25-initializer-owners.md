# Initializer ownership beside root closures

The existing borrowed-result constant closure control exposed two semantic
owners sharing the closure expression's source. Its closure contract retained
one universal lifetime, while the enclosing initializer contract had none.
The anonymous execution provider consequently consumed child aliases under
the initializer contract and rejected a valid function-pointer constant with
`invalid lifetime constraint identity`.

Declaration registration now retains an initializer's semantic source separately
from its executable value root. A root closure, including transparent grouped
or attributed wrappers, uses the initializer declaration as its enclosing owner;
the closure keeps its own expression identity. Real blocks and ordinary values
retain their existing owner. The wrapper walk consumes parsed syntax once and
is bounded by that syntax arena. No child aliases are discarded.

Initializer inference, value-selection scopes, incremental and batch region
contracts, and CFG construction use this retained owner. The checked provider
routes the separated root through the existing `Target::Initializer` identity,
checks const eligibility and the ordinary destruction/borrow/region proofs,
and retains the actual declaration body for later concrete preparation. Reuse
validates its declaration, semantic source and solved result type. Retained
body/map storage is charged before publication under the existing report budget.
Associated and generic initializer execution remain explicit boundaries.

The original borrowed-result control is unchanged. Nine new independently
object-qualified outcomes cover ordinary values, root/grouped/attributed
closures, closures inside ordinary and const blocks, static roots, dangling
locals and escaping input references. Seven accepted programs also execute
through the pinned CPPGM backend. Semantic invariants check distinct owner
contracts and full ordinary region proofs. The native test is explicitly
ignored in the backend-free Cargo lane. Draft PA24 fixtures and their manifest
retain earliest ownership; no references were generated.

Qualification passed all seven closure-pointer test groups, two suspended
storage/forward-readiness invariants, two initializer validation tests and three
required-value tests. The latter preserve linear initializer dispatch, memoized
callees, one function CFG across early/late/repeated demands and append-only
preparation at 8/32/128 roots. The two unrelated native tests in these final
invariant suites remain ignored; the affected closure native groups ran
explicitly with `--include-ignored`.

Strict release Clippy across all targets/features, the 1,358-file architecture
audit, layout and upstream checks pass. An initial Clippy-only conditional-style
failure is preserved as `clippy-first-failure.log`; it was corrected before the
final checks. Evidence is under `obj/parallel/borrowed-result/`, including
`closure-pointers.log`, `execution-storage.log`, `initializer-validation.log`,
`required-values.log`, `clippy.log`, `audit.log`, `layout.log` and `upstream.log`.
The source digest is
`e99da7c09960dbbf94eb48bace24f04cc5bfdea628d0f2a74980aec99404be6e`,
computed from the canonical compact JSON path-to-SHA256 map of Rust sources in
`dev/src` and `dev/bin`, retained in `source-sha256.json`.

This is an isolated correction for the frozen publication candidate. Full
combined release/native/source qualification remains the integrator's gate.
Anonymous specialization work is separately preserved and is not part of this
checkpoint. PA25 remains incomplete.

The standalone qualification test subsequently switched from the common
metadata helper to an explicit pinned full-program object helper. All nine
original fixture bytes pass that stronger outcome check; the seven accepted
programs already had independent native compilation. This makes the
object/metadata distinction explicit rather than relying on metadata rejection
for the two borrow errors. Follow-up evidence:
`obj/parallel/pa25-associated/owner-objects.log`.
