# Package lexical editions

This checkpoint connects the existing Rust 2021/2024 lexical context to real
package entrypoints. It does not expose a 2021 driver profile or activate PA34.
The remaining semantic and build-graph requirements stay in
[the bootstrap roadmap](library-bootstrap-roadmap.md).

`Config::in_edition` selects a physical compilation unit's edition. Parsing,
configuration, file modules, cached fresh module instances and physical includes
retain it. The standalone macro session selects both defining fragment grammar
and invocation-token context explicitly. Existing callers default to Rust 2024.

Package expansion resolves a generated macro's defining edition from its actual
declaration site. A captured macro name or body does not replace that site.
Matcher input, parsed output, nested configuration and eager builtin arguments
share the existing borrowed opaque-input context, augmented with source-edition
lookup. Captured syntax stays parsed; generated spelling is never lexed again.

The source lookup follows existing sorted parent-origin ranges. Physical source
terminates in constant time without building a line index. Generated subranges
must fit one complete, equal-length parent mapping, including legitimate tokens
inside copied opaque slices; gaps, crossing ranges, missing files and cycles
fail explicitly. Work is bounded to 4,096 ancestors, with a binary search per
generated ancestor. Ordinary and raw identifiers do not request an edition.
There is no new per-token/AST/parser field or whole-package scan. Only the cold
configuration and source unit retain an Edition value. Deep expansion chains
remain a dimension for performance measurement, not a claimed constant-time path.

## Evidence

Evidence is retained under `obj/parallel/pa34-package-editions-v1/` in the
integrator's PA29 worktree. `independent-v1/plan.json`, frozen before its run,
declared 36 pinned Rust 1.98.0 source outcomes. All matched; all 22 accepted
programs also built and executed. The plan SHA-256 is
`9a6e9eff5ff133b4583e96fccf4372b15b0aaf96934fd4b3f7e62cefce2f09e7`.

The registered `edition_packages` suite checks physical/captured/transcribed
names, generated macro definitions, selected and unselected nested cfg_attr,
raw names, modules, repeated physical modules and includes. It also reaches
ordinary semantic proof and LowIR lowering. Four native cases compare pinned
rustc with the candidate and CPPGM backend: a physical call, nested macro
forwarding and different generated-definition rule selection in both editions.
Their expected answers were declared in the test source before execution.
`closing-v1/native-plan-annotation.json` records that the separate JSON rendering
of these source-level expectations was written after the run; the source hashes
were checked against the actual executed inputs.

All 99 selected frontend tests and three private provenance/work-bound checks
pass. The first batch passed 98 frontend tests and all three private checks;
one inherited assertion matched the old debug rendering of a diagnostic file.
The corrected test checks the real source FileId, offset and include expansion
trace, and passes its exact retry. This changes no diagnostic behavior or oracle.
All-target/all-feature strict Clippy, the 1,973-file architecture audit and
harness checks pass. The assertion-only correction also passed strict Clippy.

The production CLI passes all 8,382 active byte-exact text cases. The closing
receipt binds its binary hash and individual test receipts. Source manifests,
failed attempts, diagnostics and timings remain; scoped cleanup removed generated
native artifacts. Full combined release and performance qualification remain
required before publication. These development timings are not performance
comparisons.

## Remaining scope

Mixed-edition source provenance is not an implementation of cross-crate macro
metadata. Imported macros must preserve actual definition and captured-token
identities through that real transport path. Reserved-token changes beyond
identifier admission and edition-sensitive semantics also remain: match
ergonomics, unsafe declarations/attributes, temporary scopes, opaque captures,
prelude/method selection and never-type fallback. The frozen Cargo unit graph
must eventually select the same profile for parsing, imported bodies and code
generation. Accepting a driver flag before those contracts hold would not satisfy
the original upstream crate requirement.
