# Normalize imported closure inputs before binding parameter patterns

Compiler adfb10e1c repairs closure entry ordering. A reference pattern or explicit
parameter annotation inspects its imported type before the first body expression.
Register and advance the imported signature projections in that child's existing
proof environment before binding its parameters. Reuse the original normalizer,
annotation equations, pattern and lifetime/ownership checks. No field, cache,
queue, compiler quota, backend or proof authority is added.

The [earliest PA24 packet](../pa24/tests/spec/parameter-projections/README.md)
freezes18 outcomes (15 accept,3 reject) and exact hashes before implementation,
independent execution or text generation. Free/generic methods, inherited
projections, generic parents, explicit annotations and tuple/array patterns retain
complete typed lowering. An untyped simple binding is the baseline control.
Wrong mutability, incompatible annotations and moves from shared storage must
reject by those ordinary rules. All18 Rust1.98 object outcomes agree; all15
positive Rust programs execute with their declared CHECK booleans.

[Three PA31 companions](../pa31/tests/spec/parameter-projections/README.md) retain
real Iterator/Option/callable source roles, the actual Item projection and a
provided callback method. Inferred, typed and renamed method forms independently
pass metadata/object compilation and fail frozen baselined2db at the parameter.
Diverging provided bodies claim no native execution. Early setup diagnostics are
retained: next must be a required method, callable source syntax declares the
staged/unboxed features, and Clone carries its actual const API policy. No
frontend policy was relaxed to admit those prerequisites.

Production1794326261492787741 passes9 exact tests: all18 new outcomes, three
source companions, width/noise, all previous16 cast outcomes/four source
companions/scaling, formal closure lifetime ownership, unused-body rejection and
existing complete source-body LowIR. Strict selected/CLI Clippy, full formatting
and architecture pass. Native1794326369748242930 passes39 paired Rust/CPPGM
programs:15 new callback cases,17 existing source-closure bodies and7 deferred-
cast cases. Fourfold widths8/32/128 stay within4x+256 inference/canonical work;
128 unrelated declarations add no inference or canonical work. No private data
structure changed or new private test was added; the preceding cast checkpoint's
cold private execution remains pending in the active full CI run.

Only18 new PA24 references were generated; all527 PA24 text outcomes pass with
existing oracle bytes unchanged. The three rejection oracles report reference
mutability mismatch, incompatible types and a non-Copy move out of a reference.
Harness315/468 roots/13 binaries/44 stages,128 upstream fixtures and126 Reference
files pass. Every new input appears in its earliest owner's manifest; the
integration cases extend the existing source_closures root.

Immutable dev-fast CLIf7be3585e89624c37b372189cd4e156903f99d0b66284bc2a336aacea5e855d0
binds adfb10e1c, an empty patch and3880 compiler/CLI/test Rust hashes. Thirty
A/A+twoABBA observations preserve complete simple-binding/generic-promotion
LowIR and74 ordinary survey outcomes. Candidate/base ratios0.892/0.880/0.976,
A/A0.924/0.925/0.972; no timing claim. Evidence, failed source hypotheses, native
receipts, producers and raw observations are under obj/parallel/callback-inputs.

On unchanged722 original input hashes/profile/quotas, the120s observation FAILED
at121.09s. Its partial stream retains297 failures/4987 item attempts through
function10041, stdarch x86/sse2.rs file277 byte68548, SHA256
ee870aa56a1f4f65a820c2f0b28557fc68fd72146465a8a5246e710569648964.
It moves past round_up; the first failure is now integer_decode, core num/imp/
traits.rs file36 byte6061, missing value resolution. Both earlier first-failure
bodies (parse_long_mantissa and round_up) are reached without their old failure
records. This incomplete prefix differs from the previous307/5967 observation;
raw counts are not an improvement metric or checked-body qualification. No
complete new census or original core/alloc/std artifact exists.

Five independently Rust-positive exploratory PA20 sources reduce the next issue:
T::Int::ZERO misses value resolution, Self::Int::ZERO reports an invalid method
candidate inference graph, while explicit qualified and direct-parameter
controls pass frozenf7be. They are discovery evidence, not a completed fix or
object/native qualification. Freeze their PA20 constant/selection opponents
before editing; evidence is under obj/parallel/projected-constants.

Full predecessor42826374117 passed every stage. Current full42847850912 on751ab7be1
has passed fast and all-target strict lint/architecture/harness and is still in
release/course qualification; private execution and remaining stages are pending.
The new full successor is due only after that run terminates. Do not cancel it.
Optimized nested-proof qualification remains pending. One worker continues in the
same tree, with the unchanged resource and observation limits.
