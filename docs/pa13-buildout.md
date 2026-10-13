# PA13 pattern implementation handoff

PA13 now has its implemented-draft handout and 803-case grading suite; see
[the release validation](pa13-validation.md). This file preserves the historical
implementation checkpoints in order, including their then-open boundaries and
measurements. Earlier statements that PA13 remains planned describe those
checkpoints. PA14 and PA15 remain in the active objective.

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

## Fixed-array patterns

Owned fixed-array patterns now use the same selected pattern arena. Their
constant indices and half-open subarray ranges are distinct typed projections;
rest bindings retain their solved fixed-array type. Array lengths and ordinary
binding constraints are solved once during body checking. Both parameters and
local bindings consume these facts, including nested arrays, irrefutable
alternatives, macro fragments, `@`, declarations without an initializer and
diverging source places. Bare rest and wildcard patterns do not read values.
Only a direct `name @ ..` binding forms a rest element; nested `a @ b @ ..`
bindings are rejected, as pinned rustc requires.
Borrowed and dynamically sized slice patterns remain required work below.

Owned-value checking partitions each affected array only at selected endpoints.
It sorts and deduplicates those boundaries once, visits nested element/field
paths iteratively and gives each unmentioned interval one state bit. Subarrays
alias the resulting contiguous ranges of bits; they do not receive independent
initialization state. This makes overlap checks include partially moved nested
fields. Storage is proportional to selected paths and endpoints, with
O(k log k) boundary preparation rather than storage proportional to array length.
Preparation has an explicit one-million-boundary budget, and the existing
word-charged ownership fixed point remains bounded. Bodies without these array
projections keep their earlier numbering path. The numbering and partition
scratch is discarded when each body's paths have been completed.

Disjoint elements or ranges can move in either order. A whole-array read sees
all tracked moves, while ordinary indexing still requires the containing array
to be initialized and never permits moving a non-Copy value. Empty subarrays
read no initialization bits and do not move their source. That does not make
ordinary indexing or static storage eligible for a non-Copy move. A whole-array
binding to `[N; 0]` also remains a move, unlike an empty rest projection. These
distinctions follow pinned-rustc controls rather than physical storage size.

LowIR consumes checked projections and completed layout strides, validates
constant bounds and offsets, and preserves existing alignment/copy rules.
The CPPGM backend and its pinned contract are unchanged. No new field lookup,
source parsing or inference occurs during lowering.

The maintainer tests now include 48 owned-pattern executable vectors (30 new
array cases) and 39 independent rejection controls. All 48 execute correctly
through CPPGM and rustc. An exhaustive matrix of the 371 pairs of subarray
ranges for array lengths zero through four agrees with rustc on every move
decision. Separate invariants check nested-field overlap and empty ranges.
A pattern selecting two elements of `[(); 18446744073709551615]` retains three
pattern nodes, two projections, five move paths and six move-state bits.

All 1,669 active course cases, 255 Rust tests in debug and release, strict
Clippy/rustfmt, the 167-file audit, harness and pinned-source checks pass.
The 254-test full runs and the subsequent eight-test pattern runs together
cover the final 255-test set, including the added range matrix. Existing text
oracles are unchanged and source receipts are refreshed. PA11/PA12 native
qualification, all independent rustc checks and layout probes also pass.
The frozen candidate is `obj/pa13-work/array-candidate`, SHA-256
`a487e66d40ba8ef40fd20270adcdcbad46d1d6cea05bbf89a9a6093ee303a469`.
Logs are `obj/pa13-work/array-*.log`; complete native evidence is
`obj/pa13-work/array-native-1791829953497318351/evidence.json`.
PA13 still has no partial acceptance manifest and remains planned.

The frozen A/B runs use CPU 10, A/A calibration and three ABBA blocks, retaining
all 54 observations per workload and checking complete LowIR/count equality.
Ordinary 4k/16k/64k-procedure baseline/candidate medians are
219.57/210.92, 845.96/852.84 and 3382.64/3398.09 ms, with largest-scale RSS
701,360/700,788 KiB. Records are
`obj/perf/1791830101617636316/samples.json`.
Existing owned-product 1k/4k/16k-procedure medians are
82.14/83.98, 386.17/372.68 and 2113.76/2042.75 ms, with largest-scale RSS
319,160/322,372 KiB. Records are
`obj/perf/1791830194033344073/samples.json`. The largest product input uses
approximately 1% additional peak memory. Elapsed observations are noisy and
do not establish a speedup or a reliable compile-time regression.

The new `--stage lowir --array-pattern-input` workload demands 1k/4k/16k
procedures that move the ends and middle of non-Copy arrays separately, then
destructure the moved middle. Candidate A/A medians are 152.87/422.55/1618.02 ms,
with peak RSS 27,596/97,420/382,888 KiB. Its 54 observations and full-output
hashes are retained in `obj/perf/1791830242563101701/samples.json`.
The baseline cannot compile this capability, so this is a scaling measurement
rather than an A/B comparison. Identical-binary block ratios vary from 0.646
to 1.178 at the middle size: this run cannot support a precise elapsed scaling
claim. The sparse-storage invariant is independently tested. No RustGM build
or test ran during these three measurement runs.

The earlier draft binary and all its observations are retained as
`obj/pa13-work/array-draft-candidate` and
`obj/perf/1791829512362309356/samples.json`,
`obj/perf/1791829607539190640/samples.json`,
`obj/perf/1791829650850704937/samples.json`. Final edge probes found the nested
rest-binding rejection gap, after which the candidate, independent checks and
measurements above were rebuilt. None of the draft observations were discarded.

## Remaining implementation sequence

1. Extend the selected owned-product facts with binding modes and reference
   adjustments, including borrowed arrays and slices. Bindings in different
   alternatives must have consistent written qualifiers and solved binding
   types; retain each occurrence's effective capture mode and adjustments.
   Retain region/loan obligations for PA15/PA16; an inspection result must not
   authorize unchecked reference erasure during execution. Do not infer those
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

Additional binding-mode controls are retained in
`obj/pa13-work/binding-mode-probes/evidence.json`. Rust 1.98 rejects
`let ([x] | &[ref x]) = xs` for `xs: &[u8; 1]` with E0409, so the resolver's
written-qualifier consistency check must stay. However,
`let ((x, _) | (_, [x])) = xs` for `xs: (&u8, &[u8; 1])` is accepted: one
occurrence binds a reference value and the other inherits a reference binding.
Both bind `x: &u8`. The mutable-reference version is also accepted. Do not
require identical effective capture operations across alternatives when their
written qualifiers and binding types agree. The next typed pattern model must
retain these distinct per-occurrence operations and their region obligations.

## Reference binding and slice inspection checkpoint

The checker now elaborates explicit reference patterns and ergonomic automatic
dereferences into the flat selected-pattern arena. Each binding occurrence has
its own value/shared/mutable capture operation. Its canonical definition carries
the solved local type; pattern inputs keep their matched-place type. This
distinction also matters at a let root: coercing the initializer to a borrowed
binding's local type would incorrectly reject `let ref x = 1u8`.

Default binding mode travels with iterative pattern work. Shared mode survives
subsequent mutable dereferences, bindings and wildcards do not auto-dereference,
and edition-2024 restrictions reject written `mut`/`ref`/`ref mut` or reference
patterns after an inherited reference mode. Explicit reference patterns require
exact reference mutability. Array rest bindings retain fixed-array types;
irrefutable dynamic slice rests retain a selected subslice and an unsized input
behind a sized reference. Conditional slice shapes still await match semantics.

Every borrowed capture gets an existential body region with source and body
provenance; inference of an explicit reference pattern's unknown input uses the
same identity space with a distinct origin. These IDs never substitute `static`
or a signature binder. Region equalities remain pending and capture records
retain their loan obligations. The body builder rejects these pending regions
even if an irrefutable or-pattern would choose an earlier value-capture
alternative. Explicit dereference and subslice operations also retain an
execution boundary until their body representation and proofs are implemented.

`--emit-patterns`/`--count-patterns` publish these facts deterministically. The
owning PA13 brief describes the sparse arena, local/input distinction, operation
vocabulary, provenance, counts and atomic error contract. This view does not yet
claim coverage/usefulness or lifetime/loan validity, and PA13 stays planned.

Independent controls in `dev/tests/borrowed_patterns.rs` cover 34 accepted and
18 rejected programs, compare pinned rustc outcomes, check deterministic text
and count agreement, and verify the executable boundary. Two additional cases
deliberately pass pattern inspection while rustc rejects a lifetime escape
(E0515) or mutable borrow through a shared reference (E0596); both are blocked
from LowIR. The escape control borrows a parameter: a literal-only temporary
can be promoted and is not a valid rejection control. Typed-model tests retain
per-occurrence captures, shared-mode precedence, region origin/ownership and
the unsized-input/sized-binding distinction. Earlier 48 native product/array
vectors, 39 rejection vectors and 371 overlap combinations remain required.

The frozen compiler for this checkpoint is
`obj/pa13-work/borrowed-candidate`, SHA-256
`ab2552143501de51f4f63d254cfa84cd6179c92c2f6830e635f6edab67b01696`.
Local full debug and release runs pass 259 tests, followed by the final two-test
borrowed-pattern suite in both configurations after adding the proof-boundary
control (260 total tests). Clippy, formatting, the 169-file Rust audit and
harness checks pass. Regeneration changes only compiler-source receipts; all
1,669 active PA1–PA12 oracle cases retain identical bytes. Upstream verification
checks 24 imported fixtures and all 126 vendored Reference files.

`make test-rustc test-native` passes: PA11 has 194 independent checks and 180
native executions, and PA12 has 98 LowIR checks and 48 executions (its 12 layout
views are qualified separately). The 48 additional owned-pattern programs pass
against both pinned rustc and CPPGM; their complete commands, streams and hashes
are in `obj/pa13-work/borrowed-native-1791831956755083515/evidence.json`.
PA10's 34 execution vectors pass through its test-only body-IR executor; the
LowIR-native runner correctly refuses its body-IR manifest. That inapplicable
extra invocation is retained at the end of `borrowed-independent.log`.

Performance comparisons pin CPU 10 and freeze binaries and inputs. Each run
retains warmup, A/A calibration and three ABBA blocks at each of three scales
(54 observations), checks complete rendered bytes as well as counts, and runs
without concurrent builds or tests from this task. Old executable workloads
compare against `array-candidate`; the new pattern view uses candidate A/A.
All five runs are retained under `obj/perf/`:

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate peak KiB |
| --- | --- | ---: | ---: |
| Ordinary LowIR, 4k/16k/64k | 1791831992696241148 | 3388.49 / 3391.27 | 701452 / 701432 |
| Product LowIR, 1k/4k/16k | 1791832081544806945 | 1520.75 / 1611.91 | 322412 / 322380 |
| Array LowIR, 1k/4k/16k | 1791832123733994891 | 1655.85 / 1626.64 | 382896 / 382308 |
| Borrowed-pattern A/A, 1k/4k/16k | 1791832167567592733 | 970.85 / 983.60 | 214732 / 214552 |
| Product LowIR repeat | 1791832260888940284 | 2711.26 / 2748.31 | 322412 / 322692 |

The first product run warranted a repeat: at 4k, ABBA candidate/baseline ratios
were 1.078, 1.130 and 1.030. The repeat was visibly noisier, with 4k ratios
1.005, 0.912 and 1.248; at 16k the baseline fell from 2699.8 to 1668.1 ms
within its last paired block. Absolute times also shifted substantially between
runs. These measurements do not establish a precise product-throughput effect
or support a regression-free performance claim. Memory and output equality
remain stable. The new borrowed view's candidate medians are 60.88, 251.74 and
983.60 ms, with peak RSS 16868, 56528 and 214552 KiB. Do not discard either
product comparison or infer a speedup from the noisy array timing.

Next work remains conditional patterns and match CFGs, including guarded
captures, coverage/usefulness and deterministic witnesses. Guard accesses must
use shared views before committing binding moves; failed guards retry matching
or-pattern alternatives. PA15 must develop reference coercions/outlives and
variance beyond the current pending equality model. None of these obligations
is discharged by the new inspection mode.

## Scalar match checkpoint

Scalar match selection now shares ordinary body inference and stores source
identities, arm ranges, captures, intervals and whole-arm usefulness in typed
arenas. Boolean, integer and character domains use inclusive ordered ranks;
signed values bias the sign bit and character ranks compress the surrogate gap.
The interval index keeps disjoint maximal covered ranges with ordered lookup and
an explicit one-million-operation budget per query. Guards do not add coverage.
Missing witnesses render target values, including signed minima and characters,
not internal ranks. Nested alternatives are visited iteratively without a
Cartesian product. This is a scalar specialization, not general constructor
coverage or an implementation of per-alternative redundancy diagnostics.

Unguarded scalar matches lower to source-ordered decisions with existing body
CFG joins and storage cleanup. The scrutinee's place is evaluated once; captures
happen after a successful decision. Tests exercise overlapping/open/exclusive
ranges, i128/u128 endpoints, the Unicode gap, nested matches, macros, configured
arms, function-item joins, returns and loop transfers. Direct scalar temporary
reads now participate in ownership paths and reuse immutable SSA values during
LowIR lowering. Aggregate snapshots still follow their existing copy path.

Pinned rustc probes establish a subtle initialization distinction: an unused
uninitialized scalar can match `_`, `_|0` or its whole-domain range, but `0|_`
and a binding read are rejected. The decision builder avoids full-range tests
without erasing ordered alternatives. Accepted vectors also exercise a whole
unit-struct temporary, which exposed the same direct-temporary path issue.

Scalar-only irrefutable binding coverage can be inspected; composite coverage
and conditional binding execution remain unfinished. Guard expressions are
inferred and retained, but their protected accesses are not yet proved. A
control deliberately allows inspection of a mutable binding assignment in a
guard and requires atomic rejection at body IR/LowIR publication. Do not treat
this inspection success as Rust acceptance. Guard execution still needs delayed
moves, source-place protection and retry of matching alternatives. General
products, enums, references, slices, named constants, condition-let chains and
let-else require the remaining PA13 work. PA14 and PA15 are still required.

Parsing match-arm attributes belongs to PA5. Two new declared fixtures cover
active/disabled arms and invalid syntax in a disabled arm; pinned rustc checks
both. The earlier PA4 expression mode is preserved. The PA12 enum-match boundary
still rejects, now with the more specific diagnostic `refutable constructor
requires conditional pattern semantics`. This is the only changed existing
oracle; other receipt changes record compiler sources, with PA5 additionally
recording its two fixtures and manifest. PA13 has no grading manifest yet.

The frozen candidate is `obj/pa13-work/scalar-candidate`, SHA-256
`52328222ea3cd7e242be78bfd3e7762c27192d08b2b09690f78274d0dad84543`.
Both complete Rust test configurations pass 267 tests. Strict Clippy and
formatting pass; the file/architecture audit checks 173 files with zero findings.
All 1,671 active text-oracle cases pass. The scalar tests compare 26 accepted and
17 rejected source programs with pinned rustc, test every interval against all
256 subsets of an eight-point domain, and check budget exhaustion, deterministic
views and atomic diagnostics. The 26 accepted programs plus 48 earlier
product/array programs also execute through the qualified CPPGM backend and
rustc; commands, source/binary hashes and streams are retained in
`obj/pa13-work/scalar-native-1791834818786448644/evidence.json`.

The complete independent course checks also pass: PA11 has 194 LowIR checks
and 180 executions, PA12 has 98 LowIR checks and 48 executions plus 12 separately
qualified layout views. Harness discovery, all upstream checks and pinned rustc
checks for the active stages pass; see `scalar-independent.log`.

Before generalizing coverage, retain the pinned controls in
`obj/pa13-work/constructor-validity-probes-1791834983142819960/evidence.json`.
An empty match is accepted for a directly uninhabited enum, a tuple/visible
struct containing it and a positive-length array of it. It is rejected for
`&Void`, `[Void; 0]` and a struct whose uninhabited field is private from the
matching module. A public field allows the empty match. A variant containing
`Void` need not be covered, but `[]` alone does not exhaust `&[Void]`. These
controls forbid using the layout engine's inhabitation result as the complete
coverage predicate. The unrelated `char-end-exclusive` probe is a malformed
`..=>` syntax control, not evidence about character coverage; the existing scalar
suite already covers the correctly separated open range. General coverage must
retain the matching module and place-validity context in its query identity.

Frozen measurements pin CPU 10, retain all warmups, A/A calibration and three
ABBA blocks per size, and compare complete rendered LowIR bytes and counts.
There were no concurrent builds or tests from this task during these runs.
The first three use `borrowed-candidate` as the baseline. The new scalar
workload uses candidate A/A, because the earlier compiler cannot compile it.
All 216 observations and input/binary hashes remain under `obj/perf/`:

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate peak KiB |
| --- | --- | ---: | ---: |
| Ordinary LowIR, 4k/16k/64k | 1791834999833771820 | 3582.97 / 3596.85 | 700788 / 701388 |
| Product LowIR, 1k/4k/16k | 1791835094409770775 | 1567.40 / 1587.10 | 322384 / 322592 |
| Array LowIR, 1k/4k/16k | 1791835137032824860 | 1729.75 / 1732.88 | 382484 / 382924 |
| Scalar-match LowIR A/A, 1k/4k/16k | 1791835183369469408 | 964.57 / 940.12 | 234440 / 234372 |

All nine old/new full-output comparisons are equal. The largest median changes
are approximately +0.4%, +1.3% and +0.2%; do not interpret them as proof of no
regression. The largest ordinary paired-block ratios range from 0.953 to 1.028;
product ratios are 1.007, 1.027 and 1.004. Memory remains close to baseline. The
new scalar workload's medians are 54.24, 247.18 and 940.12 ms with candidate peak
RSS 17744, 61144 and 234372 KiB. It measures scaling across independent match
bodies, not an adversarial single constructor-matrix query.

A final direct-temporary control verifies that moving the same non-Copy unit
value into two `@` bindings is rejected by both compilers, while one capture is
accepted. Evidence is in
`obj/pa13-work/direct-temporary-controls-1791835252744843721/evidence.json`.
The negative control is retained in `dev/tests/scalar_matches.rs`; its final
four-test suite was rerun in debug/release, followed by lint/audit and explicit
reference regeneration. This test-only addition does not change the frozen
compiler used for native qualification or performance.

## Constructor coverage checkpoint

Coverage now composes selected tuple, record, enum, fixed-array and reference
patterns with scalar intervals. The scalar-only fast path remains in place.
Composite queries compile pattern sets into reduced binary decision diagrams
whose variables identify typed places and whose bits encode ordered ranks.
Intersection models products, union models alternatives, and difference proves
whole-arm usefulness and finds uncovered inputs. Memoized operations and interned
nodes share unchanged subgraphs; nested alternatives are not expanded into a
Cartesian product. Dynamic slices still support only irrefutable rests, and
named/structural constants remain future work in this assignment.

The graph uses only the demanded array projections, including huge ZST arrays.
Constructor rank domains exclude variants known to be empty in the matching
context. Unused binary encodings alias the final valid rank, preserving set
complement without inventing an extra variant or character; witnesses map back
to valid ranks. Dedicated tests cover non-power-of-two domains and u128 endpoints.
The one-million-operation budget still applies to each composite coverage query.

`Members` now keeps an owner-to-variant index. Inhabitation queries share a cache
keyed by module and canonical type, and constructor domains also retain the
validity context. This avoids rescanning every declaration for each pattern or
recomputing variant domains for each arm. References terminate visible-empty
proofs; their referents may be invalid places. Private fields do not expose empty
proofs outside their permitted module. These facts are deliberately separate
from physical layout inhabitation. A match's `uninhabited` inspection field
records this contextual result, not an initialization or loan proof.

Refutable constructors are now selected and ordinary binding irrefutability can
combine constructors and scalar patterns. General match execution remains gated:
its next implementation must add typed tag decisions, conditional captures,
source-ordered retry paths and reads that enforce place validity. The control
`enum Void{} fn main(){let x:Void;match x{}}` passes coverage but is rejected by
rustc's initialization check. Body IR and LowIR must reject it until the required
validity read exists; an empty-domain proof cannot erase that obligation.
By-reference captures retain their earlier region/loan gate. Guards still add no
coverage and their access protection and retry execution remain unfinished.

Missing constructor witnesses are deterministic partial assignments, such as
`{ $ = E::B }` or `{ $.0 = false, $.1 = true }`. Names come from selected member
facts and declaration provenance only during diagnostic rendering. Tests include
macro-generated variants and Unicode fields. No diagnostic string is semantic
transport. PA13 remains planned without a partial manifest; PA14 and PA15 remain
part of the implementation goal.

The first interval-edge graph exposed an avoidable quadratic update pattern.
A bit-reversed ordering of unit enum arms took 3,697 logical operations at 32
variants, 796,657 at 512, and exceeded the budget at 1,024. Replacing complete
edge-array rewrites with shared binary domain subtrees gives 1,386, 31,146,
66,466 and 141,210 operations at 32, 512, 1,024 and 2,048 variants. The source,
complete outputs and counter observations are retained under
`obj/pa13-work/enum-coverage-scaling-1791837378450914531/`. A permanent test checks
256/1,024/4,096 variants and bounded growth; these are logical-work observations,
not elapsed-time speedup measurements.

The earlier frozen draft is retained as `obj/pa13-work/coverage-candidate`,
SHA-256 `42937a8ea790317bbd04c955c1e71cca4af2d7dac4192e909b439b5cf9ae5752`,
along with its validation logs and native evidence. It is superseded by
`obj/pa13-work/coverage-final-candidate`, SHA-256
`f4ade8cf0a423016ce6131260432ed5b92a113ed9c653b52ff9652cb31400d42`.

The constructor integration suite has 29 accepted and 16 rejected programs
checked against pinned rustc, four witness controls, all 16 subsets of Boolean
pairs for usefulness, a 64-column alternative-sharing test, a sparse trillion-
element ZST array control and the execution-validity boundary. A smaller product
and the 256-variant case are independently checked with rustc. The original
64-column rustc probe was explicitly terminated after remaining live for over
60 seconds; this is not a language rejection or passing independent check.
Its source and termination receipt remain in
`obj/pa13-work/coverage-sharing/3660725/`. The resource test now checks RustGM's
work bound directly and uses a separate smaller Rust control.

The final compiler passes all 276 Rust tests in debug and release, strict Clippy,
formatting and the 179-file audit with zero findings. All 1,671 active assignment
cases, harness discovery, upstream checks and independent Rust checks pass.
PA11 has 194 independent checks and 180 native executions; PA12 has 98 LowIR
checks and 48 executions plus 12 independently qualified layout views. The
additional 74 scalar/product/array programs pass against both runtimes, with
complete commands, hashes and streams in
`obj/pa13-work/coverage-final-native-1791837796840234379/evidence.json`.
Constructor inspection cases are not counted as native executions. The only
changed existing oracle is PA12's match boundary: it now rejects at the explicit
constructor-execution/validity gate rather than during constructor selection.
All other reference changes are compiler-source provenance receipts.

Performance runs freeze binaries/inputs, pin CPU 10, retain warmups, A/A
calibration and three ABBA blocks at each size, and compare full output bytes
as well as counts. No build or test from this task ran concurrently. The four
existing executable workloads compare with `scalar-candidate`; constructor
inspection uses candidate A/A because the baseline lacks that capability.
All 270 observations remain under `obj/perf/`:

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate peak KiB |
| --- | --- | ---: | ---: |
| Ordinary LowIR, 4k/16k/64k | 1791837971913417841 | 3503.65 / 3462.47 | 700828 / 700828 |
| Product LowIR, 1k/4k/16k | 1791838070869200344 | 1608.62 / 1588.70 | 322424 / 322640 |
| Array LowIR, 1k/4k/16k | 1791838114056467837 | 1696.07 / 1701.65 | 382924 / 382492 |
| Scalar-match LowIR, 1k/4k/16k | 1791838159852911889 | 1488.97 / 1353.54 | 234664 / 234840 |
| Constructor-pattern A/A, 1k/4k/16k | 1791838193822472693 | 1744.01 / 1735.69 | 223804 / 223900 |

All twelve existing-workload full-output comparisons are equal. Memory stays
close to baseline. Absolute wall times drift: ordinary 16k calibration samples
range from 855.9 to 888.5 ms, while paired medians are 1436.62/1443.75 ms. The
largest scalar paired-block ratios are 1.069, 0.968 and 0.895, and even the
identical-binary constructor run has a 1k block ratio of 0.763. These observations
do not support a precise speedup or regression-free claim. The new constructor
candidate medians are 137.28, 422.75 and 1735.69 ms, with peak RSS 17740, 58572
and 223900 KiB. Logical-work and sparse-storage invariants are checked separately
from these noisy wall-time observations.

Before enabling source dereference expressions or general constructor execution,
retain the root-place probes in
`obj/pa13-work/root-place-validity-1791838489370427268/evidence.json`.
Pinned rustc accepts an empty match on `*x` for `x: &Void`, but rejects covering
only `E::B` in `match *x` when `E::A(Void)` also exists. It also differs in the
unreachable-pattern warning for a wildcard on a dereferenced versus owned Void.
RustGM currently rejects those dereference expressions before coverage, so these
are explicit future controls, not acceptance-equivalence claims. Do not open
that expression boundary while treating every root as a validated owned value;
root validity and nested-reference validity need separate qualification.

## Owned constructor execution

Unguarded matches now lower tuples, records, enum payloads, fixed arrays and
nested or-patterns. The body builder consumes the selected pattern arena with
iterative decision and capture worklists. Tests keep source order and do not
move payloads. An or-pattern with captures records its successful alternative
in a local; capture branches then read that alternative's selected places.
Nested products share continuations instead of expanding combinations. These
locals are initialized before decisions, including choices conditionally skipped
by an enclosing alternative, and cleaned up on both arm success and failure.
Capture projections are reconstructed from typed selections in capture branches;
they never reuse an SSA address defined only in an earlier decision branch.

`Rvalue::IsVariant` retains the selected variant and a non-moving place test.
Lowering loads the actual layout tag and compares its stored discriminant,
including signed/extreme explicit tags. Nominal field identity now has a direct
owning-variant index, so payload offsets come from the selected variant rather
than always using the first layout group. No source or rendered IR is reparsed.

Pinned controls in
`obj/pa13-work/constructor-read-probes-1791839143746083099/evidence.json`
show that a multi-variant tag test rejects an uninitialized or partially moved
owned enum. A surviving physical tag is insufficient: the ownership proof checks
the whole tracked input, not only its shallow bit. A single-variant wildcard
pattern and a wildcard product can avoid a read. Empty matches emit a non-moving
`Validate` obligation before `Unreachable`; body inspection shows that obligation,
while ownership/LowIR modes reject an uninitialized input. References and source
dereferences retain their existing gates and are not authorized by this proof.

Irrefutable ordinary lets and parameters use the same decisions when their
patterns contain refutable alternatives or ranges. Declarations without an
initializer only create bindings; the controls in
`obj/pa13-work/uninitialized-pattern-declarations-1791839645553166297/evidence.json`
independently establish that behavior. Existing irrefutable product bindings and
scalar matches retain their simpler construction paths.

`dev/tests/constructor_matches.rs` adds 34 complete executable programs, twelve
invalid-place/move controls, and CFG-size bounds at 8/32/128 columns of independent
or-patterns. It covers variant-specific offsets, explicit tag representations,
alternative capture places, nested matches, loop transfers, delayed moves,
partial products, arrays/rests, uninitialized declarations and sparse huge ZST
arrays. The exact former PA12 match boundary source is retained here as a positive
case. PA12 now has 109 fixtures: that supported program is no longer graded as a
rejection. All remaining existing text oracles are unchanged.

Guards, let conditions/chains, let-else, dynamic slice decisions, named and
non-integral constants, and borrowed execution remain unfinished. In particular,
the current first-success or-pattern decisions must not be used for guards:
failed guards can require another matching alternative, with protected accesses
and delayed moves. PA13 stays planned; PA14/PA15 remain in the active objective.

Final local validation for this checkpoint uses frozen
`obj/pa13-work/constructor-final-candidate`, SHA-256
`0e2e255988065a05246f6bf270171f854bc7d232e9ce74e737590e913485f28d`.
Debug and release each pass 279 Rust tests. Strict Clippy/rustfmt and the
181-file audit pass. Explicit `make ref-test` regenerates provenance receipts;
review confirms unchanged bytes for every retained oracle and fixture. Only the
graduated PA12 boundary disappears. All 1,670 active assignment cases, the
24-test harness, 24 pinned upstream inputs, 126 Reference files, independent
rustc checks and 12 layout controls pass. PA11 passes 194 native checks with
180 executions; PA12 passes 97 native checks with 48 executions. Logs are
`obj/pa13-work/constructor-final-{debug,release,quality,refs,independent}.log`;
the initial quality invocation's manifest-receipt check required the subsequent
explicit reference regeneration, after which the full harness passed.

Another 108 full programs (34 new constructors plus the earlier 74 scalar,
product and array vectors) execute successfully through both compilers, with
source assertions and clean exit/streams. Their immutable sources, commands,
compiler/backend hashes and individual streams are retained in
`obj/pa13-work/constructor-final-native-1791839787686553778/evidence.json`.
Unreachable functions with initialized empty-type parameters also compile with
both frontends; that separate acceptance control is retained in
`obj/pa13-work/valid-empty-input-1791839883993677045/evidence.json`, and is not
counted as an execution of an impossible input.

A final capture-reachability review independently checked four moved-field
programs. Pinned rustc rejects all four with E0382, including alternatives it
also warns are unreachable. The existing capture CFG preserves those ownership
checks; pruning such captures from coverage alone would change acceptance.
The sources/streams are retained in
`obj/pa13-work/capture-reachability-1791840231766031208/evidence.json` and all four
are now permanent rejection vectors. The expanded constructor suite passes in
both configurations; this last change only adds test inputs and leaves the
frozen compiler bytes unchanged. Its lint/audit, explicit reference regeneration
and harness rerun are in `constructor-reach-{debug,release,quality}.log` under
`obj/pa13-work/`.

Five performance runs retain 270 observations, with CPU 10, frozen inputs,
immutable binaries, A/A calibration and three ABBA blocks per size. No test or
build from this task ran concurrently. Existing workloads compare with
`coverage-final-candidate`; the new constructor workload uses identical candidate
binaries because the baseline cannot execute it. All twelve existing-workload
full LowIR comparisons are byte-identical.

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate peak KiB |
| --- | --- | ---: | ---: |
| Ordinary LowIR, 4k/16k/64k | 1791839945077253082 | 3397.50 / 3557.01 | 701384 / 701468 |
| Product LowIR, 1k/4k/16k | 1791840037182370576 | 1575.72 / 1626.80 | 322424 / 322620 |
| Array LowIR, 1k/4k/16k | 1791840078418830933 | 1660.53 / 1644.64 | 382868 / 382604 |
| Scalar-match LowIR, 1k/4k/16k | 1791840123286000490 | 957.10 / 922.05 | 234768 / 234856 |
| Constructor-match A/A, 1k/4k/16k | 1791840149731643163 | 2838.13 / 2830.23 | 539072 / 539076 |

Memory remains close for existing workloads. Largest ordinary paired-block
candidate/baseline ratios are 1.058, 1.016 and 0.999; product ratios are 1.014,
1.044 and 1.002. The aggregate medians show a small slowdown for those two
workloads, with drift between blocks; do not claim zero regression or a precise
speedup. New constructor candidate medians are 154.91/671.91/2830.23 ms and peak
RSS is 37228/137976/539076 KiB. The independent CFG-size assertions establish
that tested nested alternatives stay linear in pattern columns; these timings
cover complete compilation at increasing function counts.

## Owned conditional patterns

Owned `let-else`, `if let`, `while let` and edition-2024 let chains now use the
same typed constructor decisions and delayed captures as matches. Sparse
conditional records retain the initializer, selected pattern and optional
failure block. A sparse set marks the enclosing conjunctions during inference;
body construction consumes those identities without another semantic walk.
Successful captures are in scope for subsequent chain operands. Every false
operand cleans the successful prefix before entering the failure branch.
While conditions recreate their bindings on reentry; break, continue and
labeled transfers use the existing control and storage machinery.

Let-else requires a divergent failure block, checked in the surrounding scope.
The builder keeps just the newly introduced success locals in shared LIFO
scratch while constructing that block. It never clones the complete surrounding
live-local list for each declaration. Iterative size controls at 16/64/256
chain operands and sequential declarations bound blocks by `8 * width + 10`.
Seven independently interpreted scalar CFG programs check actual storage
balance and declared results against rustc.

This is a block-count bound, not a claim that the entire generated body is
linear. The retained structural samples expose quadratic individual
`StorageDead` statements when each failure cleans a growing prefix: chains at
16/64/256 operands have 289/2665/35209 statements, and sequential let-else has
331/2851/35971. The shared continuation storage avoids copying live lists, but
these cleanup actions still need a compact persistent scope representation.
Address that before PA13 acceptance; the fixed-width function-count benchmarks
below do not measure this limitation. Keep the scalar storage interpreter and
native checks when changing the representation, and independently expand the
compact form when reviewing earlier body-view oracle changes.

This exposed two earlier semantic gaps. Definite control transfer is now a
transient dense byte fact separate from an expression's contextual type. A
statement with a diverging condition can make an otherwise tail-less block
never-valued, while an explicit unit-valued tail retains its type. Short-circuit
operators only inherit definite transfer from their mandatory left operand;
loop and labeled-block breaks are checked against their own frames. Retained
`NodeFacts` remain 80 bytes. Two PA9 control fixtures own the earlier-language
correction; PA9 now has 201 cases.

Pinned rustc also checks reads, moves and repeated initialization in statically
impossible pattern failures and later unreachable match arms. Body IR now has
`FalseEdge { real, imaginary }`: ownership traverses both successors, while
runtime lowering traverses only `real`. Ordinary failure edges need no extra
edge. When possible, the builder replaces an existing jump in place; a decision
entry branch is split only when an imaginary failure is necessary. Final match
arms and ordinary irrefutable declarations do not require a failure body.
Storage used solely on imaginary paths is not allocated, including a regression
with a one-billion-byte local. A diverging input has no failure continuation;
a wildcard-or-literal inside a single arm still short-circuits without a read.

The independent exploratory controls and their streams are retained under
`obj/pa13-work/conditional-divergence-1791841918357522681` and
`imaginary-flow-controls-1791842128446406073`. Permanent controls now include
40 accepted conditional programs, 32 rejected conditional programs, three new
scalar and three new constructor unreachable-arm rejections, and a scalar
irrefutable-or execution. This includes selected alternative captures, divergent
inputs, failed-prefix moves, reinitialization, nested failure scopes, macros,
shadows, disjoint fields and invalid let syntax. Borrowed conditional selection
is inspectable but still records pending region/loan obligations and cannot
publish executable IR.

This checkpoint leaves guards, refutable dynamic slices, named/non-integral
constants and borrowed execution unfinished. PA13 remains planned, with no
partial grading manifest. PA14 and PA15 remain in the active objective.

Validation uses `obj/pa13-work/conditions-final-candidate`, SHA-256
`a0bc003fbfdd7239f42319aad39cb401b13c9330ef2326010720eaaa29877afd`.
All 285 Rust tests pass in debug and release. Final source-only extensions of
the scalar/conditional corpus pass again in release. All 1,672 active assignment
cases, strict Clippy/rustfmt, the 185-file audit, the 24-test harness, 24 pinned
upstream fixtures and 126 Reference files pass. Explicit reference regeneration
updates the twelve source receipts and adds the two declared PA9 controls;
every pre-existing oracle byte is unchanged. The first harness invocation ran
before regeneration and detected the changed PA9 manifest; the final invocation
passes with current receipts. Logs are `obj/pa13-work/conditions-*.log`.

Another 149 complete programs (40 conditional and 109 existing or extended
scalar/product/array/constructor controls) execute successfully through both
the pinned CPPGM backend and rustc, with source assertions, zero exits and
empty streams. Immutable inputs, commands and frontend/backend hashes are in
`obj/pa13-work/conditions-final-native-1791842668192418448/evidence.json`.

All independent rustc checks and twelve PA12 layout controls pass. PA11 adds
194 LowIR checks with 180 executable comparisons; PA12 adds 97 LowIR checks
with 48 executable comparisons. The backend bridge qualification also passes
its parse/validate/execute control. Together with the 149 development programs,
this is 377 native comparisons, including the assignment panic outcomes.

The next guard implementation must follow the pinned Reference's
`expr.match.guard.next`, `shared-ref`, `value` and `no-mutation` rules in
`doc/rust-reference/src/expressions/match-expr.md`. A false guard retries other
matching alternatives within the same arm, including nested products; it cannot
jump directly to the following arm. During the guard, pattern bindings denote
protected views of the selected places. Actual Copy/move captures commit only
after success. Access checking must allow unrelated effects while rejecting
moves or writes that invalidate those protected places. Guard-local conditional
bindings add another success-only scope, with cleanup on retry and divergence.
Add independent controls for retry count/order, changed capture locations,
nested alternatives, disjoint mutation, prohibited overlapping writes/moves,
conditional guards and failed retries before enabling execution. Preserve a
shared decision representation rather than expanding combinations of columns.

The immediate next checkpoint is compact cleanup storage, before guard
execution. Retain a typed persistent live-scope tail so a branch can end a
prefix with one operation rather than copying or emitting every preceding
local. Preserve separate lexical scope identities across shadowing, macros,
loop reentry and imaginary ownership paths. This belongs to PA10's body
contract; any changed text vocabulary needs a handout update and reviewed
oracles, with a replay/expansion check against the previous scalar storage
sequence. Measure both function-count scaling and long-chain/declaration width;
state the remaining cost of ownership dataflow separately from body size.

Six performance runs retain all 324 observations, on CPU 10 with frozen inputs,
immutable binaries, A/A calibration and three ABBA blocks per size. No build
or test from this task ran concurrently. The five existing workloads compare
with the published constructor candidate; every full LowIR and count comparison
is byte-identical at all three sizes. The new conditional workload uses A/A
because the published baseline cannot execute it.

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate peak KiB |
| --- | --- | ---: | ---: |
| ordinary | 1791842776173261861 | 3580.27 / 3614.00 | 700864 / 701092 |
| product | 1791842872569966562 | 1573.62 / 1571.47 | 322404 / 320536 |
| array | 1791842914099061565 | 1598.54 / 1657.31 | 382484 / 382352 |
| scalar | 1791842959595557416 | 987.83 / 995.27 | 234796 / 234812 |
| constructor | 1791842986413263809 | 2863.43 / 2915.83 | 539068 / 538808 |
| conditional | 1791843065185019943 | 2126.48 / 2156.58 | 456024 / 456000 |

Largest ordinary/product/array/scalar/constructor median ratios are recorded by
these observations, not claimed as stable speedups. Array and constructor
workloads show small elapsed increases; peak memory remains close to baseline.
The new conditional workload is available as
`scripts/measure_lexer.py --stage lowir --conditional-pattern-input`; it combines
let-else, a while-let chain and multiple conditional captures. This measures
complete compilation with fixed pattern width, and does not close the separate
long-prefix storage-cleanup issue described above.

Conditional A/A candidate medians at 1k/4k/16k functions are 124.27/513.18/2156.58 ms; paired peak RSS is 31796/117024/456000 KiB.

## Compact storage exits

The long-prefix body-size issue above is now fixed. Each local retains the
predecessor that was live when the local was created. These persistent links
skip locals from already-closed nested scopes; they are not contiguous local-ID
ranges. `Action::StorageEnd { first, last }` records a reverse live-stack suffix
in one action, with inclusive endpoints. A single-local exit keeps
`StorageDead`. Ordinary bodies and synthesized constructors use the same action
selection. Cleanup emission neither walks nor copies the suffix.

The body adapter prints `storage-parent=lN|-` on local records and
`end-storage first=lN last=lN source=NODE` at multi-local exits. The PA10 handout
owns this vocabulary, and `spec.md` now requires shared storage tails. The
allocation-free range iterator follows decreasing ancestors and rejects invalid
endpoints, missing links and cycles rather than silently dropping cleanup.
Ownership transfer clears the same initialization/write state as before;
executable lowering still consumes storage actions as no-ops. The scalar test
executor checks each ended local was live and charges its execution budget.
This storage operation remains distinct from destructor execution.

The new PA10 `storage-ancestry` fixture closes nested scopes before creating
later siblings and exercises a labeled-loop exit. Its declared result 153
matches the storage interpreter, rustc and CPPGM execution. PA10 now has fifty
fixtures, including 35 interpreted/rustc execution vectors. A model invariant
also covers gaps in local IDs and rejects corrupted ancestry. Conditional body
size tests cover 16/64/256/1024 bindings and bound both blocks and statements.
At 256 bindings, chain statements fall from 35,209 to 2,569 and sequential
let-else statements from 35,971 to 3,076. At 1024, they are 10,249 and 12,292.
This is linear body representation; ownership's bounded block-by-path-word
fixed point still has its separate cost and is not claimed to be linear.

Validation uses `obj/pa13-work/cleanup-candidate`, SHA-256
`25642b3763652a428d9535cb54a3daee40f2f3d458934eca066a10c1f09b9ac4`.
All 286 Rust tests pass in debug and release. All 1,673 active assignment cases,
strict Clippy/rustfmt, the 185-file audit, harness and upstream checks pass.
Independent rustc and twelve layout controls pass. PA11/PA12 retain 194/97
LowIR checks and 180/48 executable comparisons; backend qualification passes.
Another 150 development programs pass native/rustc execution, including the
new scope regression, for 378 native comparisons in total. Frozen source,
command and compiler/backend evidence is
`obj/pa13-work/cleanup-native-1791843798881826270/evidence.json`.

Oracle changes were independently reviewed before acceptance. Expanding compact
storage into individual reverse-order ends, removing only the new parent
metadata, and restoring the expanded statement count reproduces the published
compiler's output across every one of the 1,673 cases. Receipt and raw A/B
streams are in
`obj/pa13-work/cleanup-equivalence-1791843831769614566/evidence.json`.
All forty changed pre-existing oracles are PA10 body views and independently
expand to the exact previously committed bytes; existing stderr and status
sidecars are unchanged. The new PA10 fixture was declared before explicit
`make ref-test` generation. All twelve source receipts match current sources.
The initial temporary comparison completed its checks but could not serialize
Path metadata; the corrected receipt run passed and is the evidence cited here.

The 150 development programs additionally have expanded body equivalence and
byte-identical complete LowIR against the frozen conditional-pattern compiler.
That separate evidence is
`obj/pa13-work/cleanup-dev-equivalence-1791843961195698651/evidence.json`.
Logs for this checkpoint are `obj/pa13-work/cleanup-*.log`.

PA13 remains planned. Guard execution is next, following the retry/protected
capture requirements above; refutable dynamic slices, named/non-integral
constants and borrowed execution also remain. PA14/PA15 remain in the active
objective. Compact storage does not establish any new loan or lifetime proof.

Eight frozen performance runs retain all 432 observations: CPU 10, A/A
calibration and three ABBA blocks per size, with no concurrent build/test from
this task. Every full LowIR output and count comparison is byte-identical.
The baseline is the published conditional-pattern compiler.

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate peak KiB |
| --- | --- | ---: | ---: |
| ordinary | 1791843994230819748 | 3498.33 / 3478.93 | 701100 / 703232 |
| product | 1791844086584813014 | 1572.74 / 1615.51 | 320544 / 321760 |
| array | 1791844129251485345 | 1691.47 / 1691.82 | 382584 / 383396 |
| scalar | 1791844174588790566 | 982.52 / 1001.81 | 234800 / 235016 |
| constructor | 1791844201680519812 | 2961.51 / 3027.27 | 539092 / 540680 |
| conditional | 1791844283049324888 | 2184.60 / 2132.19 | 455976 / 456692 |
| chain-width | 1791844343131777136 | 140.10 / 63.03 | 45212 / 12124 |
| else-width | 1791844345949063946 | 139.82 / 64.24 | 48028 / 14908 |

The first six workloads scale function count with fixed pattern width; ordinary
uses 4k/16k/64k functions and the others 1k/4k/16k. They show small mixed timing
changes and a small memory cost from the retained predecessor links; these
observations do not establish a general speedup or a regression-free claim.
The two new `--storage-width-input chain|let-else` workloads instead hold one
demanded function and scale to 128/512/2048 bindings. They expose the cleanup
problem directly: at the largest width, compact bodies reduce both elapsed
time and peak memory substantially while preserving complete LowIR bytes.

For chain-width at 2048 bindings, candidate/baseline elapsed ratio is 0.450 and peak-RSS ratio is 0.268.

For else-width at 2048 bindings, candidate/baseline elapsed ratio is 0.459 and peak-RSS ratio is 0.310.

## Conditional guard grammar checkpoint

Match guards now use the existing conditional-expression parser and validator.
Passing the expression context into let-scrutinee parsing preserves the guard's
unambiguous struct-literal syntax while leaving if/while's restriction intact.
There is no second parse or new syntax representation. Existing resolution and
selected conditional facts already provide the required left-to-right binding
scope, shadowing and hygienic identity; invariant tests now assert those facts.
Guard execution remains rejected until protected views and retries are proved.

Twenty-two fixtures belong to their earliest owners: PA4 has fourteen grammar
controls, PA5 one configured item/pattern case, PA6 five scope controls, and PA8
two macro-hygiene controls. Expected outcomes were declared before explicit
reference generation. All existing stdout, stderr and status oracles remain
byte-identical. The twelve source receipts match the current compiler/tests.

Validation: 1,695 active course cases; 289 Rust tests in both debug and release;
Clippy all/pedantic, rustfmt and the 185-file audit; 24 harness tests; the 38-stage
placement check; 24 pinned upstream fixtures and 126 pinned Reference files;
all independent rustc checks and twelve layout controls. Logs are
`obj/pa13-work/guard-parser-{debug,release,quality,harness,independent,refs}.log`.
No new native execution is claimed by this parser checkpoint.

Frozen baseline is `cleanup-candidate` (the preceding published compiler).
The new `guard-parser-candidate` and built release binary both hash to
`e54dca892d21a9d86f9ccbac7fe876e2cee46b7abcf1c3d768f8e1a1b1d669c6`.
Three CPU-10 names-stage measurements retain 162 observations, with A/A
calibration and three ABBA blocks per size, and no concurrent build/test from
this task. Full names output and count output agree for every A/B comparison.

| Workload | Run | Largest A/B median ms | Largest A/B median peak KiB |
| --- | --- | ---: | ---: |
| ordinary, 64k functions | 1791845586324228462 | 912.36 / 911.69 | 139332 / 139376 |
| boolean guards, 16k functions | 1791845612426525196 | 286.23 / 285.25 | 48908 / 48924 |
| conditional guards, 16k functions | 1791845620626623768 | 480.43 / 472.19 | 57708 / 57722 |

The first two compare old/new compilers; the third is candidate-only A/A
scaling because the old compiler rejects conditional guards. These observations
show no material change in the two measured existing workloads, not a general
performance guarantee. New-workload timing has visible noise; all samples are
retained under `obj/perf/RUN/samples.json`.

Independent guard investigations retain eighteen grammar cases at
`obj/pa13-work/guard-grammar-1791844762445246437/evidence.json` and twelve further
place/move cases at `obj/pa13-work/guard-places-1791845246754733816/evidence.json`.
The latter show that tests in other arms protect their places, while untested
unbound tuple fields remain mutable/movable. Ordinary array indexing is not
disjoint. Guard-let locals have normal move/mutation semantics; moving an outer
non-Copy value can fail either on retry or on fallback after a failed operand.
These are requirements for the next execution work, not proofs provided by the
current inspection mode. PA13 is still planned; PA14/PA15 remain in the goal.

## Guard-view foundation

The body model now has internal `GuardId` groups, `LocalKind::Guard` binding
views, and `GuardStart`, `BindView`, and `GuardUse` actions. A view retains its
value type and selected source place; it owns no value snapshot. A group lists
shallowly protected tested places and a contiguous range of its binding views.
Tested-place records are proof-only, including places in inactive variants;
the native path never evaluates them. Guard execution from source is still
gated while the decision builder gains retry continuations and delayed commits.

Ownership requires initialization when selecting a binding view, rejects moves
and assignments through views, and requires all selected views at guard
completion. Physical source identities are resolved before move-path numbering,
including nested views, projected fields and overlapping array ranges. Alias
expansion is bounded to one million work units. Existing path interning and
array boundary numbering are shared with ordinary ownership checking.

A backward may-use worklist computes protection liveness. Guard completion
uses the shallow tested-place protections and deep binding-view protections;
ordinary view reads also keep the relevant binding protection live. Starting
an attempt and binding/recreating view locals end preceding lifetimes. Writes,
moves and storage changes conflict only with future live protections. Thus a
path that returns without a later view/guard use can mutate the source. The
proof follows ownership CFG edges, has a 64 MB state bound and a 32-million
work budget, and skips allocation entirely for bodies without guard groups.
This is an internal guard proof, not authorization to erase user reference,
region or loan obligations.

Native view locals hold addresses. Scalar temporary views materialize already
evaluated scalar snapshots; aggregate views point to existing storage. Aliases
are rebound on repeated execution. Zero-sized values need no storage. Native
binding rejects insufficiently aligned packed origins, including zero-sized
fields whose alignment is greater than one. Pinned rustc controls for packed
scalar, product and zero-sized fields reject with E0793; evidence is retained
under `obj/pa13-work/guard-packed/`. The native check is a required physical
constraint in addition to the initialization/liveness proof.

Eight new Rust tests exercise the primitives without bypassing the source
execution gate. Six ownership matrices cover lifetime endings, divergence,
branch/loop joins, shallow versus deep protection, ancestors, nested aliases,
initialization, immutability and array overlap. A lowering test turns exactly
one owned binding into an internal view in each of ten typed development
programs, preserving the ordinary parser and selected types. Separate rustc
programs use explicit shared references as the independent component contract.
The ten programs cover scalar places/temporaries, products, non-Copy values,
arrays, branch-selected addresses, repeated bindings, zero-sized values and
function items. A final test rejects packed scalar and zero-sized origins.
These tests validate the view representation; they do not claim match-guard
source lowering, retry behavior or a complete PA13 release.

The final foundation checks pass: 297 Rust tests in debug and release; 1,695
active assignment cases; Clippy all/pedantic, rustfmt and the 189-file audit;
24 harness tests, the 38-stage placement check, 24 upstream fixtures, 126
Reference files and the backend smoke test. Explicit `make ref-test` updated
twelve source receipts; every existing oracle stream/status remains unchanged.
Logs are `obj/pa13-work/guard-views-{debug,release,quality,refs}.log`.

The frozen release candidate is `obj/pa13-work/guard-views-candidate`, SHA-256
`fd77bc62717f1b42ae8fe30f7ce1e27a92dcafc142fa8046434c08599eac5d8e`.
It passes PA11's 180 native executions/194 checks and PA12's 48 native
executions/97 checks. Their receipts under `obj/native/pa11` and `obj/native/pa12`
record that exact candidate hash. The ten new component programs additionally
pass CPPGM validation/execution and their independent rustc programs, with
receipt `obj/pa13-work/guard-views-qualified-1791847010332432110/evidence.json`.
That separate component evidence consumes test-produced typed LowIR; it does
not imply the compiler now accepts guarded source execution.

Next is the compact retry decision builder, selection of views before guards,
conditional guard scopes and commits after success. The seven pinned order
probes in `obj/pa13-work/guard-order-1791845855949986699/evidence.json` establish
source-order product traversal, including record patterns whose written field
order differs from declaration order, nested alternatives and immediate stop
on successful guards. The thirteen flow controls in
`obj/pa13-work/guard-flow-1791845753358155411/evidence.json` additionally require
imaginary ownership paths bypassing guards without inventing executable
retries after an irrefutable conditional guard. PA13/PA14/PA15 remain incomplete.

Two frozen CPU-10 LowIR-stage measurements compare the published guard-parser
compiler with this foundation. A/A calibration and three ABBA blocks per size
retain all 108 observations. No build or test from this task overlapped timing.
Every complete LowIR output and count comparison is byte-identical.

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate median peak KiB |
| --- | --- | ---: | ---: |
| ordinary, 64,000 functions | 1791847279507720634 | 3434.96 / 3468.18 | 703194 / 715176 |
| constructor, 16,000 functions | 1791847372306311583 | 2898.79 / 2900.71 | 540540 / 549478 |

These are ordinary and unguarded constructor workloads: source guards remain
gated, so the measurements cover existing-path overhead, not guard liveness
scaling. The retained per-body guard arenas have a small representation cost.
Timing varies between workloads; the observations do not establish a general
speedup or a regression-free claim. Raw samples remain in `obj/perf/RUN/samples.json`.

At the largest sizes, median peak RSS is about 1.7% higher in each workload;
median elapsed time is about 1.0% higher for ordinary code and 0.1% higher for
constructors. The body now retains two empty vector headers even when no guard
exists. Revisit that per-function cost with guarded workloads when the builder
is connected; no dense node-fact record grew in this change.

## Owned guarded execution and compact retries

Source guards now connect the protected-view foundation to owned scalar,
constructor, product and array decisions. Bindings remain read-only views during
the guard; successful continuation commits their ordinary copies/moves. Guard-let
bindings use normal local storage and remain visible to later operands and the
arm result. A false operand ends the complete attempt-local storage suffix before
resuming the next matching alternative. The original scrutinee place, including
an evaluated dynamic index, is retained once.

The retry graph gives each subtree first-match and resume entries. Products
advance in written field order and resume the rightmost field first; a field
with no initial match fails its product immediately. Or-patterns save the
successful alternative and dispatch its resume continuation after guard failure.
Their graph size is linear in pattern occurrences rather than in products of
alternatives. Runtime may still make all language-required guard attempts.
Imaginary ownership edges check guard bypasses and alternatives after successful
matches. An irrefutable conditional guard introduces no executable failure or
imaginary retry cycle. This distinction permits a one-time outer move through
an irrefutable guard-let while rejecting the repeated move possible after a
boolean operand. Diverging guard paths do not acquire future fake uses.

A resumed decision may bypass a previously emitted parent projection block.
`Action::Select` therefore retains a static selected path and its already
evaluated scrutinee anchor. Lowering caches typed byte-offset recipes for path
prefixes and forms the final address in its actual use block. It never reuses a
branch-local SSA address that does not dominate the resume, and it does not
replay every ancestor projection at each leaf. Tested-place metadata stays
proof-only, including inactive variant fields. Ordinary constructor decisions
and successful captures retain their existing projection path.

Three new integration tests add 86 source programs: 55 accepted executions and
31 rejections, all independently checked with pinned rustc. They cover guard
attempt counts, source-ordered tuple and written record fields, overlapping
ranges, nested alternatives/guards, scrutinee side effects, array rests,
non-Copy captures, shallow and deep protection, ordinary guard-let ownership,
shadowing, returns/break/continue, initialization bypasses and packed fields
including zero-sized fields. Seventeen final controls distinguish an empty
array rest from the reference used to view it: the view requires and protects
the containing array's shallow state. Disjoint element moves remain permitted,
including moves before the view is created; replacing, wholly moving or indexing
the borrowed array is rejected while that view is live. An uninitialized array
cannot supply the view, even though moving an empty rest reads no elements.
The pinned rustc controls exposed this distinction during final review, and the
shared ownership path query now supplies it to both initialization and liveness.
The [pinned rustc overlap rules](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_borrowck/src/places_conflict.rs)
provide the implementation cross-check; the retained probes verify acceptance. The former two source-execution gates now check
protected views and ownership rejection. Six additional width/depth programs
at 4/16/48 levels check bounded body/LowIR growth. Their native assertions prove
zero attempts when a final field is impossible and one attempt after immediate
success; neither case needs exponential runtime exploration.

The frozen candidate is `obj/pa13-work/guard-execution-candidate`, SHA-256
`936cdce4801c6895a4930c2d687b68f7cea80540b9ef605533848f5f7583b8dd`.
It passes all 61 new native programs through the qualified CPPGM backend.
57 have independent rustc executions; the four larger constructed stress cases
use the count invariants just described, without requiring rustc to expand their
pattern products. Receipt:
`obj/pa13-work/guard-execution-qualified-1791849800663918269/evidence.json`.
The prior 150 scalar/product/array/constructor/conditional/storage programs also
pass this frozen candidate and rustc, with receipt
`obj/pa13-work/guard-execution-previous-native-1791849808936751051/evidence.json`.

Debug and release each pass 300 Rust tests; all 1,695 active course cases pass.
Clippy all/pedantic, rustfmt, the 193-file audit, 24 harness tests, 38-stage
placement, 24 upstream fixtures, 126 pinned Reference files and the backend
smoke check pass. Explicit `make ref-test` refreshes the twelve source receipts
without changing any existing oracle stream or status. Full `make test-rustc`
passes with earlier stages' documented boundary differences retained. Logs use
`obj/pa13-work/guard-execution-{debug,release,refs-quality,course-native}.log`,
`guard-empty-quality.log`, and `guard-retry-refs-rustc.log`. The full independent
rustc course run preceded the final internal empty-rest refinement; all 86
new source vectors were rechecked against rustc after that refinement.
The final binary also passes PA11's 194 checks/180 native comparisons and
PA12's 97 checks/48 native comparisons; `obj/native/pa11/evidence.json` and
`obj/native/pa12/evidence.json` record the final candidate hash.

PA13 still needs named constants, refutable dynamic slice selection/coverage and
its final release contract and fixtures. Reference-pattern execution remains
blocked on user-reference proof obligations; internal guard views do not discharge
those obligations. PA14 and PA15 remain incomplete. No PA13 manifest or active
stage promotion accompanies this checkpoint.

Final CPU-10 performance checks retain all 216 observations: frozen binaries,
full-output equality, A/A calibration and three ABBA blocks at each scale. No
local build or test from this task overlapped timing. Existing ordinary and
constructor workloads compare against the published guard-view foundation.

| Workload | Run | Largest baseline/candidate median ms | Largest baseline/candidate median peak KiB |
| --- | --- | ---: | ---: |
| ordinary, 64,000 functions | 1791849950636856331 | 4130.08 / 3888.19 | 715178 / 715150 |
| constructor, 16,000 functions | 1791850064293473131 | 3089.90 / 3099.83 | 549564 / 549510 |

Every complete LowIR output and count agrees across those A/B binaries. RSS is
essentially unchanged. Largest-scale ordinary A/A calibration spans 3.65–5.45
seconds, so elapsed medians are noisy; these observations do not establish a
speedup or a universal regression-free claim.

The newly enabled guard workloads use final-candidate A/A scaling, since the
published baseline rejects their source execution.

| Workload | Run | Functions | Candidate median ms | Candidate median peak KiB |
| --- | --- | ---: | ---: | ---: |
| chain | 1791850147527218406 | 1,000 | 60.55 | 16308 |
| chain | 1791850147527218406 | 4,000 | 337.07 | 53950 |
| chain | 1791850147527218406 | 16,000 | 1089.99 | 206258 |
| retry | 1791850179028107310 | 1,000 | 79.32 | 20544 |
| retry | 1791850179028107310 | 4,000 | 324.31 | 71300 |
| retry | 1791850179028107310 | 16,000 | 1292.21 | 275844 |

Both guard workloads show roughly proportional memory growth across these
scales. Reproduce with `scripts/measure_lexer.py --stage lowir --guard-input
let-chain` or `--guard-input retry`. The earlier name-stage guard workloads
remain available. Raw records are `obj/perf/RUN/samples.json`.


## Named scalar constant patterns

Boolean, integer and character constant paths now select exact declared types
and participate in the existing interval, constructor and ownership machinery.
Constants also supply open, exclusive and inclusive range endpoints. This
implements the scalar portion of the Reference's
[constant translation rule](../doc/rust-reference/src/patterns.md), including
exhaustiveness and whole-arm usefulness. It does not complete structural constant
expansion or claim general CTFE support.

The evaluator previously used for enum discriminants now has one shared
post-typing query session for all pattern requests. A sparse declaration-value
cache preserves evaluated dependencies across the declaration/body boundary.
Completed enum layouts supply discriminants when there is no active declaration
query. Source expressions and types are consumed as selected facts; no source
is parsed again and initializers already checked for a discriminant are not
checked again for a pattern use. Scalar arithmetic, casts, lazy conditions,
cycle checks and the existing work budget remain shared.

Ordinary body inference still constrains each pattern immediately. Only the
constant value, normalized range and dependent coverage work are delayed.
Requests and deferred ranges/roots/matches use flat arenas with per-body counts
and file/module provenance. One reused coverage checker finishes those scopes
after initializer typing. Constant-free bodies retain immediate coverage and no
pending arena allocation. A query checker allocates local inference scratch only
when it actually types a body; post-typing pattern queries do not allocate a
second dense local table. `NodeFacts` remains 80 bytes.

The new source matrix declares 30 successful scalar programs, one successful
two-file package, 23 rejected programs and eight accepted Rust programs that
remain explicit implementation boundaries. Pinned rustc checks every declared
language outcome and executes all 31 successful programs. The independent
resource tests cover shared discriminant dependencies, normalized signed ranks,
redundant arms, fixed-width node facts, 4,096 repeated/distinct demands and an
8,192-step forward dependency chain. Their retained range/arm/decision counts
stay proportional to source requests. Privacy controls distinguish a private
empty field in its owning module from the same type outside that module.

Float domains, structural and reference constants, refutable dynamic slices and
the final PA13 contract/fixtures remain open. Expression-position constants,
text/reference values and general CTFE retain their earlier boundaries. A named
pattern inside declaration-time CTFE is diagnosed explicitly before it can
leave an unfinished coverage record. PA13 remains planned; PA14/PA15 remain
in the active goal.


The immutable candidate is `obj/pa13-work/constant-pattern-candidate`, SHA-256
`30f7ea6d2490fee11569333a56a2c0257e7f8dcb0de8770b722f71a71c4f4109`.
All 31 new programs pass CPPGM LowIR validation and native execution, with
independent rustc executions retained in
`obj/pa13-work/constant-qualified-1791851849512271660/evidence.json`.
The same candidate requalifies the 61 guard/stress programs and the prior 150
scalar/product/array/constructor/conditional/storage programs, respectively:
`obj/pa13-work/constant-guards-qualified-1791851854487753521/evidence.json` and
`obj/pa13-work/constant-previous-native-1791851862184764479/evidence.json`.
The four larger guard stress cases retain their constructed attempt-count
invariants; the other 207 earlier programs also execute independently with rustc.

Debug and release each pass 307 Rust tests. All 1,695 active assignment cases
pass. Clippy all/pedantic, rustfmt, the 196-file audit, 24 harness tests, 38-stage
placement, 24 upstream fixtures, 126 Reference files and backend qualification
pass. Full `make test-rustc` passes with the previously documented stage
boundaries retained. Explicit `make ref-test` updates the twelve source receipts;
no existing oracle stdout, stderr or status changes. The frozen executable
passes PA11's 194 checks/180 native comparisons and PA12's 97 checks/48 native
comparisons; the receipts in `obj/native/pa11/evidence.json` and
`obj/native/pa12/evidence.json` identify its binary hash, not the CLI wrapper.
Logs are `obj/pa13-work/constant-{full-debug,full-release,quality,harness,
rustc-course,refs-native,frozen-native}.log`; the final debug query tests are in
`constant-query-invariants.log`.


CPU-10 measurements retain all 324 observations from six workloads: immutable
binaries, identical frozen inputs, A/A calibration, three paired ABBA blocks
at each scale, and complete rendered-output/count equality. No build or test
from this task overlapped timing. The three existing workloads compare with
`guard-execution-candidate` (`936cdce4801c6895a4930c2d687b68f7cea80540b9ef605533848f5f7583b8dd`).

| Existing workload | Run | Largest scale | Baseline/candidate median ms | Baseline/candidate median peak KiB |
| --- | --- | ---: | ---: | ---: |
| ordinary | 1791852191478956423 | 64,000 | 3645.96 / 3641.91 | 715144 / 715172 |
| discriminants | 1791852290023687168 | 16,000 | 836.10 / 826.32 | 147940 / 147920 |
| guard-retries | 1791852312977751324 | 16,000 | 1256.64 / 1240.58 | 275878 / 275876 |

These observations show similar medians and memory, not an established speedup.
Ordinary largest-scale A/A spans 3.55–3.78 seconds; discriminant calibration
also has a transient slowdown at 4,000 functions. Complete LowIR bytes agree
with the baseline in every existing workload.

The newly accepted constant workloads use candidate A/A scaling. Reproduce
with `scripts/measure_lexer.py --stage lowir --constant-pattern-input MODE`,
where MODE is `shared`, `unique` or `chain`.

| Constant workload | Run | Functions | Candidate median ms | Candidate median peak KiB |
| --- | --- | ---: | ---: | ---: |
| shared | 1791852346777834045 | 1,000 | 36.52 | 12074 |
| shared | 1791852346777834045 | 4,000 | 153.71 | 37910 |
| shared | 1791852346777834045 | 16,000 | 642.87 | 142054 |
| unique | 1791852364000714198 | 1,000 | 47.54 | 14186 |
| unique | 1791852364000714198 | 4,000 | 200.69 | 47114 |
| unique | 1791852364000714198 | 16,000 | 824.93 | 178688 |
| chain | 1791852386321036084 | 1,000 | 48.66 | 14188 |
| chain | 1791852386321036084 | 4,000 | 202.78 | 47130 |
| chain | 1791852386321036084 | 16,000 | 957.05 | 178706 |

Memory scales roughly with source size. The largest forward-chain calibration
spans 0.97–1.43 seconds, so its elapsed growth includes host noise; this is not
a universal timing guarantee. The shared session and bounded-worklist tests
provide the algorithmic checks against per-use retyping or recursive dependency
traversal. Raw records are `obj/perf/RUN/samples.json`; the command/receipt index
is `obj/pa13-work/constant-perf-receipts.json`.

## Floating-point pattern domains

`f32` and `f64` literals, named scalar constants and ranges now use the existing
scalar interval and composite decision machinery. Bit/rank conversion is O(1)
and adds no per-value storage or ordinary-node fields. The numeric ranks order
negative infinity through positive infinity, with signed zeros sharing one rank;
a final unordered rank represents all NaN payloads for coverage. Open bounds
stop at numeric infinity. NaN constants are rejected even in an otherwise
unreachable arm, and ranges covering all numbers still require a fallback for
NaN. Witnesses identify the missing NaN class.

Lowering keeps its existing typed comparisons. A full numeric range still reads
and tests the input because it excludes NaN; only the entire domain can omit a
test. This preserves initialization and guard protection checks. Bindings capture
the original input rather than the representative rank value, retaining the sign
of zero. Exclusive bounds use the preceding representable rank, including at
zero, infinity and subnormal/normal boundaries.

The rank invariants compare 16,384 deterministic encodings per width plus edge
values against host numeric order. They check inverse nonzero bits, canonical
zero, negation, infinities, out-of-domain ranks, and positive/negative
signaling/quiet NaN payload rejection. The independent program matrix has 52
successful sources and 46 rejected sources across both widths. It covers
constant endpoints, overlapping guards, conditional selection, arrays, products,
enums, initialization, and atomic deterministic pattern/LowIR views.

The pinned compiler exposes a specific semantic discrepancy: opposite zero
spellings in a guarded or-pattern run a failing guard once, whereas the
Reference's [expr.match.guard.next](../doc/rust-reference/src/expressions/match-expr.md)
requires trying the next matching alternative. RustGM runs it twice for either
zero, as it does for two identically spelled zeros. Eight explicit source pairs
cover both widths, both orders and literal/constant spellings. The RustGM source
asserts two retries; its separate rustc partner asserts the observed one retry.
These pairs are not same-source runtime agreement. No source or expected result
is silently changed to hide the discrepancy. The original probe and MIR are
retained in `obj/pa13-work/float-retry-probe/`.

Structural/reference/text constants, refutable dynamic slices, and the final
PA13 fixtures and grading contract remain open. PA13 remains planned;
PA14 and PA15 are still part of the active goal.

The frozen executable `obj/pa13-work/float-pattern-candidate` has SHA-256
`7f10435b51748f63e0caf456341d0e18c7bf2de923ac36b99be66f14ea5ec7f3`.
All 52 parity programs pass CPPGM LowIR validation/native execution and independent
rustc execution. The eight Reference cases separately execute with the declared
two-retry result, with their one-retry rustc partners retained. Three small,
asserting performance-input programs also pass both native paths. Their 63-case
receipt is `obj/pa13-work/float-qualified-1791854460316738507/evidence.json`.

The candidate additionally requalifies all 242 earlier PA13 native vectors:
31 constants, 61 guards/stress cases and 150 earlier pattern/storage cases.
Receipts are `float-constants-qualified-1791854429715662660/evidence.json`,
`float-guards-qualified-1791854434852819495/evidence.json` and
`float-previous-native-1791854442510524636/evidence.json`, under `obj/pa13-work/`.
The same four large guard stress cases retain their previously declared
constructed-count oracle; other earlier programs also execute with rustc.

Debug and release each pass 311 Rust tests. All 1,695 active assignment cases,
Clippy all/pedantic, rustfmt, the 198-file audit, 24 harness tests, 38-assignment
placement checks, upstream/Reference hashes and the backend check pass.
Full `make test-rustc` passes, retaining earlier declared phase boundaries.
PA11 passes 194 independent checks/180 native comparisons and PA12 passes
97 checks/48 native comparisons. Both `obj/native/paN/evidence.json` receipts
identify the frozen binary hash above. Explicit `make ref-test` updates twelve
source receipts; existing oracle stdout, stderr and exit-status bytes are unchanged.
Logs are `obj/pa13-work/float-{full-debug,full-release,all-quality,rustc-course,
frozen-native,refs}.log` and `float-{patterns,constants,guards,previous}-native.log`.

CPU-10 measurements retain all 378 observations from seven workloads, each
with immutable binary snapshots, frozen inputs, A/A calibration, and three ABBA
blocks at each of three sizes. No build or test from this task overlapped timing.
The four existing workloads compare with `constant-pattern-candidate`
(`30f7ea6d2490fee11569333a56a2c0257e7f8dcb0de8770b722f71a71c4f4109`);
their complete LowIR and count outputs agree in every run.

| Existing workload | Run | Largest functions | Baseline/candidate median ms | Baseline/candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| ordinary | 1791854597539019046 | 64,000 | 3600.74 / 3587.84 | 715184 / 715178 |
| scalar-match | 1791854692547558729 | 16,000 | 1056.42 / 1046.00 | 240142 / 240092 |
| guard-retries | 1791854720451647932 | 16,000 | 1271.24 / 1278.93 | 275914 / 275862 |
| constant-shared | 1791854754666429311 | 16,000 | 654.88 / 638.24 | 142440 / 142030 |

These medians and memory levels are similar; they do not establish a speedup.
Largest-scale ordinary A/A spans 3.56–3.75 seconds. The shared-constant
calibration is faster than its later paired block, showing within-run host noise.

The new float workloads use candidate A/A scaling via
`scripts/measure_lexer.py --stage lowir --float-pattern-input MODE`:

| Float workload | Run | Functions | Candidate median ms | Candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| float-scalar | 1791854771550036971 | 1,000 | 48.00 | 15588 |
| float-scalar | 1791854771550036971 | 4,000 | 200.43 | 51278 |
| float-scalar | 1791854771550036971 | 16,000 | 802.72 | 192886 |
| float-product | 1791854793886750304 | 1,000 | 349.36 | 24026 |
| float-product | 1791854793886750304 | 4,000 | 1414.54 | 86370 |
| float-product | 1791854793886750304 | 16,000 | 5737.42 | 335304 |
| float-retry | 1791854947674408753 | 1,000 | 61.19 | 19848 |
| float-retry | 1791854947674408753 | 4,000 | 235.16 | 68800 |
| float-retry | 1791854947674408753 | 16,000 | 1101.96 | 265526 |

The product input emits 896,013 instructions at 16,000 functions; its 1,000,
4,000 and 16,000 function medians scale roughly with that emitted work. RSS
also grows roughly with input size across these workloads. These are bounded
observations, not universal timing guarantees. The three small asserting workload
programs were independently executed before measurement. Full rendered/count
outputs agree for every A/A run. Raw records are `obj/perf/RUN/samples.json`;
the command/receipt index is `obj/pa13-work/float-perf-receipts.json`.

## Refutable dynamic-slice selection and coverage

Typed selection now retains dynamic-slice length constraints, prefix indices,
one-based suffix offsets and rest projections. Exact patterns require their
stated length; a rest permits larger lengths. Rest bindings preserve the slice
input type and the inherited reference capture mode. Ordinary lets/parameters
require coverage, while match, let-else, if-let, while-let and condition chains
retain refutable decisions. `[] | [_, ..]` is irrefutable. These are selected
semantic facts; reference/slice execution still stops at the existing pending
region/loan boundary. Local reference annotation elision and reference writes
are not claimed by the pattern inspection view.

Slices use the existing shared decision graph with a length variable. Each
coverage query lazily walks its own roots once to collect tested start/end
positions by canonical slice type. Hash sets collect positions and alias lengths;
the completed alias list sorts once. Wildcards and unconstrained captures add
no alias points. At a length where `start_index + end_offset == length`, the end
projection becomes the corresponding start projection. Other length intervals
retain independent end projections, even if an interval happens to contain a
single length. This last distinction preserves identities between patterns with
different minimum lengths. Nested slices reuse the same typed-place machinery.
No whole-crate rescan, value enumeration, rendered-text transport or unbounded
length enumeration is introduced. Collection, position pairs, branches and graph
operations share the 1,000,000-work limit.

The 35-case matrix checks 23 accepted and 12 rejected sources against pinned
rustc, including prefix/suffix correlation, sparse alias gaps, nested slices,
empty element types, constants, floats, conditional selection and binding modes.
Every accepted matrix case remains rejected at executable lowering's explicit
owned-pattern or region/loan boundary. Each view is deterministic, count mode
agrees with its footer, and rejected views have no stdout.

An independent Boolean model enumerates every sequence of lengths zero through
six for 64 generated six-arm pattern sets. Those lengths cover every aliasing
partition, exact-length boundary and an independent long-tail representative for
the generated prefix/suffix widths. Each set is checked both without and with a
fallback: 128 sources independently checked by rustc and RustGM. Whole-arm
usefulness is compared with the enumerated set difference; guarded arms add no
coverage. Resource controls separately exercise 5,000 wildcard positions and a
sparse 5,001-position pattern below 100,000 coverage operations. A dense pattern
hits the work limit with matching, atomic emit/count diagnostics. These stress
controls do not claim rustc throughput or acceptance parity at RustGM's limit.

The immutable executable is `obj/pa13-work/slice-pattern-candidate`, SHA-256
`dc713ca8a2722e3dcebddecce1cffe23b7a9f05b74b7a7c71f45dc8956b77b2d`.
Debug and release each pass 314 Rust tests; all 1,695 active assignment cases
pass. Clippy all/pedantic, rustfmt, the 200-file audit, 24 harness tests,
38-assignment placement, upstream/Reference hashes, backend checks and complete
`make test-rustc` pass. Earlier phase-boundary exceptions remain explicit.
The frozen binary also passes PA11's 194 checks/180 native comparisons and
PA12's 97 checks/48 native comparisons. Both native receipts identify its hash.
Explicit `make ref-test` refreshes twelve source receipts; existing assignment
oracle stdout, stderr and statuses are unchanged.

All 305 prior PA13 native checks pass with the frozen binary: 31 constants,
61 guard/stress cases, 150 earlier pattern/storage cases, and 63 float/workload
cases. Their documented constructed-count and signed-zero discrepancy oracles
remain distinct from same-source rustc parity. Receipts under `obj/pa13-work/`
are `slice-constants-qualified-1791856297830894889/evidence.json`,
`slice-guards-qualified-1791856303003830591/evidence.json`,
`slice-previous-native-1791856310484508920/evidence.json` and
`slice-floats-qualified-1791856330330278327/evidence.json`.
Three small slice benchmark sources independently pass rustc and both RustGM
pattern views in `slice-workload-checks-1791856356981088191/evidence.json`.
Final logs are `slice-{final-debug,full-release,final-quality,rustc-native,refs}.log`.

PA13 remains planned. Structural/reference/text constant patterns and the final
published assignment contract/fixtures remain unfinished; borrowed execution
retains its later proof boundary. PA14/PA15 remain in the active goal.

CPU-10 measurements retain all 324 observations across six workloads, with
immutable binaries, frozen inputs, A/A calibration and three paired ABBA blocks
at each of three sizes. No build or test from this task overlapped timing.
The three existing workloads compare the frozen floating-pattern baseline
(`7f10435b51748f63e0caf456341d0e18c7bf2de923ac36b99be66f14ea5ec7f3`)
with the slice candidate. Complete rendered outputs and count outputs agree.

| Existing workload | Run | Largest functions | Baseline/candidate median ms | Baseline/candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| ordinary | 1791856487439477587 | 64,000 | 3693.88 / 3752.08 | 715122 / 715152 |
| constructor-coverage | 1791856588669745861 | 16,000 | 1730.78 / 1737.35 | 225038 / 225014 |
| guard-retries | 1791856635442328510 | 16,000 | 1218.04 / 1243.24 | 275904 / 276094 |

Largest-scale medians differ by 0.4–2.1%, with similar memory use. Ordinary
A/A calibration spans 3.60–3.72 seconds, while paired observations include a
4.69-second candidate sample. The guard median is 2.1% slower in this run;
these observations do not establish a speedup or a universal regression bound.

New slice workloads use candidate A/A scaling through
`scripts/measure_lexer.py --stage patterns --slice-pattern-input MODE`:

| Slice workload | Run | Size | Candidate median ms | Candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| slice-scalar | 1791856891397946620 | 1,000 | 341.99 | 10642 |
| slice-scalar | 1791856891397946620 | 4,000 | 1563.80 | 31038 |
| slice-scalar | 1791856891397946620 | 16,000 | 6500.57 | 113346 |
| slice-nested | 1791857058500219126 | 1,000 | 1021.77 | 12142 |
| slice-nested | 1791857058500219126 | 4,000 | 4010.86 | 35990 |
| slice-nested | 1791857058500219126 | 16,000 | 18239.65 | 135226 |
| slice-sparse | 1791857524591037130 | 128 | 4.16 | 3692 |
| slice-sparse | 1791857524591037130 | 512 | 4.74 | 3648 |
| slice-sparse | 1791857524591037130 | 2,048 | 6.77 | 4394 |

Scalar/nested sizes count functions; sparse sizes count unconstrained prefix
positions before one tested value in a single function. RSS scales roughly
with input size. Nested 16,000-function paired samples span 15.8–25.5 seconds
with identical A/A binaries, versus 15.8–16.4 seconds in calibration; elapsed
growth therefore includes substantial host noise. Sparse-pattern timings are
dominated by process startup at these sizes. The wide-pattern work-counter
controls provide the algorithmic check against enumerating irrelevant lengths.

Raw records are `obj/perf/RUN/samples.json`, with append-only observation logs.
The combined command index is `obj/pa13-work/slice-perf-receipts.json`; verified
hashes, sample counts, equality and summaries are in `slice-perf-verified.json`.
The initial wrapper rejected the new slice workload name before measurement;
its diagnostics remain in `slice-perf.log`. After fixing its argument allowlist,
`slice-perf-resume.log` records only the three remaining workloads. All original
observations were retained, and the 24 harness checks pass after that fix.

## Unit, tuple and array constant patterns

The shared typed query engine now retains demanded product values in flat arenas.
A repeated array stores one value and a length. Named dependencies, selected
field/index reads, lazy if branches and already computed discriminants reuse
these values; indexing a billion-element repeat does not allocate that array.
Empty plain blocks produce unit. General statements, calls and expression-position
constant execution retain their existing later assignment boundaries.

Whole-product requests are distinct from scalar range endpoints. After initializer
typing, translation fills the selected root with ordinary product/range patterns.
Repeated product subtrees share IDs within a use, with separate projected place
identities during coverage and execution. Each generated node retains the use
site's file/node, so cross-file constants do not transport declaration node IDs
into body construction. Pending irrefutability and coverage retain their original
body/module scope. The dense ordinary node facts remain unchanged.

Type-level eligibility is checked even for empty arrays, and actual scalar leaves
still reject NaN. An empty repeat containing a computed NaN has no NaN leaf and
can be a pattern. Unit and empty products can be irrefutable. The expansion budget
charges edges before allocating siblings; it cannot accumulate unbounded pending
work before discovering the limit. Shared type eligibility and compact repeated
values avoid per-use type reconstruction and repeated array materialization.

Twenty-five asserting programs cover unit, empty arrays/blocks, nested/repeated
products, exhaustive Boolean products, projections, forward constants, discriminant
reuse, captures, guards/retries, condition chains, let-else, while-let, macro
patterns, modules and signed-zero capture preservation. Eighteen negative programs
check refutability, type/length mismatches, bounds, cycles, overflow, NaN,
non-structural empty-array element types, initialization and protected guard writes.
All are independently checked with pinned rustc; successful rustc programs execute.
Selected and LowIR views are deterministic; rejected LowIR views are atomic, and
count mode agrees with selected-pattern footers.

A separate package checks use-site provenance and native behavior for constants
from an external module. Borrowed product selection remains behind the existing
executable region/loan boundary. Resource tests check compact indexed repeats,
shared repeated-product pattern IDs and diagnosed expansion limits; the largest
stress inputs do not claim rustc resource equivalence.

PA13 remains planned. Nominal structural equality/derived PartialEq, reference/text
constant patterns and the final assignment fixtures/contract remain unfinished.
PA14 and PA15 remain in the active goal.

The frozen candidate is `obj/pa13-work/product-constant-candidate`, SHA-256
`3c039049558a8ebb8c3f9837321051067c5c94e5cead02146a69552b080198d2`.
Debug and release each pass 318 Rust tests; all 1,695 active assignment cases
pass. Clippy all/pedantic, rustfmt, the 203-file audit, 24 harness checks,
38-assignment placement, upstream/Reference hashes, backend checks and complete
`make test-rustc` pass. Explicit `make ref-test` updates twelve source receipts;
existing oracle stdout, stderr and exit-status bytes remain unchanged.

The candidate passes PA11's 194 checks/180 native comparisons and PA12's 97
checks/48 native comparisons. Their receipts identify the frozen hash. It also
passes all 305 prior PA13 native checks, retaining the previously documented
four constructed-count guard oracles and eight signed-zero Reference/rustc
pairs. Those exceptions remain separate from same-source runtime parity.

Native receipts under `obj/pa13-work/` are:

- `product-patterns-qualified-1791858566170423330/evidence.json`: 25 new programs,
  the cross-file package and three small asserting performance inputs; all 29
  execute through CPPGM and independent rustc.
- `product-constants-qualified-1791858541674129462/evidence.json`: 31 earlier checks.
- `product-guards-qualified-1791858546255431444/evidence.json`: 61 guard/stress checks.
- `slice-previous-native-1791858554015234415/evidence.json`: 150 earlier checks;
  this retained directory prefix comes from the reused harness, and its receipt
  identifies the product candidate hash above.
- `product-floats-qualified-1791858573534910984/evidence.json`: 63 float/workload checks.

Logs are `product-{full-debug,full-release,quality,harness-final,rustc-native,refs}.log`
and `product-{patterns,constants,guards,previous,floats}-native.log`.

The qualified timing set contains 324 observations from six workloads, each
with immutable binaries, frozen inputs, full rendered/count equality, A/A
calibration and three ABBA blocks at each of three sizes. Existing workloads
compare the frozen slice baseline
(`dc713ca8a2722e3dcebddecce1cffe23b7a9f05b74b7a7c71f45dc8956b77b2d`)
with the product candidate, pinned to CPU 10.

| Existing workload | Run | Largest functions | Baseline/candidate median ms | Baseline/candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| ordinary | 1791858717406495913 | 64,000 | 3608.35 / 3591.62 | 715154 / 715162 |
| scalar-constants | 1791858813452490633 | 16,000 | 630.61 / 619.15 | 142376 / 142104 |
| slice-coverage | 1791858830447156015 | 16,000 | 6114.43 / 6108.36 | 113354 / 113318 |

Existing-workload medians and memory are similar. Ordinary largest-scale
A/A spans 3.59–3.68 seconds. These observations do not establish a speedup.

New product workloads use candidate A/A scaling with
`scripts/measure_lexer.py --stage lowir --product-constant-input MODE`:

| Product workload | Run | Functions | Candidate median ms | Candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| product-shared | 1791859489622459904 | 1,000 | 183.41 | 17712 |
| product-shared | 1791859489622459904 | 4,000 | 741.56 | 62038 |
| product-shared | 1791859489622459904 | 16,000 | 1439.88 | 240414 |
| product-unique | 1791859543036961776 | 1,000 | 96.71 | 20526 |
| product-unique | 1791859543036961776 | 4,000 | 391.71 | 72468 |
| product-unique | 1791859543036961776 | 16,000 | 1674.95 | 283162 |
| product-chain | 1791859587874245148 | 1,000 | 105.73 | 20504 |
| product-chain | 1791859587874245148 | 4,000 | 409.68 | 72486 |
| product-chain | 1791859587874245148 | 16,000 | 1711.95 | 283190 |

The shared case uses one constant, unique uses distinct aliases, and chain uses
forward dependencies. Memory grows roughly with input size. Unique/chain elapsed
medians also grow roughly with input size. Shared timing improves disproportionately
between 4,000 and 16,000 functions despite identical A/A binaries, so cross-size
wall times still include host variation. Largest unique/chain calibration spans
1.68–1.74 and 1.71–1.74 seconds. These are observations, not universal bounds.

An execution-environment transition removed the original tool handle while
its process continued. A local retry wrapper was interrupted, but its benchmark
child survived and completed all 54 samples in run `1791859003074821226`.
Receipt verification caught the child continuing after the wrapper had exited.
It overlapped the original three product runs and the first shared rerun
(`1791859258793627456`). Those five runs retain all 270 observations and are
excluded from the qualified timing set. All three product workloads were then
rerun after every overlapping process had finished, producing the table above.
No build/test work overlapped timing; the overlap was the duplicate benchmark.

All 594 observations are retained: 324 qualified samples plus 270 overlapping
samples. Raw records are `obj/perf/RUN/{samples.json,observations.jsonl}`.
The isolated checkout holds `product-perf-receipts.json`,
`product-perf-original-receipts.json`, `product-perf-all-receipts.json`,
`product-perf-overlapped-receipts.json` and `product-perf-verified.json` under
`obj/pa13-work/`. The three qualified existing-workload runs remain in the
original checkout; absolute receipt paths identify each location. The clean
new-workload runs use the same binary bytes and inputs in the isolated checkout.

## Nominal constant patterns and derived equality

Struct and enum constants now enter the same typed query and selected-pattern
pipeline as tuple/array constants. Flat product storage retains a selected
variant ID and declaration-ordered values. Record construction consumes typed
field selections; struct updates fill only missing fields from the shared base.
Field projections use retained nominal ordinals. Forward dependencies, nested
values, lazily selected branches and empty constructors reuse the existing query
engine. No constructor names or source fragments are reparsed.

Constructor calls also evaluate their callee expression for constant eligibility.
The previous fieldless-enum fast path could skip a callee block containing a
non-constant call. All calls now schedule the callee; a constructor identity is
a typed constant value distinct from a constructed instance. Struct updates
likewise check their base even when every field is explicitly supplied or the
struct has no fields. Negative controls cover both arities and an enum-to-integer
constant, plus zero-field and completely overridden updates.

The builtin derive hooks now recognize `PartialEq` and `Eq`. These selected
capabilities check every field of every variant; `Eq` additionally excludes
floating fields and requires `PartialEq`. Packed equality derives require Copy
fields, including a packed alignment that happens to fit the particular field.
This is a closed-profile derive capability, not general trait/operator dispatch,
manual implementations, methods or procedural derive expansion. Those remain
later assignment boundaries.

Equality and structural-value eligibility are distinct. An inactive enum field
may contain a function pointer because it implements PartialEq; a non-equality
field invalidates the derive even when inactive. Actual translated values inspect
only the active variant and reject non-structural leaves such as NaN. Empty
arrays still check the element type's PartialEq capability, but have no actual
leaves: `[fn(); 0]` and a zero-length float repeat containing NaN can match.
Nominal translation emits ordinary product patterns with variant and field IDs,
so existing coverage, ownership, delayed captures and guard retries handle them.

The new equality query uses a flat graph and reverse dependency edges. A greatest
fixed point propagates missing PartialEq/Eq capabilities through recursive
reference components before publishing any result. Positive and negative results
are both cached. Per-query work counts collection and propagation, bounded at
1,000,000 operations. Direct tests exercise mutually recursive failure propagation
in either demand order, distinctions among pointer/float/array capabilities,
20,000 nested types without recursion, and repeated queries with no additional
solver work. Ordinary node facts remain unchanged.

There are 34 asserting nominal programs and 28 independently declared negative
programs, plus external-module, recursive-derive, resource and borrowed controls.
Pinned rustc checks the supported programs and executes every asserting case.
The invalid cases also require normal diagnostic exit 1, atomic output and
matching count/full diagnostics. The old derived-nominal rejection was removed
from the later-capability test and moved into the supported coverage. The
operator test still requires the explicit scalar-only operator boundary.

Borrow tests exposed a separate pinned-rustc discrepancy in an existing rule.
The vendored Reference's `patterns.ident.binding.non-reference` and
`patterns.ident.binding.auto-deref` rules include constants of non-reference
types among patterns that automatically dereference their input. RustGM retains
that behavior. Pinned rustc instead rejects bare named scalar, unit, tuple,
array, struct and enum constants against borrowed inputs (E0308). All six
explicit `&C` controls pass both compilers, as do implicit/explicit bare unit
constructor controls. These 14 inspection checks retain the six acceptance
differences explicitly; they do not claim same-source equivalence. The earlier
product borrow test now uses the explicit form. Executing borrowed patterns
still requires the later loan/region proofs.

The frozen candidate is `obj/pa13-work/nominal-constant-candidate`, SHA-256
`d0a2340bdc200125d3b31a065a766d10631b93b424de0f955e3d3b7f84ac5c9c`.
Both debug and release pass 327 Rust tests. All 1,695 active assignment cases,
Clippy all/pedantic, rustfmt, the 207-file audit, 24 harness checks,
38-assignment placement, 24 upstream fixtures, 126 Reference files, backend
qualification and the complete independent rustc course checks pass. Explicit
reference regeneration changes twelve compiler-source receipts only; committed
oracle stdout, stderr and exit-status bytes remain identical.

Native qualification passes 38 new checks: 34 nominal programs, one cross-file
package and three small asserting performance inputs. It also passes all 334
previous PA13 native checks, retaining the four constructed-count guard oracles
and eight signed-zero Reference/rustc pairs. PA11 passes 194 independent checks
and 180 native comparisons; PA12 passes 97 checks and 48 native comparisons.
The built release binary and frozen candidate have identical hashes.

Evidence is summarized in `obj/pa13-work/nominal-validation-summary.json`.
The six native receipts under that directory's parent are:

- `nominal-patterns-qualified-1791861514054679288/evidence.json`: 38 new checks.
- `nominal-products-qualified-1791861519561879173/evidence.json`: 29 earlier checks.
- `nominal-constants-qualified-1791861523555731892/evidence.json`: 31 earlier checks.
- `nominal-previous-native-1791861527811963605/evidence.json`: 150 earlier checks.
- `nominal-guards-qualified-1791861548726774322/evidence.json`: 61 earlier checks.
- `nominal-floats-qualified-1791861556782953344/evidence.json`: 63 earlier checks.

The final validation logs are `nominal-final-{debug,release,lint,refs,report,
rustc-native}.log`, `nominal-{audit,audit-tests,harness}.log` and
`nominal-{values,products,constants,previous,guards,floats}-native.log`.

PA13 remains planned: text/reference constant values and the final assignment
contract/fixtures remain unfinished. PA14 and PA15 remain part of the active goal.
This checkpoint is developed in the writable temporary checkout while the home
checkout and network publication remain unavailable under the current session
permissions; validation does not imply a public push or hosted CI result.

Performance validation retains 378 observations from seven sequential workloads.
Every workload snapshots its binaries and inputs, checks full rendered and count
output equality, warms both snapshots, runs four A/A calibration samples, and
retains three ABBA blocks at each of three sizes. There were no concurrent builds
or test runs during measurement. `nominal-perf-receipts.json` records run
intervals; `verify_nominal_perf.py` verifies all raw observations, hashes, phase
counts and equality in `nominal-perf-verified.json`.

The four existing workloads compare the product baseline
`3c039049558a8ebb8c3f9837321051067c5c94e5cead02146a69552b080198d2`
with the nominal candidate, pinned to CPU 10:

| Existing workload | Run | Largest functions | Baseline/candidate median ms | Baseline/candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| ordinary | 1791861631992134416 | 64,000 | 6133.56 / 6152.53 | 714140 / 714116 |
| scalar-constants | 1791861762085392566 | 16,000 | 1124.91 / 1126.78 | 142090 / 142080 |
| product-shared | 1791861792054810731 | 16,000 | 1997.58 / 1954.50 | 240392 / 240428 |
| product-chain | 1791861850907062199 | 16,000 | 1780.45 / 1746.55 | 283192 / 283090 |

The paired medians and memory remain comparable; these timings establish no
speedup. Ordinary largest-scale A/A spans 3.99–5.96 seconds, while paired samples
span 4.26–6.31 seconds across the two binaries. Shared-product largest-scale
A/A spans 1.50–2.59 seconds. This substantial host variation limits any precise
regression claim; every observation is retained rather than discarding slower
samples.

New nominal workloads use candidate A/A scaling with
`scripts/measure_lexer.py --stage lowir --nominal-constant-input MODE`:

| Nominal workload | Run | Functions | Candidate median ms | Candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| nominal-shared | 1791861899880877575 | 1,000 | 99.72 | 17706 |
| nominal-shared | 1791861899880877575 | 4,000 | 640.63 | 62122 |
| nominal-shared | 1791861899880877575 | 16,000 | 1529.04 | 240052 |
| nominal-unique | 1791861945249716017 | 1,000 | 136.12 | 24630 |
| nominal-unique | 1791861945249716017 | 4,000 | 564.24 | 89498 |
| nominal-unique | 1791861945249716017 | 16,000 | 2378.39 | 347456 |
| nominal-chain | 1791862008431764013 | 1,000 | 102.12 | 19850 |
| nominal-chain | 1791862008431764013 | 4,000 | 402.29 | 69740 |
| nominal-chain | 1791862008431764013 | 16,000 | 1615.93 | 268802 |

Shared uses one type/constant, unique uses independently derived nominal types,
and chain uses forward constant dependencies. Memory grows roughly with input
size. Unique/chain elapsed times also scale roughly with input size; the shared
4,000-function case shows a calibration-to-paired shift from 374–421 ms to
376–646 ms despite identical A/A binaries. No universal complexity or throughput
bound is inferred from these wall times. The iterative solver's direct work-count
tests provide separate evidence for deep graphs and cached repeated demands.


## Text pattern inspection checkpoint

String and byte-string literals, including raw forms, now type as immutable
static references and participate in selected-pattern inspection. Demanded text
constants consume the stored decoder result and intern exact bytes in one flat
arena with compact IDs and collision-checked hash buckets. Ordinary expression
literals do not allocate semantic text values. Repeated references to a value
reuse its bytes; repeated subpatterns share IDs while fields retain distinct
places. The ordinary node record remains unchanged.

Strings translate through a typed dereference to a `text="..."` leaf. Byte
strings translate to byte ranges within fixed-array products or exact slices,
using the existing one-million-work expansion budget before allocating edges.
Literal byte patterns support slice inputs; named constants preserve their
annotation, while a byte-string constant initializer may unsize to an explicitly
declared slice reference. This initializer conversion remains a typed adjustment
with pending ordinary lifetime constraints.

Coverage collects distinct text identities from the current query's roots only.
It gives each observed string a rank and reserves one extra rank for all other
strings. Existing product/constructor/slice decision graphs combine those ranks
without expanding strings or rescanning unrelated bodies. The text view uses the
ordinary escaped-byte writer. Missing witnesses can name an observed string or
`other string`; neither is source code for later phases to interpret.

Constant/static annotations now apply Rust's static lifetime default. Nested
function-pointer binders keep ordinary input/output elision; type aliases do not
acquire a static default. Pattern value matching unifies type shape without
recording an equality between the constant's storage lifetime and the input's
lifetime. A dedicated inference operation retains preexisting expression
constraints, with no change to ordinary type equality or reference mutability.

Declared independent cases cover 27 acceptances and 18 rejections. Additional
checks cover decoded/raw duplicate usefulness, composed/decomposed Unicode,
NUL/empty strings, byte-array and slice mixing, nested products, named aliases,
macros, cross-file provenance, guard coverage, conditional patterns and static
elision. Direct invariants cover exact interning, repeated-node sharing, query
locality, unchanged expression lifetime constraints, and diagnosed expansion
limits with no partial output. One cross-file positive and three explicitly
later-stage controls also compile with rustc. These inspection checks do not
claim executable parity for references.

Nominal `&str`/slice fields still require PA22's fat-pointer layout. Sized
byte-array reference fields can be inspected. C strings still require the later
library type. General promoted references, such as `const C:&u8=&7`, remain
unfinished. The executable builder rejects text values/reference decisions until
reference lowering and loan proofs exist. PA13 remains planned, with no grading
manifest or generated PA13 sidecars published yet.


The checkpoint passes **338 Rust tests** in debug and release and **1,695 active
assignment cases**. Existing `.ref`, `.ref.stderr` and `.ref.exit_status` bytes
are unchanged; twelve compiler-source receipts were regenerated explicitly.
Clippy/rustfmt, the 211-file syntax/architecture audit (zero findings), harness
checks and 38-stage fixture placement pass. All 24 imported fixtures and 126
pinned Reference files verify. Independent rustc checks and CPPGM backend smoke
checks pass. PA11 records 194 checks/180 executions; PA12 records 97 checks/48
executions.

All 372 prior PA13 native programs also pass with the initial text candidate.
Final review gated array-to-slice probing on the reference heads already known
by ordinary coercion. The final candidate recompiles every one of those inputs
to byte-identical LowIR, preserving the linked native execution evidence;
`obj/pa13-work/text-final-native-equality.json` records the input/LowIR hashes and
prior receipts. The earlier four constructed guard-count oracles and eight
signed-zero Reference/rustc differences retain their original classification.
These are not new native executions of text-reference programs.

The final frozen candidate is `obj/pa13-work/text-final-candidate`, SHA-256
`ae1b8ed10eef4f3f3ccad03a5d3a2a5be1eb05cb42fcfe8f9daa55a0247ccdcf`.
Its complete source checks are `text-home-{debug,release,quality}-final.log`,
`text-home-oracles-final.log`, `text-home-rustc-native.log` and
`text-home-independent.log` under `obj/pa13-work/`. Native receipts are indexed
in `text-native-receipts.json`. The final build repeats the full debug/release
suites and earlier assignment checks after the coercion cleanup. No builds or
tests run alongside the qualified timing series.


The timing series retains **432 observations** from eight sequential runs on CPU
10. Each uses frozen binaries/inputs, complete rendered and count-output
equality, A/A calibration and three ABBA blocks at each size. Existing workloads
compare the nominal baseline (`d0a2340bdc200125d3b31a065a766d10631b93b424de0f955e3d3b7f84ac5c9c`)
with the final text candidate.

| Existing workload | Run | Largest functions | Baseline/candidate median ms | Baseline/candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| ordinary | 1791864380753058196 | 64,000 | 3585.32 / 3580.64 | 715180 / 715142 |
| scalar-constants | 1791864477189243835 | 16,000 | 643.72 / 645.85 | 142128 / 142076 |
| nominal-shared | 1791864494629540005 | 16,000 | 1497.77 / 1493.43 | 240766 / 240668 |
| slice-coverage | 1791864539038042600 | 16,000 | 16351.03 / 16115.51 | 135268 / 135288 |

Largest-scale medians and memory are similar. The 4,000-function nominal run
is noisy: baseline/candidate paired medians are 526.70/611.04 ms, with A/A
calibration at 355–378 ms and paired samples spanning 364–652 ms. Every sample
is retained. This series does not establish a precise regression or speedup
bound, and the largest-scale table must not hide that intermediate-size shift.

New workloads use candidate A/A scaling with
`scripts/measure_lexer.py --stage patterns --text-pattern-input MODE`:

| Text workload | Run | Functions or arms | Candidate median ms | Candidate median KiB |
| --- | --- | ---: | ---: | ---: |
| text-shared | 1791864962307744752 | 1,000 | 255.70 | 12776 |
| text-shared | 1791864962307744752 | 4,000 | 1008.43 | 39582 |
| text-shared | 1791864962307744752 | 16,000 | 4196.38 | 148036 |
| text-unique | 1791865073780817746 | 1,000 | 271.76 | 15614 |
| text-unique | 1791865073780817746 | 4,000 | 1107.55 | 52244 |
| text-unique | 1791865073780817746 | 16,000 | 4560.33 | 200440 |
| text-bytes | 1791865195344650342 | 1,000 | 349.83 | 11418 |
| text-bytes | 1791865195344650342 | 4,000 | 1434.37 | 34624 |
| text-bytes | 1791865195344650342 | 16,000 | 5809.79 | 127188 |
| text-wide | 1791865357389160970 | 128 | 4.99 | 3624 |
| text-wide | 1791865357389160970 | 512 | 9.00 | 4350 |
| text-wide | 1791865357389160970 | 2,048 | 23.23 | 7124 |

Shared and unique exercise tuple constants containing strings and byte slices;
bytes mixes literal, prefix/suffix and empty patterns. Wide has one match with
128/512/2,048 distinct string arms. The first three have 1,000/4,000/16,000
functions. Small generated controls compile independently with pinned rustc.
Time and memory broadly follow input size. Fixed process overhead is visible
in wide's small inputs. Noise remains visible: unique's smallest A/A includes
466 ms, and bytes' largest paired samples span 5.30–7.97 seconds despite
identical A/A binaries. These are scaling observations, not universal bounds.

Raw records are `obj/perf/RUN/{samples.json,observations.jsonl}`. The work index,
verified statistics and checker are `obj/pa13-work/text-perf-receipts.json`,
`text-perf-verified.json` and `verify_text_perf.py`; verification output is
`text-home-perf-verification.log`. All runs include their immutable A/B binaries
and input snapshots. No timing runs overlap each other or this checkpoint's
builds/tests.

After filesystem permissions were restored, the two validated product/nominal
commits were fast-forwarded into `~/rustgm` and pushed to public main. The earlier
local product diff is preserved in a named Git stash. All 15 temporary performance
runs are now also archived byte-for-byte under this checkout's `obj/perf/`, and
their top-level work receipts/binaries are in
`obj/pa13-work/recovered-isolated/`. Historical receipts keep their original
absolute paths; `archive-map.json` records the corresponding archive locations.
The temporary checkout remains intact, but further development uses `~/rustgm`.

## Shared reference constants and place queries

PA13 inspection now accepts shared promoted references such as `const C:&u8=&7`,
references to immutable statics, nested references, and array-to-slice initializer
coercions. Selected constant patterns reuse the existing dereference, product,
range and exact-slice forms. Reference IDs share subpatterns within a use while
selected projections retain their own places. Ordinary node facts remain 80
bytes. Only adjusted field/index nodes allocate sparse autodereference entries.

Constant evaluation separates a place from a value read. A flat place record
retains value, type, owned/shared/static access and packed alignment; an iterative
query worklist handles borrowing, dereferencing and projections. Borrowing a
non-Copy static does not move it. Reading one Copy field does not demand that the
whole container be Copy. Reads through shared/static storage check Copy before
using a cached definition value, including after another pattern has borrowed
and cached that same static. Packed provenance survives projections, while an
actual dereference uses its referent's own alignment. Immutable static origin
and alignment are checked when a reference is demanded.

Typed expression inspection also records shared borrows, dereferences, dynamic
slice indices and automatic field/index dereferences. Function-body borrows get
fresh `expression-borrow` regions; these are obligations, not proven lifetimes.
The executable builder diagnoses pending region/loan proofs for these operations,
including projected references, before producing LowIR. Mutable promotions,
mutable static access, cyclic allocation storage and general CTFE remain later
capabilities. This is no claim of native reference execution.

Review closed the root-place validity issue recorded in the constructor coverage
checkpoint above. Coverage now distinguishes an owned root from a potentially
invalid referent, using selected dereference/projection facts. It cannot prune
`E::A(Void)` from `match *x` when `x: &E`. Grouping preserves place identity;
owned value results retain their separate ownership obligations. A directly
empty root enum still permits an empty match, while a wildcard on a dereferenced
empty place remains useful. Irrefutable binding requirements retain root validity
through the deferred constant-pattern worklist, so a later constant use cannot
change the answer. Seventeen independent controls cover roots, fields, indices,
grouping, macros and deferred matches/bindings; separate typed assertions check
arm usefulness for owned versus dereferenced empty values.

The new maintainer suite declares 34 acceptances and 19 rejections, with an
additional cross-file acceptance preserving pattern use-site provenance. Together
with the 17 root-place controls these are 71 pinned-rustc acceptance/rejection
comparisons. Typed invariants also check selected scalar ranks, shared reference
subpatterns, static-cache read permissions and a 2,048-level reference chain.
A cyclic static-pattern query diagnoses a bounded dependency cycle in RustGM.
Pinned rustc crashes with SIGSEGV on that probe; this is retained as an
infrastructure failure and is never classified as language-rejection parity.
The cyclic control therefore runs RustGM only.

The `--reference-pattern-input shared|places|chain` timing generators use the
pattern stage. Shared reuses one promoted tuple reference across functions;
places repeatedly borrows an indexed field through a shared pointer to a
non-Copy immutable static; chain grows nested reference aliases/constants in one
query. Small examples of all three were independently accepted by pinned rustc.
The new workloads measure inspection, not runtime loans or reference codegen.

The final frozen compiler is `obj/pa13-work/reference-final-candidate`, SHA-256
`e730a7af794e7f8697f44ef194a50db75c311ecfd8553f8f351b320ff104793b`.
The initial `reference-candidate` preceded the root-validity fix and is not the
release candidate. Debug and release each pass 347 tests. All 1,695 earlier
assignment cases pass; explicit reference regeneration updates twelve source
receipts and changes no existing stdout, stderr or status oracle. Rustfmt,
Clippy, the 216-file audit and harness/placement checks pass. The earlier
independent run qualifies all active stages, PA11's 180 and PA12's 48 native
comparisons, the backend bridge and all 24 imported fixtures/126 Reference files.
The final candidate additionally recompiles every one of the 372 prior PA13
native inputs to byte-identical LowIR; their actual executions remain in the
linked prior receipts, with the documented Reference/rustc qualifications.

Validation records: `reference-validation-summary.json`, `reference-final-*.log`,
`reference-independent.log` and `reference-final-native-equality.json`, under
`obj/pa13-work/`. The workload controls are in `reference-workload-checks/`.

Seven sequential CPU-10 measurements retain 378 observations: each input has
warmups, four A/A calibration samples and three paired ABBA blocks. The four
existing workloads compare against `text-final-candidate`
(`ae1b8ed10eef4f3f3ccad03a5d3a2a5be1eb05cb42fcfe8f9daa55a0247ccdcf`).
The three new reference workloads use identical final-candidate A/A binaries.
No local RustGM build or test overlapped the measurements. Frozen binary/input
hashes, every raw observation, count output and complete rendered output were
verified by `verify_reference_perf.py`; the index and results are
`reference-perf-receipts.json` and `reference-perf-verified.json`.

| Workload (largest scale) | Run under `obj/perf/` | A / B median ms | A / B median peak KiB |
| --- | --- | --- | --- |
| ordinary (64000) | 1791868504115063674 | 3607.12 / 3522.85 | 715220 / 715198 |
| product-shared (16000) | 1791868603000285219 | 1446.50 / 1460.67 | 241342 / 241248 |
| nominal-shared (16000) | 1791868641774038204 | 1459.90 / 1464.79 | 240700 / 239876 |
| text-shared (16000) | 1791868680929460519 | 4058.66 / 4086.76 | 148022 / 148002 |
| reference-shared (16000) | 1791868789277617682 | 545.00 / 547.10 | 85284 / 85196 |
| reference-places (16000) | 1791868804034794487 | 699.51 / 711.06 | 124290 / 124286 |
| reference-chain (2048) | 1791868822968850538 | 33.50 / 33.78 | 11344 / 11362 |

The measured existing paths show no material change in median time or peak
memory. These are noisy wall-time observations, not a precise speedup claim.
Ordinary calibration spans 3547–3631 ms at its largest 64k-procedure
input; text calibration spans 4108–4205 ms. Reference-chain candidate medians
are 5.29/10.77/33.78 ms for 128/512/2048 levels; shared and static-place workloads
vary function count rather than the depth of a single pattern. The separately
checked flat-chain and shared-subpattern invariants establish the intended
representation independently of these timings.
