# PA34 edition context foundation

The driver still supports only Rust 2024. Accepting a `--edition=2021` flag
without carrying its semantics through parsing, expansion and imported bodies
would make the original Serde graph incorrectly appear supported.

`support::edition::Edition` now supplies the common typed identity for Rust 2021
and Rust 2024. Imported body origins re-export this identity; its discriminants
are not a metadata format. The default preserves the current 2024 profile.

`expand::macros::compile::in_edition` selects fragment grammar once at macro
definition compilation. For a 2021 definition, `expr` becomes the existing
`Expression2021` fragment kind. Explicit `expr_2021` remains that kind in either
edition. Matching, first-token eligibility, parser callbacks and capture
memoization consume that completed kind. There is no per-invocation spelling
rescan, new cache, or growth of tokens, retained definitions and capture records.
The ordinary `compile` entrypoint continues to select Rust 2024.

This API controls fragment grammar only. It does not assign the defining edition
to captured tokens. Full integration must retain source-token editions separately:
captured tokens retain their invocation provenance, while transcribed definition
tokens retain their defining provenance. Opaque captured syntax stays parsed and
must not be reconstructed from rendered text. Generated definitions and imported
macros need the same rule; a caller's global configuration is not sufficient.

## Independent qualification

The pinned Reference's [fragment edition rules](../doc/rust-reference/src/macros-by-example.md)
and [reserved tokens](../doc/rust-reference/src/tokens.md) define the hypotheses.
Before implementation, the frozen plan in
`obj/parallel/pa34-edition-context-v1/plan.json` declared 16 source outcomes and
four native consumers. Pinned Rust 1.98.0 completed all 22 commands successfully,
including the two provider builds. The plan SHA-256 is
`98dc2bb2b2e9e3b03c67746777f5af4b40b45589f4ffb0a63860837cc5296c5e`;
compiler SHA-256 is
`3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2`.
Inputs, stdout, diagnostics, commands and results remain in that evidence root;
the successful guard removed its generated native and metadata artifacts.

Both defining editions were invoked from both consumer editions. Native assertions
confirmed ordinary/subexpression acceptance, differing top-level const-block and
underscore rule selection, and edition-independent `expr_2021`. Additional source
controls confirmed 2024 rejection of `gen`, adjacent hashes and guarded strings,
with 2021 acceptance and raw-identifier controls. These independent token outcomes
are requirements for subsequent work, not claims that RustGM implements them.

The registered `edition_fragments` suite exercises the matcher rule outcomes,
committed parse failures, and memo identity. A 2021 `expr` capture shares one parse
with an `expr_2021` alternative; the distinct 2024 kinds remain separate. Current
source parsing, expansion sessions and CLI deliberately retain their existing
2024 entrypoints until the remaining context is implemented.

The combined working candidate includes the source `needs_drop` identity
prerequisite. Its focused debug batch passed all 37 selected tests (22 matcher
tests and 15 source intrinsic/layout invariants), with exact selected/executed
inventory checks. All-target/all-feature strict Clippy, formatting, the 1,931-file
architecture audit and harness checks passed. Receipts are under
`obj/parallel/pa34-edition-context-v1/checks-v3/`. Earlier formatting-order and
test-filename findings remain in the prior receipts; neither reached expensive
test code generation. Full release/native and performance qualification remain
required before publishing this combined compiler checkpoint.

The same debug candidate also passed all 8,362 active PA1–PA24 byte-exact text
comparisons in 52.96 seconds. `course-complete.json` in the evidence root binds
the source commit, binary hash and individual receipts. This cheap regression
pass reused the focused batch's CLI build; it did not invoke Cargo again.

## Token syntax and provenance checkpoint

The explicit `parse_crate_in_edition` syntax entrypoint now distinguishes `gen`
in Rust 2021 and Rust 2024. Existing entrypoints still default to 2024. Name,
path, field, use, generic/capture and attribute positions consume the same typed
keyword rule; raw identifiers keep their existing behavior. This is syntax
support, not a complete source-edition profile or a new driver flag.

`Inputs::edition` reuses the parser's existing context reference. Uniform source
uses the typed Edition; `WithEditions` borrows an immutable provenance resolver
beside the existing opaque-input provider. Only `gen` demands that resolver.
Transparent ident/tt captures do not interpret keywords. Macro fragment grammar
still comes from the definition, while eligible syntax capture uses each input
token's edition. Matcher memo entries remain invocation-local.

`Stream::token_origin` checks a complete generated token against the existing
sorted origin ranges. The caller resolves that original file/span, continuing
through parent expansion maps when necessary. No token, cursor, AST, parser or
retained macro record gains a field, and generated spelling is never lexed again.
Opaque expression forwarding reuses parsed syntax without edition lookups.
Attribute conversion likewise retains checked path-name admission in local,
ordered graft ranges; new adjacent tokens still require their own provenance.
Ordinary attributes allocate no graft-range storage.

Independent source plans and observations are retained under
`obj/parallel/pa34-token-editions-v1`. The first plan's 16 local source outcomes
matched. Its mixed-edition run exposed an invalid provider artifact filename;
those cross-crate failures are infrastructure evidence, not language rejection.
The corrected immutable `independent-v2` plan
(`9c233749f4e9ffe493248d2042f63563ffb42f10923e6817572d669e4b125a47`)
passed all 34 commands: two providers, 20 source checks and 12 native executions.
They distinguish captured names, definition-written names, raw identifiers and
generated arguments consumed by a second macro. Eight further metadata and
restricted-visibility probes matched in `qualification-v1`.

The first candidate focused batch passed 68 frontend tests and five private
destruction-query invariants; strict Clippy, the architecture audit and all 152
harness tests passed. Its debug code generation took 138.68 seconds; test
execution took 0.30 seconds. These shared-machine observations illustrate the
reason for batching code generation, not a compiler speedup measurement.
The subsequent opaque attribute-path correction passed its mixed-edition
independent plan (`b907b04c6ae9f1daf7dbbec158d3344a1dc473dfba3fec82b27e6ee6cb6ddd65`):
two provider builds and eight source checks, all matching the declared outcomes.

The combined source-trait/token-context candidate at `3da05b3c` passes all 104
selected tests: 76 frontend and 28 private checks. The first combined run passed
all frontend checks and 27 private checks; one closure-binding negative still
expected the old diagnostic wording. Its updated assertion preserves the required
rejection and passed an exact retry. The unaffected successes were retained.
All-target/all-feature strict Clippy, architecture and harness checks pass.
`qualification-v2` and `qualification-v3` retain both the failure and correction.

That same production CLI passes all 8,362 active PA1–PA24 text comparisons in
40.87 seconds, with no reference changes or additional Cargo invocation. The
`combined-text-v1/course-complete.json` receipt binds the source commit and CLI
SHA-256 `750d1f1a05178046edc7aff25b0d697eb06f3a3e4be01c3da74b5f23328f5f09`.
Scoped cleanup keeps the source inputs, diagnostics and receipts. Combined
release/native/performance qualification remains required before publication.

## Required continuation

The [package lexical edition checkpoint](pa34-package-editions.md) now connects
configured attributes, module loading, includes, persistent expansion sessions
and bounded parent-origin lookup. Metadata and actual cross-crate macro transport
remain required. Qualify mixed-edition generated definitions and opaque forwarding
through those import entrypoints before exposing the driver option. Define
reserved-token rejection at the appropriate grammar boundary without silently
changing PA1's documented boundary-only token view.

Then qualify edition-sensitive semantic behavior required by the actual crate
graph. The [explicit pattern checkpoint](pa34-pattern-editions.md) now qualifies
the modifier/reset portion of match ergonomics. Remaining work includes unsafe declarations/attributes, temporary
scopes, opaque captures, prelude/method selection and never-type fallback. The
2021/2024 profile and imported body edition must agree with the frozen Cargo unit
graph. No assignment is activated by this foundation.
