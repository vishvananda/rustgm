# PA13 pattern implementation handoff

PA12 is published at `ef3b278`. PA13 remains planned until its handout,
implementation, independent fixtures and text oracles are complete. The active
objective also includes PA14 and PA15; the identity work below is a prerequisite,
not completion of pattern semantics.

## Binding identity prerequisite

The resolver previously created one definition from an or-pattern's first
alternative but discarded the relation to its other binding occurrences.
`Resolution::pattern_aliases` now retains each additional occurrence's canonical
definition. Nested alternatives link representatives after visiting their
children, so resolving links in reverse order is linear and avoids recursive
queries or a Cartesian product. Independent tuple columns keep separate bindings.
Ordinary patterns allocate no alias map. Alias records follow source traversal,
independently of hash seeds.

Comparison uses hygienic identifier identity, with the existing name/mode
consistency checks. Captured pattern fragments keep caller binding identities;
equal spellings from different hygiene contexts cannot silently merge.
`Declarations::new` installs those identities in existing node facts, so later
type checking and lowering do not repeat name lookup. Existing definition IDs,
node-record size and earlier text outputs remain stable.

The owning controls are one PA6 nested-binding fixture and two PA8 macro/hygiene
fixtures. Maintainer invariants check the alias-to-definition handoff, nested
representatives, random hash seeds, 4096 alternatives and independent columns.

The checkpoint passes 1,668 active course cases and 247 Rust tests in both debug
and release. PA6's 173 and PA8's 91 classified programs pass independent rustc
checks. Existing oracle bytes are unchanged; all source receipts are refreshed.
Strict Clippy/rustfmt, the 162-file audit, harness and pinned-source verification
pass. The frozen candidate is `obj/pa12-work/pattern-identities-candidate`,
SHA-256 `77e4bf04367a80c213035e1c34761bd7fe75c0b4ac973c5dcde412a0ddb2107a`.
Logs are retained as `obj/pa12-work/pattern-identities-*.log`.

Two frozen A/B runs compare this candidate with PA12's publication binary on
CPU 10, with full-output equality, A/A calibration and three ABBA blocks. All
54 observations per run are retained. Ordinary name-resolution medians at
4k/16k/64k functions are 54.64/55.31, 217.22/214.51 and 900.06/891.05 ms
for baseline/candidate; largest-scale RSS is 139,408/139,372 KiB. The raw record
is `obj/perf/1791826176700433275/samples.json`.

The new nested-alternative workload has 1k/4k/16k procedures and three aliases
per procedure. Baseline/candidate medians are 27.18/27.57, 105.38/105.95 and
439.15/435.55 ms; largest-scale RSS is 63,212/63,672 KiB. It records the cost
of retaining the extra facts while preserving complete name-view bytes. These
observations show no material elapsed regression; they do not establish a
speedup. The raw record is `obj/perf/1791826201830726761/samples.json`.
Reproduce the workload with `scripts/measure_lexer.py --stage names --pattern-input`.

## Implementation sequence

The first owned-product checkpoint is implemented. `sema::patterns` retains
compact pattern IDs, source-ordered selected fields, constructor identities and
shared or-pattern bindings. The checker solves every retained type with the
containing body; body construction consumes these facts without repeating name
lookup, shape selection or inference. Group wrappers do not allocate semantic
pattern nodes. Simple bindings keep their existing path and the dense node
record remains 80 bytes. A 16,000-field tuple with two selected fields retains
three pattern nodes and two projections, without expanding the rest pattern.
Both elaboration and binding construction use reusable iterative worklists.
The checker diagnoses its explicit per-root work budget; the CLI releases the
selected-pattern arena after building body IR.

This checkpoint accepts owned tuple, record, tuple-struct and single-variant
constructor patterns, including nested patterns, rest, parameters, uninitialized
declarations, `@` bindings and irrefutable alternatives. Wildcards form places
without reading their values. Place initializers run once, including indexing
effects, and divergent initializers balance the construction stacks. Copies
from an `@` subpattern precede the containing move; overlapping non-Copy moves
remain rejected by the owned-value checker. Alternatives share binding types
and canonical definitions. Selecting their first alternative is valid only for
this currently irrefutable profile: conditional patterns must add a decision CFG
before extending that path.

A statement-macro regression exposed an earlier storage-scope bug: expansion
containers ended their bindings' storage even though the bindings belong to the
surrounding source scope. Only explicit source blocks now end that storage.
The owning PA10 fixture checks both a binding that survives expansion and an
inner block that shadows it; its declared result, body-IR interpreter and rustc
all agree on 42. No existing text oracle changed. PA10 now has 49 cases,
including 34 independently executed integer-CFG vectors.

Five new maintainer tests cover selected identities, sparse storage, deterministic
LowIR, positive runtime vectors and semantic/move errors. All 18 positive
programs execute with clean zero status and empty streams through the pinned
CPPGM backend and rustc 1.98.0. Their assertions exercise complete values rather
than relying on truncated exit codes. All 15 negative programs are independently
rejected by rustc. This is work-in-progress evidence, not an active PA13 grading
suite; no partial PA13 manifest has been installed.

The checkpoint passes all 1,669 active course cases and all 252 Rust tests in
debug and release. Strict Clippy/rustfmt, the 166-file audit, harness and pinned
source checks pass. The frozen candidate is `obj/pa13-work/product-candidate`,
SHA-256 `098deb0dae8655a518e5be2dbd4ca1b68ba67b8851ee508c651b5f203eb2fee0`.
Logs are `obj/pa13-work/product-*.log`; native product evidence, commands,
source hashes and compiler/backend hashes are retained in
`obj/pa13-work/product-native-1791827855382493997/evidence.json`.
The existing PA11 and PA12 native suites also pass: 194/98 LowIR checks and
180/48 executions respectively, including their declared panic outcomes. All
independent rustc checks and PA12 layout probes pass.

Frozen ordinary LowIR A/B measurements on CPU 10 compare the previous identity
checkpoint with this candidate. At 4k/16k/64k demanded procedures, paired
baseline/candidate medians are 207.12/209.02, 846.87/845.74 and
3547.67/3484.08 ms. Largest-scale peak RSS is 700,816/701,372 KiB. Full LowIR
bytes and count outputs agree. All 54 observations, including warmups, A/A
calibration and three ABBA blocks, are retained in
`obj/perf/1791827983669936871/samples.json`. These measurements show no material
regression and do not establish a speedup.

The new `scripts/measure_lexer.py --stage lowir --product-input` workload demands
1k/4k/16k procedures with nested non-Copy record parameters, tuple rest,
`@` bindings and shared alternatives. Candidate A/A medians are
90.61/375.23/1579.07 ms; paired peak RSS is 23,360/82,560/319,196 KiB.
The 54 observations and full-output hashes are retained in
`obj/perf/1791828079238821288/samples.json`. This records scaling of the new
capability; the previous compiler rejects that source, so it is not an A/B
performance comparison. No RustGM build or test ran during either measurement.

The remaining sequence is:

1. Extend the selected owned-product facts to fixed-array patterns and retain
   binding modes and reference adjustments. Bindings in different alternatives
   must agree on their solved type and effective binding mode. Array moves need
   typed constant-index/subarray projections with correct overlap facts, rather
   than weakening PA12's ordinary indexed-place restrictions. Do not infer those
   facts again during body construction.
2. Type match scrutinees and arms, let conditions and let-else. Validate literal,
   range, constructor, rest, reference and slice patterns, including privacy,
   field/arity checks and edition-2024 binding-mode restrictions. Reuse selected
   constants and target-width scalar operations; PA25 still owns general constant
   evaluation. Let-else failure branches must diverge, and ordinary let/parameter
   patterns must be irrefutable.
3. Implement usefulness/exhaustiveness over selected constructors, using a bounded
   worklist and memoized subproblems. Keep or-patterns shared rather than expanding
   all combinations. Split integer/character intervals at relevant boundaries;
   preserve the Unicode scalar gap and distinguish empty/uninhabited types.
   Guards do not prove coverage. Retain redundant-arm facts and missing witnesses
   deterministically. Work counters and budget failures are required by `spec.md`.
   The Rust compiler's [pattern-checking guide](https://rustc-dev-guide.rust-lang.org/pat-exhaustive-checking.html)
   and [usefulness algorithm description](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_pattern_analysis/usefulness/index.html)
   explain constructor grouping and specialization. The latter documents nightly;
   pinned stable 1.98.0 acceptance remains the course authority. In particular,
   do not equate physical layout uninhabitedness with stable pattern coverage.
   Query keys must retain the relevant visibility and place-validity context.
4. Build match CFGs that evaluate the scrutinee once, test alternatives in source
   order and delay moves until an arm and its guard succeed. Guards inspect
   bindings without prematurely moving their source. Preserve wildcard discards,
   field projection, array bounds and binding storage lifetimes. Add typed
   discriminant reads and constant-index/downcast projections as needed; the
   ordinary PA12 index operation must keep its existing semantics.
   The vendored Reference's `expr.match.guard` rules require retrying a failed
   guard for another matching or-pattern alternative, so a guard's effects may
   execute more than once within one arm.
5. Connect those projections to owned-value checking and LowIR. Variant tags and
   offsets come from completed layout queries; no AST spelling or rendered type
   becomes a backend key. Check complete values by executing the unchanged CPPGM
   backend and pinned rustc. PA14 then publishes the full partial-initialization
   and move-path inspection contract.

## Inspection and later-proof boundaries

The executable assignment remains source-to-LowIR. A supplementary typed pattern
view should expose binding modes, selected projections, coverage and witnesses
for borrowed and unsized scrutinees that cannot yet be executed. Freeze its exact
format only with the implemented typed interface and declared vectors, following
PA12's supplementary-mode harness. Such inspection proves pattern facts, not
lifetime or loan validity, and must never count as native execution evidence.

Keep region and loan obligations intact. PA15 owns lifetime contracts, PA16 owns
loans and PA17 owns drops. General fat-pointer execution remains PA22 work. Owned
fixed arrays can exercise slice-pattern syntax now without silently requiring
fat pointers. By-reference bindings need their semantic facts even while the
executable representation gate remains closed. Do not remove that gate merely
to make a match program lower.

## Test and publication gate

Declare outcomes before generating oracles. Cover nested bindings, tuple/record/
variant/array/rest shapes, guards and failed guards, overlapping ranges,
or-pattern identity and type agreement, empty enums, redundant arms, missing
constructors, cfg/macros/modules and source evaluation order. Use rustc acceptance
for full programs, explicit typed-vector expectations for inspection, and native
execution for every accepted LowIR case. Keep each future-stage difference explicit.

The current PA12 match-boundary fixture must graduate to the owning PA13 suite
when that capability is implemented; it must not continue claiming the same
source is unsupported. Earlier name and expansion fixes remain in PA6/PA8.
Until the whole PA13 contract passes, do not add an empty or partial PA13 manifest
to the active report. Run all active regressions, Clippy/audits, independent checks
and frozen scaling/equivalence measurements before publishing each compiler
checkpoint. PA14 and PA15 remain required after this work.

Initial pinned-rustc contract probes are retained under
`obj/pa13-work/contract-probes/`. Empty matches over an empty enum, a tuple
containing it, a payload enum containing it and a nonempty fixed array of it
are accepted; an empty match over a reference to that enum is rejected. A
zero-length array is accepted with its empty-array constructor. Character ranges
covering both sides of the Unicode scalar gap are exhaustive. The guard-retry
probe executes the same guard twice after both alternatives match and both
guards fail. These are observed stable-toolchain controls for the implementation,
not claims that the current RustGM pattern checker implements them yet.
