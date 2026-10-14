# PA17 destruction buildout

PA17 remains planned and incomplete. PA15 and PA16 were released at `1651a5e`;
this journal records the implementation that precedes the PA17 grading contract.
No released destruction grading suite or completed-destructor execution is
claimed here. Maintainer tests cover the foundations described below.

## Built-in Drop identity

The resolver now retains a distinct `BuiltinTrait` definition for `Drop`.
Destructor selection can use that identity, rather than matching a user trait's
name. Supported metadata exposes the same definition through `core::ops::Drop`,
`std::ops::Drop`, ordinary aliases, glob imports, reexports and expanded macros.
Metadata is created on demand, so programs that do not request it retain their
previous definition IDs and avoid allocating external scopes.

External crate metadata has its own scope. Lexical declarations can shadow
relative `core`/`std` paths and unqualified `Drop`; absolute external paths select
the external scope. Prelude names are not members of the user's crate root.
`no_std` suppresses the automatic `std` entry. Explicit `extern crate` names and
aliases use the same supported metadata. The import dependency worklist wakes a
forward reference when a later explicit extern declaration supplies its name.

Configured `no_implicit_prelude` attributes are retained as module facts and
inherited by descendants. They disable implicit trait/external lookup while
preserving explicit imports, absolute external paths and primitive types.
Module attributes are read once from configured syntax, not rescanned per lookup.
The supported external metadata remains limited to this language-trait surface;
this is not a general implementation of `core` or `std`.

Four maintainer tests exercise trait identity, negative namespace/prelude controls,
absence of metadata in ordinary programs, and the semantic boundary. Positive
and negative namespace inputs are checked independently with pinned rustc.
Both ordinary resolution and resolution after expansion consume the same facts.

The identity checkpoint initially rejected the built-in trait during declaration
analysis. The typed-header checkpoint below advances that boundary to typed body
construction. The pending-body checkpoint then advances it to completed proof
consumers; raw body construction now retains the unproved cleanup facts.

## Typed destructor headers and receivers

The declaration worklist indexes genuine Drop implementations and their methods
in two linear passes over resolved definitions. It retains the local nominal
target, Self definition, method identity, captured lifetime slots and declared
outlives requirements. Duplicate impls, Copy plus Drop, nonnominal targets,
specialization, missing/extra associated items and invalid method signatures are
diagnosed. Aliases and configured/expanded methods use the ordinary resolved AST.

Impl binders include named and anonymous lifetime slots. Method bodies capture
those slots in one flat environment, preserving their identities through Self
annotations and nested callable binders. The receiver borrows the exact Self
type with an independent late lifetime. Its binding refers to the whole mutable
reference; it does not use reference-pattern destructuring. `mut self: &mut Self`
retains mutable binding separately from mutable referent. Destructor bodies now
run through ordinary type inference, field access, patterns and call checking.
Explicit destructor calls and function values are rejected.

The existing directed outlives graph now lives in `types::region_graph`, with
the borrow module reexporting the same API. Its algorithm is unchanged. Header
coverage treats target slots as universals and unused impl slots as existential
variables. Nominal explicit and field-implied requirements are assumptions;
impl/method requirements cannot add relations between target parameters, even
through an unused lifetime. Method-declared lifetimes may be unused or late, but
cannot become extra early-bound parameters. Explicit `extern "Rust"` uses the
ordinary Rust convention; its already-decoded literal also supports raw and
escaped spellings. Foreign conventions remain deferred.

Type inspection emits deterministic `destructor`, `destructor-bound` and
`receiver` facts. Destructor rows explicitly mark `cleanup=pending`; nonempty
counts add destructor and receiver totals. Ordinary programs retain their exact
views. At that checkpoint typed body construction rejected these impls. The
pending-body checkpoint below permits raw body inspection while ownership,
region, loan and LowIR modes continue to reject outstanding elaboration.

The header matrix declares 73 source outcomes, independently compiled by pinned
rustc in the maintainer suite. It covers lifetimes, aliases, method conformance,
receiver forms, body errors and nominal coverage; import/prelude/macro identity
tests now also check typed destructor selection. A CLI boundary test checks type
inspection and all later proof/execution modes. These remain implementation
tests rather than a released PA17 assignment contract.

## Pending body requirements

Typed receiver bindings now enter the body CFG directly as whole-reference
parameters in the root scope, including explicitly named receiver lifetimes.
The builder consumes the retained receiver type, definition and mutability;
it does not reinterpret receiver syntax as a destructuring pattern.

`types::destruction::NeedsDrop` memoizes requirements over the closed type DAG.
Its explicit worklist and budget handle deep shared tuples, nested nominal
fields and diagnosed by-value cycles without recursive traversal. A custom
destructor requires cleanup; otherwise owned fields/elements propagate that
requirement. References, raw pointers and callables do not follow their
referents. Zero-length arrays require no element cleanup, while nonempty arrays
query the element once regardless of length. The store, completed definitions
and custom-destructor set define one query context.

`body::drops::Pending` records destructor signatures and captured lifetime slots,
per-body type requirements and potential replacement/storage cleanup sites.
Stores remain candidates even when the destination might not be initialized.
Live-local suffixes use persistent parent counts to avoid expanding every suffix
at every exit; scope and temporary exits retain their existing indices. Candidate
sites may overlap and their view order is not runtime destructor order. Guard
views do not own the viewed object. Existing materialized temporary places also
do not yet cover every owned rvalue's destruction scope.

Body inspection prints explicit `pending-destructor`, `drop-requirements` and
`drop-candidate` rows. Completed ownership, CFG region and loan verification
reject these programs before issuing a checked token, so LowIR still cannot
silently omit cleanup. Promotion skips values requiring destruction; ordinary
constants and zero-length arrays remain eligible.

Lifetime binders now index declared-name membership before first-use slot
assignment. Captured impl slots extend that index without erasing the method's
own lifetime declarations. Declaration order, first-use numbering and the
16-bit slot capacity diagnostic remain unchanged. This removes the measured
quadratic declaration scan for wide impl binders.

## Remaining implementation

Feed actual destructor uses into lifetime and loan checks. Consume the retained
initialization and rvalue-scope facts in conditional cleanup planning. Elaborate
drop scopes, replacement, partial moves, temporaries and early exits into typed
plans consumed by LowIR.

Ownership's existing `Word.may` records whether storage has ever been written;
the new optional initialized plane is separate. The scope-aware analysis now
integrates place/rvalue consumption with ordered cleanup endpoints. Select
conditional flags and field/array operations from that replay. The custom-destructor
field-move checker must be required by completed destruction proofs. Replacement
must evaluate the incoming value, destroy the initialized old destination and
then store the replacement.

Use the retained creation and cleanup scopes for owned rvalues, including
discarded values that never become materialized temporary places. Combine
overlapping candidate sites into one ordered plan: locals in reverse declaration
order and temporaries in reverse creation order; custom destructors before
fields; fields and array elements in their
specified forward order. Handle early exits and failed guard attempts explicitly.
Seed destructor accesses into region and loan liveness before issuing the checked
token, and retain the proved cleanup plan for lowering. Do not release the current
pending-proof guards until these obligations are implemented and tested.

## Owned values and initialization

The body builder shares one memoized destruction query with the pending-requirement
collector. A body allocates its boxed owned-value table only when it creates a
destructible value. Each value retains its creation point, lexical lifetime and
ordering relative to storage exits. Transparent result forwarding updates its
scope without creating another object. Synthesized pattern-capture reads use
the selected binding's scope. Function and constructor returns record transfer
before local cleanup. The existing borrowed-place materialization table remains
separate. Raw body views expose `owned-value` and `return-owned` records.

`ownership::initialization` uses the same move-path construction, transfer
functions and fixed-point solver as ordinary ownership checking. Its extra
may-be-initialized word is absent from the ordinary state layout. Retained entry
states and a bounded replay visitor classify ranges or individual bits as never,
conditional or always initialized. A conditional aggregate range can contain
partially moved fields; it must be split before selecting cleanup. The dataflow
matrix stays within its 64 MB budget and replay charges queries and transfers.

Initialization analysis retains the element/gap partitions already computed by
array move-path numbering. Unselected intervals share one bit, independently of
their length; observed elements retain their path identity for nested field moves.
Ordinary ownership checking does not retain these extra partitions.

The retained facts provide a custom-destructor field-move check using resolved
nominal identities. It propagates protected ancestors through existing move paths
and rejects only non-Copy field moves. Whole-object moves, Copy reads, borrowed
patterns and field replacement remain eligible. This check is available to the
destruction phase; completed proof entrypoints still reject pending elaboration.

These are not executable drop flags or a finished cleanup plan. Initialization
models explicit move/access transfers and owned-rvalue consumption. Implicit
scope cleanup and its loan/region effects must still be integrated before
removing the pending-proof boundary.

## Owned operand consumption

Initialization analysis retains a sparse path for each destructible rvalue,
including values with no borrowed-place materialization. Ordinary ownership
analysis keeps its existing path set and state layout. The bounded fixed-point
solver and replay share the consumption transfers; inspection checks that each
owned source is initialized before consuming it. A replay callback observes
return transfers before cleanup statements and ordinary operands before the
statement consumes them.

The analysis collects consumption from typed aggregate and call argument lists,
constraint adjustments, stores and retained return records. Its deterministic
effect table preserves the value, path, CFG point, return/storage-exit ordering
and disposition. It bounds collection and sorting work without expanding array
elements. Callers can distinguish transfers, immediate disposal and constant
instantiation when constructing the eventual cleanup plan.

An ordinary `[expression; 0]` still evaluates its operand. Pinned rustc destroys
that value before the next sibling expression or callee runs; the effect table
records disposal at the repeat statement. A repeat of a constant path instead
instantiates elements and has no prototype destructor, including when length is
zero. Inline const evaluation remains a later capability. The independent trace
probes under `obj/pa15-work/drop-repeat-probes-v1/` retain ten matching declared
outcomes and two rejected ordering predictions alongside rustc's actual output.
The latter refuted delaying ordinary zero-repeat destruction until statement end.

Repeat mode now comes from the type checker's resolved constant-path selection,
including parenthesized paths, rather than guessing from the final rvalue kind.
The sparse selected-node set survives runtime lifetime adjustments. Blocks that
forward a constant remain ordinary operands. This also repairs PA16 execution:
the new `thin-native/constant-repeat-lifetimes.rs` exercises zero, one and two
non-Copy constant elements with narrowed lifetimes. The preceding compiler
rejects its two-element case with an incorrect Copy requirement; pinned rustc
accepts and executes it. Its independently declared outcome precedes generation
of its LowIR oracle.

These consumption facts do not execute destruction. The later scope-aware
checkpoint adds scope and forced-initializer state cleanup; runtime flags,
destructor glue and the induced loan/region uses remain pending.

Required assignment fixtures must establish destructor/field/local order,
conditional flags, partial initialization and every cleanup path with independent
rustc and CPPGM execution evidence. Publish the complete handout, manifest and
text oracles only after those requirements and all earlier gates pass. The abort
profile remains in force; unwinding belongs to PA31.

## Identity checkpoint validation

All 3,093 PA1–PA16 course cases retain their exact stdout, stderr and exit status.
The debug and release frontend suites each pass 470 tests, with six explicit
backend-dependent tests still excluded from those suites. This checkpoint adds
24 positive namespace/identity vectors and 13 negative namespace/prelude vectors,
each independently compiled with pinned rustc. It also checks three sources
against the temporary destructor-elaboration boundary.

Formatting and Clippy all/pedantic with warnings denied pass without new lint
allowances. The syntax-aware audit reports 306 files and zero findings; all
25 harness tests pass. Upstream verification retains the checksums of all
24 imported fixtures and all 126 pinned Reference files. No assignment manifest,
reference sidecar or backend implementation changed.

Detailed logs use the `drop-identity-*` prefix under `obj/pa15-work/`.
The initial full run passed the frontend/course suites, then Clippy rejected an
explicit `panic!` in a test. That failure was fixed by returning a test error;
quality checks and the release suite then passed with the final source.

The performance check retained 162 observations: three scales per workload,
A/A calibration and three ABBA blocks. Source, immutable binary and frozen input
hashes agree after measurement. Full output hashes and count bytes agree at every
scale. This agent's builds/tests finished before timing; unrelated projects were
compiling concurrently on the shared host. Timing ranges and outliers are retained,
so these results do not establish an isolated speedup or a hardware-counter gate.

| Workload | Scale | Baseline median | Candidate median | Candidate median RSS |
| --- | ---: | ---: | ---: | ---: |
| Name resolution | 4,000 | 50.60 ms | 52.05 ms | 12,090 KiB |
| Name resolution | 16,000 | 211.72 ms | 218.41 ms | 37,582 KiB |
| Name resolution | 64,000 | 902.29 ms | 901.49 ms | 139,394 KiB |
| Ownership pipeline | 4,000 | 335.56 ms | 329.56 ms | 63,242 KiB |
| Ownership pipeline | 16,000 | 1,288.51 ms | 1,333.02 ms | 241,134 KiB |
| Ownership pipeline | 64,000 | 5,316.88 ms | 5,410.10 ms | 949,954 KiB |

Median changes range from -1.79% to +3.45%, with overlapping observed ranges.
The largest name-resolution case is effectively unchanged; the largest ownership
case is +1.75%. No consistent material regression is established across scales.

The new workload resolves 256/1,024/4,096 modules, each disabling implicit preludes
and importing `Drop` through an absolute glob. Pinned rustc accepts every frozen
input. Two immutable copies of the candidate produce identical complete views;
the preceding compiler has no equivalent accepted trait path. Candidate medians
are 8.00/24.89/96.65 ms, with 5,406/7,828/21,242 KiB median RSS. Retained definition,
reference and other reported counters grow linearly.

The baseline SHA-256 is
`abc0be79acb5f34ee5bbdb19ea98daf86fa3ac3172849ef5e679d74c4c283ada`;
the candidate is
`98fe8616d6814aac42dc220a4c728874753644bf4ff9eba6d835d70919c861d7`.
`obj/pa15-work/drop-identity-v1-perf-receipts.json` indexes all raw receipts and
observations; `drop-identity-v1-perf-verified.json` retains the summaries and the
matching identity/source receipts are stored beside it.

## Typed-header checkpoint validation

All 3,093 PA1–PA16 cases retain their exact stdout, stderr and exit status.
Both debug and release frontend suites pass 472 tests, with the same six
explicitly excluded backend-dependent tests. The header matrix contains 73
independently rustc-qualified cases. No assignment manifest, generated oracle,
backend implementation or native destruction claim changed.

Formatting, Clippy all/pedantic with warnings denied, architecture/file audits,
all 25 harness tests and upstream checksum verification pass. The syntax-aware
file audit reports 312 files and zero findings. New compiler code uses no lint
allowances. Detailed logs are `drop-headers-full1.log` and
`drop-headers-release1.log` under `obj/pa15-work/`; earlier focused/quality logs
retain the intermediate failures and their fixes.

The initial performance run retained 162 observations with A/A calibration and
three ABBA blocks at each scale. Full views and count bytes agree for the old
workloads. New destructor inputs are independently accepted by pinned rustc;
two immutable candidate copies produce identical views and linearly growing
counters. Compiler, test source, binary and input hashes were verified after
measurement. This agent's builds/tests had finished before timing. Other work
continued on the shared host; every observation, including outliers, is retained.

| Workload | Scale | Baseline median | Candidate median | Candidate median RSS |
| --- | ---: | ---: | ---: | ---: |
| Type checking | 4,000 | 141.29 ms | 139.48 ms | 32,440 KiB |
| Type checking | 16,000 | 613.27 ms | 651.14 ms | 116,304 KiB |
| Type checking | 64,000 | 2,541.92 ms | 2,754.35 ms | 457,322 KiB |
| Ownership pipeline | 4,000 | 338.88 ms | 341.25 ms | 63,576 KiB |
| Ownership pipeline | 16,000 | 1,329.89 ms | 1,348.11 ms | 240,976 KiB |
| Ownership pipeline | 64,000 | 5,640.23 ms | 5,856.38 ms | 949,888 KiB |

The larger type-checking medians increased 6.18% and 8.36%; ownership medians
increased 0.70–3.83%. Ranges overlap, but that alone does not dismiss a regression.
The new workload also showed up to 10.82% median variation between identical
binaries at its smallest scale. These observations prompted the fixed-CPU
follow-up recorded below.

The new workload checks 256/1,024/4,096 distinct nominal destructor impls, each
with two used lifetime parameters, one unused parameter, outlives requirements,
a named receiver lifetime and a field-mutating body. Candidate B medians are
22.64/74.76/319.54 ms, with 7,950/17,704/57,860 KiB median RSS. The prior compiler
rejects these inputs, so their timing comparison uses two copies of the candidate.

The baseline SHA-256 is
`98fe8616d6814aac42dc220a4c728874753644bf4ff9eba6d835d70919c861d7`;
the candidate is
`cad54f593ebb99a6bb1ad12cdd0afb613d29aee753a62e25514421adb657514f`.
`obj/pa15-work/drop-headers-v1-perf-receipts.json` indexes the initial raw receipts;
`drop-headers-v1-perf-verified.json` contains verified summaries. Identity/source
hash receipts and pinned-rustc input qualifications are stored beside them.

The follow-up fixed both binaries to CPU 2 and repeated the largest inputs with
A/A calibration and five ABBA blocks, retaining 52 additional observations.
Child user/system CPU time and context-switch counts accompany wall time.
It verified full/count byte equality and all frozen source/binary/input hashes.

| 64,000-function workload | Baseline wall median | Candidate wall median | Baseline CPU median | Candidate CPU median |
| --- | ---: | ---: | ---: | ---: |
| Type checking | 2,607.42 ms | 2,629.41 ms | 2,605.46 ms | 2,627.51 ms |
| Ownership pipeline | 5,663.58 ms | 5,817.99 ms | 5,659.72 ms | 5,815.10 ms |

The fixed-CPU type-checking difference is +0.84% wall / +0.85% CPU; it does not
reproduce the initial 6–8% type-checking gap. Ownership remains +2.73% wall /
+2.75% CPU in this follow-up. Its wall-time ranges are 5,319–5,865 ms for baseline
and 5,118–6,031 ms for candidate. Median RSS is effectively unchanged for ownership
and lower for the type-checking input. These are shared-host observations, not an
isolated performance gate; the modest ownership difference remains visible.
`drop-headers-affinity-results.json` indexes these additional raw receipts.

## Pending-body checkpoint validation

All 3,093 PA1–PA16 course cases retain exact stdout, stderr and exit status.
Both debug and release frontend suites pass 477 tests. Their six explicitly
excluded backend-dependent tests also pass in `make test-maintainer-native`
after qualification of the pinned backend. These native tests exercise existing
functionality; executable destruction remains unimplemented.

The new requirement tests independently evaluate twelve type predicates through
rustc's `std::mem::needs_drop` in constant array lengths. They cover custom and
structural destruction, zero-length and billion-element arrays, references,
pointers and callable values. A 20,000-level shared tuple graph checks bounded
iterative traversal and cache reuse; a separate graph checks diagnosed by-value
cycles. Source tests cover conditional stores, retained storage ranges, receiver
scope and promotion exclusions. The wide-binder test independently qualifies
4,096 captured impl lifetimes plus a method lifetime and checks the slot-capacity
diagnostic. Complete proof and LowIR entrypoints still reject pending cleanup.

Formatting, Clippy all/pedantic with warnings denied, all 25 harness tests and
upstream checks pass. The syntax-aware audit reports 316 files, zero findings
and no nesting warning. No new lint allowances or assignment oracle changes
were introduced. Logs under `obj/pa15-work/` include
`drop-requirements-full2.log`, `drop-requirements-release1.log` and
`drop-requirements-native1.log`. Earlier logs retain the corrected test-import
and formatting failures.

The final performance comparison retained 234 observations on CPU 2, with A/A
calibration and three ABBA blocks at each scale. Full views and count bytes agree
between the old/new binaries for all existing workloads. The new body workload
uses two immutable candidate copies because the old compiler rejects raw bodies
with destructors. All frozen source, binary and input hashes were verified after
measurement. This agent ran no builds or tests during timing; unrelated work
continued on the shared host, so these are not isolated hardware-counter gates.

| Existing workload | Scale | Baseline median | Candidate median | Candidate median RSS |
| --- | ---: | ---: | ---: | ---: |
| Type checking | 4,000 | 155.76 ms | 158.78 ms | 32,442 KiB |
| Type checking | 16,000 | 651.89 ms | 647.14 ms | 116,330 KiB |
| Type checking | 64,000 | 2,576.59 ms | 2,591.52 ms | 457,334 KiB |
| Ownership pipeline | 4,000 | 307.12 ms | 306.81 ms | 63,350 KiB |
| Ownership pipeline | 16,000 | 1,376.84 ms | 1,403.96 ms | 240,718 KiB |
| Ownership pipeline | 64,000 | 5,621.80 ms | 5,649.28 ms | 949,954 KiB |

Changes range from -0.73% to +1.97%, with nearly unchanged RSS. The largest
type/ownership inputs change +0.58%/+0.49%. The observed ranges overlap; no
consistent material regression is established in these measurements.

| Wide impl lifetimes | Baseline median | Candidate median |
| ---: | ---: | ---: |
| 512 | 4.26 ms | 4.11 ms |
| 2,048 | 7.70 ms | 6.59 ms |
| 8,192 | 32.77 ms | 15.49 ms |
| 32,768 | 322.81 ms | 55.97 ms |

These inputs were independently accepted by pinned rustc. The old lookup's
10x time growth at the largest 4x input step becomes 3.61x; the largest median
falls 82.66% with unchanged type output and effectively unchanged RSS.

The new raw-body inputs contain 256/1,024/4,096 locals with a potential early
return after each declaration. Pinned rustc accepts all three. Candidate medians
are 6.93/17.61/64.83 ms and RSS 5,038/8,546/20,506 KiB. Retained sites are
1,795/7,171/28,675 (`7N + 3`); shared destruction-query work stays at seven steps.
Body values, statements and blocks also grow linearly. This checks candidate
metadata growth, not completed cleanup execution.

Baseline SHA-256 is
`cad54f593ebb99a6bb1ad12cdd0afb613d29aee753a62e25514421adb657514f`;
candidate SHA-256 is
`34e769b8ef0858a3af1ebb4b9e710c8e953570054e0badd890aa3bd8fedbd301`.
`obj/pa15-work/drop-requirements-v1-perf-results.json` indexes all raw receipts,
including every timing, RSS and child CPU-time observation. The matching source
hash, binary identity and validation receipts are stored beside it.

## Owned-value checkpoint validation

All 3,093 PA1–PA16 cases retain byte-exact stdout, stderr and exit status. The
debug and release frontend suites each pass 484 tests; all six explicitly
selected maintainer native tests pass against the qualified pinned backend.
These native checks cover existing functionality, not finished destruction.
Formatting, Clippy all/pedantic with warnings denied, all 25 harness tests and
upstream verification pass. The syntax-aware audit reports 324 files and zero
findings. No lint exceptions or assignment reference changes were added.

Seven new tests exercise owned-value lifetimes and return transfer, first versus
replacement initialization, branch/loop joins, reinitialization, residual fields,
custom-destructor field-move rules and sparse array flags. Fifteen field-move
vectors are independently accepted/rejected by pinned rustc. Other source probes
are also rustc-qualified. An eight-element and a billion-element array retain the
same path/bit counts and three partitions after conditional first/last moves:
the endpoints are conditional and the untouched middle remains initialized.

Detailed logs are `obj/pa15-work/drop-values-full1.log`,
`drop-values-release1.log` and `drop-values-native1.log`; the validation receipt
hashes each log. Earlier focused logs retain the missing-pattern-scope failure
and its fix. That regression now passes alongside pattern, array and enum cases.

Performance measurements retain 216 observations on CPU 2, with A/A calibration
and three ABBA blocks at each scale. The old/new binaries produce identical full
views and count bytes for the existing body and ownership workloads. New views
use two identical candidate binaries; initialization analysis uses an immutable
driver linked to the same frontend library. All 330 source/configuration hashes,
binary identities and frozen input hashes were verified after measurement. This
agent ran no builds or tests during timing; unrelated builds continued on the
shared host, so these observations are not an isolated performance gate.

| Existing workload | Scale | Baseline median | Candidate median | Candidate median RSS |
| --- | ---: | ---: | ---: | ---: |
| Body IR | 4,000 | 301.00 ms | 300.76 ms | 59,860 KiB |
| Body IR | 16,000 | 1,241.70 ms | 1,243.55 ms | 227,378 KiB |
| Body IR | 64,000 | 5,091.83 ms | 5,068.56 ms | 897,416 KiB |
| Ownership pipeline | 4,000 | 335.66 ms | 340.32 ms | 63,386 KiB |
| Ownership pipeline | 16,000 | 1,387.42 ms | 1,388.94 ms | 241,456 KiB |
| Ownership pipeline | 64,000 | 5,796.46 ms | 5,812.09 ms | 951,186 KiB |

Observed changes range from -0.46% to +1.39%; the largest body/ownership inputs
change -0.46%/+0.27%. The largest ownership RSS grows by 1,214 KiB (0.13%). These
measurements do not establish a material throughput regression.

The new owned-body workload has 256/1,024/4,096 locals and an early-return option
after each declaration. Candidate medians are 6.81/18.54/67.28 ms, with median RSS
5,048/8,744/20,528 KiB. The retained site counts remain `7N + 3` and shared
destruction-query work remains seven steps.

The initialization workload has 256/1,024/4,096 functions, each with conditional
first/last moves from a billion-element array and conditional local replacement.
Pinned rustc accepts every frozen input. The driver builds typed bodies, proves
existing moves/accesses, checks custom-destructor field moves and visits every
statement/bit through the bounded initialization API. Candidate medians are
18.38/68.07/278.13 ms, with median RSS 7,148/16,312/55,488 KiB. At the largest
scale it retains 28,673 paths, 32,769 bits and 12,288 sparse segments while making
1,081,348 queries; all counters grow linearly. This exercises pending analysis,
not executable cleanup or a production CLI destruction mode.

Baseline SHA-256 is
`34e769b8ef0858a3af1ebb4b9e710c8e953570054e0badd890aa3bd8fedbd301`;
candidate SHA-256 is
`162dbe3490723e334ed5a5a1286ea4c4f187388e14e457702979623bb62c98fc`;
the initialization driver's SHA-256 is
`732ce4de27ceb1423c7e77a154d5d9e0a1ae0159d3d13f4830f8f17fa6586ec0`.
`obj/pa15-work/drop-values-v1-perf-results.json` indexes the raw receipts,
including every wall-time, RSS, child CPU-time and context-switch observation.
Source hashes, binary identities, input qualifications and validation receipts
are stored beside it. The driver's dependency versions/checksums match the
maintainer lockfile, and its source is included in the frozen source receipt.

## Operand-consumption checkpoint validation

All 3,094 PA1–PA16 course cases pass. The existing 3,093 cases retain their
byte-exact output; only the new PA16 constant-repeat regression adds oracle
sidecars. Its expected success was declared and checked by rustc before reference
generation. All 113 PA16 native cases execute successfully through both rustc
and the qualified CPPGM backend, including this regression. The preceding frozen
RustGM binary rejects that source with an incorrect Copy requirement.

Debug and release frontend suites each pass 489 tests; all six explicitly
selected maintainer native tests pass. Five new tests cover 22 independently
rustc-qualified sources, plus the ordinary/retained path distinction. They check
stores, calls, aggregates, constructors, branch joins, loops, returns, lifetime
adjustments and the three repeat dispositions. Adjusted constant paths retain
constant mode; a block forwarding the same constant remains an ordinary operand.

Formatting, Clippy all/pedantic with warnings denied, all 25 harness tests and
upstream verification pass. The file/architecture audit reports 326 files and
zero findings; no lint exceptions were introduced. Final logs under
`obj/pa15-work/` are `drop-consumption-full2.log`,
`drop-consumption-release2.log`, `drop-consumption-native2.log` and
`drop-consumption-pa16-native1.log`. The validation receipt hashes each log.
Earlier logs retain the first implementation's checks and the lifetime-repeat
probe that exposed the mode-selection defect. Final checks reran after that fix.

PA17 remains planned: these tests prove retained operand consumption and the
PA16 regression, not completed cleanup, destructor liveness or runtime Drop.

Performance measurements retain 162 observations on CPU 2, with A/A calibration
and three ABBA blocks per workload/scale. All 334 source/configuration hashes,
binary identities and frozen input hashes were verified afterward. Full views
and count bytes match between the old/new ownership binaries; both analysis
workloads use two identical copies of the new driver because their retained
facts have changed. The driver invokes the frontend library directly, including
custom-destructor field-move checks and bounded state queries. It does not run
rustc or CPPGM to implement any phase.

| Ownership pipeline | Baseline median | Candidate median | Candidate median RSS |
| ---: | ---: | ---: | ---: |
| 4,000 functions | 335.94 ms | 337.68 ms | 63,364 KiB |
| 16,000 functions | 1,366.98 ms | 1,363.09 ms | 241,456 KiB |
| 64,000 functions | 5,731.68 ms | 5,750.14 ms | 951,150 KiB |

Median changes range from -0.28% to +0.52%; the largest input changes +0.32%
wall / +0.36% child CPU time, with 260 KiB additional median RSS. Its candidate
range is 5,562–7,574 ms, including the retained high outlier; baseline is
5,708–5,777 ms. No observations were discarded. This agent ran no builds or tests
during timing, but unrelated builds continued on the shared host. These are
shared-host observations, not an isolated hardware-counter gate.

The existing initialization workload at 256/1,024/4,096 functions now retains
the owned operand paths and consumption effects. Candidate medians are
21.19/77.05/314.22 ms, with median RSS 7,152/16,314/55,524 KiB. At its largest
scale it retains 16,384 effects, 45,057 paths, 49,153 bits and 12,288 sparse array
segments, and makes 1,622,020 state queries.

The added workload combines conditional first/last moves from billion-element
arrays, local replacement, calls, tuples, ordinary and constant zero-length
repeats, and branch returns. Pinned rustc accepts all frozen sources. At
256/1,024/4,096 functions, candidate medians are 39.49/153.96/651.07 ms and median
RSS is 9,266/26,164/92,550 KiB. The largest input retains 57,346 effects (`14N + 2`),
102,404 paths (`25N + 4`) and 106,500 bits (`26N + 4`), making 6,602,766 queries
(`1612N + 14`). Its 4,096 disposals and 4,096 constant instantiations remain
distinct. These counters grow linearly without expanding array elements.

Baseline SHA-256 is
`162dbe3490723e334ed5a5a1286ea4c4f187388e14e457702979623bb62c98fc`;
candidate SHA-256 is
`1a733dfe5239a122d3d79464c04dcca31aa9980a81026be4d6e4349b81f47ef9`;
driver SHA-256 is
`a3fb13c016935f71bffdec3134ac5840577c916b140ccb45b264b812033323b3`.
`obj/pa15-work/drop-consumption-v1-perf-results.json` indexes all raw receipts,
including wall time, RSS, child CPU time and context switches. Matching source,
identity and validation receipts are stored beside it. The frozen driver's
dependency versions and checksums match the maintainer lockfile.

## Scope-aware initialization

Initialization now retains the existing validated storage analysis and shares its
ordered events. The cleanup index stores each local or tracked temporary once,
in reverse creation/declaration order within its scope. Locals and extended
temporaries share this ordering. Scope exits walk the existing ancestor ranges;
they do not retain a copy of every live owner. Index memory is capped at 64 MB
and index work at 32 million steps; transfers and replay use the existing bounded
word accounting and also charge scope/member dispatch.

The fixed point and checked replay consume the same return/storage timeline.
Each outgoing edge starts from the unmodified block result before applying that
edge's exits. Scope and let-else initializer cleanup clear owned initialization;
explicit local storage ends subsequently observe those values as uninitialized.
Loop entry cannot retain a discarded temporary from an earlier iteration.
Borrow-only materialization may start a place after another scope's endpoint at
the same point; it cannot revive an already-owned, consumed value. Unused unit
and empty-pattern places need no invented ownership path.

`initialization::Facts::visit_events` exposes cleanup candidates together with
ordinary statement points. Each candidate identifies its place, cause, location
and optional owned path, before clearing or replacing that value. Never state
suppresses a runtime drop; Conditional aggregates still require field/segment
selection. Borrowed and dynamically indexed replacement destinations have no
owned path: the containing reference/array is not the replaced value.

Scope-aware checking exposed a body-builder defect in a returned lifetime
coercion. A block's tail value retained the tail scope while the newly constrained
value inherited the block scope. The tail could therefore end before the
constraint consumed its input. Transparent forwarding now extends the input's
owned scope before applying the constraint, as well as forwarding the result.
The existing adjusted-operand regression now runs through scope-aware checking.

Seven new integration tests retain independently declared rustc destructor
traces for thirteen programs and qualify eight more sources for acceptance.
They inspect interleaved extended temporaries/locals, parameter captures and
remaining fields, partial temporary moves, wildcard disposal, replacement,
zero-length repeat disposal, returns, let-else, loops, guard views, empty patterns
and borrowed/dynamic-index replacement. The edge-isolation test moves a
let-chain failure block's initial cleanup to its unique incoming edge, an
equivalent CFG, and checks both successor entry states. This exercises a live
owner on an edge rather than claiming a candidate with Never state proves it.

The rustc ordering sources use a println observer. The frontend variants remove
only that observer and inspect typed facts; they are not native RustGM Drop
execution comparisons. All executable pending-drop guards remain. Next work
must select field/array glue and conditional flags, add destructor-induced
region/loan uses, retain the completed plan in the checked program, and lower
actual cleanup calls before PA17 can be released.

Validation passes all 3,094 PA1–PA16 course cases without oracle changes, 496
frontend tests in each of debug and release, and all six explicit maintainer
native tests through the qualified CPPGM backend. Formatting, Clippy all/pedantic
with warnings denied, all 25 harness tests and the architecture/file audit pass;
the audit reports 328 files and zero findings. No lint exceptions were added.
`obj/pa15-work/scope-cleanup-v1-validation.json` hashes the full, release, native,
audit and focused ordering logs. Earlier failing logs retain the coercion and
unused-pattern defects found while integrating scope-aware checking.

Performance evidence retains 162 observations on CPU 2: A/A calibration and
three ABBA blocks for each of nine workloads/scales. All 335 frozen source and
configuration hashes, binary identities and input hashes were verified afterward.
The unchanged ownership pipeline compares old/new binaries with matching full
and count output. The two initialization workloads compare identical copies of
the new analysis driver because ordered cleanup changes the exposed states.
No own builds or tests ran during timing; unrelated work continued on the shared
host, so these observations are not an isolated hardware-counter gate.

| Ownership pipeline | Baseline median | Candidate median | Candidate median RSS |
| ---: | ---: | ---: | ---: |
| 4,000 functions | 347.50 ms | 348.15 ms | 63,342 KiB |
| 16,000 functions | 1,394.63 ms | 1,388.18 ms | 241,192 KiB |
| 64,000 functions | 5,645.04 ms | 5,680.25 ms | 951,054 KiB |

Median changes range from -0.46% to +0.62%; the largest input changes +0.62%
wall / +0.61% child CPU time, with 150 KiB less median RSS. Its baseline range is
5,167–5,767 ms and candidate range 5,367–5,791 ms. No observations were discarded;
these measurements do not establish a material throughput regression.

The owned-consumption workload at 256/1,024/4,096 functions now replays cleanup
candidates as well as every statement bit. Candidate medians are
38.12/140.19/672.98 ms, with median RSS 9,296/26,178/93,034 KiB. At the largest
scale it exposes 176,134 candidates (`43N + 6`), retaining the same 102,404 paths,
106,500 bits and 12,288 sparse array segments as before.

The new scope workload combines interleaved extended temporaries and locals,
conditional moves from billion-element arrays, loops with continue/break,
replacement, let-else failure and returned values. Pinned rustc accepts all
frozen sources. Candidate medians at 256/1,024/4,096 functions are
70.90/292.77/1,082.17 ms, with median RSS 13,496/43,130/162,622 KiB. The largest
input exposes 286,733 candidates (`70N + 13`), 4,096 edge candidates, 122,889
paths (`30N + 9`), 126,985 bits (`31N + 9`) and 12,288 array segments. It makes
14,983,212 statement-bit queries (`3658N + 44`) plus candidate-path queries.
All counters grow linearly without enumerating array elements. Candidate counts
include Never state and are not runtime destructor counts.

Baseline SHA-256 is
`1a733dfe5239a122d3d79464c04dcca31aa9980a81026be4d6e4349b81f47ef9`;
candidate SHA-256 is
`6374bc948a8c5bc046a3f6a4da42ba1e766decad356cca409d9b79be251efe12`;
the analysis driver is
`de083804e94c719c5fe666b9d58781e9e9ebb4a835368015c58377c40d2ea300`.
`obj/pa15-work/scope-cleanup-v1-perf-results.json` indexes every raw receipt,
including wall time, RSS, child CPU time and context switches. Its versioned
driver project preserves the previous driver's source and uses the maintainer's
locked dependency versions/checksums. It invokes the frontend library directly;
rustc and CPPGM do not implement any frontend phase.

## Shared recipes and ordered conditional flag writes

The typed-body layer now owns `body::drops::glue::Catalog`: an iterative,
memoized recipe query over completed types, members and selected methods.
A custom destructor precedes its fields; struct/tuple fields and active enum
variant fields retain declaration order. Arrays retain one element recipe and
a length, including zero-sized element types, rather than expanding elements.
References and empty arrays require no destruction. Nominal lifetime
instantiations share recipes; structural query keys remain canonical type IDs.
The query diagnoses cycles, inconsistent field facts and work/storage limits.

`borrow::destruction::select` requires the custom-destructor field-move rule,
then selects operations from retained initialization. Whole initialized values
use shared glue. Partially moved products select fields, enums dispatch before
accessing variant fields, and arrays reuse the sparse physical segments from
ownership. Gap conditions sit outside element loops. Never state suppresses
runtime operations. Original cleanup types remain available for destructor
borrow checking even after every runtime field call has been pruned: moving all
fields individually does not have the same lifetime effect as moving the whole
aggregate.

Initialization replay additionally exposes the exact ordered writes used by the
proof. Ordinary statement transfers share one mutation interface with the
fixed-point solver; scope ends, materialized births and operand/return consumption
use the same observed write primitive. Existing point/candidate inspection APIs
preserve their order and answers. The destruction plan retains statement markers,
cleanup identities and only the flag writes required by selected conditional
operations. Flags start false. Each write selects a half-open interval in the
sorted sparse flag table using binary search; it never enumerates array elements.
Edges replay independently from their predecessor's outgoing state.

Nine new maintainer tests cover shared nominal lifetime recipes, method/field
ordering, enum dispatch, zero-length and billion-element arrays, 20,000 nested
shared types, memoized cycle rejection, conditional whole/field drops, array
subfields, required custom-destructor move rejection and the difference between
runtime pruning and lifetime obligations. A separate test-only flag interpreter
executes only the plan's emitted stores over the CFG. Its joins are compared with
retained initialization at every statement/cleanup observation for eight
independently rustc-qualified branching, loop, return and replacement programs.
It does not reconstruct missing writes from body statements or move paths.

All 3,094 PA1–PA16 course cases pass with unchanged text oracles. Debug and release
each pass 505 frontend tests with six explicitly backend-dependent tests ignored;
all six pass in `make test-maintainer-native`. Formatting, Clippy all/pedantic with
warnings denied, the 335-file architecture/file audit, all 25 harness tests and
upstream provenance checks pass. No lint exceptions were introduced. The initial
audit caught recipe construction importing semantic members directly into borrow;
construction now belongs to the typed-body layer and borrowing consumes its typed
interface. `obj/pa15-work/drop-plan-v1-validation.json` hashes the validation logs.

Four additional pinned-rustc probes in
`obj/pa15-work/drop-region-selection-probes-v1/qualification.json` establish the
next borrow-checking requirement. For a noncustom wrapper containing a Drop field
and an unrelated plain reference, only the Drop field keeps its borrowed region
live at destruction. A custom wrapper requires both. Moving the Drop field alone
still leaves the wrapper's drop-check obligation. Consequently, blindly using
all free regions of a cleanup type, or only regions of emitted runtime calls,
would each be incorrect.

This remains an intermediate typed plan. Pending-drop proof and execution guards
are still in place. PA17 requires a query for Drop-relevant lifetime components,
ordered destructor/storage points in liveness and loan checking, retention of the
proved plan at the executable boundary, actual LowIR destructor/glue calls, native
comparisons and the released handout/manifests/oracles. In particular, existing
statement-level loan points cannot distinguish a destructor using an extended
temporary from the temporary's later endpoint at the same statement position.

Performance evidence uses 342 frozen source/configuration hashes, separately
hashed versioned driver sources, immutable binaries, frozen inputs and CPU 2.
A/A calibration plus three ABBA blocks at each of nine workload/scale pairs retain
162 observations. Ordinary ownership and the existing initialization inspection
compare old/new binaries with identical full and count output. The new planning
workload compares identical candidate copies; it is scaling evidence, not a
before/after speedup. No own builds/tests ran during timing; unrelated activity
continued on the shared host. Every observation is retained.

| Workload / size | Baseline median | Candidate median | Candidate RSS |
| --- | ---: | ---: | ---: |
| Ownership / 4,000 functions | 329.46 ms | 342.67 ms | 63,360 KiB |
| Ownership / 16,000 functions | 1,390.41 ms | 1,395.28 ms | 241,442 KiB |
| Ownership / 64,000 functions | 5,701.17 ms | 5,689.79 ms | 951,000 KiB |
| Initialization / 256 functions | 65.60 ms | 65.26 ms | 13,488 KiB |
| Initialization / 1,024 functions | 298.34 ms | 299.51 ms | 43,084 KiB |
| Initialization / 4,096 functions | 1,213.93 ms | 1,237.07 ms | 163,170 KiB |
| Planning / 256 functions | same candidate | 75.82 ms | 14,168 KiB |
| Planning / 1,024 functions | same candidate | 300.69 ms | 43,140 KiB |
| Planning / 4,096 functions | same candidate | 1,187.59 ms | 163,348 KiB |

Ownership medians change +4.01% / +0.35% / -0.20%; the smallest A/A calibration
ranges 336.62–609.59 ms, so its small regression does not establish a stable cost.
At the largest size, CPU time changes -0.21% and median RSS decreases 148 KiB.
Retained initialization changes -0.51% / +0.39% / +1.91%; the largest CPU change
is +1.89% with 270 KiB more median RSS. This small observed replay cost is retained
rather than described as a speedup or hidden by dropping samples.

At 4,096 functions, planning retains 286,733 cleanup candidates, 94,208 nodes,
24,576 array intervals, 12,288 flags and 1,261,615 ordered events. These counts
follow `70N+13`, `23N`, `6N`, `3N`, and `308N+47`, respectively. Planning work is
`1255N+121` steps. The recipe query retains `N+3` recipes and takes `9N+25` steps;
structural tuple types with distinct reference regions still have distinct query
keys. No measured operation enumerates the billion-element array. Counts describe
typed planning, not executed destructor calls.

`obj/pa15-work/drop-plan-v1-perf-results.json` indexes every raw receipt; all
frozen source, driver, binary and input hashes were checked after measurement.
The ordinary candidate SHA-256 is
`44c7a8231c36ebcc34bbd68ce2f9e298d1a0b50e5b2595d1a8854d271cf8fa0c`;
the initialization driver is
`147fbf3275729804dd7bf7cebf7c0c5ccb43aa28a0c21aa7ff8f552df920af08`;
the planning driver is
`4b1bfa40eaf8eff02e77faf496f174be6483228f47524a0550b5b64e020706db`.


## Ordered destructor lifetimes and loan checking

A pending destruction plan now supports a separate lifetime/loan analysis.
`types::destruction::regions::Regions` selects the actual non-static lifetime
arguments needed by custom destructors and recursively owned fields. Nominal
field templates are cached in bound-slot form and substituted at each use;
references do not own their referents. Iterative query work, canonical result
slices and memoized failures bound deep and repeated requests. The runtime glue
representative never supplies lifetime arguments for this query.

Initialization replay now exposes individual storage births and endpoints,
including each scope in an exit range and each initializer temporary. The plan
also retains sparse deinitialization intervals independently of whether a
runtime flag exists. One ordered point index supplies block/edge lanes to both
backward liveness and forward loans. Drop entities remain distinct from ordinary
use entities. Whole moves and defining stores kill the corresponding Drop use;
field moves preserve the containing aggregate's obligation. Destructor access
checks and temporary storage checks occur in their actual cleanup order.

The lifetime proof for replacement exposed an operand/consumer distinction.
A coercion temporary previously seeded the destination binding's region before
the store, keeping the old borrow alive across replacement. The pending-drop
analysis retains canonical operand requirements through coercions, aggregate
construction and repetition, while preserving separate definition/liveness bits.
Destination locals retain their own type requirements after the store. This
permits replacing a whole value and then using its old referent, without losing
loans held by copied aliases, moved owners or the new value. Partial-field
replacement still retains the parent's obligation. The pinned rustc NLL dump
for the minimal replacement probe independently identifies the old borrow's gap
at the defining store; the dump and diagnostic driver are retained under
`obj/pa15-work/destructor-diagnostic-v1/`.

This follows the distinction between ordinary use requirements and destructor
requirements described in the [Rust compiler development guide](https://rustc-dev-guide.rust-lang.org/borrow-check/drop-check.html).
Behavioral decisions are checked against the repository's Rust 1.98.0 compiler
and pinned Reference. In particular, tuple-struct constructor arguments can
receive temporary lifetime extension in this profile. An ordinary function call
returning the same type remains the negative control; the local Reference's
`destructors.md` documents that distinction.

Eight new maintainer tests add query invariants, 49 independently classified
loan/temporary/counter programs, and an explicit conditional flag clear on one
CFG edge. Replacement controls include old-owner moves, live and ended shared
aliases, borrowing the slot, both/one branch replacement, loop exits, reborrowing
the old field, indirect calls, projected stores and whole-tuple replacement.
The flag interpreter still executes only emitted flag stores, and checks every
observed state against retained initialization; the other successor remains
conditional after the cleanup edge clears its own flag.

Validation for this checkpoint:

- All 3,094 PA1–PA16 text cases pass with unchanged committed oracles.
- All 513 frontend tests pass in debug and release; six backend-dependent tests
  remain explicitly ignored there and all six pass via `make test-maintainer-native`.
- Clippy all/pedantic, rustfmt and the architecture/file audit pass: 342 files,
  zero findings. All 25 harness tests and pinned upstream/Reference checks pass.
- The largest new scaling input is independently accepted by Rust 1.98.0.
- Logs are `destructor-liveness-full1.log`, `-release1.log`, `-native1.log`,
  `-audits5.log`, and `-focused8.log` under `obj/pa15-work/`.

These APIs deliberately retain the pending-drop execution boundary. PA17 still
needs the completed plan attached to checked bodies, selected destructor calls
and cleanup flags lowered to LowIR, native counter/order comparisons, and the
released handout, manifests and text oracles. The new operand-lifetime table is
currently used by the ordered destruction analysis; integrating it with the
published ordinary-loan path needs separate PA16 regression/oracle review.

Performance uses immutable binaries, frozen inputs, CPU 2, two warmups, four
A/A calibration observations and three paired ABBA blocks per workload/size.
Every observation is retained (162 total); full rendered output and count output
match across old/new ordinary-loan and initialization binaries. New destructor
proof measurements compare two copies of the same binary for scaling, not a
before/after speedup. No own builds or tests ran during timing; unrelated work
continued on the shared host.

| Workload / functions | Baseline median | Candidate median | Candidate RSS |
| --- | ---: | ---: | ---: |
| ordinary-loans / 256 | 30.23 ms | 30.41 ms | 9,276 KiB |
| ordinary-loans / 1,024 | 102.06 ms | 106.93 ms | 24,766 KiB |
| ordinary-loans / 4,096 | 396.29 ms | 405.76 ms | 85,634 KiB |
| initialization / 256 | 58.51 ms | 58.66 ms | 13,496 KiB |
| initialization / 1,024 | 300.91 ms | 300.89 ms | 43,098 KiB |
| initialization / 4,096 | 1169.01 ms | 1143.49 ms | 163,176 KiB |
| destructor-proof / 256 | same candidate | 133.76 ms | 16,298 KiB |
| destructor-proof / 1,024 | same candidate | 484.94 ms | 51,094 KiB |
| destructor-proof / 4,096 | same candidate | 2309.54 ms | 190,210 KiB |

The largest ordinary-loan workload changes +2.39% wall time and +2.40% CPU,
with 740 KiB more median RSS. The paired wall ranges overlap, but this small
observed cost is recorded rather than called a speedup or hidden. Initialization
changes -2.18% at the largest size, also with overlapping ranges. The new proof
workload includes nested whole-value replacement, conditional initialization,
loop/continue/break cleanup and extended borrowed temporaries.

At 4,096 generated functions, the new pass checks 32,768 source loans and
1,351,794 ordered points. Retained region words are 131,083; summed proof work is
39,736,942 steps. These follow `8N`, `330N+114`, `32N+11` and `9701N+1646`.
Drop-component query work is `33N+2`, with cached template substitution.
Planning retains `37N+13` cleanup candidates and `N` flags. Counts describe the
staged proof, not generated or executed native destructors.

`obj/pa15-work/destructor-liveness-v1-perf-results.json` indexes the raw receipts;
source, driver, binary and input hashes were verified after measurement. The
ordinary candidate SHA-256 is
`96e18eb4577eb6850566d22292622bf4cbc927878ee03772af8f8bb1261e372d`;
the initialization driver is
`c4bae6ea300101a892a645702819b2edcffffe3a6bebbd4ea3826edb265b3547`;
the destructor proof driver is
`56c864120b14c828c2414e75615e023144030cef437eb177af986897f90978a2`.


## Native cleanup lowering from completed destruction proofs

The staged `borrow::verify_destruction` entry point now combines initialization,
selected cleanup, lifetime contracts and ordered loan checking into a `Checked`
value. It owns immutable recipe arenas and selected plans in body order. The
ordinary PA14–PA16 entry points retain their existing pending-drop boundary;
the CLI is not yet switched to the staged verifier. Bodies with no runtime
cleanup discard their event tape after proof and use ordinary lowering.

LowIR consumes the same ordered events used by the proof. Statement evaluation,
old-value cleanup and conditional flag writes remain in their selected order.
Real edges with cleanup receive deterministic extra blocks; imaginary proof
edges do not become runtime branches. Each demanded glue recipe has one stable
`rg.drop.ID` identity and is emitted through the existing demand queue, which
also demands custom destructor bodies. Field/variant traversal is iterative;
array intervals produce counted loops rather than unrolled element calls.
Custom `Drop` runs before automatic field cleanup. Tag dispatch visits only the
active enum variant. Packed fields move into aligned backing for glue calls,
and zero-sized receivers receive stable aligned addresses.

`native_destructors.rs` checks 24 classified programs against pinned rustc
acceptance, then lowers every accepted program through the staged library API.
Its explicit native suite executes 22 accepted programs through both rustc and
CPPGM, with counters and source assertions checking replacement, moves, partial
arrays/tuples/enums, conditional initialization, loops, return, let-else,
zero-length repeats, packed fields, custom-before-field destruction and extended
borrowed owner ordering. Both partial-move live-borrow negative controls remain
rejected. The edge case relocates an equivalent return cleanup onto its incoming
CFG edge before proof; runtime checks cover both successors and initialization
states. Repeated lowering must produce identical text.

A separate default test compares an aligned zero-sized destructor array of
length eight with length one billion. Both pass pinned-rustc acceptance, retain
aligned backing, and produce the same instruction count with under 12 KB of
LowIR. This is a compile-time bound, not a claim that executing one billion
destructor calls is constant time. Field traversal order is retained in typed
recipes; broader observable sibling-order tests remain part of PA17 release
qualification. No timeout or unavailable backend counts as a passed execution.

PA17 remains planned until the CLI, assignment handout, manifests and independently
qualified text/native fixtures are published together. This checkpoint provides
the executable library path required for that release.


Validation for native cleanup lowering:

- All 3,094 PA1–PA16 text cases pass with unchanged committed oracles.
- All 515 frontend tests pass in debug and release. The seven explicitly
  backend-dependent tests pass through `make test-maintainer-native`.
- Clippy all/pedantic, rustfmt and the file/architecture audit pass: 349 files,
  zero findings. All 25 harness tests, 24 upstream fixture checks and 126 pinned
  Reference file checks pass.
- The largest scaling input passes independent Rust 1.98.0 acceptance. The
  256-function input also validates, compiles and executes through CPPGM.
- Logs and their hashes are indexed in
  `obj/pa15-work/drop-native-v1-validation.json`.

Performance uses immutable binaries and frozen inputs, CPU 2, two warmups,
four A/A calibration samples and three paired ABBA blocks per workload/size.
All 162 observations are retained. No own builds or tests ran during timing.
The shared host continued running unrelated work. Old/new ordinary LowIR output
matches byte-for-byte, as do counts; destructor lowering compares identical
candidate copies for scaling because the baseline had no executable cleanup.
The staged driver uses the exact dependency versions in the course lockfile.

| Workload / functions | Baseline median | Candidate median | Candidate RSS |
| --- | ---: | ---: | ---: |
| ordinary thin references / 256 | 30.48 ms | 30.89 ms | 9,220 KiB |
| ordinary thin references / 1,024 | 116.82 ms | 116.32 ms | 22,638 KiB |
| ordinary thin references / 4,096 | 488.52 ms | 490.60 ms | 76,362 KiB |
| ordinary wide references / 256 | 54.32 ms | 54.09 ms | 12,058 KiB |
| ordinary wide references / 1,024 | 206.80 ms | 204.90 ms | 34,418 KiB |
| ordinary wide references / 4,096 | 877.35 ms | 854.18 ms | 124,512 KiB |
| destructor lowering / 256 | same candidate | 162.93 ms | 22,636 KiB |
| destructor lowering / 1,024 | same candidate | 608.66 ms | 78,278 KiB |
| destructor lowering / 4,096 | same candidate | 2596.68 ms | 299,208 KiB |

At 4,096 ordinary functions, thin-reference wall time changes +0.42% and
wide-reference time -2.64%; paired ranges overlap in both cases. Median RSS
changes by 48 KiB and 66 KiB respectively. These observations show no material
regression in these workloads, rather than proving a universal performance
bound. The same-binary destructor measurements also retain scheduling outliers.

Destructor output scales as `3N+9` functions, `18N+11` blocks, `90N+31`
instructions, `18N+7` slots and `2N` globals. At 4,096 functions this is 368,671
instructions and 299,208 KiB median peak RSS. Nominal recipes erase lifetimes
and share code; structural tuples/arrays currently share by canonical type ID,
so lifetime-distinct structural types can still demand separate glue. Further
sharing must preserve layout distinctions and deserves a separate measured
change before type-generic destruction.

`obj/pa15-work/drop-native-v1-perf-results.json` indexes every raw receipt.
Frozen source, driver, binary and input hashes were checked after measurement.
The ordinary candidate SHA-256 is
`4c1503906b98310473546971658c11c8216d28ee1cecbb86f8dd8342566e81de`;
the staged lowering driver is
`bb3e6c85227602d59e4a2daf74eba96904f7e1f08b1111270737948873438181`.
