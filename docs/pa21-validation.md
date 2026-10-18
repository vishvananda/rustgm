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

Declaration environments must retain the explicit quantified carrier. Proving a
universal obligation needs fresh scoped universals and escape checks; using a
universal assumption needs fresh instantiation in the actual caller scope.
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
