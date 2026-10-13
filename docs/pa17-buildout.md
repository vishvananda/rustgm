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

Feed actual destructor uses into lifetime and loan checks. Preserve conditional initialization
facts independently of immutable-write history. Elaborate drop scopes, replacement,
partial moves, temporaries and early exits into typed plans consumed by LowIR.

Ownership's existing `Word.may` records whether storage has ever been written;
it is not a may-be-initialized drop flag. Keep the new initialization transfer
facts separate, including residual fields after moves and branch/loop joins.
Reject non-Copy field moves out of a type with a custom destructor. Replacement
must evaluate the incoming value, destroy the initialized old destination and
then store the replacement.

Retain creation and cleanup scopes for all owned rvalues, including discarded
values that never become materialized temporary places. Combine overlapping
candidate sites into one ordered plan: locals and temporaries reverse creation
order; custom destructors before fields; fields and array elements in their
specified forward order. Handle early exits and failed guard attempts explicitly.
Seed destructor accesses into region and loan liveness before issuing the checked
token, and retain the proved cleanup plan for lowering. Do not release the current
pending-proof guards until these obligations are implemented and tested.

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
