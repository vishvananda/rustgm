# Frontend performance integration and library bootstrap handoff

This checkpoint integrates all 18 commits of `origin/pr/frontend-perf-v2`, ending
at `d43ca889f4f51a7429fb36ae2cfe8033298a6601`, while retaining the newer source
builtin-macro bindings, checked opaque storage heads and C variadic metadata
transport. It does not activate PA25 or another unfinished assignment.

The qualified compiler source is `46d8b82a013536682a3ec9b87070d60e2a9bd7ec`.
Its immutable executable SHA-256 is
`427a7877b936fa848dc3f66ac8bffe8468a41d68ed73509c14e2b91f51623e28`.
Later reference and documentation commits do not change that compiler.

## Architecture and integration corrections

The imported series uses compact syntax and optional identities, parse-time name
interning, byte-oriented lexing, flat resolver tables, deterministic fast hashing,
body-local inferred-region numbering, compiled signatures, shared structural
analysis, reusable scratch buffers and cold boxed diagnostics. Same-width checked
addition/subtraction also reduces emitted LowIR. Caller-specific ownership,
initialization, destruction, lifetime and loan proofs remain required.

Integration preserves exact body ownership when resolving nested region facts,
invalidates cached empty numbering units when units join, and adapts the newer
consumers to compact IDs and the compiler hash-table aliases. Cached facts remain
inside their owning phase; production lowering does not depend on test-source
helpers. Strict lint expectations and record-layout controls match the resulting
representation rather than retaining stale pre-merge sizes.

Full private validation exposed a genuine allocation regression: an empty loan
expiration index attempted to allocate by a point domain of `u32::MAX`, exceeding
the guarded 12 GiB memory limit. The index now uses no table for no events, sparse
rows for sparse exits, and a bounded direct table for dense exits. Construction,
sorting and lookup work are charged; sorting is budget-checked before it starts.
Maximum-domain, sparse/dense, regeneration and near-exhausted-budget controls pass.
The original failed run and kernel OOM evidence remain retained.

The performance section of `spec.md` makes this series a precedent for future
work. `docs/parallel-work.md` requires each hot-path handoff to identify removed
work, cache and ownership boundaries, multiplying dimensions, bounded work/storage
and compact record sizes. A faster common case does not establish removal of a
quadratic path, and structural reuse does not grant another caller proof authority.

## Functional qualification

The unchanged compiler passes:

- 2,825 regular maintainer tests, including all 1,016 non-ignored private tests;
- 294 explicitly selected native maintainer tests;
- all 8,362 active assignment text cases;
- formatting, strict all-target/all-feature Clippy, the architecture/file audit
  (1,897 Rust files, zero findings), upstream fixture checks and the harness;
- all 8,161 independent source-runner checks, plus builtin-macro, literal, layout
  and upstream controls;
- all 4,858 native-runner cases, including 2,746 executable comparisons, 317
  explicitly compile-only checks and ten object comparisons.

Existing documented stage/runtime differences remain explicit; these totals do
not turn those cases into Rust equivalence claims. PA13 retains its sixteen
documented runtime differences. No planned assignment is counted as passing.

Two merged stdout references needed reconciliation. PA6's `for_binding` names
view retains the already implemented mutable-iterator declarations: four names,
eight scopes and 27 definitions/bindings. PA20's `trait-lifetime-proven` view
retains the published canonical dispatch arguments and separate opened witnesses,
using the new body-local numbers. Their diagnostics and exit statuses are exact.
Independent source/native checks preceded full PA6/PA20 regeneration through the
course runner. Generated receipts supersede historical scoped batches by their
retained hashes; no historical receipt or oracle was hand-edited.

Functional evidence is in the validation worktree's
`obj/parallel/frontend-perf-combined-v2` and
`obj/parallel/frontend-perf-reference-reconciliation-v1`.
The two workers retain their independent source/native lanes under
`obj/parallel/perf-integration-independent-v1/lanes/`.
The combined qualification record is main's
`obj/parallel/frontend-perf-integration-preparation/qualified-B.json`, SHA-256
`28169d82683abede3f84721d51907e288cdcc0b4ed0c80ea60d639f4039b3087`.

## Measurement scope

The primary comparison uses the published immutable `a3c95faa…` executable,
whose production source is `5e498da9…`. It measures the combined publication,
including intervening functionality, rather than attributing every difference
to the imported series alone. Frozen source inputs and every raw observation
remain retained.

The original 33-input preflight disproved four old-baseline acceptance hypotheses:
annotation depths 8/32/128 and the depth-128 foreign-owner case. The old compiler
rejects those inputs; the candidate accepts and independently object-compiles
them, with the expected inspected constant `N = 1`. The original failed preflight
is preserved. Only the 29 supported shared cases enter A/B comparisons. All four
newly supported inputs receive separate candidate-only absolute/scaling
measurements, with no baseline speedup claim. Three untimed negative controls
retain their distinct old unsupported and new type-error frontiers.

Lexer bytes are identical. Three configured-AST count changes follow compact
single-name/path-type representation; full representative output checks preserve
all 4,000 configured items, names, spans, attributes and parent roles. Nine changed
LowIR outputs differ only by checked-u32-add rewrites and consequent SSA numbering;
an exact pattern/operand/use review checks all remaining IR. Both retained variants
of all eighteen executable benchmark sources validate, build and execute against
the declared results and fresh pinned Rust. Unequal text remains unequal; native
observations and machine-artifact identities are recorded separately.

The exclusive study pins one CPU, uses AABB warmup observations, four A/A
calibration observations and five ABBA blocks, with three process executions per
observation. RSS uses two ABBA blocks; Callgrind runs once per binary/input.
Candidate-only observations retain B/B calibration and the same per-observation
execution count. Exclusivity covers the registered resource controller; other
projects may still affect wall time. No favorable retiming is permitted.

Measurement observations and analysis are retained under main's
`obj/parallel/frontend-perf-common-v1`; the original failed preflight remains in
`obj/parallel/frontend-perf-combined-v1`.

## Measured results and remaining algorithmic work

All 29 shared inputs retire fewer instructions: reductions range from 22.7% to
68.5%. No input crosses the declared instruction or joint RSS regression gate;
the largest median RSS increase is 158 KiB. Selected results are:

| Frozen input | Instructions | Paired median wall time | Median RSS change |
| --- | ---: | ---: | ---: |
| Lexer, 256,000 lines | -62.5% | -64.5% | -420 KiB |
| Configured parser, 64,000 items | -60.1% | -57.5% | -20,942 KiB |
| Flat resolver, 64,000 items | -68.5% | -62.4% | -36,798 KiB |
| Generic empty bodies, width 128 | -41.8% | -26.0% | -1,094 KiB |
| Observer, width 128 and 128 contexts | -49.2% | -32.6% | -22 KiB |
| Rigid context, 128 contexts | -55.1% | -39.9% | -706 KiB |
| Ordinary scalar control | -22.7% | -5.6% | -238 KiB |

Wall times remain observations from this machine, not universal speedups. The
scalar control's A/A spread is 27.0%, exceeding its measured wall-time difference;
the generic-empty spread is 15.0%. The three large frontend inputs above have
A/A spreads of 2.2–2.8%. Every block and calibration observation remains retained.

The four newly accepted inputs have candidate-only medians of 7.13, 13.57 and
31.88 ms for depths 8/32/128, and 62.26 ms for depth 128 with three foreign owners.
Their instruction counts are 3,526,546; 12,133,138; 46,841,095; and 92,309,949.
The depth series has 3.44x/3.86x instruction growth per 4x depth and an incremental
ratio of 4.03. Their B/B wall spreads are 18.7–40.4%, so the instruction scaling
is stronger evidence than fine-grained timing comparisons.

One scaling trigger remains: increasing tuple width 8/32/128 with eight observer
contexts gives a positive incremental instruction ratio of 6.42 (published
baseline: 7.55). A single bounded attribution pass used the existing A/B profiles
at those widths and the ordinary scalar control; it ran no additional compiler
measurements. Both binaries call `borrow::regions::points::Solver::insert`
exactly 374/2,624/35,366 times. Candidate inclusive insertion costs are
49,447/435,118/6,118,085 instructions versus 189,898/1,776,312/24,434,284 in the
baseline. The ordinary control has no calls to that routine.

The source explains the retained multiplying work: liveness `Population::emit`
enumerates each active region/parameter at every point into pair vectors, then
`points::infer` inserts individual seeds into sparse 64-bit words. Faster hashing
reduces its cost but does not eliminate that expansion. A follow-up must carry
bounded compressed word/range or transition facts across both producer and
consumer, preserving borrow births, CFG joins, parameter/storage requirements,
budgets and caller-specific proofs. It must compare against an independent
small-domain point model and retain separate width/context and ordinary controls.
This checkpoint improves an existing superlinear path; it does not claim that
the path is now linear. Attribution and review remain in main's
`obj/parallel/frontend-perf-observer-attribution-v1`.

## Required continuation

The active goal now includes every assignment requirement needed to compile
original upstream core, alloc and std and execute their consumers. PA25–PA30
retain their complete owning contracts. Generic/const body transport, validated
source-library bindings and the actual pinned source blockers determine priority;
ordinary upstream APIs must come from library source. Any internal, target or
edition feature required by that source must work at its bootstrap milestone.
Optional self-hosting remains outside this completion boundary.
