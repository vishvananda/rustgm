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
