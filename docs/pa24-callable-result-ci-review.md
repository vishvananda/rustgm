# Callable result CI reference review

Full CI42948079201 on publication2d94 completed all3,065 private tests and all
regular integration binaries successfully. The release/course step then failed
13 PA20/PA21 text comparisons. Native, debug and independent successor stages
were skipped; this run does not provide their qualification.

The [callable result contract](pa24-callable-results.md), compiler31cfbe08b /
immutableCLI8b73f15a, progresses each call's own output requirements before its
structural consumer. Existing PA20 and PA21 reductions already own all thirteen
affected contracts; no new fixture, language outcome or implementation change is
needed for this review.

Eleven accepted semantic inspections record the resulting earlier constraints:
seven add a reflexive region equality; one adds a universal alias and its equality;
two add an outlives requirement alongside a reflexive equality; and one allocates
the quantified Copy witness earlier. The last case preserves all three scoped
proof blocks, their parent scope and the witness-to-bound relation under the new
region numbering. No proof block or obligation disappears. Two existing invalid
calls now diagnose incompatible concrete results before their former projection
or trait-selection diagnostic. Their sources and rejection statuses are unchanged.

Before generation, `obj/parallel/callable-ci/expected-outcomes.json` froze all13
source hashes, existing manifest contracts, anticipated changes and7,293 old
sidecar hashes. Independent Rust object compilation agrees with all thirteen
outcomes. The eleven accepted sources reach complete RustGM LowIR and execute
with both pinned Rust and the locked CPPGM backend, status0 and empty output.
The two negative Rust diagnostics are E0308 type mismatches.

Exact generation through the existing course runner updates only11 semantic
stdout sidecars and2 diagnostic sidecars. The other7,280 PA20/PA21 sidecars,
every fixture input and every declared outcome stay byte-identical. Scoped
generation receipts are committed with these references. Full1,564 PA20 and867
PA21 text comparisons pass guarded1794367453687235048; paired independent/native
qualification passes1794367199157806503. No compiler rebuild, backend change,
new original-core observation or artifact claim is part of this repair.

The first pre-generation driver compared full local diffs with CI's forty-line
display excerpts and stopped before writing any oracle. Its corrected successor
retains full diffs and checks CI's display limit only for that log comparison.
Both scripts and observations remain under `obj/parallel/callable-ci/`; the full
reviewed diff includes the quantified proof blocks hidden by CI's truncation.
