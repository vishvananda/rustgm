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
and grading-suite work is tracked in `docs/pa8-progress.md`.

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
fragment-parser callbacks from matching, or the single argument-list parse for
an eager builtin. A forwarded opaque fragment does not need another callback.

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
report `ERROR: file FILE: OFFSET: MESSAGE` on stderr. Error offsets refer to
normalized physical source, not the appended expansion image. When applicable,
subsequent lines report innermost-first expansion ancestry:

```text
  in macro NAME at file FILE: OFFSET; defined at file FILE: OFFSET
  in macro NAME at file FILE: OFFSET; builtin
```

A captured expression keeps its original physical error location. A failed
expansion can report its selected definition even before an invocation record
has been committed. Module parse errors retain their introducing expansion,
including when the invalid file has not yet entered the source registry. These
are RustGM diagnostic messages; rustc's wording and note layout are not an oracle
contract.

The count mode emits just the footer's counts, without `end-expanded-crate`.
`make perf-expanded-crate` measures independent functions whose macro matcher
rejects its first rule, retains an expression capture, duplicates its typed
syntax and resolves the generated body. It verifies full output hashes as well
as counts, freezes binaries/inputs, and retains A/A and paired observations.

The prototype implements `cfg!`, `line!`, `column!`, `file!` and `module_path!`
using typed literals. Location builtins accept no arguments. `file!` follows the
physical invocation cause and retains the requested source path, including
symlink aliases, leading `./` and `..`; redundant interior `.` components are
normalized. Stdin is named `<anon>`. `module_path!` uses the module receiving the
expansion, including modules generated around a captured invocation. Function
names do not become module-path components.

`--crate-name NAME` supplies that path's first component. Otherwise the first
root `#![crate_name = "NAME"]` wins, then the input filename stem (with dashes
replaced by underscores), or `rust_out` for stdin. An explicit name must agree
with a root attribute. Empty names, characters other than letters/digits/`_`,
malformed duplicate attributes and `crate_name` inside `cfg_attr` are errors.
These identities do not require source reparsing or filesystem calls in the
compiler library.

`stringify!` consumes classified tokens without expanding its arguments. It walks
retained opaque fragments by identity and produces an owned string literal, with
no fragment-parser callback. Captured tokens occupy one shared arena; repeated
or forwarded substitutions do not duplicate that storage. Unicode identifier
spellings normalize to NFC. Source spacing, macro-template spacing, delimiter
edges and invisible fragment boundaries remain distinct. Direct documentation
comments retain their spelling; matching them as token trees can desugar them
into attributes before stringification.

The pinned stringification layout uses a 78-byte target width, with at least
60 bytes available after indentation, consistent brace edges and independently
wrapping contents. Long individual tokens may exceed that target. The iterative
walker diagnoses more than 1,000,000 token visits or 4,096 nested continuation
frames. These formatting decisions follow the pinned compiler's observable
[token printer](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_ast_pretty/src/pprust/state.rs)
and are checked with independent runtime values. Rendered output remains a
builtin value and diagnostic view, never a syntax transport.

`concat!` and `compile_error!` use eager argument expansion. The ordinary parser
reads the comma-separated argument list once. Nested invocations enter the same
resolution worklist; an explicit dependency count resumes the builtin after
those invocations finish. Delayed definitions and imports do not cause another
argument parse. Temporary argument nodes retain their source spans and typed
literal storage and are retired after the resulting value is installed.
For an eager builtin, `parser-calls=1` records that argument-list parse.

`concat!` accepts strings, characters, booleans and numeric literals, including
unary-negative numeric literals. Integer spellings become decimal magnitudes;
float spellings keep their decoded decimal text without suffixes or underscores.
A trailing comma is allowed. Parenthesized literals, blocks, arbitrary constant
expressions, bytes, byte strings and C strings are rejected. `compile_error!`
requires one eagerly expanded string and diagnoses its contents. Both preserve
ordinary macro shadowing. `concat_bytes!` remains unstable in Rust 1.98.0 and is
outside this stable-language profile. Library-dependent builtin expansion is
owned by the later language/library hooks rather than fabricated external names.
