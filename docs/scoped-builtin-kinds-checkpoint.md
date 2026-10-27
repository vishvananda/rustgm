# Source builtin declaration-kind join

Qualified source `091f90614649fee1670dcd374ff72b3d44cc2e2c` accepts genuine
scoped bang and legacy attribute declarations. The source declaration still
controls visibility, scope, matcher syntax and header provenance; its checked
builtin role supplies the extension kind. This follows pinned
`rustc_resolve/src/macros.rs::compile_macro`, which compiles the source declaration
before replacing the extension kind from the builtin registry.

One field-free `MacroTarget::local` mapping serves both source forms. Unqualified
invocation lookup checks actual textual candidates against the requested kind,
resumes each rejected candidate's real preceding scope, and keeps the existing
unfiltered cache. Imports retain their unfiltered query. Each skipped, warm and
failed candidate query pays the existing source macro work budget. The helper
diagnostic visitor independently authenticates the current source item form:
scoped declarations require their real item-definition mapping; legacy macros
retain their real node/scope/namespace identity without inventing that mapping.
No owner, field, registry role, feature or wire profile was added.

The standard six-control gate passed in 149.90 seconds, including metadata,
formatting, global architecture, production/integration/CLI strict Clippy,
private strict Clippy, the unoptimized build and all exact tests. It covers
opposite-kind legacy shadows, repeated lookup, real unresolved scope wakeup,
failed-attempt accounting, original scoped formatting/byte and legacy autodiff
positives, feature/matcher/unknown-role negatives and the preserved original
Iter declaration. Iter invocation still rejects before expansion permission.
The former staged-policy checks remain on the original staged fixtures; the
unstaged legacy autodiff declaration separately completes real semantic/lowering
checks without pretending to have a library policy.

Actual layouts in bytes are MacroTarget16, Definition136, Body96, Scopes128,
Call168 and Engine4088. Existing ordinary bootstrap capacity remains exactly24
slots/384 bytes; no new persistent storage is introduced. Filtered lookup with
8/32/128 wrong-kind source candidates observes 30/102/390 attempted steps and
30/102/390 underlying lookup steps across its warm/failure/pending controls.
The separate existing 8/32/128 declaration control observes source work
791/3071/12191 and retained-accounting bytes13056/52224/208896. These are logical
work/storage observations, not allocator-instrumented allocation counts or a
hardware performance claim.

Fourteen new pinned-rustc metadata observations have zero infrastructure failures
and zero status disagreements. Eight warning/category predictions differed and
remain retained: legacy sources emit unused_features (one also unused_variables),
and the malformed matcher says missing fragment specifier. Final expected
contracts bind those actual observations; original hypotheses/raw streams stay
immutable. No original formatting/byte/autodiff observations or Try native inputs
were repeated. Builtin execution, compiler-profile and foreign metadata fences
remain independent. The next unchanged-core observer is owned by the integrator;
this checkpoint does not claim a crate artifact or coroutine execution.

Evidence: `obj/qualification/fast/1793069606684326581/receipt.json`, SHA
`7b398fb0bf80748739be2bbc4ebfedd133a561ea2d0d0f080b06013787fb6bdb`,
2838 exact source/configuration fingerprints. Frozen predictions, fourteen raw
streams, prior failed gates and the final source map remain under
`obj/parallel/pa31/builtin-kinds-v1/`. The actual fixtures and expected contracts
live in `pa31/tests/spec/scoped-builtin-declarations/`.
