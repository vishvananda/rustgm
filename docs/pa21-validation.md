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


## Distinct universal goals and shared structural keys

A growing family of distinct `for<'a> u8: A<TailN<'a>>` obligations exposed two
quadratic costs. At 16, 32, 64 and 128 goals, the published kernel charged
1,856, 5,248, 16,640 and 57,856 steps and created 152, 560, 2,144 and 8,384
canonical types. All semantic and final lifetime checks succeeded; the new
linear-growth invariant failed. Sharing only the universal opening reduced type
growth to 32, 64, 128 and 256 but left work at 1,361, 3,233, 8,513 and 25,217.
Every independent selection was still rebuilding its structural key graph.

The candidate shares openings only for independent conjunctive requirements in
one immutable State, grouped by arity and proof mode. These scopes add no local
hypotheses. Selections keep private inference and fresh witnesses, and commit a
new scope only after success. Empty binders, outer captures and caller permission
remain explicit. All selected lifetime debts remain obligations for the final
proof, including on a completed-query hit.

Trait index and cycle keys now share a lazy canonical-store fold. Its opaque key
cannot be used as a predicate. Actual success/failure answers, active cycles,
caller assumptions and header availability stay outside this cache. The original
one-million-step cumulative folding limit bounds its derived storage, and each
request charges the incremental transformation work. A failed request restores
the previously completed structural cache.

With both changes, the reference-tail family takes 1,041, 2,081, 4,161 and 8,321
steps and creates 32, 64, 128 and 256 types. At 256 and 512 goals, the expanded
probe takes 16,641 and 33,281 steps and creates 512 and 1,024 types. This is
linear growth for this family, not a claim about the whole frontend.

A separate nested-callable family exposed unnecessary binder-depth entries in
the lifetime-erasure fold: 488, 1,744, 6,560 and 25,408 steps at widths 16 through
128. Erased keys are context-free, so only that fold now canonicalizes its cache
depth. Capture-sensitive substitution continues to distinguish binder depth.
The corrected fold takes 98, 194, 386 and 770 steps on the same family,
with a retained invariant enforcing linear work.

Nine added invariants cover the two growth families, independent structural-key
requests, malformed requests, complete non-lifetime inputs, goal order, unpaid
static debts, independent/captured slots, empty and mixed-arity scopes, nested
callable captures and interleaved absent/ordinary goals. Existing caller,
availability, cycle and suspension tests remain required. The focused run passes
all 146 library invariants, strict Clippy, formatting,
the 813-file audit with zero findings and the harness checks. Twelve pinned
rustc object controls match their independently declared outcomes: eight accept
and four reject. The original capture negative also had an unrelated explicit
late-lifetime argument error; its source
and receipt remain retained, and a corrected control plus positive caller were
qualified separately; the corrected rejection reports only the intended E0277.

PA21 remains planned. This does not activate HRTB source support or establish
native execution of these source controls. Probes and raw observations are under
`obj/pa21-work/universal-growth-v1/`; the full immutable regression study is
retained under `obj/pa21-work/universal-growth-final/`.


Full functional validation passes all 28 gates: **1,321 regular tests**, **91
native groups**, **6,218 PA1–PA20 exact cases**, formatting, strict Clippy,
the **813-file audit with zero findings**, harness, backend and upstream checks.
All **816 source/configuration hashes** and **18,654 oracle sidecars** remain
fixed. PA21–PA25 still fail explicitly as planned. The frozen candidate is
`d5d955a9eed168e09a9dae94a5b33fba1ef09423773e4146edcad887f0508f6f`,
compared with published `19a33f7` /
`0de3223807e3c80b281d84a6114dfc3aad9f1856087aa6bfed289416da2b3f94`.
The immutable study completes all 31 timing jobs and 27 instruction pairs.
Instruction changes range from **−1.28% to +0.29%**, with no increase above the
1% investigation threshold. Main paired wall medians range from **−5.74% to
+2.19%**; none crosses the predeclared 3% follow-up threshold. Same-binary
medians range from **−1.45% to +1.05%**, and the largest A/A spread is **17.46%**.
Median peak RSS deltas span **−116 to +100 KiB**. All outputs and binary, input
and source hashes match. No extra wall runs were selected by the fixed policy.

The two predeclared Massif comparisons record these peaks:

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,261 | 2,432,261 | 2,524,192 | 2,524,256 |
| constant-chain-uses-256 | 5,382,097 | 5,381,921 | 5,544,032 | 5,544,280 |

These two storage peaks are essentially unchanged. The ELF text/data/BSS sum
increases by 16,344 bytes. Neither measurement establishes the cause of the
−116..+100 KiB RSS range or a portable wall-time improvement.

The study retains **806 timing observations** and **124 qualification
observations**, comprising **3,224 timed** and **496 qualification compiler
invocations**, plus **54 Callgrind** and **four Massif** executions. No observations
were discarded and no measurements were repeated until favorable. All source,
input, output and binary hashes agree. Full-batch cleanup reclaimed **6.90 GiB**.
Receipts and complete observations remain in
`obj/pa21-work/universal-growth-final/`. This comparison uses `19a33f7` as its
baseline; it does not resolve the earlier streamed-proof wall-time effect against
`b1410a8`. Source formation, declaration producers, scoped implementation
conditions and concrete instance witness integration remain required before
PA21 source activation.


## Quantified declaration carriers and supertrait composition

The next checkpoint carries whole clauses through generic declarations, retained
original predicates, trait headers and body-environment capture substitution.
Supertrait closure composes adjacent universal binders without dropping their
subjects, trait inputs, equality results or enclosing captures. Associated-result
classification checks the complete captured formal parent. Flat declaration,
conformance, nominal and body consumers explicitly reject quantified input until
they have a scoped implementation. Source syntax gates remain closed, and PA21
remains planned.

Four composition invariants and six carrier/ownership invariants cover multiple
universal slots, empty binders, nested callables, non-Self requirements, caller
isolation, application memoization at different depths and malformed parent slots.
The isolated composition family uses `6n+1` folder steps plus `2n` store validation
steps and creates `n` canonical types through width 512.

A broader supertrait expansion test found an additional quadratic walk: each
parameter mapping independently shifted the same growing argument tails beneath
an inner binder. At widths 16, 32, 64 and 128 this cost 528, 1,568, 5,184 and
18,560 contract steps (`n²+17n`). Moving context-free shifts to a lazy canonical
store cache reduces this family to `18n+1` steps: 289, 577, 1,153, 2,305, 4,609
and 9,217 through width 512, with `n` new canonical types. Three shift invariants
check distinct amounts, nested callable locals, simultaneous parameter mappings
and retained state after failures. These work counts are internal scaling
measurements, not whole-frontend speedups.

Eleven independent pinned-rustc object controls qualify seven accepted and four
rejected sources. An initial negative expectation for diagonal use was corrected:
a universal lifetime can be instantiated with an enclosing lifetime; the reverse
does not prove independence. A separate generic reference-formation example
receives rustc's explicit current-type-system-limitation/implied-static diagnostic.
Its language/profile meaning remains an investigation rather than a declared
invalid fixture. The original controls and diagnostics are retained alongside the
corrected set in `obj/pa21-work/quantified-contracts-v1/`.

All 28 full functional gates pass: **1,335 regular tests**, **91 native groups**,
and **6,218 PA1–PA20 text-oracle cases**, plus formatting, strict Clippy, the
820-file audit, harness, backend qualification and upstream verification. All
**823 source/configuration hashes** and **18,654 oracle sidecars** remain fixed;
PA21–PA25 still fail explicitly as planned. The frozen candidate is
`af86f1d5dc81d8532cbae134ddadbcd94452e52badb661b955a6afa459098cc9`,
compared with published `76587c4` / `d5d955a9eed168e09a9dae94a5b33fba1ef09423773e4146edcad887f0508f6f`.
The main study completes all 31 timing jobs and 27 instruction pairs. Instruction
changes range from **−0.59% to +0.24%**, with no increase above the 1% investigation
threshold. Paired wall medians range from **−18.99% to +5.68%**; same-binary medians
range from **−2.74% to +2.27%**, with a largest A/A spread of **131.24%**. The large
apparent wall improvements are not reliable speedup evidence. Median peak RSS
deltas span **−164 to +22 KiB**. All output and binary, input and source hashes match.

The predeclared follow-up selects `trait-view-96-count` (+5.68%), `pending-chain-8`
(+5.05%) and `trait-view-8-count` (+4.19%) for one AB/AA/BB batch each. The follow-up
AB changes are **−0.17%, +1.31% and +1.90%**, respectively. Corresponding same-binary
AA/BB medians are +0.24%/+0.16%, +0.92%/−0.83% and +0.19%/−2.45%. The initial
above-threshold increases do not recur in this fixed batch. No additional timing
repeats were selected.

The two fixed Massif comparisons record:

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,261 | 2,432,261 | 2,524,256 | 2,528,240 |
| constant-chain-uses-256 | 5,381,921 | 5,381,873 | 5,544,248 | 5,540,008 |

Heap peaks are unchanged or 48 bytes lower; combined peaks change by +3,984 and
−4,240 bytes. The ELF text/data/BSS sum increases by 24,592 bytes. These measurements
do not establish the cause of the RSS range or a portable wall-time improvement.

All **1,040 study observations** and **160 qualification observations** are retained,
comprising **6,968 timed** and **1,072 qualification compiler invocations**, plus
**54 Callgrind** and **four Massif** executions. All source, binary, input and output
hashes agree; no observations were discarded or repeated until favorable. Main-run
cleanup reclaimed **6.90 GiB**. Receipts remain in
`obj/pa21-work/quantified-contracts-final/`. This comparison does not resolve the
earlier streamed-proof wall-time effect against `b1410a8`. These functional and
regression results do not establish end-to-end RustGM HRTB source or native
acceptance; scoped implementation conditions, formation and concrete body witness
integration remain required before source activation.

## Scoped implementation conditions

Implementation selection now retains each quantified condition as a whole clause.
Ordinary trait and equality conditions settle first; the parent application and
all type/const arguments are then resolved once. Universal children have isolated
inference contexts on the existing iterative search stack. Their local witnesses
cannot refine a parent hole. Candidates whose parameters can be determined only
through a quantified equality remain deferred; this is a remaining capability,
not an independently established language rejection.

A candidate shares immutable substitution and opening work across conjunctive
conditions of the same arity and parent scope. Different candidates retain distinct
universes. Scope-aware completed answers preserve their debts, while an empty
quantifier still prevents a containing proof from entering the flat cache. Inductive
cycle cutoffs remain stack-relative. Header waits preserve the exact child cursor,
universe and allocation frontier, excluding allocations from other work.

Sparse node ownership and an ordered allocation ledger retain only the selected
graph's scoped witnesses and constraints. Universal allocation becomes monotonic
so an alternative cannot reuse a discarded alias. The reserved envelope can include
unreferenced identities from attempts; discarded equalities and permissions are
never published. Proof assembly uses linear passes over scopes, nodes and recorded
allocations, without sorting or repeatedly copying ancestor proofs. Reserved block
placement checks ordering, overlap, scope identity and foreign gaps.

Eighteen added library invariants cover universal/static-only implementations,
independent slots and captures, nested conditions, ordinary equality inference,
empty binders, pending headers, discarded alternatives, shared and separate
universes, cycles, caller assumptions, associated results, outlives conditions,
reserved ownership and structural growth. All **178 library invariants**, strict
Clippy and the **828-file audit with zero findings** pass. Sixteen independently
object-qualified rustc controls match their predeclared outcomes: **eight accepted
and eight rejected**. The controls and diagnostics are retained in
`obj/pa21-work/implementation-quantifiers-v1/{source-controls,clause-controls}/`.
These typed solver invariants and rustc controls are not RustGM HRTB source or
native execution coverage.

| Family | Sizes | Charged selection work | Additional canonical storage |
| --- | --- | --- | --- |
| Distinct shared reference tails | 16–512 | `65n + 30` | `2n` types |
| Wide parent argument lists and condition counts | 8–128 | `53n + 73` | Parent resolves once |
| Nested universal implementation conditions | 8–128 | `61n + 44` | `n` scopes; region frontier `2n + 2` |

These bounds describe internal work and storage, not a whole-frontend speedup.
All 28 full functional gates pass: **1,353 regular tests**, **91 native groups**,
and **6,218 PA1–PA20 text-oracle cases**, plus formatting, strict Clippy, the
828-file audit, harness, backend qualification and upstream verification. All
**831 source/configuration hashes** and **18,654 oracle sidecars** remain fixed;
PA21–PA25 still fail explicitly as planned. The frozen candidate is
`8bfcf38b30b22cc6f723896f64b87cd0f1f860ae265464096fb9b1036ef76618`.
Its immutable comparison with published `39cad8c` /
`af86f1d5dc81d8532cbae134ddadbcd94452e52badb661b955a6afa459098cc9`
completes all 31 timing jobs and 27 instruction pairs. Instruction changes range
from **−0.18% to +0.44%**, with no increase above the 1% investigation threshold.
Main paired wall medians range from **−9.36% to +8.68%**, while same-binary medians
range from **+0.10% to +6.06%**. The largest A/A spread is **72.77%**; this study
does not establish a portable wall-time speedup. Median peak RSS deltas span
**−14 to +420 KiB**. All output and source, binary and input hashes match.

The predeclared follow-up selects `generic-reference-256` (+8.68%),
`constant-chain-uses-256` (+4.26%) and `trait-view-32-count` (+3.07%) for one
AB/AA/BB batch each. Follow-up AB changes are **−0.70%, −2.16% and −1.87%**.
Corresponding AA/BB changes are +0.33%/+0.74%, +4.53%/−0.24% and +0.001%/+0.44%.
None of the initial positive threshold crossings recurs in that fixed batch;
no additional timing repetitions were selected.

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,261 | 2,432,261 | 2,528,240 | 2,528,240 |
| constant-chain-uses-256 | 5,381,873 | 5,381,873 | 5,540,072 | 5,539,976 |

Both heap peaks are unchanged; combined peaks are unchanged or 96 bytes lower.
The ELF text/data/BSS sum increases by **40,984 bytes**. These two fixed measurements
do not establish the cause of the RSS range on the broader input set.

All **1,040 study observations** and **160 qualification observations** are retained,
comprising **6,968 timed** and **1,072 qualification compiler invocations**, plus
**54 Callgrind** and **four Massif** executions. No observations were discarded or
repeated until favorable. Receipts remain in
`obj/pa21-work/implementation-quantifiers-final/`. Cleanup reclaimed **6.90 GiB** from 5,754 generated files. This comparison does not resolve the earlier streamed-proof
wall-time effect against `b1410a8`. Source gates, scoped Sized retention,
formation/conformance consumers and concrete-body witness integration remain
explicit boundaries. PA21–PA25 remain planned.


## Quantified nominal formation requirements

Immediate nominal requirements now retain whole clauses. Owner substitution maps
complete lifetime/type/const arguments under the requirement's own binder; an
empty quantifier still establishes a binder depth. Callable deferral inspects
captures outside that binder. A local of the quantified requirement cannot hide
an independent caller obligation, while an actual surrounding callable local
retains the existing deferred-formation rule. Independent ancestor captures lower
through callable scopes without capturing or discarding the quantifier.

Occurrence queries retain compact ordinary application keys and allocate a
separate depth-keyed table only for scoped clauses. The same DAG traversal visits
subjects, associated equality results, type-outlives bounds and nested callable
arguments. Nominal proof selection retains universal evidence for the final
lifetime gate and rejects quantified obligations without an explicit caller scope.
Quantified outlives clauses are not omitted by the separate ordinary lifetime
contract path. Body consumers without scoped support still reject them explicitly.

Seven added invariants cover universal versus static-only implementations,
required explicit scopes, independent obligations under callables, local deferral,
ancestor lifting through empty/nonempty binders, mixed-depth cache queries, all
clause roots, quantified outlives and growing shared argument tails. Eight pinned
rustc object controls match their predeclared outcomes: four accepted and four
rejected. Sources and diagnostic receipts remain under
`obj/pa21-work/nominal-clauses-v1/{source-controls,outlives-controls}/`.
For widths 16–512, requirement traversal/lifting counts `12n+6` steps and
parameter substitution counts `5n+2`, for combined measured work of `17n+8`.
Additional canonical type storage is `2n`; a repeated demand adds no traversal
or substitution work. These counters do not represent all frontend work or a
whole-compile timing claim.

These are typed formation tests and independent language controls, not RustGM
HRTB source/native coverage. PA21 remains planned. All 28 full functional gates
pass: **1,360 regular tests**, **91 native groups**, and **6,218 PA1–PA20 exact
text-oracle cases**, plus formatting, strict Clippy, the 829-file audit, harness,
backend and upstream checks. All **832 source/configuration hashes** and **18,654
oracle sidecars** remain unchanged; PA21–PA25 discovery still fails as planned.
The frozen candidate SHA-256 is
`1f9f6c170c6494d8cca516e6fa29e2837566ba7e02408a4c3be575b7ebd44229`.
Its immutable performance comparison with published `cd23188` /
`8bfcf38b30b22cc6f723896f64b87cd0f1f860ae265464096fb9b1036ef76618`
completes all 31 timing jobs and 27 instruction pairs. Instruction changes span
**−0.19% to +0.40%**, with no increase above the 1% investigation threshold.
Paired wall medians span **−5.87% to +2.90%**, while same-binary medians span
**−0.17% to +0.47%**. No paired wall increase crosses the predeclared 3% threshold,
so the fixed policy selects no follow-up timing runs. The largest A/A spread is
**66.63%**; this does not establish a portable speedup. Median peak RSS deltas span
**−116 to +136 KiB**. Output bytes and source, binary and input hashes all match.

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,261 | 2,432,261 | 2,528,240 | 2,528,288 |
| constant-chain-uses-256 | 5,381,873 | 5,381,873 | 5,540,104 | 5,540,200 |

Both fixed heap peaks are unchanged. Combined peaks increase by 48 and 96 bytes;
the ELF text/data/BSS sum increases by 4,076 bytes. These samples do not explain
the broader RSS range. All **806 study observations** and **124 qualification
observations** remain retained, comprising **3,224 timed** and **496 qualification
compiler invocations**, plus **54 Callgrind** and **four Massif** runs. No
observations were discarded or repeated until favorable. Receipts remain in
`obj/pa21-work/nominal-clauses-final/`. Cleanup reclaimed **6.90 GiB** from 5,751
generated files. This comparison does not resolve the earlier
streamed-proof wall-time effect against `b1410a8`. Whole-clause declaration
formation, ambiguity, nominal outlives propagation, conformance and concrete-body
integration remain required before source activation. PA21–PA25 remain planned.


## Declaration formation and nested callable definition bounds

Quantified trait formation now has its own cached structural result, separate
from logical implication and implementation selection. It defers each whole
subject/type argument containing the clause's local lifetime and lifts independent
inputs, requirements and region pairs to the surrounding owner. Independent trait
arguments remain checked even when Self contains a local. Nested requirement and
callable binders retain their identities, including empty binders. Ordinary
queries reuse the existing requirements cache; the scoped cache is allocated on
demand and invalidated with completed header bindings.

Five typed invariants cover subject/argument independence, captured versus local
requirements, nested and empty binders, cache invalidation, exhausted budgets and
shared tails. Widths 16–512 take exactly `14n` counted steps and `n` new canonical
types in this family; repeated requests add no traversal work. These are internal
query counters, not whole-frontend complexity or source-level HRTB coverage.

The pinned compiler investigation corrected five initial expectation mismatches.
In particular, an unused nonempty binder defers a false global bound, whereas an
empty binder does not; and a local-dependent quantified trait subject defers its
whole formation input, even if it contains independent nominal requirements.
An explicit nominal type used in a body still requires its normal proof. The
original expectations and diagnostics are retained. Revised controls reuse all
26 original sources and add five independent-input cases: all 31 match pinned
rustc object outcomes (17 accepted, 14 rejected). The exact pinned implementation
supports the distinction between [trait-input formation](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_trait_selection/src/traits/wf.rs#L417)
and [false global bounds](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_hir_analysis/src/check/wfcheck.rs#L2379).
Source hashes and investigation receipts are retained in
`obj/pa21-work/formation-v1/`. The earlier implied-static reference-formation
qualification remains separate.

That investigation exposed an existing source-level bug in supported function
pointer bounds. The published compiler rejects five valid unused declarations
because it checks escaping lifetime depth but misses nested callable binders.
Definition classification now reads cached properties of the original subject,
complete trait arguments and equality result. Two additional typed invariants
check all roots, captured lifetimes and unused/empty quantifier identities.
Closed projections remain eligible for subsequent normalization and proof.

Ten authored source controls were independently qualified before reference
generation: six accepted, four rejected. Eight are installed in PA19's
`declaration-proofs` cluster; two associated equality controls belong to PA20's
`contracts/equations/lowir` cluster. Their source bytes are unchanged from the
qualified controls. The candidate accepts the five previously rejected unused
declarations, rejects the unsatisfied use sites and accepts the implementation
positive. All six accepted programs execute with matching clean outcomes through
rustc and CPPGM. Generated LowIR emits only demanded functions; unused invalid
conditions never enter the executable. Every old oracle sidecar remains unchanged.
PA19 now has 1,080 cases and 254 native executions; PA20 has 1,345 cases and 484
native executions, for 6,228 active course cases overall.

This checkpoint does not open PA21 source gates. Declaration ambiguity, scoped
lifetime consumers, conformance and concrete-body integration still need their
whole-clause implementations; independent language controls do not substitute for
RustGM source/native coverage of those paths.


Full validation passes all 33 recorded gates: **1,367 regular tests**, **91 native
groups**, **6,228 PA1–PA20 exact cases**, formatting, strict Clippy, the 831-file
zero-finding audit, harness, backend and upstream checks. The four affected
language/native gates retain their successful earlier receipts for the identical
candidate bytes, manifests, sources and harness configuration rather than repeating
them. PA21–PA25 discovery still fails as planned. The expanded frozen snapshot
verifies **7,171 source/configuration hashes** and **18,684 oracle sidecars**.
The immutable candidate is
`02d6034bf8c2c14673a909fffec3d8ca0406491cc661ccf2e081ba9b513bbfc0`,
compared with published `3c71c4c` /
`1f9f6c170c6494d8cca516e6fa29e2837566ba7e02408a4c3be575b7ebd44229`.

All 33 timing jobs and 29 instruction pairs complete. On the existing 27 inputs,
instruction deltas span **−0.40% to +0.32%** and paired wall medians span
**−7.15% to +1.52%**. Two added, predeclared workloads repeat unused callable
bounds whose implementation is available, so both binaries accept and emit equal
LowIR. They independently pass pinned rustc object generation:

| Repeated declarations | Baseline instructions | Candidate instructions | Instruction change | Paired wall change |
| --- | ---: | ---: | ---: | ---: |
| 64 | 14,004,782 | 9,849,509 | −29.67% | −17.79% |
| 256 | 53,957,653 | 37,342,264 | −30.79% | −24.43% |

These workloads exercise removal of definition-time satisfaction work for bounds
that defer satisfaction until use. They do not measure real-library compilation.
No instruction increase exceeds the 1% investigation threshold, and no paired wall
increase exceeds 3%; the predeclared policy therefore selects no timing follow-ups.
Same-binary medians span **−0.64% to +0.65%**, but the largest A/A spread is
**102.22%**. Shared-host timings do not establish a portable speedup. Median peak
RSS deltas span **+28 to +146 KiB**. All full/count outputs and frozen hashes match.

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,261 | 2,432,261 | 2,528,288 | 2,528,288 |
| constant-chain-uses-256 | 5,381,873 | 5,381,881 | 5,540,136 | 5,540,168 |

The fixed ordinary heap/combined peaks are unchanged. The constant-chain heap
increases by eight bytes and its combined peak by 32 bytes; ELF text/data/BSS
increases by 4,128 bytes. These samples do not explain the broader RSS range.
All **858 study observations** and **132 qualification observations** are retained,
comprising **3,432 timed** and **528 qualification compiler invocations**, plus
**58 Callgrind** and **four Massif** runs. No observations were discarded or
repeated until favorable. Receipts remain in `obj/pa21-work/formation-final/`.
The earlier streamed-proof wall-time effect against `b1410a8` remains unresolved.
PA21–PA25 remain planned; no unsupported source gate was opened by this checkpoint.

Successful guarded cleanup reclaimed **6.90 GiB** from 5,753 generated files;
timing observations, diagnostics and the current immutable comparison pins remain.


## Whole-clause declaration consumers

This checkpoint keeps the PA21 source gates closed. Declaration ambiguity,
trait-input formation and recursive equality/outlives term formation now consume
whole clauses without discarding their binders. An independent own-trait
requirement is checked against caller assumptions in scratch inference; local
conditions remain deferred until use. Canonically identical opened alternatives
keep a real source premise and its owned interval. Distinct correlations remain
ambiguous when demanded, and scratch equations grant no permissions.

Recursive terms retain independent nominal requirements beneath local references.
Scoped shape lookup handles captured assumptions at the actual root depth, with
separate formation, ordinary-use and normalized caches. A constructor's local
Sized requirement may defer, but the type remains unsized for use checks.
Scoped projection normalization shares its cursor and opening/closing folds
across roots, preserves nested and empty binders and retains selected lifetime
evidence until the final region proof. Closing immutable owner aliases avoids
repeated scans of an accumulated proof. Foreign free identities fail.

The ambiguity investigation qualifies 28 pinned-rustc source controls (20 accepted,
8 rejected), and the recursive nominal investigation qualifies 13 (8 accepted,
5 rejected). Twelve corrected projection/Sized controls qualify (5 accepted,
7 rejected). Original hypothesis mismatches and controls rejected for E0582 are
retained. Static-only projected outlives and equality controls trigger internal
compiler errors in pinned rustc 1.98.0; they are excluded from semantic oracle
evidence, not counted as language rejections. Typed static-only projection tests
remain internal proof-soundness checks. No PA21 source or native success is claimed.

The new typed tests exercise actual declaration, nominal, projection and lifetime
consumers. Declaration ambiguity widths 64–1024 take 12n counted steps without new
canonical types. Adding the independent own-trait goal makes the earlier formation
family take 17n steps and n new types. Growing scoped projection terms at widths
16–1024 take 11n+9 counted steps and 3n+1 new types; repeated roots add request work
but no new types or scopes. These are bounded internal families, not a proof of
whole-frontend linearity. The focused run passes 213 library tests, strict Clippy,
the 837-file zero-finding audit and every PA19/20 text oracle.

Full validation passes all **32 recorded gates**: **1,388 regular tests**,
**91 native groups**, **6,228 PA1–PA20 exact cases**, formatting, strict Clippy,
the **837-file zero-finding audit**, harness, backend and upstream checks.
Fresh PA19/PA20 independent runs check **1,080/1,345 source cases** and execute
**254/484 native comparisons**. The seven/one documented acceptance differences
are unchanged. All **7,177 source/configuration hashes** and **18,684 oracle
sidecars** match. PA21–PA25 discovery still fails as planned.

The frozen candidate is
`db33c1af84ff6db968a508d6351983a8b447def9183ed81c81e2ad88d89ae3c5`,
compared with published `328f419` /
`02d6034bf8c2c14673a909fffec3d8ca0406491cc661ccf2e081ba9b513bbfc0`.
All **33 timing jobs** and **29 instruction pairs** complete with matching outputs.
Instruction deltas span **−0.34% to +0.70%**; none exceeds the 1% investigation
threshold. Initial paired wall medians span **−21.16% to +9.38%**, while same-binary
medians span **−10.84% to +0.38%** and the largest A/A spread is **64.69%**.
These shared-host measurements do not establish portable wall-time changes.
Median peak RSS deltas span **−58 to +68 KiB**.

Four initial timing increases exceed 3%. The predeclared policy selects the three
largest once, with batch-16 A/B, A/A and B/B follow-ups:

| Workload | Initial A/B change | Follow-up A/B | Follow-up A/A | Follow-up B/B |
| --- | ---: | ---: | ---: | ---: |
| recursive-copy-64 | +9.38% | −0.02% | +0.25% | +0.35% |
| trait-view-32-full | +7.96% | −2.87% | −2.61% | −1.32% |
| shared-depth-callers-96 | +5.08% | −4.67% | +1.90% | −2.94% |

Those three increases are not reproduced by their fixed follow-ups. The fourth,
`coupled-48` at **+3.63%**, remains recorded without an additional run under the
three-input policy. No observation is discarded or repeated until favorable.
The earlier streamed-proof wall-time effect against `b1410a8` remains unresolved.

| Workload | Baseline heap bytes | Candidate heap bytes | Baseline combined heap/overhead/stack | Candidate combined |
| --- | ---: | ---: | ---: | ---: |
| ordinary-256 | 2,432,261 | 2,432,021 | 2,528,288 | 2,528,360 |
| constant-chain-uses-256 | 5,381,881 | 5,381,521 | 5,540,072 | 5,540,080 |

The fixed combined peaks increase by **72/8 bytes**, while observed heap peaks
fall by **240/360 bytes**. These samples are not an explanation of the broader
RSS range. ELF text/data/BSS increases by **28,656 bytes**. Retained evidence totals
**1,092 study observations**, **168 qualification observations**, **7,176 timed**
and **1,104 qualification compiler invocations**, plus **58 Callgrind** and
**four Massif** runs. All outputs, frozen sources and binary hashes agree.
Receipts remain in `obj/pa21-work/clause-consumers-final/`; language investigations
remain in `obj/pa21-work/ambiguity-v1/` and `obj/pa21-work/term-formation-v1/`.

Scoped lifetime assumptions, nominal outlives propagation, conformance, body
integration and binder-aware source production remain unfinished. Independent
source controls and typed proof tests do not establish end-to-end PA21 acceptance.
PA21–PA25 remain planned.

Successful guarded cleanup reclaimed **6.90 GiB** from **5,751 generated files**.
Inputs, diagnostics, observations and current immutable comparison pins remain.


## Binder-independent nominal implication — 2026-10-18

Nominal outlives inference now distinguishes a complete quantified obligation
from a lifetime bound implied by a nominal input. It filters a predicate that
mentions its own local lifetime before extracting any components. Independent
components of that filtered predicate do not become caller assumptions. Empty
and unused quantifiers lift into the owner scope without capturing nested
callable locals. Quantified trait conditions do not obstruct independently
implied field lifetimes; formation still checks those trait conditions.

This follows the pinned compiler's
[implied-outlives query](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_trait_selection/src/traits/query/type_op/implied_outlives_bounds.rs),
which filters the complete normalized predicate before extracting components.
The ordinary nominal propagation worklist is unchanged. The new scoped helper
is allocated on demand and shares capture summaries, lowering folds and complete
clause results. It caches inferred requirements, never permission to use them.

Independent pinned-rustc qualification contains **48 controls: 31 accepted and
17 rejected**, with no mismatches or infrastructure failures. All nine ordinary
controls also match the published `db33c1af84ff` compiler. Earlier incorrect
hypotheses remain alongside their diagnostics in the investigation records.
For example, an explicit `for<'a> T: 'a` assumption can justify `T: 'static`,
but a nominal carrying that condition does not grant the same assumption to
its caller. An ordinary nominal `T: 'static` condition can be implied by an
input type; declaring a nested field still has its separate formation proof.

Explicit caller assumptions require their own rules. The pinned
[environment elaborator](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_type_ir/src/elaborate.rs)
does not decompose a predicate with a locally bound target. With a nonlocal
target it can retain independent components. The original predicate remains
available to the
[outlives verifier](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_infer/src/infer/outlives/verify.rs).
The nominal implication filter must not substitute for that separate consumer.

Eight new integration tests cover whole-predicate filtering, empty and unused
binders, independent fields, nominal argument substitutions, opaque projections,
nested callable scopes, invalid identities and shared captured type graphs.
The existing universal nominal formation test now verifies both the absence of
an inferred scalar bound and retention of the actual quantified obligation;
its non-static instance still fails the final lifetime proof.

For growing opaque projected subjects with duplicate clauses, widths 16 through
1,024 use exactly **18 × width work steps** and **2 × width new types**.
This establishes additive capture/lowering work for that family; it does not
establish whole-frontend linearity or remove the cost of arbitrary component
collection. Focused validation passes 23 integration tests, 213 library tests,
strict Clippy, the 839-file audit with zero findings, and harness checks.

Full validation passes all **32 gates**: **1,396 regular tests**, **91 native
groups**, **6,228 exact PA1–20 course cases**, formatting, strict Clippy, the
839-file audit, harness, backend and upstream checks. Fresh PA19 qualification
covers 1,080 source checks, 641 LowIR checks and 254 native executions; PA20
covers 1,345 source checks, 799 LowIR checks and 484 native executions. The same
seven PA19 and one PA20 documented acceptance differences remain. All **7,179
source/configuration hashes** and **18,684 oracle sidecars** are unchanged.

The immutable candidate is `67c2184be048`; the baseline is published
`db33c1af84ff`. Across **33 timing jobs** and **29 instruction-count pairs**,
instruction changes range from **−0.45% to +0.19%**. Paired wall-time changes
range from **−10.61% to +1.51%**; same-binary comparisons range from **−0.66% to
+1.12%**, with maximum A/A spread **19.10%**. No positive timing median crosses
the predeclared 3% follow-up threshold, and no instruction increase crosses 1%.
These shared-host timings do not establish a portable speedup. The earlier
streamed-proof wall-time effect against `b1410a8` remains unresolved.

Observed RSS changes range from **−186 to +150 KiB**. Fixed Massif comparisons
measure ordinary-input heap 2,432,021 → 2,432,261 bytes and combined
heap/overhead/stack 2,528,360 → 2,528,288 bytes; the constant-chain input measures
heap 5,381,521 → 5,381,701 and combined 5,539,648 → 5,539,832 bytes. These samples
do not explain the broader RSS range. ELF text/data/BSS increases by **8,184
bytes**. Evidence retains **858 study observations**, **132 qualification
observations**, **3,432 timed** and **528 qualification compiler invocations**,
**58 Callgrind** and **four Massif** runs. All outputs and frozen hashes agree.
Receipts remain in `obj/pa21-work/implied-outlives-final/`; language controls and
pinned compiler-source provenance remain in `obj/pa21-work/outlives-consumers-v1/`.

PA21 source gates remain closed. Caller assumptions, declaration region
consumers, conformance, body integration and binder-aware source production
remain unfinished. PA21–PA25 remain planned.

Successful guarded cleanup reclaimed **6.90 GiB** from **5,751 generated files**.
Inputs, diagnostics, observations and the current comparison binaries remain.


### Quantified caller lifetime consumers

The next checkpoint adds typed caller outlives assumptions and scoped required
clauses. Direct parameter bounds over every lifetime retain their meaning;
composite bounds do not accidentally distribute across a locally bound target.
Projected premises preserve complete inputs, repeated-slot identity and target
correlations. Fixed captures become conditional equality tests, never equations
introduced by a trial match. Independent universal requirements use fresh graph
scopes. Declaration trait lifetimes, receiver compatibility and nominal static
requirements now consume these whole clauses. Source HRTB, conformance and body
integration remain incomplete; this does not release PA21 or its source tests.

The matcher indexes rigid lifetime-erased shapes and then checks complete typed
structure. Its shared recipes include type pairs, argument lists and callable
parameter lists, with the requesting proof scope in the cache key. Canonical
persistent radix maps retain slot bindings and guard conjunctions without copying
growing prefixes. Radix recursion is bounded by the 32-bit key space; type walks
are iterative and budgeted. Ordinary projection equality shares its exhaustive
structural dispatch while retaining its existing proof behavior.

A new wide-input invariant exposed an earlier quadratic walk in quantified slot
validation: 1,024 nominal owners sharing 1,024 arguments exhausted the capture
budget before reaching the matcher. Capture/slot occurrence queries now cache
argument and callable-list summaries by binder depth. The same setup takes
4n slot-validation steps at widths 16, 64, 256 and 1,024. Matching work at those
widths is 699 / 2,891 / 12,043 / 50,187 steps. Matched growing reference tails use
846 / 3,470 / 14,350 / 59,406 steps for local substitutions and
842 / 3,448 / 14,214 / 58,692 for fixed-capture guards. Repeated matching roots add
less than 3n work. These are typed-query invariants, not whole-frontend linearity
or compile-time speedup claims. Persistent-map unions are independently checked
against ordinary maps, including conflicts, insertion order and 32-bit endpoints.

Independent source investigation remains recorded under
`obj/pa21-work/outlives-consumers-v1/`. The 28 late-matching controls qualify
12 acceptances and 16 rejections with pinned rustc. Pinned rustc rejects four ordinary nested-callable controls accepted by both
current RustGM and its preceding immutable baseline; thus these differences
were not introduced by nominal implication. Reusing one
callable alias and spelling two equivalent aliases can differ in rustc's late
syntactic matcher. Six subsequent concrete controls preserve three incorrect
initial hypotheses: a lifetime-generic monomorphic function-pointer impl does
not automatically cover a higher-ranked pointer, and the mono/poly impl overlap
control produces the historical `coherence_leak_check` warning. No source oracle
is blessed from these hypotheses. The generic nested-slot acceptance alone does
not authorize leaking an inner placeholder into an outer premise substitution.
These compatibility boundaries remain explicit before source activation.

Full validation passes all 32 gates: 1,415 regular tests (including 221 library
and 11 new caller-outlives tests), 91 native groups, all 6,228 course cases,
strict all/pedantic Clippy, the 852-file audit, harness, upstream/backend checks,
and fresh PA19/PA20 independent source/native checks. PA19 retains 1,080 source,
641 LowIR and 254 native cases; PA20 retains 1,345 source, 799 LowIR and 484 native
cases. Their seven plus one documented acceptance differences remain unchanged.
The frozen run verifies 7,192 source/config hashes and 18,684 unchanged oracle
sidecars. Planned PA21–PA25 discovery continues to fail deliberately.

Immutable candidate
`8b840ce8a8eb04657370ba5cfae9ff099a70ea0cbaf195ef6b3cc1fb5d9e39ea`
was compared with published `a83ce00` / binary
`67c2184be0481681688bafde1fe7576e0c003b0dd872083313cd9482f0310091`.
Across 33 wall jobs and 29 instruction pairs, instructions change
−0.2634% to +0.2537% and paired wall medians −2.6949% to +1.5556%.
Same-binary medians range −0.5646% to +1.7101%, with maximum within-job A/A
spread 16.3340%; these results do not establish a portable speedup. No >3% wall
or >1% instruction trigger fired, so the fixed follow-up policy selected zero
jobs. RSS changes +24 to +148 KiB. Fixed Massif ordinary/constant-chain heap
peaks are unchanged at 2,432,261 / 5,381,881 bytes; combined heap/overhead/stack
peaks change +32 / +96 bytes. Reported ELF text/data/bss size grows 61,436 bytes.
The comparison retains 858 study and 132 qualification observations, 3,432 timed
and 528 qualification invocations, 58 Callgrind profiles and four Massif profiles.
Every compared output matches and binary/input/source hashes remain stable.
This does not resolve the previously documented streamed-proof effect versus
`b1410`; it measures the immediate published baseline.

Evidence: `obj/pa21-work/universal-outlives-final/`, with final-summary SHA-256
`d5a091ff96918e48486686f169840963a9fa71833ff7aa0b5c54d1631e5e2340`.
Successful wrapper cleanup removed 5,751 generated files (6.90 GiB unshared),
receipt `obj/cleanup/1792347678739943842.jsonl`. About 453 GiB remained free.
Only the current baseline and candidate pins are retained; the superseded pin
is retired after the replacement commit passes CI.


### Whole-clause method conformance

Method override checking now keeps complete clauses through owner/method
substitution, environment construction and final lifetime validation. Trait
requirements use scoped selection evidence; outlives requirements introduce
their actual universal scopes in the directed lifetime graph. Implementation
requirements never enter the caller assumptions. Captured owner lifetimes,
independent quantified inputs and correlated inputs keep their distinct meanings.

The type/trait environment checks type and const parameter membership in every
quantified lifetime clause, then leaves its proof to the retained lifetime
consumer. It does not create Copy, Sized or user-trait permissions from that
clause. Quantified property and equality premises remain explicit unsupported
boundaries. A small clause classifier routes obligations without exposing an
unbound predicate to the caller.

Twelve predeclared source controls all match pinned rustc: six accept and six
reject. They cover equivalent and weaker method requirements, forbidden stronger
requirements, captured owners, independent versus correlated lifetimes, parameter
all-lifetimes versus static bounds, composite subjects and projected input/target
correlations. Five typed conformance groups exercise those twelve rules through
the production mapping and proof path while source HRTB remains gated. Negative
typed controls require the intended conformance or final lifetime diagnostic;
an unrelated unsupported-feature error cannot satisfy them.

Universal projected caller assumptions also deduplicate complete predicate bodies
whose top-level quantifiers differ only in unused slot count. This does not merge
requirement scopes or change definition-time empty/nonempty binder treatment.
At widths 16, 64, 256 and 1,024, N equivalent premises followed by N goals visit
exactly N candidates, with less than 5N additional matcher steps. The invariant
checks candidate scans, not merely cache hits or whole-frontend running time.

Source qualification and focused results remain in
`obj/pa21-work/conformance-clauses-v1/`. Focused checks pass 228 library tests,
11 caller-outlives tests, strict Clippy, the 853-file audit with zero findings,
harness checks and both affected course suites. The source producers, body
consumers, associated families and opaque types still need implementation;
PA21–PA25 remain planned.

Full validation passes all 32 gates: **1,422 regular tests**, **91 native groups**,
all **6,228 PA1–PA20 course cases**, formatting, strict all/pedantic Clippy,
the 853-file audit, harness, upstream and backend checks. Fresh PA19 qualification
covers 1,080 source / 641 LowIR / 254 native cases; PA20 covers
1,345 / 799 / 484. The existing seven plus one documented acceptance differences
remain. All **7,193 source/configuration hashes** and **18,684 oracle sidecars**
are stable, and planned PA21–PA25 discovery still fails deliberately.

Immutable candidate `28d7ec53f8bf8a8a8f66c60d0af71cfd02c7e114b21414b00bdbf57dcac694c7`
was compared with public `56bcc41` / binary `8b840ce8a8eb`. Across 33 wall jobs
and 29 instruction pairs, instructions change **−0.1360% to +0.2342%**; no
instruction increase crosses 1%. Original paired wall medians range
−4.3160% to +6.4168%, while same-binary medians range +0.4252% to +10.3808%.
Maximum A/A spread is 45.8260%, so these timings do not establish a portable
speedup or isolate the smaller effects of this change.

The predeclared >3% policy selected all three positive outliers. Its single
batch-16 AB/AA/BB follow-up measures pending-chain-8 at +1.0765%,
inherent-nominals-256 at −0.3166%, and inherent-closed-256 at −1.2778%.
Their same-binary medians range −0.9458% to +0.5872%; maximum follow-up A/A
spread is 17.6515%. All original and follow-up observations are retained.
The original >3% increases did not repeat in this scheduled comparison; no
additional favorable-result reruns were made. The earlier streamed-proof effect
against `b1410` remains a separate unresolved measurement.

Observed RSS changes range −96 to +78 KiB. Fixed Massif ordinary/constant-chain
heap peaks remain 2,432,261 / 5,381,881 bytes. Combined heap/overhead/stack peaks
are unchanged at 2,528,320 bytes for the ordinary input and rise by 64 bytes to
5,540,200 for the constant chain. Reported ELF text/data/BSS size increases by
24 bytes. Evidence retains **1,092 study observations**, **168 qualification
observations**, **7,176 timed** and **1,104 qualification invocations**,
58 Callgrind profiles and four Massif profiles. Every compared output matches;
all frozen input, binary and source hashes remain stable.

Receipts remain in `obj/pa21-work/conformance-final/`. Final-summary SHA-256:
`566dbf508523e7938124ef8926ecacf1b60bf21a4d6e24802a5457894c995ec3`.

Successful guarded cleanup removed 5,751 generated files (6.90 GiB unshared),
receipt `obj/cleanup/1792350213766578672.jsonl`; about 450 GiB remained free.
Only the current comparison binaries stay pinned. The preceding pin was retired
after exact public-baseline CI passed, with both binary hashes recorded.


### Retained instance projection debts

Concrete instance normalization now extends each retained lifetime proof inside
its existing scope. It resumes at the actual allocator frontier and excludes
intervening intervals owned by other proofs. Original universal identities and
private witnesses remain connected; a fresh sibling quantifier cannot replace
them. Selected implementation lifetime requirements remain debts for the final
body proof, including those discovered beneath nested callable binders.

Type-debt positions identify an existing vector element or scoped block element.
The normalizer snapshots those positions before extending the proof, then checks
and replaces exactly those roots. New evidence keeps its own raw types and target
lifetimes. Appending root-level debts cannot redirect a saved scoped position;
stale positions fail explicitly. These positions are local mutation locations,
not permission tokens that authorize a different proof.

Roots are grouped once by their actual proof scope, with one shared normalization
session per group. Work includes root visits and structural normalization. At
8, 64 and 512 roots, separate shared-graph, sibling-scope and nested-depth families
stay below 128 × root count steps. Tests distinguish scope identity even when
canonical type IDs match, preserve foreign allocation gaps, and reject unpaid
static requirements at the original body source. This is a bound on the measured
normalization work, not whole-frontend complexity.

Seven predeclared source controls match pinned rustc: four accept and three
reject. The ordinary callback example already succeeds on published `f7dfb90`;
it is a compatibility control, not a newly supported source program. Its LowIR
matches the focused candidate byte-for-byte. The other controls qualify the
universal result/target and implementation lifetime rules used by the typed tests.
No public fixture or oracle was changed, and PA21 source gates remain closed.

Focused validation passes 233 library tests, 11 caller-outlives tests, seven
instance-body tests, eight quantified-lifetime tests, strict Clippy, the 856-file
audit with zero findings, harness checks and both affected course suites.
Receipts remain in `obj/pa21-work/instance-proof-scopes-v1/`. The first two attempts
caught test-code mistakes (a nonexistent boolean type constant and an unnecessary
Result wrapper); those were corrected without lint allowances. Successful cleanup
removed three generated executables, receipt
`obj/cleanup/1792350803994193985.jsonl`; earlier qualification objects were removed
with their own hash receipt. The full validation run also includes the added
outer-capture/empty-binder invariant and independent callback execution.


Full instance-proof checkpoint validation passes all **33 gates**: **1,428 regular
tests** (including 234 library invariants), **91 native groups**, and **6,228
course cases**, strict Clippy, formatting, the **856-file audit with zero findings**,
harness, backend and upstream checks. The added outer-capture invariant passes
for both empty and nonempty binders. The existing callback control has identical
872-byte LowIR on baseline and candidate; CPPGM and pinned rustc executables both
exit cleanly with the independently declared result. PA19 source / LowIR / native
counts remain 1,080 / 641 / 254 and PA20 counts remain 1,345 / 799 / 484, including
the same seven plus one documented acceptance differences. All **7,197 frozen
source/configuration hashes** and **18,684 oracle sidecars** are stable.

Immutable candidate `44f2289ed952d0b9eb9cef0c62eb4d90f7f0333643976af3ba02d42f083f0d3f`
was compared with public `f7dfb90` / binary `28d7ec53f8bf`. Across 33 wall jobs
and 29 instruction pairs, instruction changes range **−0.3771% to +0.7794%**;
none crosses the 1% investigation threshold. Original paired wall medians range
−1.2935% to +4.6406%; same-binary medians range −0.5206% to +1.0094%, with a
maximum A/A spread of 70.2129%. Shared-host timings do not establish a portable
speedup or isolate small changes.

The fixed >3% policy selected two inputs for one batch-16 AB/AA/BB follow-up:
trait-view-8-count measures +1.0430%, and callers-48 measures −0.0810%.
Their same-binary medians range −1.4731% to −0.0254%, and maximum follow-up A/A
spread is 6.1828%. Both original >3% increases did not repeat in that scheduled
comparison; all original and follow-up observations remain available. The earlier
streamed-proof effect against `b1410` remains a separate unresolved measurement.

Observed RSS differences range −124 to +38 KiB. Fixed Massif ordinary and
constant-chain heap peaks remain 2,432,261 and 5,381,881 bytes. Combined
heap/overhead/stack peaks change by +256 and +496 bytes respectively. Reported
ELF text/data/BSS size decreases by 36 bytes. Evidence retains **1,014 study
observations**, **156 qualification observations**, **5,928 timed invocations**,
**912 qualification invocations**, 58 Callgrind profiles and four Massif profiles.
Every compared output matches, and all frozen binary, input and source hashes
remain unchanged.

Receipts remain in `obj/pa21-work/instance-proof-final/`. Final-summary SHA-256:
`a4636376e46cc78761147ab8c1181468a0fa0a38fe106ea7ed7fe4332aa4ec28`. PA21–PA25 remain planned; this checkpoint does not release a new source
or grading contract.

Successful guarded cleanup removed 5,753 generated files (6.91 GiB unshared),
receipt `obj/cleanup/1792352484998281343.jsonl`. Only the current comparison
baseline and candidate binaries remain pinned; superseded pins are retired after
their replacement passes exact-commit CI, with hash and publication receipts.

## Body caller clauses and shared component work

Body lifetime contracts now carry whole outlives clauses through collection,
dispatch, specialization and the final region check. An original parameter's
all-lifetimes permission is elaborated before specializing that checked body,
so replacing the parameter with a reference does not discard the permission.
Ordinary callee witness facts use the same clause representation; quantified
trait-witness import still has its explicit unsupported-scope guard.

Concrete projected caller premises retain their original quantified clauses.
Normalization shares a session for each binder-arity group and records new
requirements as debts in that proof. Only the exact declared premise supplies
independent normalized components; normalization requirements cannot become
assumptions. Tests distinguish unrestricted and static-restricted implementations,
empty and nonempty binders, captured inputs, unrelated targets and absent premises.

The first growing-clause test exposed quadratic traversal in the final lifetime
consumer: normalization shared its work, but checking each retained composite
premise separately exhausted the one-million-step outlives budget. The batch
consumer now groups quantified composite subjects by independent target and
walks all roots in a group together. It retains opaque projected components and
keeps top-level projected matching and ordinary assumptions on their own paths.
It neither fabricates aggregate types nor caches flattened vectors for every
prefix. Body, declaration, receiver, nominal and override lifetime gates use
this consumer. Distinct targets and caller environments retain distinct
permissions. Component traversal is bounded by the graph visited for each
target, not by separately revisiting every growing prefix for that target.

Independent qualification comprises **36 pinned rustc source controls: 23
accept and 13 reject**. Four original hypotheses were contradicted and remain
recorded with their diagnostics. Projected assumptions can remain opaque when a
matching caller trait premise is present; the paired case without that premise
can normalize through its implementation. A generic default body may use its
original all-lifetimes Self permission, but an override on a reference cannot
strengthen its mapped contract to static—or even repeat the identical quantified
Self requirement in the qualified example. The speculative conformance
pre-elaboration was therefore removed. Conformance continues to elaborate after
mapping; a dedicated regression distinguishes that operation from specializing
an already-checked body. These are rustc qualifications and typed-consumer
regressions, not newly released RustGM source capabilities.

Focused checks pass 239 library tests, 14 caller-outlives tests, instance-body
tests, strict all/pedantic Clippy, the 858-file audit and harness checks. Earlier
failed hypotheses and the initial budget-exhaustion regression remain in
`obj/pa21-work/body-clauses-v1/`; focus4 and focus6 cleanup receipts are
`1792354020924719329` and `1792354545103691468`.

The prior compiler checkpoint's CI reached its final PA20 rustc comparison but
hit the 20-minute job limit. Workflow-only commit
[`9763d07`](https://github.com/vishvananda/rustgm/commit/9763d07715abde1ffe62ee6d2e48157705df3865)
raises the limit to 30 minutes and puts release tests under the storage guard,
preserving all gates. Its exact-commit
[CI run](https://github.com/vishvananda/rustgm/actions/runs/39162122913) passes.
The previous compiler pin was retired only afterward, recording both binary
hashes and verifying that the successor changed only the workflow.

PA21–PA25 remain planned. Own-clause body formation, canonical body requirements,
quantified call inference and specialized trait-witness import still need their
scoped consumers before source activation. PA21's associated type families and
opaque types also remain release work. No public text oracle changed here.

Full validation passes **34 gates**, **1,436 regular tests**, **91 native groups**
and all **6,228 active course cases**. The isolated component-work observation
records **82 / 642 / 5,122 / 20,482 steps** for **16 / 128 / 1,024 / 4,096 roots**.
PA19 independently checks 1,080 source controls and 641 LowIR controls, with 254
executable comparisons; PA20 checks 1,345 source controls and 799 LowIR controls,
with 484 executable comparisons. Their existing seven and one documented
acceptance differences remain. All 7,237 source/config hashes and 18,684 oracle
sidecars are unchanged during validation and measurement; PA21–PA25 discovery
continues to reject planned assignments.

Against the published compiler at `8429732` (identical compiler source in the
CI-only successor `9763d07`), 33 immutable timing jobs and 29 instruction pairs
retain full/count output equality. Paired wall medians range from **−1.653% to
+2.089%**, same-binary medians from **−1.398% to +0.864%**, with maximum A/A spread
**13.877%**. No wall result crosses the predeclared 3% follow-up threshold; no
follow-up was run. Instruction changes range from **+0.071% to +0.383%**, below
the 1% review threshold. This does not establish a portable source compile-time
speedup; the new quantified consumer's improvement is demonstrated by its
bounded structural work and the prior budget-exhaustion regression. This
comparison does not settle the earlier `b1410` streamed-proof performance question.

Two fixed Massif controls retain identical heap peaks of **2,432,261** and
**5,381,881 bytes**. Simultaneous combined peaks change from **2,528,576 to
2,528,304 bytes** and **5,540,488 to 5,540,152 bytes**; these small differences do
not establish a general memory improvement. Median RSS changes range from
−538 to −34 KiB. ELF totals grow from **6,346,753 to 6,359,037 bytes** (+12,284).
The study retains **858 observations**, **132 qualification observations**,
**3,432 timed invocations**, **528 qualification invocations**, **58 Callgrind
invocations** and **four Massif invocations**.

Receipts are in `obj/pa21-work/body-clauses-final/`. Final-summary SHA-256:
`52db286b95096389a39741977fa50f4a5c585d08f75c5819d3880cbb9d9108d0`.
The candidate SHA-256 is
`1e4a4ee04b08b92344c36a0ccdcd8268e5427ead16be40f8e7d3b52148857d1d`;
the retained comparison baseline is
`44f2289ed952d0b9eb9cef0c62eb4d90f7f0333643976af3ba02d42f083f0d3f`.

Successful cleanup removed **5,753 generated files (6.91 GiB)**; receipt
`obj/cleanup/1792356093283230403.jsonl`. Only the current baseline and candidate
remain pinned. Qualified probe objects and the preceding pin were retired with
separate hash receipts under the storage guard.

## Scoped body formation and shared requirement traversal

Body contracts now retain equality-result and outlives-subject formation together
with the complete clause binder. Trait clauses consume the declaration's cached
whole-input filter. Substitution and dispatch preserve local lifetimes while
lifting actual argument captures; final region checking defers a whole
local-dependent outlives debt before component decomposition and still visits
independent requirements nested in references, nominal arguments, callable
binders and projected Self/type arguments. Formation supplies no new permission.

One body-owned query streams newly discovered debts with their actual source.
Structural visits and component cursors share graph edges per lifetime target
and component policy, keeping binder depth distinct; no flattened prefix vectors
are retained. Four structural controls measure 290, 2,306, 18,434 and 73,730 steps
for 16, 128, 1,024 and 4,096 prefixes with two targets (`18n + 2`). Repeated roots
still validate their binder arity and consume budget. This is a structural work
invariant, not a source compile-time speedup measurement.

Concrete specialization detects projections occurring only in a lifetime contract,
including manually constructed contracts with no interned declaration clauses.
Normalization shares an actual proof scope/session within each binder arity,
retains every selected implementation debt, and adds no assumptions. Regressions
cover a restrictive implementation, empty binders, captured actual arguments,
source attribution and separation between body environments.

Thirty predeclared pinned rustc 1.98.0 controls all match: seventeen accepted and
thirteen rejected. Hypotheses, sources and diagnostics remain in
`obj/pa21-work/body-formation-v1/{source,debt,projection-input}-controls/`. The
public HRTB syntax gate remains intentional. PA21 still needs its remaining
canonical body requirements, quantified call inference and witness consumers,
associated families, opaque types, remaining destruction integration, and reviewed
course contract. PA21–PA25 remain planned.

All **34 functional gates pass**: **1,447 regular tests** (including 242 library
invariants), **91 native groups**, **6,228 PA1–PA20 course cases**, strict
format/Clippy checks, the **866-file audit with zero findings**, harness, upstream
and backend checks. Eleven new maintainer tests cover the formation/query and
specialization paths. Independent PA19 checks cover 1,080 sources and 641 LowIR
programs with 254 executable comparisons; PA20 covers 1,345 sources and 799 LowIR
programs with 484 executable comparisons. Their existing seven and one documented
acceptance differences remain. All **7,239 frozen source/config hashes** and
**18,684 oracle sidecars** match; PA21–PA25 discovery continues to reject planned
assignments.

Against published `ecad7ad`, **33 immutable wall-time jobs** and **29 instruction
pairs** retain full/count output equality. Paired wall medians range from
**−2.851% to +1.086%**, same-binary medians from **−0.742% to −0.098%**, with
maximum A/A spread **22.641%**. No paired baseline/candidate result crosses the
predeclared 3% follow-up threshold, so no follow-ups were selected. Instruction
changes range from **−0.250% to +0.021%**, with none above the 1% review threshold.
These supported-source controls establish no portable compile-time speedup; the
new quantified formation path's structural test uses two fixed lifetime targets.
It does not establish a general bound when target count also grows, and that
separate scaling family remains to be investigated before PA21 release. This
comparison does not settle the earlier `b1410` streamed-proof performance question.

The two fixed Massif controls record heap peaks of **2,432,261 → 2,432,261 bytes**
and **5,381,701 → 5,381,881 bytes**. Simultaneous combined peaks are
**2,528,304 → 2,528,304 bytes** and **5,540,104 → 5,540,072 bytes**. Stack peaks
are **69,304 → 70,848 bytes** and **83,976 → 82,832 bytes**. These small sampled
differences establish no general memory improvement. Median RSS changes range
from −178 to −18 KiB. ELF totals are **6,359,037 → 6,375,421 bytes** (+16,384).
All **858 study observations**, **132 qualification observations**, **3,432 timed
invocations**, **528 qualification invocations**, **58 Callgrind invocations** and
**four Massif invocations** remain retained.

Receipts are in `obj/pa21-work/body-formation-final/`. Final-summary SHA-256:
`3b890502aef8c5e7db2c2645c2e7291f44211b36bec3dc553157b5e1dc7dece4`.
Candidate SHA-256:
`d5bc985cc02c02b844e9f5f9ed6ed4aa49ead133c0bd05cf13d0220053ddb16c`;
comparison baseline:
`1e4a4ee04b08b92344c36a0ccdcd8268e5427ead16be40f8e7d3b52148857d1d`.

Successful cleanup removed **5,753 generated files (6.91 GiB)**; receipt
`obj/cleanup/1792359183261286808.jsonl`. The preceding `44f2289e` pin was retired
only after `ecad7ad` passed exact-commit CI, with verified replacement hashes in
`obj/pa21-work/body-formation-v1/baseline-retirement.json`. Keep only the current
baseline and candidate pins until this replacement passes its exact-commit CI.

## Shared formation components across lifetime targets

This maintainer change fixes repeated traversal in PA21 body formation. It does
not activate PA21's source or grading contract; PA21–PA25 remain planned.
A typed body probe with growing tuple prefixes containing only `T`, distinct
captured lifetime targets and the caller assumption `T: 'static` declared ten
positive outcomes before execution. The published compiler exhausted its
one-million-step traversal budget at 1,024 and 4,096 distinct targets, although
4,096 prefixes with a common target passed. The actual requirement output was
linear in target count; the per-target structural walks repeated shared prefixes.

Formation now uses canonical component sets shared across targets. The existing
radix-map implementation moved from `types/relations` to `support/radix_map`, with
its own compact error type and conversion back to lifetime-constraint errors;
its map algorithm and invariants are unchanged. Structural summaries distinguish
types, argument groups, parameter lists, binder depth and component policy. Each
target visits shared set nodes instead of walking the original prefix again.
One target/policy retains the incremental walk, promoting once when another pair
appears. Published debt identities survive promotion, preserving their original
source. This avoids flattened component vectors for every prefix.

Whole-debt local-capture filtering still precedes decomposition. Independent
nested formation, opaque projection identities/depths, repeated-root slot
validation and separate body permissions remain intact. Seven new maintainer
tests cover independent growth axes, comparison with the original component
walk across scopes/policies, shared wide parameter lists, late promotion, policy
changes and the actual 4,096-target body proof with a uniquely sourced unpaid
debt. Both policy orders and source retention after promotion are exercised.

The charged-work invariants include:

| Prefix family | Scale | New parameter debts | Charged steps |
| --- | ---: | ---: | ---: |
| Repeated component, distinct targets | 1,024 | 1,024 | 23,564 |
| Repeated component, distinct targets | 4,096 | 4,096 | 94,220 |
| Distinct components, common target | 4,096 | 4,096 | 45,056 |
| Distinct components and targets | 128 | 8,256 | 28,617 |

The last family has quadratic output. These controls establish the
bounds for these prefix families, not a universal linearity claim for every type
graph or caller population. No resource limit was raised. The preexisting
4,096-prefix/two-target control passes at 118,793 charged steps.

A frozen typed driver compares the actual body proof on inputs both compilers
accept. Initial unconditional set construction improved distinct targets but
regressed the common-4,096 case by 31.51% wall time; those observations remain in
`obj/pa21-work/formation-targets-v1/probe-comparison`. The revised strategy's nine
jobs retain 234 observations and 936 invocations, A/A calibration, five paired
ABBA blocks and same-binary controls. Every recorded output hash matches its
exact declared accepted output; ten separate correctness cases also match.

| Typed driver case | Paired wall change | Instruction change |
| --- | ---: | ---: |
| Common target, 128 prefixes | +0.86% | −1.44% |
| Common target, 1,024 prefixes | −1.80% | −1.88% |
| Common target, 4,096 prefixes | +0.64% | −2.00% |
| Distinct targets, 128 prefixes | −32.97% | −72.80% |
| Distinct targets, 512 prefixes | −82.55% | −92.44% |

Same-binary medians span −2.06% to +1.86%; the largest calibration spread is
21.33%. These are typed-probe measurements, not general source compile-time
numbers or a rustc comparison. Four Massif profiles include graph construction
and the body proof: common-4,096 peak heap changes from 1,617,209 to 1,616,641
bytes, while distinct-512 changes from 3,840,231 to 303,935 bytes. Combined peaks
change from 1,624,984 to 1,624,528 and 3,867,168 to 312,520 bytes, respectively.

Full validation in `obj/pa21-work/formation-targets-final` passes 35 gates:
strict formatting and all/pedantic Clippy, the 868-file audit with zero findings,
harness/backend/upstream checks, 1,454 maintainer tests (247 library invariants),
91 native groups and all 6,228 released course cases. PA19 independently checks
1,080 sources and 641 LowIR cases, including 254 executable comparisons; PA20
checks 1,345 sources and 799 LowIR cases, including 484 executable comparisons.
The existing seven plus one documented acceptance differences remain unchanged.
Planned PA21–PA25 discovery still fails explicitly. The run verifies 7,256 frozen
source/configuration hashes and 18,684 unchanged oracle sidecars.

The broader immutable compiler comparison has 33 timing jobs and 29 instruction
pairs, with full/count output equality. Paired wall medians range from −0.68% to
+3.44%, same-binary medians from −1.51% to +1.11%, and instructions from −0.139%
to +0.087%. The largest original A/A spread is 102.89%, so shared-host timings
support no portable speedup claim. The sole above-3% case, shared-depth-callers-96,
receives the predeclared batch-16 AB/AA/BB follow-up: +1.27%, −0.19% and +2.41%,
respectively, with at most 15.39% calibration spread. All original observations
remain. No instruction increase exceeds 1%.

RSS medians decrease by 88–268 KiB; they do not establish a general heap saving.
The ordinary-input Massif heap peak is unchanged at 2,432,261 bytes, with a
16-byte combined-peak increase. The constant-chain heap peak decreases by 180
bytes and its combined peak by 240 bytes. ELF text/data/BSS totals grow from
6,375,421 to 6,391,801 bytes. The broad study and its fixed follow-up retain 936
study observations, 144 qualification observations, 4,680 timed invocations,
720 qualification invocations, 58 Callgrind runs and four Massif runs, separate
from the typed driver measurements above. This incremental comparison does not
settle the earlier b1410 streamed-proof baseline question.

The frozen application hashes are baseline
`d5bc985cc02c02b844e9f5f9ed6ed4aa49ead133c0bd05cf13d0220053ddb16c`
and candidate
`01d66e6c9c87843b82754bb09d143299a4050fd34861e19d99d8e010edb97f1e`.
The final typed driver rebuild matches the measured candidate exactly:
`3c6f51a91aa39d4d6cf7f688dd678caa8b31ffd7b1b1deb63dd375beb561b674`.
The complete summary SHA-256 is
`4ab471fbc75d848689788ddb8219560475265404eb3e914c8748309d587a9aa8`.
The older caller-clause baseline was retired only after exact-commit CI succeeded
for its published replacement, with verified binary hashes and a retained receipt.

The successful storage wrapper removed 5,753 generated files (6.91 GiB of
unshared blocks); receipt: `obj/cleanup/1792361976441780011.jsonl`.


## Diagnostic origins in shared body proofs

Shared quantified proofs now retain per-obligation diagnostic origins without
copying earlier proof or scope prefixes. The lifetime core owns an optional dense
block-label table; the body owns the corresponding source locations. Source zero
is the enclosing fallback. Single-source proofs allocate no label table, and the
final checker resolves a failed source without flattening an index for successful
proofs. Checked source ranges carry no lifetime or allocation authority.

Append checkpoints label only newly produced evidence. New root constraints move
into a constraint-only root block while their variables retain their original
allocation scope. Extension, grafting, rebasing and substitution preserve root
scope and labels. Projection normalization attributes new selection debts to the
first demanding root, including when later roots reuse its normal form. Scoped
formation normalization retains each original obligation's source.

Six new core invariants and two final-body controls cover uniquely unpaid debts,
source-range rejection before identity registration, grafted ambient scopes,
caller substitution, foreign allocation gaps and first-demand provenance. The
4,096-label control retains one scope and 4,097 source slots; ordinary evidence
keeps no table. The existing growing-projection control now uses distinct labels
while preserving its additive work bound. These are typed maintainer controls,
not a release of higher-ranked source syntax.

Thirteen predeclared source controls were independently qualified with pinned
rustc: six accept and seven reject. They distinguish universal/static-only
implementations and callers, captured-lifetime equality, independent/diagonal
slots and required associated equalities. Both frozen RustGM applications still
reject all thirteen at the intentional higher-ranked source gate with identical
diagnostics. None count as semantic-negative coverage for RustGM. Canonical body
quantifier consumption and refining call inference remain unfinished; PA21–PA25
remain planned.

`obj/pa21-work/proof-origins-final` records 34 successful gates: strict formatting
and all/pedantic Clippy, the 870-file audit with zero findings, harness/backend/
upstream checks, 1,462 maintainer tests (255 library invariants), 91 native groups
and all 6,228 released course cases. PA19 checks 1,080 independent sources and
641 LowIR cases, including 254 executable comparisons; PA20 checks 1,345 sources
and 799 LowIR cases, including 484 executable comparisons. Existing acceptance
exceptions remain unchanged. Planned-stage discovery still fails. The run verifies
7,270 frozen source/configuration hashes and 18,684 unchanged oracle sidecars.
The focused audit initially flagged a seventh generic `tests.rs` filename; the
new module is named `source_tests.rs`, with no audit or lint exemption.

The immutable compiler comparison has 33 timing jobs and 29 instruction pairs,
with equal full/count outputs. Wall medians span −4.69% to +3.40%, same-binary
medians −2.55% to +0.44%, and instructions −0.256% to +0.235%. The largest A/A
spread is 71.36%, so these timings establish no portable speedup. The sole
above-3% case, callers-48, receives the predeclared batch-16 AB/AA/BB follow-up:
−1.98%, −3.89% and −1.34%, with at most 11.23% calibration spread. All observations
remain; no instruction increase exceeds 1%.

RSS median changes span −28 to +92 KiB. Massif peak heap is unchanged for both
fixed inputs: ordinary-256 at 2,432,261 bytes and constant-chain-uses-256 at
5,381,881 bytes. Combined peaks increase by 16 and 96 bytes respectively; these
samples do not establish a general memory bound. ELF text/data/BSS totals grow
from 6,391,801 to 6,424,613 bytes. The study and its fixed follow-up retain 936
study observations, 144 qualification observations, 4,680 timed invocations,
720 qualification invocations, 58 Callgrind runs and four Massif runs. This is
an incremental compatibility comparison, not a rustc performance comparison or
a resolution of the earlier b1410 streamed-proof baseline question.

Baseline SHA-256:
`01d66e6c9c87843b82754bb09d143299a4050fd34861e19d99d8e010edb97f1e`.
Candidate SHA-256:
`59a477f77ef506ff183cc510dc5b66157b7c957986e2f64ddee16e3db509d150`.
Summary SHA-256:
`eed4de734aaa598ac2ccac7de8be6cfffebd6a5bb0322a012ec4655e2e2bba73`.
The superseded baseline was retired after exact-commit CI succeeded for its
replacement; the retirement receipt is in `obj/pa21-work/body-quantifiers-v1`.
Successful cleanup removed 5,753 generated files (6.91 GiB of unshared blocks),
recorded in `obj/cleanup/1792365161789258313.jsonl`.

### Canonical body consumption of universal requirements

Completed body nominal requirements now consume canonical quantified clauses
through the declaration query engine. Quantified state is lazy and owned by one
body with immutable caller assumptions. Queries retain isolated selection
inference, share conjunctive universal openings, reserve their actual lifetime
intervals and label only newly appended evidence. Intervening ordinary selections
remain foreign allocations, including those after the last universal query. The
body publishes its shared proof once.

Opening aliases now name their universal slots directly rather than adding an
existential equality debt. The old representation could attribute a later bad
requirement to an earlier good binding. Caller lifetime aliases are restored at
publication in one charged fold under the actual block and callable depths,
preventing their structural equalities from stealing the diagnostic source.
Private witnesses, binder identity and ownership intervals remain intact.

Two core regressions cover source retention through graft/extension/rebase and
substitution, malformed alias intervals and outer-variable escape. Ten body
regressions exercise the actual completed-body consumer, collection and final
lifetime gate: universal/static implementations and callers, independent slots,
outer captures and mutual equality, associated results, failing-root attribution,
interleaved allocation, independent functions and nested callable locals. Fixtures
install canonical completed nominal contracts and matching declaration formation
metadata at the production phase boundary; they do not bypass the final proof.

At 16, 128 and 1,024 distinct growing captured-type roots, nominal work is exactly
`2n + 1`, query work `74n + 1`, and retained type growth `2n`. Each run uses one
shared universal scope. Repeated roots add no queries, sources or allocations.
These invariants bound this graph family; they do not establish a bound for every
trait search or a portable wall-time speedup.

Eighteen independently qualified source controls cover the corresponding Rust
requirements (eight accepted, ten rejected by pinned rustc). All still stop at
RustGM's intentional higher-ranked source gate and count as no RustGM semantic
negative coverage. Four separate investigative rustc controls also show that an
extra static premise alongside the universal premise is ambiguous (E0283); the
canonical caller fixtures therefore replace their ordinary seed with the single
qualified universal premise. PA21–25 remain planned. Source call inference,
associated families and opaque results are not delivered by this slice.

The focused run passes strict all-target/all-feature Clippy, 267 library tests,
37 affected integration tests, the 872-file audit with zero findings and all
33 harness checks. The audit initially reported repeated `body` and `quantified`
filename stems; the new module and tests now have specific names, with no lint
or audit exemption.

The frozen full run passes 36 gates, 1,474 regular tests, 91 native groups and
all 6,228 course cases. PA19 independently checks 1,080 rustc sources and 641
LowIR cases, including 254 executable comparisons; PA20 checks 1,345 sources
and 799 LowIR cases, including 484 executable comparisons. Existing seven/one
acceptance exceptions remain unchanged. Planned-stage discovery still fails.
All 7,286 frozen source/configuration hashes and 18,684 oracle sidecars match.

The immutable comparison has 33 timing jobs and 29 instruction pairs, with
matching full/count outputs. Paired wall medians span −16.90% to +1.13%,
same-binary medians −2.99% to −0.13%, and instructions −0.267% to +0.156%.
No positive wall median reaches the predeclared 3% follow-up threshold, and no
instruction increase reaches 1%. The largest A/A spread is 51.10%; the faster
wall samples establish no portable speedup. These are compatibility measurements
against the published proof-origin baseline, not a rustc comparison or a
resolution of the earlier b1410 baseline question. The new source-gated consumer
is covered separately by the typed work/storage invariants above.

RSS medians increase by 208–404 KiB. Peak heap is unchanged on the two fixed
Massif inputs: ordinary-256 at 2,432,261 bytes and constant-chain-uses-256 at
5,381,881 bytes. Their combined heap/stack peaks change by −16 and +272 bytes;
separate stack peaks rise by 1,088 and 664 bytes. These profiles do not explain
the RSS increase or establish a general memory bound. ELF text/data/BSS totals
change from 6,424,613 to 6,424,581 bytes. The study retains 858 observations,
132 qualification observations, 3,432 timed invocations, 528 qualification
invocations, 58 Callgrind runs and four Massif runs. No measurements were dropped
or repeated until favorable.

Baseline SHA-256:
`59a477f77ef506ff183cc510dc5b66157b7c957986e2f64ddee16e3db509d150`.
Candidate SHA-256:
`b4a665daccccfa231fb953062773db7dfd125928220c0190ac4c12c78ea61e26`.
Summary SHA-256:
`0bb98f816cd48f9e1051b3f346c7717a816aafece6ed0c736045ea67ff008522`.
Evidence is retained in `obj/pa21-work/body-consumer-final`; focused failures,
source qualification and the prior baseline retirement receipt are in
`obj/pa21-work/body-consumer-v1`. Successful cleanup removes 5,755 generated
files (6.91 GiB of unshared blocks), recorded in
`obj/cleanup/1792368503573429539.jsonl`. Current comparison binaries remain pinned.

### Partial inference refinement transport

A separate refining transport now reconciles selected type/const constraints
against the caller's current inputs. It copies only reachable holes, retains
partial structures and shared mixed arguments, and validates lifetime closure
and joint constraints before caller allocation or binding. Fresh holes remain
shared; unchanged holes retain their original identities. Query lifetime debts
stay separate from new equations between already closed caller types. This does
not weaken the existing non-refining probe or authorize selection evidence.

The first prototype passed its eight transport controls but failed three added
regressions: shared partial prefixes had quadratic charged work, callable locals
could escape during ordinary equality, and alpha renaming created caller debts.
These failures and their inputs remain recorded. Reconciliation now batches
structural equations and checks the final graph for cycles once when it contains
no callables. Callable graphs use the existing strict scoped comparison.
Ordinary single-equation cycle checking is unchanged.

At 16/64/256/512 roots, the initial prototype charged 840/7,968/105,600/407,808
work units, including query setup. The new batch construction and transport use
exactly `54n`, through 2,048 roots. With independently refined parent prefixes,
commitment costs `49n + 8`, or `71n + 8` with an explicit refresh, through 2,048.
These bounds cover the tested function-free graph families, not callable fallback
or general trait search. They are work counts, not a wall-time or rustc speedup.

Seventeen new regressions cover partial type/const sharing, numeric narrowing,
resumption, conflict/cycle rejection before caller mutation, lifetime closure,
callable capture and alpha equivalence, dependency wakes and growing graphs.
The small-graph test compares all 2,592 two-equation systems across unrestricted
and integer domains with ordinary unification; a three-equation cycle rejects
in all six orders. The focused run passes 32 affected integration tests, strict
all-target/all-feature Clippy, the 878-file audit with zero findings and 33
harness checks.

A subsequent review found that retaining the bulk pair memo could make later
small equations repeatedly clear its large hash-table capacity. A new invariant
first failed at 16 roots with capacity 28. The batch now owns a separate temporary
memo and restores ordinary scratch on success and failure. Its growing regression
checks both exits and repeated small comparisons. Retained ordinary capacity
stays at 3 through 2,048 roots on both exits; the final full run includes this fix.

Pinned rustc independently qualifies ten source hypotheses (six accepted, four
rejected), including partial nominal/const inference and direct or indirect
universal dependence. This slice does not integrate partial selection with source
calls or open the HRTB source gate. Complete predicate transport, retained partial
selection continuations and scoped witness export remain required. PA21–25 stay
planned.

The revised frozen run passes 39 gates, 1,491 regular tests (268 library tests),
91 native groups and all 6,228 course cases. Strict all-target/all-feature Clippy,
the 878-file audit and 33 harness checks pass. PA19 independently checks 1,080
rustc sources and 641 LowIR cases, including 254 executable comparisons; PA20
checks 1,345 sources and 799 LowIR cases, including 484 executable comparisons.
The existing seven/one acceptance exceptions remain unchanged. All 7,306 frozen
source/configuration hashes and 18,684 oracle sidecars match. Planned stages
continue to fail discovery.

The ten refinement controls preserve baseline behavior. Nine retain the explicit
higher-ranked source gate; the ordinary partial-inference positive still rejects
because selection discards unresolved input refinements. It is a known PA19 gap,
not newly supported source behavior or a successful semantic negative control.
Six further pinned-rustc lifetime-export hypotheses match (three accepted, three
rejected), distinguishing independent private witnesses, directional outlives
requirements and indirect universal dependence. Their actual call consumer is
still to be implemented.

The revised executable was measured independently: 33 calibrated timing jobs and
29 instruction pairs preserve full/count output equality. Paired wall medians
span −3.01% to +5.43%, same-binary medians −2.64% to +0.99%, and instructions
−0.389% to +0.262%. Five wall cases cross 3%; the predeclared three-largest policy
adds nine fixed follow-up jobs. Those paired medians are +0.68% (ordinary-256),
+0.86% (trait-view-32-full) and +2.29% (pending-chain-8). The two smaller outliers,
lifetime-depth-64 (+3.16%) and trait-view-8-full (+4.25%), retain their original
observations. Follow-up same-binary medians span −3.56% to +0.85%; largest A/A
spread is 42.89% in the original study and 8.77% in the follow-up. There is no
portable speedup, general wall-time regression bound or rustc comparison here.
These compatibility inputs do not exercise the new source-gated transport;
its algorithmic evidence is the typed growth and scratch invariants above.

RSS medians change by −536 to −294 KiB. Ordinary-256 peak heap remains 2,432,261
bytes; constant-chain-uses-256 changes from 5,381,881 to 5,381,701 bytes. Combined
heap/stack peaks rise by 224 and 176 bytes, with separate stack peaks rising by
2,728 and 24 bytes. These two profiles establish no general memory bound or RSS
attribution. ELF text/data/BSS total changes from 6,424,581 to 6,412,277 bytes.
The revised study retains 1,092 timing observations, 168 qualification observations,
7,176 timed invocations, 1,104 qualification invocations, 58 Callgrind runs and
four Massif runs. Earlier measurements remain separate historical evidence.

Baseline SHA-256:
`b4a665daccccfa231fb953062773db7dfd125928220c0190ac4c12c78ea61e26`.
Candidate SHA-256:
`e996fd161d2f6ad075fdc950ed2fd364dc009a471b10a37736d048e77666acf6`.
Final summary SHA-256:
`4f40647fa27ba7c68071f634fa974a9b8ac7b854462d2d351fd1a8c2a812f0c8`.
Final evidence is in `obj/pa21-work/refining-call-scratch-final`; prototype
failures and focused observations are in `refining-call-transport-v1`, and the
superseded full measurement is in `refining-call-final`. The superseded draft
executable was retired after replacement validation with a hash receipt; the
published baseline and final candidate remain pinned for publication.

The final cleanup receipt records 3,381 removed generated files before the
interruption (`obj/cleanup/1792372449367052584.jsonl`). A subsequent protected
qualification run completed cleanup, removing four newly generated files
(`obj/cleanup/1792372683471361886.jsonl`). No extra historical executable pins
were retained.

### Retained ordinary partial trait inference

Ordinary calls can now infer a partial constructor or const argument from a
unique compatible implementation while retaining its unfinished conditions.
Selection tries every compatible header before exposing progress; pending or
ambiguous alternatives prevent publication, and an unknown Self is not guessed.
The candidate's lifetime debts remain mandatory through final proof. Seven new
PA19 cases cover four independently executed positive programs and three
independently rejected programs, including bound reordering, nested conditions,
const results, failed conditions, ambiguity and unknown Self.

Deferred conditions and body goals now use their own type/const dependency
notifications instead of rescanning all siblings. Child continuations occupy a
flat arena with stable slots, preserving iterative traversal and destruction.
Body projection and trait notifications retain separate owners. Header waits
keep the exact continuation and freeze local body inference until resumption;
this is not a promise to reconcile arbitrary caller mutation during header work.
Private lifetime allocations survive suspension and exclude foreign gaps.

The initial condition retry prototype had quadratic work: recorded counts were
7,254 at depth 8 and 89,538 at depth 32. Those two historical numbers are explicitly
labelled transcribed tool output because the original log was overwritten.
The final retained-chain regression measures initial work `166d + 170`, resumed
work `82d + 62`, and exactly `d` flat slots through depth 2,048. Body notification
registration costs `5n`, and registration plus delivery costs `12n`, through
4,096 independent goals. Actual reverse-dependency source chains at 8/32/128/512
use 15/63/255/1,023 selections (`2n - 1`); pinned rustc also accepts those programs.
These are bounds for the tested graph families, not general trait-search bounds.

Import records original holes during the existing traversal, maps complete
predicates directly and initializes owned contexts in place. Empty queries avoid
allocating retained mappings and wakeup tables. Mixed input reconciliation keeps
the original lifetime-equation order. Three initial oracle-order changes were
caught and corrected before this frozen run; no old sidecar was hand-edited.

A proposed single-canonical-root shortcut failed a strengthened scaling test
when earlier constraints had already expanded its shared descendants: matching
cost 170 steps at width 16 and 8,514 at width 128. The shortcut is now restricted
to a single variable against an unexpanded canonical leaf. Composite patterns
retain joint cycle checking. Matching the expanded shared-prefix regression now
costs exactly `10n + 8` at 16/128/1,024/4,096 inputs. Its result verification uses
the existing shared finalizer; resolving every prefix separately had exhausted
the test's work budget. No production resource limit was raised. Callable scoped
comparison and ordinary single-equation occurs checks remain intact.

The frozen run passes 41 gates, **1,507 regular tests** (276 library tests),
**91 native groups**, and **6,235 course cases**. Strict all-target/all-feature
Clippy, the 887-file audit with zero findings, 33 harness tests and upstream/backend
checks pass. PA19 checks 1,087 rustc sources and 648 LowIR cases, including 258
native executions; PA20 checks 1,345 sources and 799 LowIR cases, including 484
native executions. The existing seven/one acceptance exceptions remain explicit.
All 7,337 source/configuration hashes and 18,705 sidecar hashes remain unchanged
during validation; a separate check preserves all 18,684 pre-existing sidecars.
The additional 21 sidecars belong to the seven new PA19 fixtures.

The ordinary partial-inference source control now accepts. Nine higher-ranked
controls still hit the explicit source gate; their rejections are not semantic
negative coverage. Quantified witness export, refining higher-ranked calls,
associated type families and opaque types remain unfinished. PA21–25 stay planned
and fail suite discovery; this checkpoint does not release PA21.

Immutable comparison with the published compiler covers 36 calibrated timing
jobs and 32 instruction pairs, including reverse-dependency source chains at
8/32/128. Every measured full/count output matches. Instructions change by
+0.024% on ordinary-256 and +0.032% on callable-definition-256. Other increases
above 1% span +1.101% to +2.524% on the existing trait-heavy inputs; reverse-body-8
and reverse-body-32 cost +4.058% and +3.943%, while reverse-body-128 costs **6.114%
fewer instructions**. All increases above 1% have saved exclusive annotations and
function deltas. Matching/import, frame-copy, hashing and notification costs
remain visible; inlining changes also move attribution between symbols. These
costs are retained as measured overhead, not dismissed using favorable timing.
The earlier prototype's pending-chain-32 overhead was +5.704%; removing duplicate
walks and unused retained state reduced the final observation to +1.962%.

Initial paired wall medians span −2.58% to +8.76%; same-binary medians span
−0.16% to +0.46%, with a maximum A/A spread of 86.49%. Twelve comparisons exceed
+3%. The predeclared three-largest follow-up policy selects trait-view-8-full,
reverse-body-32 and pending-chain-8; their follow-up medians are −0.82%, +0.30%
and −4.06%. Follow-up same-binary medians span −3.52% to +3.85%, with maximum A/A
spread 29.77%. All original outliers remain in the record. These shared-host
measurements establish neither a portable speedup nor a general regression bound;
they are not a comparison with rustc.

RSS median differences span −254 to +696 KiB. The two predeclared Massif inputs
show heap peaks changing from 2,432,261 to 2,432,704 bytes (ordinary-256), and from
5,381,881 to 5,382,561 bytes (constant-chain-uses-256). Their combined heap/stack
peaks fall by 232 and 536 bytes; separate stack peaks change by +1,512 and −2,008
bytes. These two profiles do not establish a general memory bound. ELF text/data/
BSS total grows from 6,412,277 to 6,498,305 bytes.

Evidence is in `obj/pa21-work/partial-selection-final`, with prototype failures
and independent controls in `partial-selection-v1` and three separate historical
preflight studies. The final study retains 1,170 observations, 180 qualification
observations, 7,488 timed invocations, 1,152 qualification invocations, 64 Callgrind
runs and four Massif runs. Earlier draft observations are not pooled into the
final result. Successful guarded cleanup removes 5,769 generated files (6.91 GiB
of unshared blocks), receipt `obj/cleanup/1792377590592496587.jsonl`.

Baseline SHA-256:
`e996fd161d2f6ad075fdc950ed2fd364dc009a471b10a37736d048e77666acf6`.
Candidate SHA-256:
`0bd99225d8282e54cc4ed80d1557b2b40c5a1edf6238466608acb4287c574af1`.
Final summary SHA-256:
`02b406d7619c6e978d6cdd2619b443bb68c09277ba4cc230cc1cc2b13497208a`.

### Quantified call inference and private witness export

Call inference now consumes a whole quantified clause and opens its universal
lifetimes inside an isolated trait-search scope. A unique selected implementation
can refine caller type and const holes while retaining unfinished conditions.
Lifetime witnesses appearing in the inferred input receive stable identities in
the older caller scope; an equality connects each exported witness to its private
origin inside the original proof scope. Universal aliases cannot be exported.
The final region proof remains mandatory. A structurally successful search is
not evidence that its lifetime debts are satisfiable.

The export fold allocates only encountered witnesses and survives suspension.
It preserves previous substitutions, skips foreign allocation gaps, and adds
bridges only for newly encountered witnesses. Later conditions may discover new
private witnesses without replacing earlier caller identities. Nested callable
binders remain local, unused private witnesses remain private, and an empty
quantifier retains its scope. Root completion reconciles the selected predicate
inside its opened scope rather than equating original bound slots with free
aliases in the caller. Ordinary queries retain their existing path and leave the
new scope/obligation metadata unallocated.

Thirteen independent full-source controls are compiled to objects by pinned
rustc 1.98.0 in the regular test suite. They cover independent/longer/shorter
private lifetimes, older caller captures with and without a static permission,
indirect equality, unused witnesses, nested callable inputs, partial inference
and witnesses introduced by a later condition. The corresponding internal tests
install exactly the canonical requirement at the declaration/body boundary,
then run actual body inference and the final borrow-region proof. These are
qualified typed-consumer checks, not end-to-end HRTB source or native coverage.
Thirty-one public source-gate diagnostics remain byte-identical to the published
compiler, and their rejection is not counted as semantic negative coverage.

Retained universal dependency chains through depth 2,048 take initial work
`198d + 224 + 2 log2(d)`, resumed work `144d + 189`, and own `d + 4` lifetime
identities, including precisely one caller export. Waiting without changed
inputs performs no search work. The complete scoped proof passes its unchanged
100,000-step lifetime budget. Exporting shared canonical prefixes costs `3n + 5`
through 4,096 roots and still allocates just one witness when the private source
interval contains thousands of unused IDs. These bounds describe the tested
families, not arbitrary trait search.

The frozen validation passes 34 recorded gates, **1,524 regular tests** (286
library tests), **91 native groups** and **6,235 PA1–PA20 course cases**. Strict
all-target/all-feature Clippy, the 896-file audit with zero findings, 33 harness
tests, stage layout and upstream/backend checks pass. PA19 independently checks
1,087 rustc sources and 648 LowIR cases, including 258 executable comparisons;
PA20 checks 1,345 sources and 799 LowIR cases, including 484 executions. The
existing seven/one acceptance exceptions remain documented. All 7,318 frozen
source/configuration hashes and 18,705 oracle sidecar hashes remain unchanged.
PA21–PA25 discovery continues to fail explicitly as planned. No course fixture,
manifest or oracle was changed in this checkpoint.

The immutable comparison with the published compiler covers 36 calibrated timing
jobs and 32 instruction pairs. All measured full/count outputs match. Instruction
changes range from **−0.241% to +0.235%**; ordinary-256 changes by +0.021% and
callable-definition-256 by −0.052%. No increase exceeds the 1% instruction-review threshold. The original profiles and all observations remain
available.

Paired wall medians range from −2.98% to +2.92%; same-binary medians range from
−0.54% to +0.17%, with maximum A/A spread 46.71%. No comparison crosses the
predeclared +3% follow-up threshold, so the fixed follow-up policy schedules no
additional timing jobs. These shared-host results are not a portable speedup,
a general regression bound, or a comparison with rustc. RSS median differences
range from −262 to +182 KiB.

The two predeclared Massif inputs change peak heap from 2,432,704 to 2,432,261
bytes (ordinary-256) and from 5,382,561 to 5,381,881 bytes
(constant-chain-uses-256). Their combined heap/stack peaks increase by 56 and
232 bytes; separate stack peaks change by −1,704 and +3,816 bytes. These two
profiles do not establish a general memory bound. ELF text/data/BSS grows from
6,498,305 to 6,518,785 bytes.

Final evidence is in `obj/pa21-work/witness-export-final-v3`; focused controls and
failed preliminary validation runs remain separately recorded. The final study
retains 936 observations, 144 qualification observations, 3,744 timed compiler
invocations, 576 qualification invocations, 64 Callgrind runs and four Massif
runs. No historical draft observations are pooled into this result.

Baseline SHA-256:
`0bd99225d8282e54cc4ed80d1557b2b40c5a1edf6238466608acb4287c574af1`.
Candidate SHA-256:
`06fa33e119a665d8e7cd4ede9cfb970f40bd46dbbe693fd721c67cb0653ff1f0`.
Final summary SHA-256:
`62a8b201f6b31829527d39d11fc74e764e971c819a1af8c9e32bf689a8127413`.

PA21 remains planned. Source binder production, remaining scoped declaration and
instance-witness consumers, associated type families, opaque types, their
variance/destruction integration and the public inspection/fixture contract
remain unfinished. PA22–PA25 remain planned as well.

Successful guarded cleanup removes 5,759 generated files (6.90 GiB of unshared
blocks), receipt `obj/cleanup/1792383374320749115.jsonl`.

## Parameter defaults and shared occurrence summaries

This checkpoint converts the parameter-default consumer to whole clauses and
corrects its eligibility rule. That rule differs from ordinary closed-bound
validation: any lifetime occurrence in the original bound defers the default
check, including static lifetimes and nested callable locals. An unused clause
quantifier alone does not defer it. A lifetime introduced only by substituting
the default cannot change the original decision. Eligible quantified requirements
retain their complete binder through substitution and final lifetime proof.

Projection normalization requires two distinct inputs to this decision. Retained
original bounds supply lifetime eligibility: erasing a lifetime does not permit
an early check, and introducing one does not suppress that check. The normalized
clause supplies type/const parameter occurrences. If normalization erases the
parameter entirely, that parameter's default does not create a new obligation.
Independent source probes exposed both directions before final qualification.
Ordinary aliases retain their existing expansion behavior.

The occurrence index now computes one compact summary per shared canonical type,
list, argument group and trait application within an owner. Its values distinguish
no parameter, exactly one identity, and multiple identities. Equality results,
const parameters and non-injective projection inputs participate. This is an
occurrence index, never proof or an inference of injectivity. Iterative traversal
charges nodes and edges against the existing million-work budget; no growing
prefix retains a flattened parameter set.

The prefix invariant measures 115, 899, 7,171 and 28,675 work steps at widths
16, 128, 1,024 and 4,096: `7n + 3` for this family. The former fresh traversal
measures 152 and 8,384 steps at widths 16 and 128; the larger old cases were not
run. Retained summary entries stay within `2n + 1`. A separate 4,096-wide argument
list shared by 4,096 distinct traits requires 28,676 work steps. These are index
work/storage checks, not bounds on whole-frontend work or heap allocation.

Five internal invariants cover classification, whole-clause proof and shared
occurrences. The unused-quantifier controls install canonical clauses at the
consumer boundary and independently qualify their source hypotheses with pinned
rustc; they do not claim higher-ranked source admission. PA18 adds three public
controls, PA19 eight and PA20 ten, each in its earliest owning default-requirement
cluster. Their 21 declared outcomes agree with rustc, and all ten accepted
programs execute through rustc and CPPGM. Failed actual uses reach the intended
bound diagnostic, rather than an unsupported-feature gate. No new acceptance
exception is introduced.

Final qualification passes **1,531 regular tests** (291 library tests), **91 native
groups** and **6,256 course cases**, strict all-target/all-feature Clippy, the
900-file audit with zero findings, 37 harness tests, layout and upstream/backend
checks. PA18/PA19/PA20 independently qualify 441/1,095/1,355 source outcomes and
441/656/809 LowIR cases, with 271/262/489 executable comparisons and three PA18
codegen-only controls. Their existing one/seven/one acceptance exceptions remain
explicit. The run records 30 cumulative gates plus seven qualification/reference
gates. It verifies 7,446 frozen source/configuration hashes and 18,768 oracle
sidecar hashes; all 18,705 sidecars predating this checkpoint retain their bytes.
The 31 higher-ranked source-gate diagnostics are unchanged, and planned PA21–PA25
discovery still fails explicitly.

Evidence is in `obj/pa21-work/parameter-defaults-final-v2`, with exploratory and
focused controls in `obj/pa21-work/parameter-defaults-v1`. Preliminary snapshot,
fixture-loading and audit failures remain separately recorded; they are not
included as successful gates. The final test reads public source fixtures through
ordinary test I/O, respecting the include-macro vocabulary audit. Successful
independent checks remove their generated object/metadata files; the guarded run
cleans remaining disposable outputs and retains inputs, diagnostics and receipts.

The immutable comparison with the published compiler covers 46 calibrated timing
jobs and 40 instruction pairs. All measured full/count outputs match. The eight
new source inputs vary the number of distinct bounds and shared type depth
independently; each also passes pinned-rustc object generation. Instruction
changes across the complete matrix range from **−6.563% to +0.082%**. Ordinary-256
changes by −0.016%. Growing default-bound/depth pairs 8/8, 32/32, 64/64 and 96/96
change by −0.173%, −3.098%, −5.679% and −6.563%; 256 bounds sharing depth 64 change
by −6.489%. No increase reaches the 1% review threshold. These whole-frontend
measurements complement the linear index invariants; they do not establish
linear complexity for other phases.

Paired wall medians range from −5.50% to +1.64%; same-binary medians range from
−3.96% to +0.71%, with maximum A/A spread 71.50%. No increase crosses the
predeclared +3% follow-up threshold, so no additional timing jobs are scheduled.
Retain the original observations; this noisy shared-host study does not establish
a portable speedup or compare RustGM with rustc. RSS median differences range
from −886 to +612 KiB.

The two predeclared Massif inputs have unchanged peak heaps: 2,432,261 bytes for
ordinary-256 and 2,984,230 bytes for default-bounds-256-depth-64. Combined
heap/stack peaks are unchanged for the first and increase by 384 bytes for the
second. Separate stack peaks decrease by 3,128 and 32 bytes. These two inputs
do not establish a general memory bound. ELF text/data/BSS grows from 6,518,785
to 6,535,165 bytes.

The final study retains 1,196 timing observations, 184 qualification observations,
4,784 timed compiler invocations, 736 qualification invocations, 80 Callgrind runs
and four Massif runs. Historical draft measurements are not pooled into it.
Baseline SHA-256:
`06fa33e119a665d8e7cd4ede9cfb970f40bd46dbbe693fd721c67cb0653ff1f0`.
Candidate SHA-256:
`81b0cd2e2d357f8dc32e03b1b74dbfa0be4686098551dae34a72ca4b3eb67643`.
Final summary SHA-256:
`a0e7222ae03b7f60a6708d95b8167bd25b84bd378771361276d2109d182dee05`.

The cadence review in `cadence-review.json` records approximately 10 seconds for
all PA1–PA20 text suites, 260 seconds for release Cargo tests (including the
36-second build), 164 seconds for native maintainer tests and 111 seconds for
the affected PA18–PA20 native/source checks. These are local elapsed times,
excluding cleanup. The final receipts for this checkpoint, witness-export-final-v3
and partial-selection-final contain no course failures; this is not an audit of
every intermediate attempt. Focused independent source probes found the current
compiler defects; preliminary broad-run failures were evidence preparation and
fixture/audit wiring. Full text checks remain a cheap regression gate. Related
implementation edits can share a comprehensive checkpoint, with focused checks
and early audits between them, under the testing cadence documented at the root.

PA21 remains planned: higher-ranked source production, remaining scoped consumers,
associated type families, opaque identities/capture, variance/destruction
integration and the public inspection/fixture contract are still unfinished.
PA22–PA25 also remain planned.

## Builtin clauses and proved lifetime identity

The builtin consumer now retains complete quantified `Copy`, `Clone` and `Sized`
clauses. Formation records the normalized declaration input once; body consumers
read that record. Ordinary composite bounds containing lifetimes use the same
structural matcher, with an explicit distinction between caller slots and locally
quantified slots. Matching preserves repeated-slot correlations, enclosing
captures and nested callable binders. It does not infer an element bound from a
bound on a containing tuple.

Declared and signature-implied lifetime equalities are computed by an iterative
strongly connected component traversal. Inferred call arguments retain pending
builtin requirements in their body contract. Only the completed body lifetime
proof may supply additional equality evidence for those requirements. The abstract
local-region sentinel never equates separate runtime borrow extents. Template
ownership and cleanup use the resulting scoped environment; concrete instances
substitute, normalize and independently validate their retained requirements.
Ordinary environments without region-bearing builtin assumptions skip this path.

The shared matcher memoizes type/list/argument pairs and persistent slot maps.
Its growing-prefix invariant covers widths 16, 64, 256 and 1,024, with bounded
linear charged work and one lookup per repeated query. SCC tests compare all 512
small graphs against transitive closure and cover cycles through 16,384 nodes
under a linear budget. Caller environments own their caches; substitution closes
colliding lifetime equivalence classes without capturing new nested locals.

Nine independently declared PA18 fixtures cover mutual, transitive, static and
implied equality, direct and inferred calls, nested tuples, and rejection of
unrelated/one-way lifetimes or a `Clone`-only premise. Rejected bodies are unused,
so demanding a concrete instance cannot hide a template error. All nine agree
with pinned rustc object generation. All six accepted sources execute with the
qualified CPPGM backend and rustc, alongside fourteen typed-boundary builtin
controls. The old 18,768 oracle sidecars retain their bytes; the new fixtures add
27 sidecars. PA18 now has 450 cases: 280 accepted and 170 rejected, including
277 executable cases and three deliberately divergent compilation-only cases.

The frozen candidate is
`38981d5c10c56aca1430841f244dfe75cdedc820c79488c634a447735945250b`.
Evidence is retained in `obj/pa21-work/region-permissions-final-v1/`; earlier
focused failures remain in `builtin-clauses-v1/`. Public PA21 syntax gates remain
closed. User-trait quantified instance witnesses, lexical binder contexts,
projection equalities, associated families, opaque types and the new semantic
view still require implementation and release qualification.

The complete functional checkpoint passes 1,560 release tests, 91 native
maintainer groups, all 6,265 PA1–PA20 text cases, strict all-target/all-feature
Clippy, the file/architecture audits and 39 harness tests. PA18–PA20 independent
acceptance suites pass all 2,900 declared checks with their existing documented
stage differences. Their native lanes pass 1,028 executable comparisons and
three code-generation-only comparisons. Upstream verification passes. All 31
retained higher-ranked source-gate diagnostics match the published compiler;
PA21–PA25 suite discovery still reports planned stages instead of empty success.

The initial 22-input performance study found a repeatable cost on existing
lifetime-bearing composite bounds: 6.40–6.74% more instructions at 16/64/256
functions, with the predeclared 64-function follow-up at +5.06% wall time
(A/A +1.34%, B/B −0.06%). The profile identified redundant structural matching
of exact assumptions and deep copies of immutable indexes. Exact Copy/Sized
facts now bypass matcher allocation after checking caller identity; Clone alone
still consults other evidence for Copy. Environment clones share immutable
builtin indexes and lifetime classes, while extension uses copy-on-write and
substitution constructs its replacement directly. A dedicated invariant checks
1,024 exact queries without matcher allocation, extension isolation and rejection
of a foreign caller's cache.

The resulting compiler is
`e2ffd24f2c885310bbe7cd279ba2ee15185433ff3913ae3f4ff6598c9870fa94`.
The seven-input changed-path remeasurement is in
`obj/pa21-work/region-permissions-fast-v1/`. Both studies retain frozen binaries,
inputs, full-output equality, A/A calibration, five ABBA blocks of four executions
per observation, and Callgrind counts. All observations from the earlier version
remain available; this is an implementation change followed by a new measurement.

Against the published compiler, exact-composite instruction differences are now
+3.69%, +3.69% and +3.89% at 16/64/256 functions. Their initial paired wall medians
are −1.05%, +1.33% and +3.50%. The predeclared 256-function follow-up is +1.41%
(A/A −1.08%, B/B −1.65%); it does not establish a portable throughput improvement
or zero-regression bound. The ordinary control changes −0.52% in instructions;
the lifetime-depth control still costs +4.20%. This remaining cost is reported,
not treated as timing noise. Peak RSS medians for the largest composite control
are 17,970 KiB before and 19,092 KiB after; all seven jobs retain individual
memory observations separately from timing.

The newly accepted inferred-equality inputs have no comparable successful old
compiler run. Before the reuse optimization, their 16/64/256-function instruction
counts are 40.4/159.5/641.2 million, approximately proportional to input width.
The final 256-function input uses 640.7 million instructions, measures 122.9 ms
median wall time and 21,182 KiB peak-RSS median. This is new-capability scaling
evidence, not an A/B speedup claim. Every new scaling source passes rustc
qualification; the largest direct and inferred-equality sources execute through
both compilers, and all fourteen typed-boundary controls retain their previously
executed source and LowIR bytes after the reuse change.

Final publication qualification for `e2ffd24f` passes **1,561 regular release
tests**, **91 native maintainer groups**, **6,265 course cases**, and the same
2,900 independent acceptance checks and 1,028 executable comparisons described
above. All 31 source gates retain their expected diagnostics. Exact final source,
binary and fixture hashes are verified in `region-permissions-fast-v1/validation/`;
`final-summary.json` records this final compiler separately from the preceding
measurement candidate. PA18 provenance is regenerated for this compiler, while
all existing and newly qualified oracle bytes remain unchanged. Strict Clippy,
audits and the 39 harness checks passed for these exact compiler sources before
the final release/native run. The PA21–PA25 goal remains incomplete.

## Lexical higher-ranked source clauses

The source decoder now connects where-clause and trait-position lifetime binders
with the existing canonical clauses and scoped proof consumers. This is an
implementation checkpoint, not a PA21 release. GATs, opaque types, the new semantic
view, the public manifest and the remaining quantified consumers are incomplete.
The earlier entries describing a closed source gate are historical checkpoints.

A where binder owns its subject and every following bound; a trait-position
binder owns its application and associated equalities, while its subject captures
the enclosing scope. Empty written `for<>` binders and unused declared slots are
retained explicitly. Canonical tests inspect alpha renaming, distinct captured
and local slots, and three lifetime depths inside nested callable arguments.
Shared subjects are scheduled once. Their relocation uses one cached structural
shift, rather than walking the same wide type again for every bound.

Bound preparation, original predicates, associated binding records and deferred
projection roots retain these contexts. Projection naming stores complete
clauses. A local annotation can instantiate quantified lifetime arguments with
fresh body inference identities; repeated identical clauses in one query share
that instantiation. A signature cannot choose unspecified quantified arguments.
Quantified impl requirements enter the scoped proof machinery and retain their
lifetime debts. A concrete callee premise with no enclosing lifetime captures
needs no imported witnesses; normalization independently selects and proves its
validated implementation. Captured quantified instance witnesses remain a gap,
not a permission to flatten local witnesses into global assumptions.

Independent probes also qualified invalid binder combinations. Rustc rejects
both `?for<...>Sized` and `for<...>?Sized`, and a binder applied to a lifetime
bound. Three PA5 fixtures now cover those parser rejections. Named quantified
lifetimes cannot shadow an enclosing lifetime. The new source regression suite
has 43 outcomes, with four additional canonical binder tests and a legacy-view
profile test. An earlier callable-outlives hypothesis was corrected using object
emission: rustc accepts the unused generic declaration but rejects its demanded
call. Local shorthand projection inference is likewise distinct from shorthand
in an item signature; both source hypotheses and original diagnostics remain in
the retained investigation.

The frozen candidate is SHA256
`c3e10f673f6ac16eb42dab5fa6abf3921c584eb3d7a885d4a3a0beee98b69014`.
All 22 accepted source controls executed through both rustc and CPPGM. All 6,268
PA1–PA20 text cases pass. The sweep identified one stale PA19 LowIR boundary:
its program now legitimately compiles. That unchanged rejection oracle now runs
in PA19's semantic inspection lane, whose ordinary-predicate vocabulary cannot
represent quantified clauses. The independent-outcome reason is explicit.
PA19 has 440 inspections and 655 LowIR cases; its total and 262 executable
positives are unchanged. PA5 has 219 cases. All 18,795 preexisting oracle sidecars
remain byte-identical, with nine new PA5 sidecars generated by the ordinary
reference runner. Strict all-target/all-feature Clippy, the Rust audit, 39 harness
unit checks, layout and upstream verification pass for this snapshot.

Measurements compare immutable published `e2ffd24f…` and candidate `c3e10f67…`
binaries, frozen inputs, full output equality, A/A calibration, five paired ABBA
blocks and Callgrind. Four supported controls changed instruction counts by
−0.047% (ordinary), +0.014% (lifetime depth), +0.104% (parameter defaults) and
−0.134% (exact composite bounds). Median wall changes were −0.43%, −1.34%, −2.31%
and −0.94%; the defaults control had 102.8% A/A spread. These noisy wall samples
are not a portable speedup claim. Median peak RSS changed by 60, 2, 62 and 40 KiB
respectively in the separate retained ABBA observations.

New controls grow both the shared subject width and trait-bound count to
16/64/256, with roughly linear source growth. They took 3.62/12.03/45.76 million
instructions, 3.43/4.93/17.56 ms median wall time, and 7,316/8,080/8,818 KiB median
peak RSS. This supports approximately linear work across the measured sizes;
it is not a whole-frontend complexity proof. The old compiler rejects these
sources, so no A/B performance claim is made for their new acceptance.

Evidence lives under `obj/pa21-work/quantified-instance-v1/`, with the frozen
snapshot and qualification, performance and memory receipts in `candidate-v1/`.
Final checkpoint validation passes **1,567 regular release tests**, **92 native
maintainer groups**, **3,119 independent PA5/PA18–PA20 acceptance checks** and
**1,028 PA18–PA20 executable comparisons**, plus three codegen-only comparisons.
Fourteen regenerated typed builtin controls retain their previously executed
source and LowIR hashes. Source, fixture and compiler hashes match the frozen
candidate at both ends of validation. The release suite took 307.7 seconds,
native maintainer suite 192.1 seconds and affected independent suites 151.7
seconds; the successful PA1–PA20 text sweep took 10.2 seconds. These are local
validation timings, not compiler performance comparisons.

Remaining source gaps include captured quantified instance witnesses,
associated equality environments, quantified associated result bounds and
remaining conformance consumers. PA21–PA25 remain incomplete, and all five
planned-stage discovery checks still fail with the required status 2.

## Outer conditions of quantified declaration premises

Captured quantified user-trait premises now supply their necessary outer lifetime
conditions to a specialized body. This uses the existing directed region solver
to eliminate private existential witnesses in their original universes. The
result contains only relations between the body's original lifetimes and static;
local witnesses never become unrestricted body universals. A separately proved
call must establish the exact declared premise before these conditions can be
assumed. Manually constructed instances and normalization-discovered debts still
require proof. Unresolved projected conditions remain unsupported by this summary.

Four newly accepted controls cover a static capture, a private witness forcing a
capture static, a private intersection permitting a local capture, and a directed
outer relation. Three independent negative controls check missing required
relations. All seven agree with pinned rustc object/native qualification, and all
four positives execute through CPPGM. The higher-ranked source suite now contains
48 outcomes, including 26 executable positives. A manual-instance regression
checks that a quantified declaration cannot grant caller authority to a forged
instance. Independent finite-set models compare eliminated conditions for 512
directed graphs against exhaustive root assignments and universal/existential
quantification.

The frozen candidate is SHA256
`fa7621350474e9ca213d030d52991c815b13087955ae4c69859a6f3b5016191e`.
Evidence lives under `obj/pa21-work/quantified-consequences-v1/`. Qualification
passes **1,570 regular release tests**, **92 native maintainer groups**, all
**6,268 PA1–PA20 text cases**, strict all-target/all-feature Clippy, Rust audits,
harness checks and audit self-tests. Independent PA18–PA20 suites pass 2,900
declared acceptance checks with their documented stage differences, 1,028
executable comparisons and three codegen-only comparisons. Fourteen regenerated
builtin sources and LowIR outputs retain their previously executed hashes. Source,
fixture and compiler identities match the frozen snapshot before and after the
full validation. No fixture or reference changes are required for this checkpoint.

Seven supported controls compare immutable `c3e10f67…` and `fa762135…` binaries,
with frozen inputs, full output equality, A/A calibration, five paired ABBA blocks
and Callgrind. Instruction changes range from −0.2013% to +0.1287%; no predeclared
slowdown condition is triggered. Wall measurements remain noisy, with A/A spreads
up to 16.27%, and do not establish a portable speedup. New witness chains at
16/64/256 nodes use 3.53/5.76/14.62 million instructions; private intersections use
4.03/6.94/18.66 million. Incremental work grows approximately fourfold for fourfold
additional witnesses. All six scaling sources compile and execute through rustc
and CPPGM. The baseline rejects them, so these are new-capability scaling results.

Separate ABBA memory observations retain a median peak-RSS increase from 8,016 to
8,672 KiB on the lifetime-depth control; the other supported controls increase by
54–174 KiB. A predeclared DHAT follow-up profiles each immutable compiler once on
that input and the ordinary control, checking exact output equality. Both pairs
have identical allocation totals, block counts and peak live heap. Lifetime-depth
allocates 2,656,772 bytes in 25,921 blocks with a 646,909-byte peak; the ordinary
control allocates 7,763,281 bytes in 36,351 blocks with a 2,432,888-byte peak. This
finds no heap-allocation growth on these controls; it does not establish the cause
of the observed RSS difference. All original memory observations and raw profiles
are retained in `candidate-v1/memory/` and `candidate-v1/heap/`.

The full release suite took 303.5 seconds, native maintainer validation 157.2
seconds and affected independent suites 136.4 seconds. These successful checks
were run once for the completed checkpoint; focused checks and the cheap PA text
sweep served the implementation loop. PA21–PA25 remain incomplete and their
planned-stage discovery checks still return the required status 2. Quantified
associated equalities, result/conformance consumers, projected premise conditions,
GATs, opaque types and PA21's public semantic view still require implementation.

## Quantified associated equalities and result contracts

The maintainer now supports associated equalities inside complete quantified
clauses, including `for<'a> T: A<'a, Out = &'a u8>`, and quantified bounds on
declared associated results. Each clause binds its subject, trait inputs and
result together. Normalization retains the selected equality's private interval
and parent proof; result-bound evidence is checked against the owning result
contract. Impl conformance checks these clauses even in unused definitions.
Quantified lifetime requirements do not become unconditional scalar outlives
promises. The original declaration binders, captures and result correlations
remain available to later consumers.

Equivalent quantified alternatives compare through cached keys that ignore unused
local slots and consistently rename used slots. These keys retain outer captures,
nested locals and result correlations. Each call deduplicates equivalent quantified
requirements before constructing its obligations, retaining the first declaration's
original binder and proof. Ordinary clauses and different calls remain separate.
The maintained source suite contains **75 independently checked outcomes**,
including **43 executable positives**. Earlier inspection formats still reject
higher-ranked records; PA21's new public inspection contract remains unreleased.

Completed inference graphs now retain structural summaries without allocating
canonical types early. Graphs with unresolved type/const holes or projections
remain excluded. Lifetime-only mapping reuses region-free completed roots, and
successful isolated queries can return identities they actually resolved for
those immutable inputs. Failed reconciliation cannot publish them. Batch equations
with a finite, non-callable, projection-free anchor for every pair establish
finiteness during shared matching; other batches retain the joint cycle check and
callables retain scoped equality. Six new invariants cover repeated-query work,
unresolved inputs, lifetime isolation, failed publication, shape mapping, finite
anchors and unanchored cycles.

The frozen candidate is SHA256
`e01b1e79eeb85a06b0dd7bd51fac113dc1edd3f34c6e7f80b6249a6a30e09b07`.
Evidence is under `obj/pa21-work/quantified-equalities-v1/candidate-v4/`.
All **1,589 regular release tests**, **92 native maintainer groups**, **6,268
PA1–PA20 text cases**, strict Clippy, the 928-file audit, harness checks and audit
self-tests pass. Independent PA18–PA20 checks cover 2,900 declared acceptance
outcomes with their existing documented differences, 1,028 executable comparisons
and three codegen-only comparisons. Fourteen regenerated builtin sources/LowIR
outputs retain their previously executed hashes. All source, fixture and compiler
identities match the frozen snapshot before and after qualification. No fixtures
or text references changed.

All 18 scaling programs at widths 16/64/256 now compile and execute through CPPGM.
This includes the two quantified width-256 cases that previously exhausted the
unchanged one-million-work budget. The 16 previously accepted LowIR outputs match
their prior executed bytes. Pinned-rustc compilation/execution results are reused
only for identical source hashes; all 18 CPPGM executions were freshly checked.

The performance matrix compares immutable `fa762135…` and `e01b1e79…` binaries on
eight supported controls and measures 13 newly accepted inputs separately. It
retains full output equality, A/A calibration, five ABBA blocks, software
instruction profiles and eight RSS samples per input. Supported instruction counts
decrease by 0.036%–21.14%. The ordinary width-64 control improves by 15.62% in the
paired wall samples, with a 0.38% A/A spread and 21.14% fewer instructions. Other
wall samples are noisier, with A/A spreads up to 67.29%; they do not establish a
portable speedup. Supported median peak-RSS changes range from −444 to +344 KiB.
No predeclared wall, instruction or memory follow-up threshold is triggered.

Instruction counts in millions for newly supported families are:

| Family | 16 | 64 | 256 |
| --- | ---: | ---: | ---: |
| Quantified equalities | 17.38 | 91.10 | 753.25 |
| Quantified wide subjects | 21.18 | 121.76 | 1,121.41 |
| Quantified result contracts | 17.64 | 66.56 | 262.01 |
| Equivalent wide requirements | 24.84 | 153.56 | 1,519.25 |

Result-contract scaling is approximately linear over this range. Equality and
wide-subject families still grow superlinearly. Retained profiles attribute 41.8%
of quantified width-256 instructions to projection opening, including 32.4% in
whole-environment substitution. The ordinary equality width-256 control spends
79.0% in opening, including 61.8% in environment substitution. These inclusive
figures overlap. Avoiding repeated environment copies and index reconstruction is
the next algorithmic task; this checkpoint does not establish whole-compiler
linear scaling.

The complete checkpoint checks took 548.1 seconds, including 261.4 seconds for
regular release tests and 159.8 seconds for native maintainer validation. The
development text sweep took 14.6 seconds. Successful expensive checks were batched
once for the final candidate. PA21–PA25 remain planned and return status 2.
Remaining PA21 work includes environment sharing, projected premise conditions,
GATs, opaque identities/capture, variance/drop integration, and the public semantic
view with reviewed manifests and oracles. PA22–PA25 follow that release.

## Sharing closed caller environments during projection opening

Projection opening now treats the request and caller environment independently.
A request that needs fresh lifetime aliases reuses the caller's original Arc when
the retained environment summary proves that no outer lifetime can change.
Quantified local binders remain intact, and request opening does not create a new
caller assumption. Environments with captured lifetimes still use the same checked
mapping as the request and retain its restoration equations.

Two typed invariants cover shared environments at widths 16/64/256/1024, independent
request intervals, preserved clauses, absent requested permissions, and captured
environments requiring a distinct mapped value. Their constant-work assertion
measures charged substitution work; it does not include rebuilding the semantic
assumption index in `State::new`.

The immutable candidate is
`b3f3cda9eae009af3c19b69c66bf35982a1dfd268716e7e4f6c144885dd325f8`,
with evidence in `obj/pa21-work/closed-environments-v1/candidate-v1/`.
The development sweep passes all 6,268 PA1–PA20 text cases in 10.5 seconds.
All 18 frozen scaling sources produce exactly the published LowIR bytes. Their
prior CPPGM/rustc execution evidence is linked by exact source/output hashes;
this development comparison does not claim 18 fresh native executions.

Seven focused A/B measurements compare with published 3177cbae (`e01b1e79…`).
They retain A/A calibration, five paired ABBA blocks, four executions per
observation, Callgrind counts, eight RSS observations, and exact output equality.
Quantified-wide widths 16/64/256 reduce instructions by 6.85%/19.38%/33.44%, and
paired wall medians by 6.01%/16.05%/31.91%; their A/A spreads are 3.22%/2.87%/6.05%.
The width-256 candidate median is 116.95 ms. Ordinary captured-lifetime equality
controls and the ordinary width-256 control remain essentially unchanged. RSS median
differences range −126 to +510 KiB; no predeclared follow-up threshold is triggered.

The quantified width-256 profile reduces opening to 105.2 million instructions (14.14%
of the total); `State::new` still accounts for 103.98 million (13.97%) while rebuilding
the assumption index. These inclusive counts overlap. Ordinary captured caller
environments still need repeated mapping. This establishes reduced work for
closed environments, not linearity of the complete projection path.

The complete checkpoint passes all 37 checks: 1,591 regular release tests,
92 native maintainer groups, all 6,268 active text cases, strict Clippy,
the 929-file audit, formatting, harness checks and audit self-tests. Independent
PA18–PA20 checks cover 2,900 source outcomes with their documented stage
differences, 1,028 executable comparisons and three codegen-only comparisons.
Fourteen builtin source/LowIR outputs retain their previously executed hashes.
The exact frozen source, fixture and CLI identities match before and after
qualification. The checkpoint took 708.0 seconds, including 383.4 seconds for
regular release tests and 159.3 seconds for native maintainer validation.

PA21 remains unreleased. Remaining work includes scoped projected conditions,
GAT requirements and normalization, opaque identities/capture, variance/drop
integration, and reviewed public semantic oracles. PA22–PA25 remain planned.

## Projected trait inputs beneath callable binders

Trait input normalization now preserves the original predicate and replaces
projection occurrences in their actual callable scope. The same projection term
under sibling binders has separate evidence. Empty binders, enclosing captures,
nested returned function types and unrelated inference holes keep their identity.
Callable bound syntax waits for the complete caller environment without taking
lifetime-elision ownership away from the callable signature. Universal publication
compares the retained original query, so normalization that erases a lifetime does
not discard its formation or outlives requirement.

Selection uses its existing stack and scope tree for the new requests. Each scope
keeps its normalizer/restoration cache; a shared incremental equality index visits
selected proof nodes once. The index only restores identities and retains every
equation for the final lifetime proof. Closing admits original caller identities
and actual ancestors, excluding earlier sibling allocations. Work invariants cover
widths 16/64/256/1024. Header suspension tests retain these scopes across foreign
allocation gaps and still reject static-only or restricted universal evidence.

The release candidate is
`0e2a2e0fc20619798a0d54990d14b141525eade68a1024a2891d3ac1979fe6a7`,
with immutable sources, binaries and observations under
`obj/pa21-work/family-frontier-v1/release-candidate-v1/`.
The higher-ranked source suite independently checks 96 outcomes, including 60
positive programs. Development validation freshly executes 12 new positive
callable controls through rustc and CPPGM. The full checkpoint executes all 60
accepted higher-ranked programs as part of the native maintainer suite.

Thirteen frozen performance inputs include ten supported comparisons with
published 0abe6ff6 (`b3f3cda9…`) and three newly accepted callable widths.
Supported inputs retain byte-identical LowIR; instruction changes range −0.36%
to +1.24%. Wall samples retain A/A calibration and five ABBA blocks, but several
calibration spreads are large, so these do not establish wall-time improvements.
The predeclared RSS threshold triggers a paired heap follow-up on quantified
width 64 and the ordinary width-256 control. Both have identical peak heap bytes
across versions: 2,145,380 and 2,432,888 respectively. Total allocation changes are
−1,215 and +29 bytes. This does not explain the separate RSS variation.

New callable-root widths 16/64/256 take medians 10.65/13.43/132.06 ms and
13.65/63.85/461.54 million instructions. Their pinned-rustc acceptance is checked;
old compiler rejection time is not a baseline. The width-256 profile attributes
245.04 million inclusive instructions (53.09%) to rebuilding assumption indices,
versus 15.44 million at width 64: approximately quadratic growth. The enclosing
`State::new` count overlaps this cost. The selected-proof traversal invariant does
not imply that complete projection compilation is linear. Sharing immutable
caller indices is the next algorithmic target; captured-environment remapping
remains a separate cost.


The full checkpoint passes all 36 publication commands, plus preflight strict
Clippy and the 936-file audit: 1,602 regular release tests, 92 native maintainer
groups, all 6,268 active PA text cases, formatting, harness and audit self-tests.
PA18–PA20 independent checks cover 2,900 source outcomes with their documented
stage differences, 1,028 executable comparisons and three codegen-only cases.
Fourteen builtin source/LowIR pairs retain their previously executed hashes.
Exact compiler, source and affected fixture hashes match before and after the
checks. Upstream verification covers 24 imported fixtures and 126 Reference files.
The publication commands take 726.0 seconds, including 405.9 seconds for regular
release tests and 177.6 seconds for native maintainer groups; the text sweep takes
10.4 seconds. Cleanup removes 3,433 generated files occupying 7.14 GiB, preserving
sources, diagnostics and receipts. These expensive checks ran once for this
coherent compiler batch after focused development checks passed.

Two source compatibility frontiers remain recorded with their independent results:
rustc rejects one redundant alpha-equivalent callable-bound case that the frontend
accepts, and accepts an unused static-only callable-projection declaration that
the frontend rejects during formation. Neither is counted as a passing negative
control. No public assignment fixture or oracle changes. PA21 remains planned,
including GAT requirements/normalization, opaque identities/capture, variance/drop
integration and the semantic view with reviewed public manifests. PA22–PA25 remain
part of the active implementation goal.


## Sharing immutable caller clause indices

Projection queries now reuse the structural trait/subject index owned by their
immutable caller clauses. The canonical subject key and index live in `types`;
complete matching, inference, universal opening and proof selection remain in
`sema`. Both declaration normalization and body projection queries consume the
same shared representation. Each query retains its own lifetime equations and
implementation availability. No selected result or permission is cached globally.

Trait clause storage uses copy-on-write ownership. An actual clause insertion or
substitution invalidates the derived index; repeated identical insertion preserves
it. Parameter-trait lookup belongs to that same index and is invalidated on change.
Substitution can change coarse subject keys and collapse duplicate clauses without
turning them into ambiguous alternatives. Existing callers retain their original
clauses and index. Builders that alias body lifetimes or filter their clauses
continue to construct their distinct environments.

Four caller-index invariants cover shared storage, caller extension, private
mutation, mapped subject keys, local/captured lifetimes, duplicate collapse and
fresh evidence intervals. The existing opening control now charges initial index
construction at widths 16/64/256/1024, then requires constant additional lookup and
opening work. This replaces its former accounting limitation: index construction
is no longer omitted from the query's work. Existing ambiguity, header-wait,
refinement and separate lifetime-proof controls remain required.

The immutable release candidate is
`53ed1d132ffe33866236c380143eacb3dcb7b7ec883ca1ea125f76118671997c`,
compared against published
`0e2a2e0fc20619798a0d54990d14b141525eade68a1024a2891d3ac1979fe6a7`.
Evidence is retained under
`obj/pa21-work/family-frontier-v1/index-sharing-v1/release-v1/`.
All thirteen frozen inputs are accepted by both compilers and produce identical
full LowIR bytes. The experiment retains four executions per observation, A/A
calibration, five ABBA blocks, instruction profiles and eight RSS observations
per input. No predeclared regression or RSS threshold triggered a repeat.

| Callable roots | Previous instructions | Shared-index instructions | Previous median | Candidate median | Paired wall change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16 | 13.646 million | 12.672 million | 5.618 ms | 5.288 ms | -7.39% |
| 64 | 63.837 million | 46.777 million | 15.258 ms | 12.130 ms | -21.95% |
| 256 | 461.471 million | 182.390 million | 74.255 ms | 36.225 ms | -51.80% |

The corresponding A/A spreads are 3.77%, 19.23% and 0.71%. The other ten inputs
also reduce instruction counts, by 0.095%–15.56%; noisy wall observations remain
in the evidence and do not establish a universal wall-time improvement.
The callable family now grows approximately linearly over the measured widths.
Its width-256 clause insertion work falls from 245.035 million to 2.254 million
inclusive instructions, and `State::new` falls from 255.829 million to 2.428 million.
These counts overlap and must not be added. The result removes the identified
index-rebuilding quadratic path; captured-environment remapping remains separate,
and this does not establish whole-language linearity.

A paired DHAT follow-up checks the largest changed case and an ordinary control.
At 256 callable roots, total allocation falls from 43,139,941 to 16,847,445 bytes;
peak live allocation changes from 4,895,267 to 4,905,435 bytes (+10,168).
The ordinary control changes by +72 total bytes and +40 peak bytes.
The development checkpoint passes all 6,268 PA1–PA20 text cases and twelve scoped
callable executions compared independently with rustc. Planned PA21–PA25 stages
continue to return failure. This optimization does not release PA21 or implement
GAT arguments, opaque types or their remaining contracts.

The publication checkpoint passes 1,606 release tests, 92 native maintainer groups,
all 6,268 PA text cases, formatting, strict Clippy, the 939-file audit and harness
checks. PA18–PA20 add 2,900 independent source outcomes with their existing nine
documented acceptance differences, plus 1,028 executable comparisons and three
codegen-only checks. Fourteen builtin source/LowIR pairs match their retained
executed identities. Candidate, source and fixture hashes are unchanged before
and after validation. The 36 checkpoint commands take 947.34 seconds in this run;
23.84 seconds cover the complete active PA text sweep.

## Associated family argument and declaration groundwork

Selected definitions retain separate parent and member argument groups through
definition waits. Structural substitution maps both owners simultaneously and
uses the complete captured lifetime prefix, including unnamed parent slots.
Its cache key includes both owners and both argument groups. A shared substitution
contains no selection proof or permission to form an application.

Family parameter reservation now precedes trait result bounds and follows impl
owner preparation. Member scopes preserve the parent's slots before assigning
their own lifetime slots in declaration order; type and const parameters retain
their member identity. Lifetime-only families have explicit parameter lists.
Ordinary associated types retain their existing context. Type/const defaults are
rejected, and unused family parameters do not inherit ordinary alias restrictions.

Input and result clauses use one bound worklist. The typed family contract retains
only its input requirements; the associated result contract retains result
promises. Completing these in separate passes would replace the first set of
predicates and discard its retained syntax. Clause classification includes any
associated equalities emitted from a written bound.

Eight internal invariants cover joint substitution, nested and captured lifetimes,
argument kinds, retained selections, independent cache keys, member reservation,
defaults and input/result separation. Ten existing continuation/header checks
also pass. Five source expectations are independently checked with Rust 1.98.0:
three valid declarations and two prohibited generic defaults. Evidence is under
`obj/pa21-work/family-frontier-v1/family-parameters-v1/`.
Strict Clippy and the 943-file audit pass. All 6,268 active PA text comparisons
pass in 18.19 seconds against the current debug candidate, with compiler/source
identities unchanged during validation. Scoped cleanup removes the three probe
objects and preserves their source and diagnostic evidence.

These are declaration and substitution boundary checks, not completed public GAT
support. Application formation, required where-clause analysis, conformance and
all normalization consumers still need integration before removing the source
gate. PA21 and the later assignments remain planned. The full release/native
checkpoint and changed-path measurements remain due before publication.

## Family applications and shared lifetime opening

Qualified and shorthand associated paths now prepare separate parent and member
arguments through the existing declaration worklist. Original applications retain
both argument groups. Trait result substitution maps `Self`, parent parameters
and member parameters together, avoiding sequential replacement of actual arguments
that themselves name parameters.

Projection selection opens parent predicates, member arguments and caller clauses
with one lifetime mapping. Retained selections carry the opened member arguments
through definition waits while preserving the original normalization request.
Nested callable-local lifetimes remain local. Immutable closed caller indices stay
shared; selection permissions remain query-local.

The focused checks pass 24 tests, including argument arity, qualified/shorthand
identity, simultaneous substitution and shared parent/member lifetime opening.
Strict Clippy and the 945-file audit pass. Six source expectations are independently
qualified with Rust 1.98.0; those source/hash receipts are reused for the subsequent
diagnostic-only correction. An ordinary nonparameterized `Out<>` application also
passes the candidate's semantic view.

The cheap text sweep caught two PA20 diagnostic regressions. Restoring the existing
wording fixes both without changing references. The final debug candidate
`399e251a09deb474370a8d0d142d377f30c349e6e5b0e752db1cd4ba9c84c032`
passes all 6,268 PA1–PA20 cases in 14.53 seconds, excluding its 30.87-second build.
Source and candidate identities remain unchanged throughout validation. Evidence
and earlier failed runs remain under
`obj/pa21-work/family-frontier-v1/family-applications-v1/`.

The public GAT source gate remains. Family input obligations, equality consumers,
conformance, body inference and required where-clause analysis still need work;
internal application tests do not establish complete source support. These edits
share a future full publication checkpoint rather than repeating release/native
suites for each consumer change.

## Complete family equality inputs and lifetime promises

Equality selection now imports Self, trait arguments and member arguments through
one shared inference mapping. Trials match every input before committing equations;
failed or ambiguous alternatives leave the caller's variables unconstrained.
Equal results from different family applications do not identify their inputs.
The proved-equality transport and canonical normalizer consume the same complete
application. A direct normalization invariant checks both a selected family result
with restored caller lifetimes and a different argument that must remain opaque.

Declared result outlives promises now retain whether a lifetime belongs to the
parent or the member. Declaration preparation records that distinction once;
queries index the corresponding argument group directly. Parent type/const
arguments cannot displace family lifetime slots. Nested callable-local lifetimes
do not escape as outer promises, and malformed templates or argument kinds fail.
These promises still depend on a well-formed complete application.

Validation passes 49 focused tests plus the added direct normalization invariant,
31 affected integration tests, strict Clippy and the 948-file audit. Nine new
source expectations match pinned rustc: four accepted and five rejected, with
negative diagnostics checked for argument/type mismatch and missing outlives
proof. The candidate's public GAT source gate remains; these independent outcomes
and internal tests do not establish complete public source support.

A shared-graph work invariant grows both argument count and graph depth through
16, 64 and 256. Complete input import/matching charges 267, 1,035 and 4,107 work
units respectively, retaining the original caller's unsolved variable. This
checks linear growth in that family; it does not measure wall time or selection
across growing numbers of distinct equality candidates.

The debug candidate
`f8a26116d7f5ef6afd4c06fa2371bcb61c2aaec78f0c03c148e0fbff25971d7f`
passes all 6,268 active PA text cases in 15.89 seconds. Compiler/source identities
remain unchanged during that sweep. Only the direct normalization test changes
afterward; its focused run, formatting, Clippy and audit pass separately.
Evidence is retained in
`obj/pa21-work/family-frontier-v1/family-equalities-v1/` and
`obj/pa21-work/family-frontier-v1/family-outlives-v1/`.
Full publication validation and performance measurements remain due for the
coherent GAT checkpoint. PA21–PA25 remain planned.

## Retained family input requirements

Canonical family normalization now proves member input requirements after its
parent trait proof, including when that parent comes from a caller assumption.
Joint substitution maps implicit Sized bounds, written predicates and lifetime
relations. An unfinished input contract remains distinct from a completed empty
contract. Result promises never substitute for input obligations.

Member conditions use child frames on the existing iterative selection stack.
Universal conditions retain private scopes and header continuations; completion
preserves their lifetime debts. Family ancestry includes the complete application
and scope. A family's failure or composite proof is not cached as an answer to
the parent trait alone. Ordinary queries allocate no family ancestry entries.

Validation passes 168 focused tests, 31 affected integration tests, strict Clippy
and the 951-file audit. New invariants cover missing and caller-provided member
bounds, implicit versus relaxed Sized, unpaid lifetime relations, universal versus
static-only evidence, unfinished contracts and repeated header waits with foreign
allocation gaps. Eight independently declared source outcomes match pinned rustc
(four accepted and four rejected); these remain source expectations rather than
claims that the candidate publicly supports GATs.

The debug candidate
`31f7415126d7d9df6874f049efbf1c9e7d7d0d474a6f00757cd0abfcf093a98d`
passes all 6,268 active PA text cases in 14.65 seconds. Source and candidate hashes
remain unchanged during validation. Receipts and earlier failed lint/audit/test
compilation runs remain in
`obj/pa21-work/family-frontier-v1/family-formation-v1/`.

The public source gate remains. Internal selection normalization still needs
complete family transport through callable scopes; body inference, conformance,
required where-clause analysis and all input-formation consumers remain unfinished.
Full publication checks and performance comparisons remain due for the combined
implementation batch. The cheap compatibility sweep continues to run at useful
checkpoints; release/native qualification is not repeated for each related edit.

## Nested family normalization and shared parent proofs

Trait selection now retains each opened family application alongside its inferred
parent goal. Callable scopes open parent and member inputs with the same lifetime
folder. Equality selection and selected implementation definitions consume those
opened arguments, while the original request remains unchanged for normalizer
resumption. Child frames prove member requirements on the existing iterative
stack, including nested applications and repeated header waits.

The scaling checks exposed repeated parent proof construction across distinct
family applications. Growing both applications and parent conditions from eight
to 32 increased charged work from 1,550 to 15,302 units. Selection now caches the
completed parent answer before attaching prior input evidence or checking member
conditions. Existing scope, cycle and allocation restrictions still govern reuse;
member failures and composite proofs never enter the parent cache. The resulting
work is 752, 2,840 and 11,192 units for widths eight, 32 and 128, respectively.
Separately, increasing one family's argument count through 16, 64 and 256 costs
746, 2,666 and 10,346 units. These are algorithmic work checks, not wall-time claims.

Validation passes 177 focused tests, 31 affected integration tests, strict Clippy
and the 952-file audit. Nine new nested-selection invariants cover distinct
applications, nested requirements, callable lifetime correlations, selected
definitions, erased-result debts, cycle termination, retained header waits and
both scaling dimensions. Thirteen independent source controls match pinned rustc:
eight accepted and five rejected. Function-pointer implementations explicitly use
`for<'a>`; an implementation for a single outer lifetime is a different contract.
A standalone signature mentioning a restricted family was accepted by rustc, so
the negative control explicitly demands normalization under a universal trait
bound. The earlier signature-only observation remains in the evidence and must
not be treated as a demonstrated rejection rule.

Debug candidate
`9fc07c9d549a26fc01e6e7db1f73a84f188ca6baa43907ab674d46b53c40e148`
passes all 6,268 active PA text cases in 14.66 seconds, with unchanged source and
binary identities during validation. Receipts, source plans and failed development
runs remain under `obj/pa21-work/family-frontier-v1/family-search-v1/`.

The public GAT source gate remains. Body inference, complete original-input
formation, associated-definition conformance and required where clauses remain
unfinished, along with the other PA21 deliverables. Local conformance providers
currently bypass ordinary selection and still reject family arguments; their
future support must also prove member requirements. Full publication validation
and immutable-binary performance comparisons remain due for the combined batch.

## Inferred family result shapes and parent input progress

A uniquely compatible implementation can now expose a family definition's shape
using inferred parent and member arguments together. Canonical normalization and
inferred templates share member arity, owner and lifetime-slot checks. One
simultaneous type/const substitution preserves shared input variables; complete
parent lifetime captures and member lifetimes remain distinct beneath nested
callable binders. Ambiguous or input-refining header probes cannot export a
speculative shape. Erased results leave unused inputs and their requirements
attached to the original projection query.

A completed parent proof can separately refine the body's inputs before member
arguments are determined. Caller equalities can also determine complete member
inputs. These changes retain their proof and wake the original query; they do not
complete member formation or publish a normalized result. Unchanged inputs do not
report progress. Scoped input matching keeps ambient lifetime slots rigid, and
nested projections in member type arguments now wait for their own normalization.

Twelve new internal tests cover mixed type/const inference, sharing across both
argument groups, independent applications, erased requirements, ambiguous trials,
independent headers, callable captures, parent-to-member progress, caller equality
refinement, nested input waits and ambient lifetime matching. The complete focused
selection passes 189 tests; 34 affected integration tests, strict Clippy and the
954-file audit pass. At 16, 64 and 256 distinct inferred member arguments, shape
exposure charges 512, 1,952 and 7,712 work units (`30n + 32`). This is an algorithmic
work invariant, not a wall-time comparison.

Ten source controls independently match Rust 1.98.0 (six accepted, four rejected).
Two additional controls qualify the next inference case: a unique parent header
can constrain an input while its own premise remains pending until the member
result determines another input. Rustc accepts the satisfied Copy case and rejects
the mutable-reference case. The completed-parent handoff above does not yet
implement that retained partial-proof continuation; it must not be counted as
complete inferred-family source support.

Debug candidate
`8a7ab990bde87940ee427cef1a0745b5171085fff27cd6c542a30e842be44d35`
passes all 6,268 active PA text cases in 14.69 seconds. Eight existing symbolic
projection programs execute successfully through rustc and CPPGM, with repeated
LowIR equality. Source and candidate hashes remain unchanged through these
checks. Evidence is in
`obj/pa21-work/family-frontier-v1/member-inference-v1/`; scoped cleanup removes
seven independent probe objects and sixteen native executables, retaining receipts.

The public GAT source gate remains. Retained partial-parent inference, complete
original-input formation, definition conformance and required where-clause rules
still need completion, along with the other PA21 deliverables. Full publication
validation and immutable-binary performance comparisons remain due for the combined
GAT batch. PA21–PA25 still fail planned-stage discovery explicitly.


## Retained partial parents and qualified annotation inference

A body projection that stalls on a unique partial parent can now retain the
solver's exact continuation. Header dependencies and later input changes resume
that search with its lifetime reservation, excluding intervening foreign
allocations. A pending parent may expose a uniquely determined result shape, but
cannot complete normalization. Its proof must finish before the original whole
application is accepted; a failed retained premise rejects the use.

Direct qualified annotations such as `<W<_> as Q<_>>::Out` retain resolved
subject, trait/member paths and member identity instead of demanding a canonical
subject during declaration elaboration. Their arguments reuse the existing
annotation worklist, shared type/const variables and ordered defaults, including
trait Self defaults. The complete original operation remains a normalization
root even when the associated result erases an input. Ordinary fully known paths
retain the existing declaration path.

Parent refinement preserves every surrounding callable scope, including empty
binders. The initial implementation opened each scope separately and retraversed
wide input graphs. A scaling invariant exposed quadratic charged work. The final
implementation opens the entire stack through one shared type-graph traversal;
slot identities, nested locals, existing free lifetimes and older captures remain
distinct. At widths 16, 64 and 256, charged work changes as follows:

| Binder depth and input width | Separate opening | Shared opening |
| ---: | ---: | ---: |
| 16 | 790 | 174 |
| 64 | 9,142 | 510 |
| 256 | 134,710 | 1,854 |

These observations fit `2n² + 14n + 54` and `7n + 62`, respectively. They measure
algorithmic work, not compile-time speed. Nested partial-parent tests also retain
private witnesses, preserve correlated versus independent slots, and exclude a
17-region foreign interval on resumption.

Strict Clippy, the 958-file audit and 194 focused tests pass. Forty-four affected
integration tests pass; two native groups are explicitly ignored in that run.
Twenty new source controls independently match Rust 1.98.0, and their seven
negative checks require the intended diagnostic class. Six maintainer programs
execute with runtime assertions through rustc and CPPGM, with repeated LowIR
byte equality.

The same debug candidate
`a0f740abb9c3afabf66976fb0a9b7117364473d0425e251eb08348b9063e6b74`
passes the existing 6,268 PA text cases in 16.26 seconds. The owning PA20 suite
then gains 26 LowIR fixtures: those 20 controls and six executable programs. All
26 declared source outcomes are independently qualified again from their actual
fixture files; all 19 positives execute with both compilers. PA20's 1,381 text
cases compare exactly, and all 4,065 pre-existing PA20 sidecars remain unchanged.
Together with the unchanged PA1–PA19 run, this validates 6,294 active text cases.
Upstream provenance and harness/layout checks pass. Final reference provenance
matches the manifest and compiler source snapshot; no compiler source or binary
changes during qualification.

Evidence, including failed development checks and the quadratic baseline, is in
`obj/pa21-work/family-frontier-v1/parent-continuation-v1/`. Scoped cleanup removes
50 native executables across the maintainer and fixture qualification batches,
retaining source, diagnostics, hashes and receipts. No full release/native gate
or broad performance study is repeated for these related edits.

PA21's family source gate remains. Complete family-input formation, unused
associated definition conformance and required where-clause analysis still need
integration, followed by opaque/capture, variance/drop and public semantic-view
work. Full publication validation and immutable-binary performance comparisons
remain due for the combined GAT checkpoint. PA21–PA25 remain planned and fail
suite discovery explicitly; the goal through PA25 remains incomplete.

## Unused associated family definition contracts

The declaration checker now validates each family before publishing the checked
impl mapping. Trait and impl parameters share simultaneous substitutions into
the impl's full lifetime scope. A first proof uses the declared rigid identities
and input requirements to reject stronger impl bounds. A second proof uses the
impl's own identities and requirements to establish the declared application's
formation, result well-formedness and result bounds. This keeps implicit Sized
permissions on the correct side and excludes result promises from assumptions.
Captured parent parameters are registered once; own lifetimes remain distinct
from unconstrained impl lifetime captures. Each family solves its local proof
instead of retaining that proof under the smaller parent binder.

The input-contract mapper is shared with projection selection. Explicit
conformance environments reuse the existing trait-bound expansion and parameter
scope construction. Expansion retains and filters its work vector in place;
there is no second implication engine or extra copy of its completed work list.
Parameter preparation now explicitly completes empty family input contracts,
while an unprepared contract still fails.

The original independent investigation had seven incorrect acceptance hypotheses:
declared input bounds are not silently inherited by an impl definition, even when
the result erases the input. The original hypotheses and rustc diagnostics remain
in `obj/pa21-work/family-frontier-v1/family-conformance-v1/`. The revised source
matrix has **42 independently qualified outcomes: 16 accepted, 26 rejected**.
Accepted controls compile to objects with pinned Rust 1.98.0. Controls cover type,
const and lifetime parameter matching; stronger and omitted bounds; implicit
Sized; supertraits; higher-ranked input and result bounds; parent captures; and
families with where clauses but no own parameters.

`dev/tests/invariants/unused_families.rs` prepares typed declarations and invokes
the production impl-type conformance entry point. All 42 outcomes agree, and
rejected cases must reach their intended conformance or lifetime diagnostic
class. This is internal declaration integration, not released GAT source support.
Initial checks exposed a duplicate parent-parameter registration, which was fixed;
fixture preparation was also corrected to complete deferred bounds and nominal
field templates rather than treating missing facts as valid negative outcomes.

Final validation passes **409 unit tests**, **56 affected integration tests**
(six native groups remain explicitly ignored in this development run), strict
Clippy, formatting, the **961-file audit**, layout checks and all **6,294 PA1–PA20
text cases**. The text sweep takes **22.81 seconds**. All 32 checkpoint commands
have their declared status, including planned-stage failures for PA21–PA25.
Source and candidate hashes remain unchanged throughout validation. The debug
candidate is `074c7a47b7b37fd58ac67b0630d7c634965100a34ad1ab0369cbfcb19a57d54a`;
it is not an immutable performance pin. Qualification cleanup removes all 28
objects produced by the two revised rustc batches, retaining source, diagnostics
and receipts. No full release/native gate or broad performance study is repeated.

The family source gate remains closed. Checked local definitions in method and
constant conformance must still normalize family applications with their complete
formation proof; required where-clause analysis and remaining formation integration
also remain. The opaque/capture, variance/drop and public semantic-view work,
PA21 release checks, and PA22–PA25 are still required. Publication validation and
immutable performance comparisons remain due for the combined compiler checkpoint.


## Local family method and constant conformance

Checked local definitions now use the ordinary whole-application normalizer for
families, including those with where clauses and no own parameters. The checked
parent mapping alone never authorizes a member application. Distinct family
arguments, parent captures and nested callable scopes retain their own proofs.
Ordinary associated definitions retain their direct local substitution path.

When families are present, conformance consumes the impl's written method or
constant type as well as its normalized shape. A scalar result cannot erase the
original application's requirements. The declared method signature supplies only
its implied lifetime conditions; it supplies no Copy, Sized or user-trait evidence.
Captured conditions may cross nested callable binders, while their local lifetime
conditions cannot escape. Associated constants do not gain method-style implied
permissions. Erased impl lifetime slots receive independent universal identities
instead of defaulting to static during written-type conformance.

The internal source matrix has **36 independently qualified cases: 22 accepted,
14 rejected**. Rust 1.98.0 object generation agrees with the declared outcomes.
The cases cover local identity and distinct applications, type/const/lifetime
arguments, parent captures, nested callable correlations, omitted requirements,
constant array shapes, declared versus actual-only lifetime conditions and erased
late parameters. The production conformance entry point agrees on all 36 cases;
negative controls require semantic conformance or lifetime diagnostics. This
fixture prepares typed declarations internally, so it does not claim that the
public GAT source gate or all declaration/body consumers are complete.

Independent investigations also distinguish explicit family contracts from
nested nominal conditions that normalization erases. Rustc accepts some unused
signatures with erased nominal inputs, while rejecting corresponding local
annotations. A demanded instantiation of one accepted unused signature causes a
rustc ICE; its two observations remain infrastructure failures, not passing
qualification. Constants do not inherit the signature lifetime permissions
observed for trait methods. Original hypotheses, corrected outcomes, sources,
diagnostics and object cleanup receipts remain under
`obj/pa21-work/family-frontier-v1/local-family-contracts-v1/`.

An initial full text sweep caught one PA20 diagnostic regression: an ordinary
constant's retained projection debt was diagnosed prematurely during conformance.
Programs without families retain their established conformance path and later
annotation diagnostic; no reference output is changed to hide the failure.
The corrected development candidate passes **410 unit tests**, strict Clippy,
the **963-file audit**, layout checks and all **6,294 PA text cases** in
**14.70 seconds**. All 31 development commands have their declared status,
including planned-stage failures for PA21–PA25. Source and binary hashes are
unchanged through validation. Combined publication evidence is being recorded
under `obj/pa21-work/family-frontier-v1/conformance-checkpoint-v1/`.

The source gate remains closed. Shared implied family lifetime formation across
declaration, call and body consumers, required where-clause analysis, opaque
identities/capture, variance/destruction integration and the public semantic view
remain PA21 work. PA22–PA25 are still required by the active goal.


## Combined inferred-parent and family-conformance checkpoint

The pinned release candidate
`b207b78dd676960aed8d522f2a3b4101f6c72de829557d20b1568ab8cfe44889`
passes all **37 qualification commands**: **1,663 regular release tests**, **93
native maintainer groups**, formatting, harness and audit self-tests, upstream
provenance, all **6,294 PA1–PA20 text cases**, and expected planned-stage failures
for PA21–PA25. Strict Clippy and the 963-file compiler audit passed the unchanged
source snapshot during preflight. The checks take **616.53 seconds**, of which
**10.32 seconds** cover the active PA text sweep.

PA18–PA20 independently qualify **2,926 source outcomes**, retaining their nine
existing documented acceptance differences. Native qualification covers **1,047
executable comparisons** and **three codegen-only cases**. Fourteen builtin
source/LowIR pairs match their previously executed identities. Source, candidate
and fixture hashes are unchanged throughout validation. All **4,065 pre-existing
PA20 sidecars** and 1,355 original fixture receipts remain unchanged; the 26 new
inferred-parent fixtures were independently qualified before oracle generation.

Evidence is in `obj/pa21-work/family-frontier-v1/conformance-checkpoint-v1/`.
The immutable comparison uses the published baseline
`53ed1d132ffe33866236c380143eacb3dcb7b7ec883ca1ea125f76118671997c` and
thirteen frozen shared inputs. Every full LowIR output matches. Initial A/A
calibration and five paired ABBA blocks retain every execution, followed by one
Callgrind observation per compiler/input and eight RSS observations. Instruction
changes range from **−0.04% to +1.07%**. No predeclared RSS or instruction-growth
threshold triggers. The largest shared cases show paired median wall changes
from **−0.26% to +2.10%**, without establishing a universal regression bound.

Four short cases trigger the single predeclared longer timing follow-up. Its
paired median changes are **+7.18%** for ordinary-equalities-16, **+3.86%** for
quantified-wide-16, **+10.09%** for shared-inputs-16 and **+4.51%** for
shared-inputs-64. Original observations remain alongside the follow-up. The first
case exceeds both follow-up calibration spreads; the others have wider calibration
noise. This checkpoint does not claim zero wall-time regression. One allocation
investigation of ordinary-equalities-16 finds **+1,519 total bytes**, three fewer
allocations and unchanged **432,495-byte peak**. The ordinary-256 control adds
23 total bytes with unchanged allocation count and **2,432,928-byte peak**. These
observations do not explain away the small-case wall slowdown.

The first candidate-only scaling fixture incorrectly assumed the standalone
profile supplied `assert!`. Rustc accepted it, but RustGM diagnosed the missing
macro before any timing observation. That source, original plan and stderr remain
recorded. Three corrected fixtures use an explicit conditional and independently
compile with rustc; the thirteen shared measurements are not repeated. At 16,
64 and 256 inferred-parent uses, candidate instruction counts are **17,114,659**,
**64,225,373** and **253,137,276**, respectively. The measured instruction growth
is nearly linear across these sizes. Candidate wall medians are 8.42, 18.96 and
105.83 ms, but calibration spreads reach 33.65% and 52.88% on the outer sizes;
those times support neither a clean scaling claim nor a comparison with rustc.

The existing wide-equality controls expose a separate algorithmic cost in both
binaries. On ordinary-equalities-256, opening and preparing the caller consumes
about 76% of candidate instructions; environment region substitution alone takes
about 62%, and building its premise index takes about 14%. The published baseline
shows essentially the same substitution/index costs. Each lifetime-bearing
projection currently copies and reindexes the complete caller environment. The
linear-source-size matrix therefore still exhibits superlinear total instructions.
Sharing this structural preparation while keeping each query's proof permissions
separate is the next performance task; the earlier closed-caller improvement does
not cover these open lifetime environments.

The full qualification cleanup removes **3,433 generated files (7.14 GiB)**;
performance cleanup removes its three probe objects. Immutable comparison pins,
source inputs, diagnostics, all observations and receipts remain. PA21 is still
planned, and the goal through PA25 remains incomplete.
