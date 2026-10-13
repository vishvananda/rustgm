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
construction, which still rejects outstanding destructor elaboration.

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
views. Typed body construction rejects these impls until cleanup liveness and
elaboration exist. Therefore body, ownership, region, loan and LowIR modes cannot
silently emit code or claim a proof that omits destructor effects.

The header matrix declares 73 source outcomes, independently compiled by pinned
rustc in the maintainer suite. It covers lifetimes, aliases, method conformance,
receiver forms, body errors and nominal coverage; import/prelude/macro identity
tests now also check typed destructor selection. A CLI boundary test checks type
inspection and all later proof/execution modes. These remain implementation
tests rather than a released PA17 assignment contract.

## Remaining implementation

Feed actual destructor uses into lifetime and loan checks. Preserve conditional initialization
facts independently of immutable-write history. Elaborate drop scopes, replacement,
partial moves, temporaries and early exits into typed plans consumed by LowIR.

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
