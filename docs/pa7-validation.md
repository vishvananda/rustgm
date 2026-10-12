# PA7 implementation and validation

PA7 implements ordered `macro_rules!` matching and one-step typed transcription.
Its 181 complete-program fixtures have independently declared outcomes and
committed stdout/stderr/status oracles. All 181 were checked with Rust 1.98.0;
two explicitly documented differences defer parsing generated syntax to PA8.
The active PA1–PA7 total is 1,006 fixtures. Earlier text sidecars are unchanged;
explicit regeneration refreshes their compiler-source receipts.

## Architecture and ownership

- `syntax/cursor` adds borrowed classified-token input and resumable compound
  token positions. The ordinary source path still uses bounded lookahead and
  does not retain a whole-file token stream. `syntax/expression/fragment` calls
  the ordinary grammar; captured literals stay deferred until needed.
- `expand/macros/model`, `compile` and `follow` retain flat rule/variable arenas,
  repetition ancestry and bounded FIRST/follow facts. Rules share these arenas;
  a 10,000-rule invariant checks their shape without per-rule whole-arena copies.
- `nfa`, `matcher` and `capture` use a bounded frontier, lazy control graphs and
  memoized fragment parsing. A reusable session workspace retains frontier
  capacity. The matching memo is released on success; a single large capture
  record gets exact initial capacity instead of Vec's default four records.
- `bindings` replays only the winning trace into flat iteration ranges and
  sparse `(iteration, variable/repetition)` facts. `transcribe` checks nested
  and zipped shapes and emits typed pieces, including opaque capture identities.
- `prepare` constructs doc-attribute tokens only for demanded invocation
  comments. It keeps original spans and never lexes the generated spelling
  image. Definitions retain typed doc nodes; template docs await PA8.
- `session` handles active source-written invocations and textual shadowing;
  `render` is a diagnostic adapter. Generated syntax, imports and hygiene are
  PA8 consumers, not silently simulated by source rewriting.
- NFC name interning and `FileId` now live in `support`; resolution keeps its
  existing reexports. Expansion does not depend on resolution internals.

Clippy all/pedantic runs with warnings denied. The one new lint expectation is
for a cohesive FIRST-token grammar dispatch. Authored code remains safe Rust;
no new dependencies, compiler fallbacks or library environment access were added.
The file/phase audit covers 78 Rust files with zero findings. The compact
ordinary syntax node layout stays unchanged.

## Independent checks

There are 85 Rust invariants in debug and release, including 22 new PA7 tests.
They check classified input without relexing, parser boundaries, split tokens,
rule order and memo reuse, ambiguity, 10,000-element repetition work, typed
opaque reuse, empty/nested captures, cardinality errors, scope restoration and
comment origins. The 14 harness regressions, fixture discovery, 24 imported
fixtures and 126 vendored Reference-file hashes also pass.

Pinned-rustc probes identified compatibility details that are not apparent from
Reference prose alone: literal trailing dollars; raw identifier/token spelling;
empty visibility lookahead; fatal fragment-parser failures; no extra follow-set
check across an unseparated iteration boundary; template repetition operator
behavior; and literal doc comments inside matchers. They are represented in the
fixtures and handout. The course implementation does not call rustc to implement
any frontend phase.

Evidence is retained under `obj/pa7-work/`: `final-validation.log`, `release.log`,
`final-independent.log`, `reference-refresh.log`, build logs and performance logs.
`check_syntax.py` retains per-fixture rustc diagnostics under `obj/rustc-syntax/pa7`.

## Measurements

On the shared Linux host, 2026-10-12, measurements ran sequentially with frozen
binaries/inputs, full text-output equality, A/A calibration and three ABBA
blocks. All observations, including outliers, are retained.

Ordinary crate syntax against the committed PA6 binary:

| Input bytes | PA6 median ms | PA7 median ms | PA7 peak KiB |
| ---: | ---: | ---: | ---: |
| 316,000 | 30.11 | 28.96 | 8,580 |
| 1,264,000 | 110.75 | 109.45 | 25,472 |
| 5,056,000 | 433.09 | 434.62 | 94,452 |

Receipt: `obj/perf/1791773746646328712/samples.json`. Full earlier-stage output
hashes agree. At the largest scale A/A spans 434.48–484.28 ms; these medians do
not establish an ordinary-parser regression or speedup. Later changes affect
only the macro matcher.

The macro workload repeats an invocation whose first rule fails after parsing
an expression, then reuses that fragment in the selected rule and twice in its
transcription. The baseline is the first correct PA7 build, not a PA6 macro
implementation. Final measurements include reduced capture allocation and
shared matcher scratch:

| Invocations | Input bytes | Initial median ms | Final median ms | Initial/final peak KiB |
| ---: | ---: | ---: | ---: | ---: |
| 4,000 | 92,072 | 29.17 | 31.93 | 14,204 / 9,296 |
| 16,000 | 368,072 | 118.16 | 89.65 | 49,396 / 28,992 |
| 64,000 | 1,472,072 | 463.06 | 394.91 | 190,928 / 108,552 |

Final receipt: `obj/perf/1791774026095735514/samples.json`. Peak retained memory
fell about 43% at the largest input, with identical output. The largest A/A
range was 441.95–676.12 ms; do not claim a universal time improvement from this
noisy host. Initial A/A scaling is in `1791773761329396100`; the intermediate
capture-allocation-only comparison is in `1791773862275654630`. All three
receipts remain available and identify their distinct binary hashes.
