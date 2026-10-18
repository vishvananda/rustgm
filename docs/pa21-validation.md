# PA21 implementation evidence

PA21 remains **planned**. Its handout describes the required higher-ranked
relationships, associated type families, opaque types, variance and generic drop
checking. The proposed generic-semantic inspection mode is not implemented yet;
there is no public PA21 manifest or generated grading contract. PA1–PA20 retain
their released modes and exact text oracles.

## Quantified clause representation

The first implementation checkpoint adds a canonical `Clause` with distinct
ordinary-predicate and quantified variants. A quantified identity owns its local
lifetime count and the whole predicate: subject, trait application, outlives
region or equality result. Storage is allocated only when a quantified clause is
interned. Canonical identity is representation, not a trait proof.

Enclosing substitution preserves the clause's local binder, including empty
binders, and transforms its complete inputs at one consistent depth. Inserted
type arguments shift beneath both the predicate binder and nested callable
binders. Ordinary and scoped application caches remain separate; the same
canonical input can have different results at different depths.

Binder validation shares memoized required-slot summaries across different
clauses with common type graphs. It compares that summary with each clause's
own count. The first draft used a fresh occurrence traversal for every clause;
that could repeatedly scan growing shared tails. The implementation instead
shares the existing capture-query traversal with a separate maximum-slot summary
operation. Ordinary capture queries retain boolean summaries and their existing
short circuit. No summary authorizes trait selection or lifetime escape.

`dev/tests/quantified_predicates.rs` adds six invariant tests for canonical
identity and invalid slots, mixed lifetime/type/const substitution, both cache
encounter orders, projection equalities and outlives inputs, empty binders and
missing arguments, and repeated wide applications. Two unit tests in
`dev/src/types/parameters/clauses/slot_validation.rs` check both predicate sides,
independence from previous binder counts, and growing shared tails at 64, 128,
256 and 512 clauses. The latter bounds work linearly and verifies that doubling
the chain does not double a rescan of every preceding tail.

## Independently qualified source requirements

Before this checkpoint, fourteen complete source programs were qualified against
the pinned Rust 1.98/edition-2024 profile. Rustc accepts and cleanly executes ten
positives: parameter bounds, both where-binder positions, caller forwarding,
reference subjects, independent slots, enclosing captures, implementation
conditions, result bounds and associated equality. Four negative controls reject
static-only evidence, an ordinary premise used universally, incorrectly correlated
slots and a result lifetime escaping its permitted input.

The published PA20 compiler rejects all fourteen at its explicit higher-ranked
feature boundary. The ten positives are implementation gaps; early rejection of
the four negatives does not prove their intended lifetime or trait failure.
Sources, predeclared outcomes and independent diagnostics remain in
`obj/pa21-work/quantified-premises-v1/` for the subsequent solver work. These are
development requirements, not passing PA21 course cases.

## Remaining proof integration

Declaration environments must retain the explicit quantified carrier. The
canonical universal-goal kernel described below retains scoped universals and
isolates inference. Using a universal assumption still needs fresh instantiation
in the actual caller scope.
Normalization, pending implementation headers, selected evidence and concrete
instance witnesses must retain those scopes. Existing unsupported-feature gates
remain until their consumers implement those obligations. GATs, opaque capture
and the complete PA21 inspection contract remain later work within this stage.

## Checkpoint validation

The candidate passes all 28 validation gates: 1,286 regular maintainer tests,
91 explicit native groups and 6,218 exact PA1–PA20 course cases. Formatting,
Clippy all/pedantic with warnings denied, the 803-file architecture audit,
harness, backend and upstream checks pass. All 806 frozen source/configuration
hashes and 18,654 earlier oracle sidecars remain unchanged. PA21–PA25 still fail
as planned stages.

The frozen candidate SHA-256 is
`7c12528f948ab00d88fb03980fe1d6fd864c222af1ae9769f9c59655215db5a1`.
Receipts are under `obj/pa21-work/binder-carrier-final/`. Immutable comparisons
against published PA20 `ef7021c` use its pinned binary
`fd2864f065b8c72a0e4f48ac92bb2b79bc0570fd8a6a84b8cccd6dc8775ba292`.
No PA21 source compile-time claim is made because the source proof integration
remains incomplete.

## Performance and memory

The immutable comparison preserves every input and all full/count outputs.
Twenty-seven existing workloads cover ordinary and generic LowIR, recursive
projections, captured lifetimes, storage, shared callers, constant dependencies,
pending headers and PA20 semantic views. Four same-binary jobs supply additional
A/A and B/B controls. Each timing job has warmups, four A/A calibration
observations and five paired ABBA blocks; the main batch size is four.

- Instruction changes across the 27 comparisons: **−0.12% to +0.49%**.
- Median paired wall changes: **−5.04% to +3.14%**.
- Same-binary median changes: **−1.86% to +9.61%**; the largest initial A/A spread
  is **28.52%**. Shared-host wall time does not establish a portable speedup.
- Median peak RSS changes: **+186 to +314 KiB** across the compared workloads.

The predeclared policy triggers one focused follow-up for each of at most three
largest AB increases above 3%. Only `constant-chain-uses-256` qualifies. Its
original **+3.14%** becomes **−0.25%** in the single follow-up, with **+0.53% A/A**
and **−1.19% B/B** controls at batch sixteen. The increase did not recur. Its
focused RSS delta remains **+172 KiB**, so the memory observation is retained.
No observations were discarded and no run was repeated until favorable.

Two additional Massif pairs investigate the consistent RSS increase. These are
instrumented heap/stack profiles, not RSS measurements or timing comparisons:

| Workload | Baseline peak heap bytes | Candidate peak heap bytes | Baseline combined heap/overhead/stack peak | Candidate combined peak |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,245 | 2,432,253 | 2,524,288 | 2,524,208 |
| constant-chain-uses-256 | 5,382,057 | 5,381,905 | 5,544,256 | 5,543,848 |

Peak heap and combined storage are essentially unchanged on these two inputs.
The ELF text/data/BSS sum grows by 12,236 bytes; this does not establish the cause
of the larger RSS increase. Executable residency was not directly measured.
The retained RSS result must not be presented as either proved heap growth or
an explained-away measurement artifact.

The two timing studies retain **884 study observations and 136 qualification
observations**, comprising 4,472 timed compiler invocations and 688 qualification
invocations. There are 27 instruction comparisons and four additional Massif
executions. All binary, input and 806 source/configuration hashes remain fixed;
every compared output is equal. Receipts, per-invocation observations, profiles
and the predeclared follow-up policy remain under `binder-carrier-final/`.
Full validation cleanup reclaimed 6.90 GiB; only the current baseline and candidate
remain pinned until publication and exact-head CI permit retirement.

## Scoped universal-obligation implementation

The next checkpoint connects the explicit clause to canonical declaration/instance
trait selection. It substitutes enclosing arguments beneath the local binder,
opens the whole predicate with private aliases, and selects in a fresh inference
context. The retained lifetime proof puts those aliases and all implementation
witnesses beneath the corresponding universal scope. Selected equations still
need the ordinary directed lifetime proof; selection alone does not establish
that a static-only implementation works for every lifetime.

A lazy cache reuses a completed selection only within the same declaration state,
substituted clause, closed/open environment mode and implementation-availability
mode. Its pending lifetime proof stays in that state, including debts that will
fail final verification. Repeated selection does not allocate another scope or
replay a recursive implementation chain. Absent/deferred selections are not
cached. Ordinary states allocate no cache. Root builtin requirements now retain
explicit builtin provenance, which cannot authorize a user-trait method.

Ten invariants in `dev/tests/invariants/quantified_obligations.rs` exercise actual
lifetime-proof discharge: general, static-only, restricted and reflexive impls;
independent slots and enclosing captures; empty binders; ordinary versus universal
premises; closed/open and separate caller caches; a 16-level implementation chain
queried 512 times; type-outlives requirements; correlated reference subjects;
associated equality results; and nested callable binders. The repeated-chain test
checks one retained scope and bounded private identities without canonical type
growth. Invalid cached results still fail lifetime verification.

Twenty-five full programs independently qualify the corresponding intended rules
with pinned rustc object compilation: twelve accept and thirteen reject. These
are source controls for the kernel invariants, not RustGM source acceptance or
native execution coverage. The initial additional equality controls contained an
unrelated explicit late-bound lifetime call and hit E0794; corrected controls
infer that lifetime from a reference argument, preserving their intended outcomes.
Both attempts and diagnostics remain under
`obj/pa21-work/scoped-obligations-v1/`.

This route requires canonical inputs and an explicit caller scope. It rejects
scoped alias Sized retention instead of flattening those obligations. Source
bound collection, universal caller-assumption instantiation, pending scoped
normalization and concrete instance witness import still need integration.
No source feature gate or course oracle changes in this checkpoint.

All 28 validation gates pass: 1,296 regular maintainer tests, 91 native groups,
6,218 exact earlier course cases, strict Clippy/formatting, the 805-file audit with
zero findings, harness, backend and upstream checks. All 808 frozen source/config
hashes and 18,654 oracle sidecars remain unchanged; PA21–PA25 fail as planned.
The frozen candidate SHA-256 is
`cc023bac2ba771e9050ffe5cb4a8b5d831177f839397590a0033d07f7fef83cf`.
Receipts are under `obj/pa21-work/scoped-obligations-final/`; the baseline is
published carrier `b1410a8`, binary `7c12528f948ab00d88fb03980fe1d6fd864c222af1ae9769f9c59655215db5a1`.
The main immutable comparison passes all 31 timing jobs and 27 instruction
comparisons. Instruction changes range from −0.13% to +0.60%, paired wall medians
from −3.01% to +3.69%, and median peak RSS deltas from +36 to +234 KiB.
Same-binary medians range from −1.98% to −0.09%; the largest A/A spread is 15.95%.
All full/count outputs and frozen hashes agree. The main study retains 806 study
and 124 qualification observations at batch four.

Two wall increases trigger the predeclared single follow-up at batch sixteen:

| Workload | Original paired wall change | Follow-up AB | Follow-up A/A | Follow-up B/B | Instruction change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Constant-chain uses, 256 | +3.69% | +2.16% | −0.47% | −0.12% | +0.10% |
| Streamed proofs, 48 | +3.06% | +4.32% | −2.30% | +0.26% | +0.24% |

The streamed-proof result remains above the 3% investigation threshold. Its five
follow-up AB blocks range from −3.02% to +5.04%; the corresponding same-binary
paired blocks range from −25.31% to +4.28%. This noisy host does not establish the
cause of the positive wall median. Instruction counts alone do not rule out a
wall-time regression; no throughput-parity claim is made for this workload.
There is no second follow-up or selective removal of observations.

The two fixed Massif heap/stack comparisons retain these measured peaks:

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,253 | 2,432,253 | 2,524,208 | 2,524,224 |
| constant-chain-uses-256 | 5,381,905 | 5,382,073 | 5,543,848 | 5,543,584 |

Storage peaks are essentially unchanged on these two inputs. The ELF text/data/BSS
sum grows by 28,692 bytes. Neither result establishes the cause of the +36..234 KiB
RSS observations; focused RSS deltas remain +186 KiB and +78 KiB, respectively
for constant-chain uses and streamed proofs.

All outputs and source/input/binary hashes agree. The combined studies retain
962 study observations and 148 qualification observations, comprising 5,720 timed
and 880 qualification compiler invocations, plus 54 Callgrind and four Massif
executions. Full validation/performance cleanup reclaimed 6.90 GiB. These are
ordinary-source regression measurements; quantified source integration remains
incomplete. The repeated-clause invariant does not establish scaling for many
distinct quantified goals with growing shared type graphs.

## Universal assumptions and immutable environments

The next checkpoint implements the other direction of higher-ranked reasoning:
using a universal caller premise. Each use opens its whole predicate with fresh
existential aliases at the caller's actual region frontier. Candidate inference
is isolated until a unique complete application wins. The retained evidence
contains the original quantified clause, instantiated subject and complete trait
application, with the selected private interval and ordinary lifetime equations.
It creates no new universal proof scope. Body validation verifies that the clause
belongs to the same caller before using that evidence.

Assumption indexes retain ordinary and quantified competitors. Flat entry points
fail explicitly when they cannot handle the quantified environment; exact ordinary
and projected-subject shortcuts cannot silently omit a quantified competitor.
Unknown Self remains deferred. A unique premise may infer type/const arguments,
while ambiguous probes leave those variables and lifetime equations untouched.
Child conditions in implementation search use the current allocation frontier,
including after header waits that interleave unrelated allocations.

Immutable declaration environments now expose whole trait clauses. Quantified
entries use lazy storage alongside the existing compact ordinary predicates;
mixed declaration order survives substitution. Local slots, empty binders and
nested callable binders remain intact while outer captures map to caller aliases.
Type/const membership uses one shared occurrence walk across the batch. Constant
time cached region summaries distinguish local slots from captures. Quantified
builtin/equality assumptions remain explicitly unsupported at this boundary.

Sixteen new invariants in `universal_premises.rs` and `premise_scopes.rs` check
actual lifetime-proof discharge, repeated use, independent/correlated slots,
captures, complete reference subjects, ambiguity, foreign callers, declaration
order, projection competitors, type/const inference, suspended header ownership
and malformed/unsupported inputs. Two growth families at widths 64, 128, 256 and
512 bound shared-tail opening or environment-mapping work and canonical storage.
These prove sharing within an assumption query or environment substitution;
they do not resolve the separate distinct universal-goal growth question above.

Nineteen full programs independently qualify the intended rules with pinned
rustc object compilation: eleven accepted and eight rejected. The initial twelve
controls and seven inference/implementation controls are retained under
`obj/pa21-work/quantified-assumptions-v1/`. They are kernel controls, not RustGM
higher-ranked source acceptance or native execution coverage.

Source declaration producers, scoped normalization continuations, quantified
implementation conditions and concrete instance witness import remain incomplete.
PA21 stays planned, with no changed source gate, public CLI or earlier oracle.
All 28 functional validation gates pass: **1,312 regular maintainer tests**,
**91 native groups**, **6,218 exact PA1–PA20 cases**, formatting, strict Clippy,
the **810-file audit with zero findings**, harness, backend and upstream checks.
All **813 source/configuration hashes** and **18,654 earlier oracle sidecars**
remain unchanged, and PA21–PA25 still fail as planned. The frozen candidate is
`0de3223807e3c80b281d84a6114dfc3aad9f1856087aa6bfed289416da2b3f94`,
compared with published `8664ca0` / `cc023bac2ba771e9050ffe5cb4a8b5d831177f839397590a0033d07f7fef83cf`.
The immutable comparison completes all 31 timing jobs and 27 instruction pairs.
Instruction changes range from **−0.22% to +0.38%**; no workload crosses the 1%
instruction investigation threshold. Main wall medians range from **−4.86% to
+5.61%**, with same-binary medians from **−0.85% to +1.63%** and a largest A/A
spread of **20.42%**. Median peak RSS deltas range from **+34 to +220 KiB**.
All full/count outputs and binary/input/source hashes agree.

Two wall increases trigger exactly one predeclared follow-up at batch sixteen:

| Workload | Main paired wall change | Follow-up AB | Follow-up A/A | Follow-up B/B | Instruction change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Recursive Copy proofs, 64 | +5.61% | +0.004% | +1.79% | +0.03% | +0.31% |
| Inherent nominal types, 256 | +4.22% | −0.73% | +0.95% | −0.74% | −0.03% |

Neither increase recurs. Every original observation remains in the study; there
are no additional timing repeats or discarded samples. The same-binary controls
do not support a portable speedup claim. Focused RSS deltas remain +84 KiB and
+90 KiB, respectively. This comparison does not resolve the preceding checkpoint's
separate streamed-proof wall-time observation against its own baseline.

The two predeclared Massif comparisons retain these measured storage peaks:

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,253 | 2,432,261 | 2,524,224 | 2,524,192 |
| constant-chain-uses-256 | 5,382,073 | 5,382,097 | 5,544,272 | 5,544,032 |

Paired heap/stack peaks are essentially unchanged on these inputs. The ELF
text/data/BSS total grows by 28,680 bytes. Neither measurement establishes the
cause of the larger peak RSS observations; executable residency was not measured.
The studies retain **962 timing observations and 148 qualification observations**,
comprising **5,720 timed and 880 qualification compiler invocations**, plus 54
Callgrind and four Massif executions. All 813 source/configuration hashes and
compared outputs remain fixed. Full validation cleanup reclaimed 6.90 GiB.
Receipts and raw observations are under
`obj/pa21-work/universal-environments-final/`.
