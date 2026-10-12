# Maintainer buildout handoff

## Current implemented slice

The repository is initialized with the CPPGM layout and 38 assignment briefs.
PA1's lexer, PA2's literal interpretation, PA3's token trees/attributes and
PA4's expression/statement parser, PA5's crate/item/type/pattern parser, PA6's
module/name resolution and PA7's macro matching have executable text contracts.
PA8 onward remain visibly planned. No student export or Rust source
to LowIR lowering exists yet. The external backend smoke exercises handwritten
LowIR through the pinned CPPGM tools.

The original compiler's source and native binaries remain outside this repo.
`backend.lock.json` pins them; `backend.local.json` is the ignored host routing.
Backend files are never mutated. The active v5rust conversion is independent.

## Evidence from the initial build

- 51 course fixtures, including 18 unchanged upstream lexer fixtures.
- Eight Rust invariant tests in debug and release, including 20,000 nested
  comments and 1,000 deterministic malformed Unicode input sequences.
- Nine harness regression tests: exact channels, failure outcomes, timeout,
  empty/missing suites, undiscovered inputs and reference-write protection.
- Five imported full-program outcomes independently checked with rustc 1.98.0.
- Strict rustfmt/Clippy, syntax-aware file audit and provenance checks pass.
- CPPGM LowIR parse/validate/native bridge executes the exit-42 control.

Initial shared-host A/A scaling observations (2026-10-11), tokenization without
rendering, including process startup and source reading:

| Input bytes | Median candidate time | Peak RSS |
| ---: | ---: | ---: |
| 736,000 | 16.89 ms | 2,940 KiB |
| 2,944,000 | 66.06 ms | 5,056 KiB |
| 11,776,000 | 211.00 ms | 13,460 KiB |

Raw samples, warmups, A/A calibration and paired blocks are retained in
`obj/perf/1791759733565975209/`; each scale also checks complete rendered token
output hashes between immutable binary snapshots. The machine was shared with
other compiler work. These measurements establish a starting point, not a
regression claim or a comparison with another Rust compiler.

## PA2 buildout

PA2 adds reusable typed integer/float/character/byte/string/C-string/boolean
facts and target conversion. Plain/raw strings borrow source memory, scalar
escapes decode on the stack, and escaped strings allocate one output buffer.
Integer magnitudes are checked u128 arithmetic; floating conversion uses the
Reference-specified standard-library parsers directly at the demanded width.
There is no new dependency or compiler fallback.

The architecture was checked against CPPGM's `lex/number_literal.cpp` and
`lex/text_literal.cpp`: separate token boundaries, typed interpretation and
text adapters, with checked digit accumulation and iterative escapes. Rust's
inference, unsigned magnitude, suffix and Unicode rules stay language-specific;
C++'s candidate-type selection and host `long double` assumptions do not apply.

Evidence: 139 PA2 course fixtures (six unchanged upstream cases), 10 additional
Rust invariant tests, 69 independently evaluated rustc value/bit comparisons
and 58 expression acceptance/rejection checks. All 51 PA1 oracles are unchanged.
There are now 11 imported whole-program rustc outcome checks, 24 imported
fixtures overall, and 126 checked files of the pinned Rust 1.98.0 Reference.
Both debug/release invariants, strict Clippy, architecture audit and harness
checks are required. See the performance observations below for this build.

### PA2 performance observations

Measurements on the shared Linux host, 2026-10-11, include startup/source reads
but exclude dump rendering. Every scale separately checks full dump hashes
between frozen A/B executables before warmups, A/A calibration and three ABBA
blocks. All raw observations are retained, including outliers.

| Interpreter/input bytes | Baseline median ms | Candidate median ms | Candidate peak KiB |
| --- | ---: | ---: | ---: |
| Tokens / 736,000 | 17.32 | 20.53 | 2,940 |
| Tokens / 2,944,000 | 59.79 | 61.18 | 5,052 |
| Tokens / 11,776,000 | 228.12 | 231.59 | 13,500 |
| Literals / 1,232,000 | 36.23 | 28.51 | 2,940 |
| Literals / 4,928,000 | 113.16 | 104.17 | 7,164 |
| Literals / 19,712,000 | 428.70 | 399.73 | 21,244 |

Token baseline is the original PA1 executable; the literal baseline is the
first PA2 implementation before removing scalar escape allocations. The token
calibration itself ranges from 16.11–20.36 ms at the smallest scale and
226.12–273.68 ms at the largest: do not interpret the small median differences
as a proved lexer regression or speedup on this shared machine. Literal results
support roughly linear scaling, not a universal performance claim. Reproduce
with `scripts/measure_lexer.py --baseline BINARY [--stage literals]`.

Raw receipts: `obj/perf/1791761626407207630/samples.json` (tokens) and
`obj/perf/1791761639083291610/samples.json` (literals), with append-only
`observations.jsonl`, executable hashes, input hashes, snapshots and inputs.

## PA3 buildout

PA3 adds flat preorder delimiter forests, typed doc desugaring, attribute input
ranges and configuration facts. It deliberately leaves macro contents opaque
and exposes a position-based attribute API for the ordinary syntax parser.
Configuration settings are explicit and the target profile is fixed; no
compiler phase reads environment variables or invokes another frontend.

Evidence: 88 required oracles, 77 independent rustc program/predicate checks,
nine additional Rust invariants (including 30,000 nested groups) and a new
harness regression for custom argument forwarding. All earlier oracles remain
unchanged. Manifest hashes now protect reference provenance against silent
configuration/outcome changes. Debug/release tests, strict Clippy, file audits,
provenance, full active reports and oracle reproducibility pass.

Shared-host A/A tree scaling, including source reads and excluding rendering:

| Source bytes | Median ms | Peak RSS KiB |
| ---: | ---: | ---: |
| 284,000 | 13.33 | 4,352 |
| 1,136,000 | 44.44 | 11,376 |
| 4,544,000 | 172.23 | 38,136 |

All output hashes match between frozen snapshots. Warmups, calibration,
three ABBA blocks, inputs and raw observations are in
`obj/perf/1791763002707070858/`. These measure an intentionally retained
diagnostic forest; the ordinary source parser must not retain a whole forest.

## PA4 buildout

PA4 adds a three-token cursor, Pratt expression parser, explicit statement
boundaries and typed syntax arenas with compact IDs/ranges. It distinguishes
method calls from field calls and preserves path absoluteness. Only actual
macro captures retain token trees. The shared builder consumes already lexed
tokens; tuple projections and compound punctuation split typed tokens without
relexing source. Condition checks reuse scratch capacity. Recursion has a
256-call diagnosed budget; flat chains and captured groups remain iterative.

Evidence: 181 course oracles and rustc expression checks, with seven explicit
phase-boundary acceptance differences (deferred semantics and opaque macro
literal contents). Twelve new structural/resource invariants pass in debug and
release, including 20,000-operator chains and 20,000-level macro groups. All 278
earlier course outputs are unchanged. The full total is 459 course fixtures,
39 Rust invariants and 12 harness tests. Clippy, architecture/file audits,
upstream provenance, full reports and oracle generation pass.

The initial builder extraction measured an approximately 11% tree slowdown.
Removing repeated validation of lexer-owned spans and enabling inlining removed
that regression. Frozen PA3 baseline/final-candidate medians in milliseconds:

| Tree source bytes | Baseline | Candidate | Candidate peak KiB |
| ---: | ---: | ---: | ---: |
| 284,000 | 15.90 | 13.76 | 4,340 |
| 1,136,000 | 48.40 | 44.47 | 11,372 |
| 4,544,000 | 161.97 | 160.21 | 38,140 |

New expression-parser A/A scaling (startup and source reads included):

| Source bytes | Median ms | Peak RSS KiB |
| ---: | ---: | ---: |
| 140,004 | 13.89 | 5,056 |
| 560,004 | 49.19 | 14,200 |
| 2,240,004 | 184.96 | 49,400 |

These are shared-host observations, not portable speed guarantees. Full text
output equality, frozen inputs/binaries, warmups, calibration and every ABBA
observation are retained in `obj/perf/1791764190532643755/` (initial regression),
`obj/perf/1791764334673790599/` (fixed trees) and
`obj/perf/1791764197895932737/` (new parser baseline). Final minor condition
validation changes do not occur in the parser scaling input. Logs of all course,
independent, audit and release checks are under `obj/pa4-work/`.

## PA5 buildout

PA5 adds crate, item, generic/qualified type and pattern syntax with the same
body parser. Attributes attach at grammar-owned positions; expansion evaluates
actual owners top-down. Disabled nodes stop later sibling cfg predicates and
subtree traversal. Active literals decode exactly once through PA2. Dense
configuration effects retain global parent IDs and their original capture ID,
with reusable owner scratch. No new dependency or frontend fallback was added.

There are 213 new course fixtures and independent rustc checks, with two explicit
phase-boundary differences for external-module loading and unresolved types.
Eleven new Rust invariants verify typed child ordering, source spans, global
attribute provenance, deferred work, compound-token splits, compact nodes,
large item lists and diagnosed recursion. The full total is 672 course fixtures,
50 debug/release Rust invariants and 13 harness tests. All PA1–PA4 text sidecars
remain unchanged. Full reports, independent checks, Clippy, file/architecture
audits, provenance and reference generation pass; logs are in `obj/pa5-work/`.

The initial crate parser exhausted the debug test-thread stack on adversarial
nesting. A tested 128-call crate grammar budget now diagnoses these inputs; the
PA4 fragment budget remains 256. Macro groups and long item lists stay iterative.

The first enlarged syntax union raised large-expression peak RSS from
approximately 49,400 KiB to 56,400 KiB.
Generic parameters, function types, enum variants and use trees now live in
separate dense payload arrays. Ordinary nodes are again 36 bytes, guarded by a
layout invariant. A global configured-effect array also avoids one effect-vector
allocation for every attribute owner.

Final shared-host measurements, including startup/source reads and excluding
rendering, use frozen A/B binaries, full-output equality, A/A calibration and
three paired ABBA blocks at every scale:

| Path / source bytes | Baseline median ms | Candidate median ms | Candidate peak KiB |
| --- | ---: | ---: | ---: |
| PA4 expressions / 140,004 | 14.49 | 14.99 | 5,056 |
| PA4 expressions / 560,004 | 48.33 | 49.47 | 14,208 |
| PA4 expressions / 2,240,004 | 179.39 | 182.70 | 49,408 |
| PA3 trees / 284,000 | 14.39 | 14.85 | 4,352 |
| PA3 trees / 1,136,000 | 44.48 | 41.45 | 11,392 |
| PA3 trees / 4,544,000 | 153.06 | 150.40 | 38,144 |
| PA5 crates / 316,000 | 31.47 | 29.70 | 7,872 |
| PA5 crates / 1,264,000 | 112.90 | 108.78 | 24,768 |
| PA5 crates / 5,056,000 | 439.55 | 426.18 | 93,760 |

Expression/tree baselines are the committed PA4 release; crate baseline is the
first PA5 implementation before compact payloads. These observations establish
roughly linear scaling and the removal of the measured memory inflation, not
portable speed guarantees. Expression medians remain slightly higher; the raw
calibration and paired samples are retained rather than claiming a speedup.
Final span/absolute-use-path corrections do not occur in these benchmark inputs.

Raw receipts: `obj/perf/1791766314535848383/` (expressions),
`obj/perf/1791766636998230853/` (trees), and
`obj/perf/1791766643878982292/` (crates). Initial/intermediate observations remain
in `1791765924773113448`, `1791765931748986569`, `1791766065753161891`,
`1791766072771781373`, and `1791766321529900146` under `obj/perf/`.

## Next implementation boundary

First audit PA1 against the pinned grammar's malformed-token boundaries and
decide exactly which reserved-prefix errors belong to PA1 versus PA2/PA3. Add
minimal independent vectors before widening acceptance claims. The handout
explicitly labels the current boundary as a draft.

PA2 now freezes that initial value contract. Its decoder deliberately preserves
integer magnitudes and unsuffixed decimal spelling until context is known.
PA3 exposes typed token trees, attributes and configuration. PA4 parses
expression fragments with bounded lookahead and typed arenas. PA5 adds the
`--emit-crate-ast` entry point, item/type/pattern ownership and attribute
attachment. Its active-node traversal skips disabled owners and their later
cfg predicates, and defers literal interpretation until selection. PA4
fragment oracles remain under `--emit-ast`.

PA6 should consume the configured typed crate and its active map, introduce
canonical NFC name IDs and explicit module/scope identities, and load external
source modules at the driver boundary. Audit the normalization dependency/data
before adding it. Resolution must not consult rendered AST records or reparse
function bodies. Preserve unresolved macro placeholders for PA7/PA8's expansion
worklist rather than pretending their names have already been resolved.
Literal decoding is demanded by syntax/expansion, never applied eagerly to all
macro input. Expand reserved-prefix and grammar-edge tests before claiming
complete token conformance. PA9/PA10 must enforce inferred-type agreement and
contextual overflow linting; PA25 owns general constant expression arithmetic.

For every subsequent assignment, develop its exact output contract, fixtures,
reference implementation and independent language checks together. Preserve
accepted earlier case identities; do not inflate progress with empty suites or
new passing fixtures while existing failures remain. Borrow/move checks require
positive as well as negative cases, and generic/macro work needs scaling inputs.

## Before student export

Build the complete relevant maintainer solution and reference release bundle.
The export must strip the solution, install scaffolds and pinned reference
wrappers, retain required fixtures/licenses and validate the resulting student
checkout. Run plans and local measurements stay out of that export. The current
`rustgm-ref` wrapper is intentionally maintainer-local, not a distributable
reference tool or an implementation available to student code.

## PA6 and PA7 buildout

Module/name resolution evidence is in [pa6-validation.md](pa6-validation.md).
Macro matching, typed transcription and parser input evidence is in
[pa7-validation.md](pa7-validation.md). PA7 preserves `--emit-expanded` as its
one-step matching view; PA8 introduces `--emit-expanded-crate` for integrated
expansion, keeping earlier oracles stable.
