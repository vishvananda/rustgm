# Scoped receiver and qualified method names

This PA31 corpus declares hypotheses before independent execution. Scoped
invocation remains unsupported in RustGM; an early unsupported error does not
qualify a predicted semantic rejection. The contract is the input manifest.
Preserve any failed hypothesis and record its observation separately.

The cases compare captured and introduced method identifiers under a captured
nominal owner, actual receiver and type-qualified calls, sibling opaque
expansions, captured duplicates, and definition-site versus caller privacy.
Every positive requires metadata, object and native stdout `7`; each negative
requires its declared structured diagnostic code in both language emissions.

Pinned implementation guidance is `rustc_hir_typeck::method::associated_value`,
`method::probe::push_candidate`, `TyCtxt::hygienic_eq` and
`adjust_ident_and_get_scope`. Name comparison and privacy both use the actual
impl or trait container. They are separate from receiver applicability and
implementation readiness. No text oracle or PA activation is produced here.

The first independent run matched nine case predictions: four positive native
programs print `7` and five semantic negatives report the declared error codes.
The two predicted private-method positives were falsified: both pass the earlier
method-visibility decision but the pinned compiler rejects the private function
item type at the physical call site, without a diagnostic code. Their original
sources and positive hypotheses remain unchanged. `review.json` records all
32 executed observations, including both failed native compilations. This later
privacy requirement must be qualified before full scoped execution is admitted.
