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

## Required continuation

Carry explicit source editions through the lexer, bounded parser cursor, module
loading and includes. Preserve per-token edition provenance across macro expansion
and metadata without growing every ordinary token or reparsing generated text.
Qualify mixed-edition generated definitions and opaque forwarding before exposing
the driver option. Define reserved-token rejection at the appropriate grammar
boundary without silently changing PA1's documented boundary-only token view.

Then qualify edition-sensitive semantic behavior required by the actual crate
graph, including match ergonomics, unsafe declarations/attributes, temporary
scopes, opaque captures, prelude/method selection and never-type fallback. The
2021/2024 profile and imported body edition must agree with the frozen Cargo unit
graph. No assignment is activated by this foundation.
