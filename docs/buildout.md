# Maintainer buildout handoff

## Initial implemented slice

The repository is initialized with the CPPGM layout and 38 assignment briefs.
PA1's first lexical-boundary implementation and text contract are executable;
all later assignments remain visibly planned. No student export or Rust source
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

## Next implementation boundary

First audit PA1 against the pinned grammar's malformed-token boundaries and
decide exactly which reserved-prefix errors belong to PA1 versus PA2/PA3. Add
minimal independent vectors before widening acceptance claims. The handout
explicitly labels the current boundary as a draft.

Then make PA2 normative: typed literal values, target-independent integer and
floating behavior, escape checking, string/byte/C-string restrictions, suffix
validation and exact value text. Implement those facts as reusable typed data;
the parser must not consume the rendered literal dump. Expand the existing
fixtures with both valid and rejected boundary cases.

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
