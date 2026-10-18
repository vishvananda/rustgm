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
