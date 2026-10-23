# PA34 explicit pattern modifiers

The typed frontend now applies the Rust 2021 rules for explicit binding and
reference patterns when their parsed introducer belongs to that edition. This
checkpoint does not enable a Rust 2021 driver flag or activate PA34. Imported
macros and the remaining crate-graph edition semantics still require completion.

For a non-move default binding mode, Rust 2024 rejects explicit `mut`, `ref`,
`ref mut` and reference patterns. Rust 2021 permits them subject to the ordinary
type, mutability, move and loan checks. Explicit `mut` resets that binding to move;
an explicit reference pattern resets its child default mode. An `@` subpattern
retains its own incoming mode. Allowing legacy syntax supplies no extra permission
to move a non-Copy value out of a shared reference.

Semantic analysis consumes the existing parsed pattern flags and node span.
Only an edition-sensitive modifier demands provenance. Its parsed ASCII
introducer selects the first byte's existing origin range, continuing through
the bounded parent-origin walk. The remainder of a pattern may contain captured
syntax from another edition and does not need one uniform origin. This adds no
ordinary AST/token/semantic field, cache, source rescan or rendered-text parse.
The private ancestry control requires exactly two lookups for its mixed-origin
example and rejects malformed/empty spans.

## Focused qualification

`pa34/tests/pattern-edition-outcomes.json` declares 26 paired source outcomes
over 13 original fixtures before execution. Pinned Rust 1.98.0 independently
agrees with every outcome. All accepted cases also execute through both rustc
and CPPGM: 13 native comparisons check binding values, mutation and reference
depth. The negative controls retain shared-mutability and non-Copy move rejection.
Macro-generated, opaque-forwarded, shorthand, tuple and `@` cases are included.

Evidence is retained in the root implementation checkout under
`obj/parallel/pa34-pattern-editions/`. The initial `development-v1` passed strict
all-target/all-feature Clippy, formatting, architecture, harness/upstream and
20 focused tests. Native qualification then found a fixture helper calling an
unavailable libc `exit` symbol. The failure remains recorded; no compiler or
backend rule was changed to hide it.

`development-v2` replaces only the authored fixture helper with the existing
portable checked-division assertion pattern. It preserves the old inputs and
declarations, predeclares unchanged outcomes for the new bytes, and checks that
the source hash map differs only in those fixtures and their manifest. The exact
existing test executable passes the revised semantic and native cases without
Cargo code generation. Harness/upstream checks and all 8,382 active byte-exact
text cases also pass. Static results and 19 unaffected focused cases remain tied
to their unchanged sources and binary hashes. Cleanup removes native artifacts
while retaining inputs, diagnostics, receipts and the current candidate pin.

Combined release/native and performance qualification remains required before
publishing this compiler checkpoint. Full source-edition support also requires
the [external declaration rules](pa34-extern-editions.md), temporary scopes,
opaque captures, prelude/method selection and never-type fallback as demanded
by the frozen acceptance graph.
