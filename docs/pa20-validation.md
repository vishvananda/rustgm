# PA20 trait solving validation

The PA20 release published 1,343 text-oracle cases from 797 independently qualified source
programs. Its separate `trait-semantics 1` format preserves PA19's original
`semantic-crate 1` contract. The public suite contains 546 semantic inspections
(484 accepted, 62 rejected) and 797 full LowIR checks (483 accepted, 314 rejected).
Every accepted LowIR program executes successfully through both pinned rustc
1.98.0 and the qualified CPPGM backend, with clean stdout, stderr and exit zero.
There are no compilation-only or skipped accepted LowIR cases.

The later callable-definition correction adds two associated equality controls
under `contracts/equations/lowir`: an unused declaration with a nested callable
binder accepts, while demanding its incompatible equality rejects. The accepted
control also executes through both compilers. That checkpoint has 1,345 cases
from 799 sources, with 546 semantic inspections, 799 LowIR checks and 484 native
executions. Every earlier sidecar remains unchanged. Counts below record the
original release evidence; see [PA21 validation](pa21-validation.md) for this fix.

The parameter-default checkpoint adds ten LowIR controls under
`default-requirements/`, bringing the current suite to 1,355 cases from 809 sources:
546 semantic inspections, 809 LowIR checks and 489 native executions. Eligibility
uses the original bound: an erased projection lifetime still defers satisfaction,
while a lifetime introduced only by normalization does not. Parameter indexing
uses the normalized clause separately; an erased parameter creates no new default
obligation. Positive and negative pairs cover both directions, use-site debts and
satisfying implementations. All ten declared outcomes agree with pinned rustc
object generation, and all five positives execute with rustc and CPPGM. Full
independent qualification preserves the existing one semantic-phase exception.
The previous 1,345 cases keep their exact sidecar bytes. Evidence is in
`obj/pa21-work/parameter-defaults-final-v2`; see
[PA21 validation](pa21-validation.md) for the implementation and performance review.

The inferred-parent checkpoint adds 26 LowIR cases under
`selection/inferred-parents/lowir`, bringing the suite to **1,381 cases from
835 sources**: 546 semantic inspections, 835 LowIR checks and 508 native executions.
The new controls cover partially inferred qualified paths, retained parent
conditions, const arguments, dependent/Self defaults and nested callable scopes.
All 26 outcomes match pinned rustc object/executable generation; all 19 positives
execute through both compilers. Seven negative controls require the intended
unsatisfied-parent, unresolved-input or lifetime diagnostic in the maintainer
suite. All 4,065 earlier PA20 sidecars retain their bytes. Evidence is under
`obj/pa21-work/family-frontier-v1/parent-continuation-v1/`; see
[PA21 validation](pa21-validation.md) for the shared inference implementation.

## Language and phase boundaries

The seven families cover associated type formation and normalization, result
contracts and equality, constant defaults/overrides and dependencies, caller
selection, coherence, recursive obligations and integration with fields,
borrowing and destruction. The manifest records each source's owning feature,
maintainer origin, inspection mode and independently declared outcome.

A semantic inspection records typed decisions and pending lifetime facts before
complete region/ownership proof. `integration/inspection/semantics/pending.rs`
deliberately succeeds while its LowIR sibling and rustc reject an escaping
reference. Its manifest reason makes this acceptance difference explicit.
Every other fixture agrees with its declared independent source outcome.

Three intentionally unused erroneous constant initializers carry an explicit
`allow(unconditional_panic)` attribute. Their earlier maintainer tests supplied
that policy through a lint cap. The public fixtures retain all other default
lints; this is not a language acceptance exception. Two distinct selection
programs previously used the same test name; both source programs are preserved
under distinct manifest paths. These packaging corrections were made before
oracle generation and their first qualification failures remain recorded.

## Text contract and independent review

The writer consumes existing typed declaration facts and the sparse retained
original-type store. It adds no solver replay, source reparsing or duplicate
normalization history. Rows distinguish constant uses from completed dispatches,
retain complete generic arguments and preserve original types alongside normal
forms. Sparse map rows have explicitly sorted canonical identities. Count mode
reads the same facts without rendering or sorting.

Four new maintained tests check hand-derived member/selection identities, result
bounds, complete mixed arguments, early value dispatch, repeatability, count/full
agreement, empty counts, output failures and atomic diagnosed rejection. The new
native group executes three inspected programs. The 797-program qualification
also checks repeated semantic output, count/full agreement and native results.
Sixteen additional semantic rejection checks cover ambiguous projections and
method selection without changing the complete LowIR outcomes.

All 4,029 new oracle sidecars are generated only after these declarations pass
qualification. Review compares every stdout/stderr sidecar byte for byte with the
independently qualified run and checks every added row count against the emitted
rows. The generation receipt hashes compiler sources, manifest and all fixtures.
Earlier PA1–PA19 reference bytes are checked separately and are not regenerated.

Maintainer invariants check completed negative proof reuse, inductive-cycle
cutoffs, budget diagnostics, inference retries, disjoint proof scopes and retained
header/definition continuations. Scaling tests distinguish structural work from
caller-specific proof; no counter appears in the public output. The existing
projection and storage worklists share completed nodes and bound their charged
work. This does not assert whole-program linearity for arbitrary Rust programs.

## Scope and evidence

The bounded course profile remains edition 2024, Linux x86-64, panic abort,
overflow checks enabled and debug assertions disabled. PA21–PA25 remain planned:
higher-ranked trait premises, GATs, opaque types, full generic drop checking,
unsized execution, trait objects, closures and broader constant evaluation are
separate milestones. PA20 does not establish Cargo or real-library compatibility.

Qualification receipts and the preserved failed packaging pass are under
`obj/pa20-work/course-v1`, `course-v2` and `course-semantic-negatives`. Installation,
full validation and immutable performance receipts are under
`obj/pa20-work/course-final`. Raw observations, inputs, hashes and diagnostics are
retained; generated executables and intermediate objects are cleaned by the
storage guard. Only current comparison binaries remain pinned.

Full validation passes 1,278 regular maintainer tests, 91 native groups and all
6,218 PA1–PA20 course cases. The installed PA20 harness passes 1,343 independent
rustc checks and 797 complete LowIR checks, including 483 clean native executions.
Its one documented semantic lifetime exception remains explicit. Formatting,
strict Clippy, the 800-file audit, harness, backend and upstream checks pass.
All 803 frozen source/configuration hashes and all 18,654 oracle sidecars remain
fixed; PA21–PA25 still fail as planned assignments. The thirty completed gate
receipts and final candidate hash are recorded in `course-final`.

The validated candidate SHA-256 is
`fd2864f065b8c72a0e4f48ac92bb2b79bc0570fd8a6a84b8cccd6dc8775ba292`.

## Performance review

The immutable published `c13f64c` / `1cbe6b9c` binary is compared with candidate
`fd2864f0`. Twenty-one existing workloads cover ordinary/generic compilation,
recursive projections, lifetime/storage depth, shared callers, constant
initializers and pending headers. Their instruction deltas range from −0.06%
to +0.23%; median peak-RSS differences range from −350 to +196 KiB. All full/count
outputs match, and binary, source and input hashes remain fixed.

Main paired wall medians range from −5.36% to +6.14%, with same-binary controls
from −0.20% to +1.29% and maximum A/A spread 15.40%. Three positive deltas prompted
one declared follow-up with sixteen invocations per observation. The original
observations remain included; none of those increases reproduced at that size:

| Workload | Original A/B | Follow-up A/B | Follow-up A/A | Follow-up B/B |
| --- | ---: | ---: | ---: | ---: |
| Recursive projections, 64 | +6.14% | +0.97% | +0.17% | −0.02% |
| Nested declaration depth, 48 | +4.85% | −0.25% | −0.55% | +0.87% |
| Projected storage depth, 96 | +4.75% | −1.89% | +1.71% | +0.64% |

These shared-host timings establish no portable speedup. The instruction results
show no substantial increase in the work of the existing compilation paths.

The new inspection format has no published-format baseline. Independently
qualified pending-projection chains measure its scaling on the candidate:

| Depth | Count wall, ms | Full wall, ms | Count instructions, M | Full instructions, M |
| --- | ---: | ---: | ---: | ---: |
| 8 | 9.13 | 8.97 | 5.79 | 6.11 |
| 32 | 12.74 | 12.94 | 15.62 | 16.43 |
| 96 | 22.27 | 22.87 | 40.88 | 43.03 |

Wall values are per invocation from median batched observations and include
process startup and source reading. Instruction growth is approximately linear
across these samples, consistent with the retained-query invariants; it does not
prove a complexity bound for arbitrary programs.

The main study retains 806 timing and 124 qualification observations with four
invocations each, across 31 wall jobs and 27 instruction comparisons. The focused
study retains 234 timing and 36 qualification observations with sixteen
invocations each. Every observation and all equality/hash checks are preserved.
Full validation cleanup removed 6.90 GiB of disposable artifacts. The prototype
binary was retired after transfer to the matching final validated pin.
