# Core macro and projection prerequisites

Compiler snapshot `65f20e8b` combines scoped declaration publication, cfg_select,
literal source formatting, contextual member conformance and the shared
projection-opening repair. Functional qualification is complete. The broad
performance run and the one required RSS attribution review are complete.
The original memory increase remains recorded as an accepted measured cost. This snapshot does not enable complete scoped
macro invocation, nonempty formatting, foreign macro transport or a core artifact.
PA1–PA24 remain implemented drafts; PA25–PA44 remain planned.

## Functional evidence

The full release run found one stale ordinary Clone rejection assertion. The
independent source contract permits this non-const Clone implementation; the
correction preserves the actual constant-callability flag. Recovery executes
354 fresh frontend cases and authenticates 3,412 retained passes from unchanged
release targets, for 3,766 total. This is complete qualification with explicit
retained provenance, not a fresh execution of every target.

All 8,382 active text cases pass. One PA20 inspection oracle was regenerated
after reviewing the entire retained type graph: shared projection opening removes
two unused arena types. The original source, previous and current LowIR output,
and independent native answer agree. No semantic obligation was removed to
preserve a historical arena ID.

Formatting, file/architecture audit, all-target strict Clippy, harness, upstream
provenance, backend and affected independent PA7/8/12/19/20/21 checks pass.
The transitive metadata consumer returns 7; all three release adapters use the
same qualified library. The inventory binds 399 required exact test names.

Evidence lives in the validation worktree under
`obj/parallel/core-prerequisites-release-v1/` and
`obj/parallel/core-prerequisites-recovery-v2/`.

| Artifact | SHA-256 |
| --- | --- |
| CLI | `c52de59b042a362c821048e81c7f3751bf8642ce5457725a6cd11c29ef02df60` |
| Functional receipt | `ba64da66b3ad140f59d9c5d0cf8712b85625501be6d8e65b5a067e81661cb29b` |
| Checks | `6036e15b63c3c4842a035399ff781cfcb8ea39cf69b60068269093088af23345` |
| Inventory | `4a3e739f422ecfcea0106ee5a922434a758fb99c731ddacfdee9e189d2699bc7` |

## Performance evidence

The immutable comparison against `5602dc27` contains 110 inputs: 101 successful
byte-equal pairs and nine newly supported candidate-only inputs. All 85 native
preflights and 58 declared growth controls pass. A/A calibration, paired ABBA
observations, instruction counts and RSS observations are retained in
`obj/parallel/core-prerequisites-perf-v1/`. Unsupported baseline cases are never
reported as speedups.

The former quadratic projection case improves from 550,412,747 to 103,395,319
instructions at 128 independent subjects: 81.2% fewer. Its paired median wall
time is 73.9% lower, with 3.16% A/A spread. The 32→128 subject growth ratio is
3.732 instead of 10.04, within the unchanged 6× limit. This is a measured
algorithmic improvement on that workload, not a whole-core compile-time claim.

One input, `selector-ctfe-repeated-8`, crosses the RSS review threshold:
11,096→11,716 KiB, +620 KiB (+5.59%). Instructions change by +0.169%; paired
median time changes by −0.91%, within its 3.99% A/A spread. This trigger remains
recorded; the following bounded attribution reviews it without replacing the
original observations or claiming zero regression.

A predeclared two-observation follow-up runs one Massif heap/stack profile and
one GDB exit mapping per unchanged binary, retaining exact original outputs.
Peak live heap is 275,355→276,402 bytes (+1,047); total profiled heap, allocator
overhead and stack is 366,480→370,392 bytes (+3,912). Under GDB, resident compiler
pages increase by 128 KiB and stack by 4 KiB, while resident heap stays 300 KiB.
Those mapping changes account for the instrumented 132 KiB process difference;
they do not numerically reproduce the original uninstrumented 620 KiB delta.
The evidence rules out a corresponding large live-heap increase in this input
and supports executable-page residency/layout as a contributor. Retain both
observations and accept this bounded cost for the qualified feature checkpoint;
no source change or timing rerun was used to obtain a favorable comparison.
Memory evidence: `obj/parallel/core-prerequisites-memory-v1/`, results SHA-256
`85eb387c714268796295a2ce7178d8987a05d41d59d493118c6da31a96a07dac`.

Results SHA-256: `1216f378f62b43b9e7c265204c5c313756c4e77ef6bb4908e26e5acbf39df48a`.
Analysis SHA-256: `9ed4ea733e511521d0036ed539faa92d6b233b2c81c73480d8fe1ee416a90a50`.
The guarded exclusive command finished successfully and cleaned its disposable
outputs; the 150 GiB reserve and three-job limit remain unchanged.

## Original core and subsequent work

The qualified compiler passes the original formatting declarations and stops at
`concat_bytes`, `core/src/macros/mod.rs` line 1126, byte 40135. It opens only
four original files, all unchanged. There is no semantic completion or artifact.
The whole-core record is `obj/parallel/core-prerequisites-whole-core-v1/`, result
SHA-256 `a86cba79fa5dad8410094861146ca4d5bda0f13009741f2e2eb05ea82df7a29d`.

The separate concat_bytes component `a3d7dc49` subsequently passes its source
and native controls and advances unchanged core to the attribute declaration
`autodiff_forward`, line 1553, byte 57236. It is not part of this combined
snapshot. Its handoff is in worker C's
`obj/parallel/pa31/concat-bytes-handoff-v1/handoff.json`; integration and combined
qualification remain required. Neither line movement nor test counts establish
a completion percentage for core.

The [blocker tracker](library-build-blockers.md) retains the actual remaining
work: scoped invocation and physical type privacy, attribute declaration and
application kinds, early implementation readiness, generic source artifact
transport and full formatting arguments. Ordinary upstream library algorithms
must continue to be compiled from their original source.
