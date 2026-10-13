# PA17 destruction buildout

PA17 remains planned and incomplete. PA15 and PA16 were released at `1651a5e`;
this journal records the implementation that precedes the PA17 grading contract.
No destruction tests or completed-destructor execution are claimed here.

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

Until typed destructor checking and elaboration are available, semantic analysis
rejects references to the built-in trait. In particular, it cannot accept an empty
or explicitly typed `drop` method as an ordinary function and silently erase its
destruction obligation. This temporary boundary must be replaced as PA17 proceeds.

## Remaining implementation

Retain validated destructor methods, type `Self` and receivers with lifetime-only
impl binders, and reject invalid or overlapping Drop implementations. Feed actual
destructor uses into lifetime and loan checks. Preserve conditional initialization
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
