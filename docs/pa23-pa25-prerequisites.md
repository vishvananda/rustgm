# PA23–PA25 prerequisite checkpoint

This checkpoint starts from PA22 release `e151b1e9`. It fixes shared frontend
prerequisites for the PA23–PA25 buildout. Those three assignments remain planned:
this checkpoint does not provide trait-object dispatch, closure environments or
compile-time function execution.

## Changes

Const permission now follows resolved function identity, including generic
specialization. Constant contexts retain that fact in their typed bodies.
Eligibility checks inspect the selected CFG before evaluation demand, and
destructor restrictions consume the existing proved cleanup schedule. A function
pointer does not inherit const permission. Both sides of a conditional are
checked, while source after an unconditional transfer follows pinned rustc's
reachability behavior.

The closure parser attaches attributes to parameter nodes so configuration
removes disabled parameters before resolution. Eight new PA5 cases cover plain,
nested, pattern, conditional and malformed attributes; six accept and two reject.
These cases qualify syntax and configuration, not closure execution.

An existing PA19 defect prevented implicit borrowed calls on some generic
receivers from compiling: the synthesized storage place had no prepared field
instance. The selected dereferenced receiver now enters the existing field
preparation path. Ten new PA19 cases cover seven accepted receivers and three
rejected mutability, borrow-conflict and input-type controls. All seven accepted
public controls execute through rustc and the unchanged CPPGM backend.

## Independent controls

Maintainer tables add 46 const-function source controls and four native controls,
plus ten receiver source controls and three native controls. Source outcomes were
checked with pinned Rust 1.98.0 before public oracle generation. The receiver
defect was reproduced in the immutable PA22 binary, including a non-const method
and a successful equivalent free-function call.

A work invariant varies field count (1, 32, 128) independently of call count
(1, 32, 256). Repeated receiver uses share completed field instances; fresh
borrow regions may add constant work per call, without substituting every field
again.

PA5's 227 independent source checks and PA19's 1,105 independent source checks
passed. PA19 native qualification also passed. The generated sidecars add only
the new fixtures; no existing public text oracle changed. The PA19 inventory is
440 semantic cases and 665 LowIR cases, including 269 native executions.

Detailed observations are retained under `obj/pa24-pa25-work/`: `check-v1/`,
`check-v2/`, `public-v1/`, `probes-v1/`, `oracles-v1/` and
`prerequisites-v1/`. The initial receiver work assertion incorrectly required
constant total work despite fresh per-call regions; the final assertion checks
the intended additive bound. The first public audit found a repeated filename
stem; the maintainer suite was renamed to `borrowed_receivers.rs`.

## Checkpoint validation

All 38 gates passed for candidate SHA-256
`7beb2470a90c1afd68db18b1846db17fcada873c08589aae081c5a2e717a3997`.

| Check | Result |
| --- | --- |
| Release Cargo tests | 1,770 passed |
| Native maintainer groups | 114 passed |
| PA1–PA22 text oracles | 7,386 passed |
| Affected independent suites | PA5 and PA19 source; PA19 native; PA12, PA16 and PA17 source/native passed |
| Formatting, strict Clippy, file/architecture audit | Passed; 1,063 audited Rust files, zero findings |
| Audit tests, harness, discovery and upstream checks | Passed |

The complete source/binary receipt and individual logs are in
`obj/pa24-pa25-work/prerequisites-v1/validation/`. Independent totals preserve
each earlier assignment's documented acceptance differences; they do not claim
unqualified rustc parity. The backend and its contract did not change.

## Focused performance

Immutable PA22 and checkpoint binaries were measured with frozen, rustc-qualified
inputs, four A/A calibration observations, five ABBA blocks and three executions
per observation. Every shared input produced byte-identical output. Raw timings,
RSS observations, Callgrind counts and hashes remain in
`obj/pa24-pa25-work/prerequisites-v1/performance-v1/`.

| Input | Candidate median | Candidate instructions | Instruction change |
| --- | ---: | ---: | ---: |
| Ordinary 256-function control | 19.65 ms | 66.78 million | +0.067% |
| 32 const functions | 3.93 ms | 6.92 million | +0.36% |
| 128 const functions | 7.32 ms | 25.58 million | +0.35% |
| 512 const functions | 20.79 ms | 100.16 million | +0.42% |
| 32 borrowed calls, 128 fields | 5.30 ms | 11.50 million | Baseline rejects |
| 128 borrowed calls, 128 fields | 9.31 ms | 34.73 million | Baseline rejects |
| 512 borrowed calls, 128 fields | 24.63 ms | 127.63 million | Baseline rejects |

The receiver workloads type-check an unused function and therefore measure
frontend preparation and ownership work; native execution is qualified
separately. Their incremental instruction cost is approximately linear in call
count. Candidate peak RSS medians range from 8.13 to 11.53 MiB.

Wall-time calibration was noisy on several small controls, including a 22.34%
A/A spread on the ordinary input. These observations do not establish a speedup.
The instruction counts show the small cost of the additional eligibility facts
and checks, while the work invariant specifically guards against repeating full
field substitution for each call.

Successful validation cleanup removed 4,275 generated files (7.98 GiB in
unshared blocks); scoped measurement cleanup removed its object files. Current
immutable baseline and candidate binaries remain pinned, with source inputs,
diagnostics and observation receipts retained.
