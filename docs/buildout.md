# Maintainer buildout handoff

## Current implemented slice

The repository is initialized with the CPPGM layout and 38 assignment briefs.
PA1's lexer, PA2's literal interpretation and PA3's token trees/attributes have
executable text contracts. PA4 onward remain visibly planned. No student export or Rust source
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

## Next implementation boundary

First audit PA1 against the pinned grammar's malformed-token boundaries and
decide exactly which reserved-prefix errors belong to PA1 versus PA2/PA3. Add
minimal independent vectors before widening acceptance claims. The handout
explicitly labels the current boundary as a draft.

PA2 now freezes that initial value contract. Its decoder deliberately preserves
integer magnitudes and unsuffixed decimal spelling until context is known.
PA3 now exposes typed token trees, attributes and configuration. PA4 should
parse expressions/statements using bounded token lookahead and typed arenas;
PA5 adds item/type/pattern ownership and applies disabled-item decisions.
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
