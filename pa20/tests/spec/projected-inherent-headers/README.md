# Projected inherent headers (PA20)

Reduces pinned core adapters such as `MapWhile::try_fold`, whose
`.into_try()` receiver selects `impl<R: Try> ControlFlow<R, R::Output>`. The
implementation header contains a projection. Method applicability normalizes it
as rustc's probe does: each associated-result leaf becomes a projection-equality
condition, proved under the caller's assumptions. The selected owner header then
relates through the body's normalizer, never structurally.

Seven executable positives check returned values: generic, bound, concrete,
explicit-path, inferred-path (a baseline control), call-result and by-reference
receivers. A concrete output mismatch and a missing `Try` implementation reject
because no inherent method applies. `pending.json` keeps an unbound-output
rejection whose RustGM cause is still the generic normalization failure. See
[the checkpoint](../../../../docs/pa20-projected-inherent-headers.md).
