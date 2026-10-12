# RustGM Programming Assignment 8: Expansion and hygiene

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA7. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Expansion contexts, mixed-site hygiene, macro resolution and builtins.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-expanded-crate -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

dollar-crate; generated imports/items; recursion; definition/call-site spans.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.

## Maintainer prototype view

The maintainer CLI currently implements `--emit-expanded-crate` and
`--count-expanded-crate`, including module loading and final ordinary-name
resolution. This does not activate the planned grading suite. Remaining builtin
and diagnostic work is tracked in `docs/pa8-progress.md`.

The draft view begins with `expanded-crate 1`. Its sections are:

```text
macro ID name=NAME module=MODULE context=CONTEXT introduction=CONTEXT source=FILE:NODE|builtin rules=COUNT local-inner=BOOL
macro ID name=NAME module=MODULE context=CONTEXT introduction=CONTEXT source=builtin builtin=NAME
invocation ID context=CONTEXT parent=CONTEXT macro=ID source=FILE:NODE original=START:END generated=START:END rule=ID|builtin parser-calls=COUNT
file FILE
[PA5 configuration and syntax records]
origin START:END source=FILE:START:END context=CONTEXT|inherit
end-file FILE
[PA6 name-resolution records]
end-expanded-crate files COUNT modules COUNT macros COUNT invocations COUNT captures COUNT definitions COUNT references COUNT
```

The two macro forms distinguish declarative definitions from builtins. Macro and
invocation IDs follow creation/expansion order. Context 0 denotes physical source;
context N identifies invocation N−1. Introduction/parent contexts record expansion
ancestry, while token origins retain syntax hygiene independently. Builtins have
no invented source definition node or matcher rule. `parser-calls` counts ordinary
fragment-parser callbacks from matching; a forwarded opaque fragment does not
need another callback.

Per-file syntax IDs index retained arenas. The PA5 `active` record identifies the
configured tree; inactive original nodes may remain as provenance. Offsets use
the normalized, append-only spelling image, with origin records mapping generated
ranges to their parents. These images and records are diagnostic views, never
inputs reparsed by a later phase. Literal values are read directly from the
source unit's typed storage without cloning its syntax arena.

Name IDs use the PA6 spelling table, now including the early builtin prelude and
ordinary item declarations. Name definitions can additionally have kind
`builtin-macro`. Successful output contains fully resolved ordinary names and no
pending expansion references. Source failures emit no view bytes, exit 1 and
report `ERROR: file FILE: OFFSET: MESSAGE` on stderr. Diagnostic provenance is
still being completed before the grading contract is frozen.

The count mode emits just the footer's counts, without `end-expanded-crate`.
`make perf-expanded-crate` measures independent functions whose macro matcher
rejects its first rule, retains an expression capture, duplicates its typed
syntax and resolves the generated body. It verifies full output hashes as well
as counts, freezes binaries/inputs, and retains A/A and paired observations.
