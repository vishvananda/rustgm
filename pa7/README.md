# RustGM Programming Assignment 7: Macro matching

Status: **implemented-draft**. Prerequisite: PA6. Profile: Rust 1.98.0,
edition 2024, Linux x86-64.

## Responsibility and boundary

Implement `macro_rules!` definition checking, ordered matching, parser fragment
callbacks and typed transcription. `dev/rustgm --emit-expanded [-o OUTPUT]
[SOURCE|-]` reports one matching/transcription step for source-written calls.
The input is a configured crate source image. Earlier views remain available.

PA7 does not parse the transcription as a new item, expression, type or pattern,
recursively invoke generated macros, load generated modules, resolve qualified
macro imports, provide builtins, or apply hygiene. PA8 owns that expansion
session and consumes these typed results. Ordinary name/type errors in captured
and discarded arguments are not demanded. Qualified calls and calls without a
preceding textual definition are diagnosed in this view. External source modules
are not loaded by this single-image matching view; PA6 retains its package view.

Walk active source in grammar order. A definition becomes textually visible after
its declaration, may shadow an earlier definition, is inherited by nested inline
modules/blocks, and stops at the end of its containing scope. Disabled definitions
are neither compiled nor published. Validate every enabled definition, including
ones that are never called. Template repetition shape errors are invocation-time
errors, since unused templates may contain nested macro definitions.

## Matcher and transcriber

Retain flat matcher/template arenas, stable variable IDs and repetition ancestry.
Outer matcher delimiters match any invocation delimiter; inner groups must agree.
Match literal tokens by category and spelling, with NFC identifier equality and
raw-identifier flags preserved. A raw string and a cooked string with the same
decoded value are different matcher tokens.

Implement all 15 fragment specifiers: `block`, `expr`, `expr_2021`, `ident`,
`item`, `lifetime`, `literal`, `meta`, `pat`, `pat_param`, `path`, `stmt`, `tt`,
`ty`, and `vis`. Grammar fragments call the ordinary parser over classified
tokens, including resuming inside `>>`/`>>=` after generic arguments. Preserve
opaque parsed fragments by identity; only `ident`, `lifetime` and `tt` remain
transparent. An empty visibility capture requires a following eligible token;
`($v:vis)` alone does not match an empty invocation.

Try rules in order. After an eligible fragment parser reports an error, do not
try later rules. Diagnose local ambiguity before speculative parsing. Memoize a
fragment by kind and input position across failed rule alternatives. Use a
bounded NFA frontier or equivalent shared-state algorithm, not exponential
recursive backtracking. Reject empty-consuming repetitions and illegal follow
sets, including nullable paths and nested groups.

Reconstruct captures from the winning trace. Transcription indexes nested
repetition instances, checks zipped cardinalities, preserves empty sequences,
and permits outer scalar captures inside repetitions driven by another binding.
Substituting a still-repeated binding is an error. A repetition must have a
repeating binding, and `+` must produce at least one iteration. Unknown template
substitutions retain `$` and identifier tokens for generated nested definitions.
`$crate` remains a typed definition-site marker for PA8.

Invocation documentation comments become typed `#[doc = RAW_STRING]` or inner
attribute tokens. Preserve their origin spans. Allocate a spelling image only
for a capture needing generated doc tokens; never lex that generated image.
Definition matcher comments remain literal doc nodes, while template comments
remain typed documentation pieces awaiting expansion. Do not stringify ASTs.

The pinned compiler has several observable details beyond the Reference's
prose: a trailing `$` is a literal matcher/template token; unseparated repetitions
do not add a follow-set check across their iteration boundary; `*` captures may drive `+` (when nonempty)
or `?` templates; `$crate:ident` denotes a crate marker followed by literal
`: ident`, rather than a binder. Tests pin these behaviors. Documentation in a
matcher cannot match an invocation comment after its attribute desugaring.

See the pinned [macros-by-example chapter](../doc/rust-reference/src/macros-by-example.md)
and [macro ambiguity chapter](../doc/rust-reference/src/macro-ambiguity.md).
The pinned compiler's [fragment dispatch](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_parse/src/parser/nonterminal.rs)
informs conservative token eligibility; this frontend calls its own parser.

## Typed interface and exact text view

`expand::macros::compile` builds definitions; `matcher::Compiled` lazily caches
rule control graphs. `Matched` owns memoized captures and the winning linked
trace. `bindings::Bindings` gives flat repetition instances. `Transcription`
contains origin-tagged tokens, opaque capture IDs, doc pieces and crate markers.
`session::analyze` returns definitions and source invocations in traversal order.
Later phases consume these records directly.

IDs and positions are zero-based decimal. Invocation token positions are
`INDEX:BYTE_OFFSET`; the offset represents a remainder inside one classified
compound token. Definition spans and invocation spans use normalized source
offsets. Quoting follows PA1. Emit all definitions first, then all invocations:

```text
macro-matching 1
definition ID START END "NFC_NAME" rules COUNT
invocation ID START END definition ID rule INDEX
capture ID FRAGMENT START_POSITION END_POSITION "SPELLING" nodes COUNT
token definition|invocation CATEGORY "SPELLING"
opaque CAPTURE_ID
crate definition DEFINITION_ID
doc definition inner=BOOL "BODY"
end-invocation parser-calls COUNT
end-macro-matching definitions COUNT invocations COUNT
```

Capture IDs follow first demand, including captures memoized from failed earlier
rules. `nodes` is the retained parsed syntax node count, or zero for transparent
fragments/visibility without a path. Each result piece emits exactly one line of
its appropriate kind. Repeated opaque substitutions reuse the capture ID.
Output is deterministic and independent of hash iteration order or filesystem
location. Analyze completely before writing; on a source error stdout is empty,
stderr is `ERROR: OFFSET: MESSAGE\n`, and status is 1. Resource errors are
diagnosed: definition nesting 128, matcher work 1,000,000 plus 128 per input
token, and transcription work 1,000,000. Internal failures cannot become empty
successful output.

`--count-expanded` runs the same work and reports definition, invocation, capture,
piece and input-byte counts for measurement; it is not the grading view.

## Validation

```sh
make test-pa7
python3 scripts/check_syntax.py pa7
make test lint audit-rust test-harness test-upstream
```

Tests declare acceptance before reference generation. All complete fixtures are
checked with pinned rustc; the manifest explains the two cases whose generated
syntax is intentionally left to PA8. Rust invariants check fragment reuse,
typed-token input without relexing, opacity, nesting/cardinality, textual scope,
doc origins and bounded work on a 10,000-element invocation. Measure both new
matching throughput and regressions in ordinary parsing against frozen binaries.
