# PA34 never-coercion fallback

This edition checkpoint selects never-coercion fallback from the physical
compilation unit: Rust 2021 defaults an unconstrained coerced general root to
`()`, while Rust 2024 defaults it to `!`. Macro token provenance does not choose
this rule. The pinned rustc implementation uses the session edition in
`compiler/rustc_hir_typeck/src/fn_ctxt/mod.rs::default_fallback`.

The existing inference root and dependency wakeup path remain shared. Numeric
variables keep their numeric domain, constrained roots retain their actual type,
and unrelated inference holes remain unknown. No persistent record or auxiliary
cache grows; the existing body-finalization pass applies the selected default.
Already checked imported bodies retain defining inference results. This does
not enable a 2021 driver profile or claim cross-crate macro transport.

`pa34/tests/never-edition-outcomes.json` freezes sixteen paired source outcomes
and twelve local native programs. Trait implementations for `()` and `!` produce
different runtime tags, so acceptance alone cannot hide the wrong default.
Opposing unit-only/never-only bounds, nested closures, local macros, explicitly
constrained types, numeric inference and unrelated unknowns are included.
Four external-macro native controls independently cover both provider editions
against both consumer editions. All observations match pinned Rust 1.98.0;
receipts are under `obj/parallel/pa34-never-editions/independent-v2`. The original
assert-macro fixtures and observations remain under `inputs` and `independent-v1`.

Candidate source/native and inferred-root readiness tests pass focused validation. The
native suite additionally uses retained token origins from a donor in the other
edition, with identical source spans and matched delimiter provenance, while the
physical compiling unit stays unchanged. These are provenance-consumer controls;
actual external macro transport remains required.

The first static attempt stopped at a test helper's private `physical_len` API;
using the original fixture byte length corrected that helper. `development-v2`
then passed the unchanged inference-dependency and temporary-edition suites, but
never-edition sources stopped at the unavailable bootstrap `assert!` macro.
The fixtures now use the existing authored `check`/`fail` runtime observer. All
revised source and native outcomes were independently qualified before candidate
execution. `development-v3` passed formatting, audit, all-target strict Clippy,
harness and upstream checks, then exposed a diagnostic assertion that omitted
`type annotations needed` for the expected ordinary inference-hole rejection.

`recovery-v1` changes only that assertion. Formatting, affected-target strict
Clippy, all sixteen source outcomes and fourteen candidate native executions
pass, including the two opposite-origin controls. The unchanged production,
audit, harness, upstream and independent results are retained separately from
that recovery. The inference-root tests retain their `development-v2` result;
no full-suite pass is claimed here. Combined release and performance checks
remain the next integration checkpoint. No record or cache size changes.

The later full CI run 40377221175 exposed stale test setup: the opposite-edition
helper appended an entire parsed crate without passing its attribute owners
through expansion. Production lint-owner validation correctly rejected it.
The helper now takes the actual macro-definition span, parses those unchanged
bytes through `Unit::parse_include`, and retains that unit only as a provenance
template. Redirected token spans are rebased into that exact fragment; the
compiling crate's feature/lint owners and physical edition remain unchanged.
This still tests the provenance consumer, not imported-macro transport.

An ordinary test now exercises both opposite-edition sources through validation
and LowIR, so this frontend failure does not require native qualification to
discover. Two ordinary test groups pass in 35.13 seconds including strict
Clippy, formatting and architecture checks. The existing native group then
passes all fourteen executions in a cached 27.30-second batch, including both
opposite-origin cases; generated executables are cleaned. Receipts in the
integrator tree are `1793041100110258225` and `1793041148102147156` under
`obj/qualification/fast/`. No compiler validation was weakened.
