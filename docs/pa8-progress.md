# PA8 maintainer buildout notes

PA8 remains planned. These are implementation notes, not a released grading
contract. PA1–PA7 remain the active assignments; `--emit-expanded` keeps PA7's
one-step output. PA8 will expose `--emit-expanded-crate` after its complete
contract and independently checked fixtures are ready.

## Implemented groundwork

- Syntax transfer copies typed arenas with fresh occurrence IDs and mapped
  source spans. It restores destination arenas on transfer failure.
- Expansion contexts are retained on source macro nodes. The generated parser
  consumes complete expression, statement, item, type and pattern inputs.
- Opaque captures enter the ordinary parser as typed nonterminals. Metadata
  and visibility also have typed consumers. Captured expression precedence is
  preserved without lexing its spelling again.
- A persistent capture registry moves demanded payloads out of a successful
  match. Repeated substitutions and forwarded tokens retain one dense handle;
  generated macro forests carry those handles through subsequent invocations.
- Parser source and interpolation-provider lifetimes are separate. Returned
  syntax borrows its generated spelling, not a temporary registry view.
- Configuration accepts existing parsed syntax. Installation appends a semantic
  spelling view while preserving original physical bytes, and transfers active
  flags, attribute owners, effects, source ranges and decoded literal ownership.
- The crate expansion worklist expands textual macro calls, visits generated
  definitions, restores block/module textual scopes, loads generated file
  modules using the retained loader, and bounds recursion. Disabled generated
  syntax does not demand literals, macro calls or files.
- Reused physical files receive independent expansion instances. A sparse
  overwrite journal and original arena extents preserve the parsed source;
  only another requested instance copies its original typed facts and decoded
  pool prefix. Configuration flags and CRLF physical bytes survive this copy.

## Current evidence

`dev/tests/crate_expansion.rs` exercises the complete worklist and feeds several
expanded crates into PA6 name resolution. Other new invariant suites cover
syntax transfer, installation rollback, opaque syntax, streams, forwarding and
registry lifetime/identity behavior.

Pinned rustc probes are retained under `obj/pa8-work/forwarding-probes/` and
`metadata-probes/`. The forwarding test contains the independently observed
15-by-15 fragment compatibility matrix. The `stmt` sample excludes its trailing
semicolon: the first probe with a semicolon was a rejected *outer invocation*,
not evidence about forwarding. Successful matching does not imply that a
forwarded fragment is legal in every eventual syntax/semantic context.

`obj/pa8-work/engine-regression.log` records all 1,006 earlier fixture checks,
the Rust invariants and 14 harness regressions. `engine-build.log` records
strict Clippy and the 92-file audit with no findings at this checkpoint.
No earlier reference sidecars were changed or blessed.
`instance-validation.log` supersedes that checkpoint with 93 audited files,
all earlier suites and the repeated-physical-module regression passing.

Performance investigation found that a `u32` payload in lexical `Kind` enlarged
every token and slowed unrelated source parsing. Expansion identities now use a
checked 24-bit representation; normal tokens remain twelve bytes. The registry
and defining-crate token constructors diagnose overflow rather than truncating.
Ordinary parsers also skip interpolation-provider lookups when they have no
opaque input. `dev/tests/interpolation.rs` checks representation and boundary
values; the lexer storage invariant is tightened to twelve bytes.

Retained immutable-binary measurements against the PA7 baseline include:

- Initial PA7 macro regression: `obj/perf/1791777667158323702/samples.json`.
- Initial ordinary parser regression: `obj/perf/1791777725361648092/samples.json`.
- First lookup guard: `obj/perf/1791777780044638434/samples.json`.
- Provider-demand guard: `obj/perf/1791777838194961727/samples.json`.
- Compact token identity: `obj/perf/1791778041363098466/samples.json`.
- Compact identity with PA7 matching: `obj/perf/1791778093198763079/samples.json`.

The last ordinary-parser run retains equal complete outputs and 3 ABBA blocks
after A/A calibration at each scale. At 64,000 items, A/B medians were
370.06/386.74 ms; A/A observations ranged 378.24–419.13 ms. This is evidence of
substantial improvement over the initial change, not a universal no-regression
claim. Full PA8 expansion throughput still needs its own measurement contract.
The compact-identity PA7 run's 64,000-call A/B medians were 338.39/326.85 ms,
with equal full output and candidate RSS back to 108,572 KiB peak. Retain the
individual observations when drawing conclusions.

## Work still required before activation

1. Complete final-context coverage for composed opaque fragments and macro
   hygiene. Finish expansion-time scope ambiguity checks, including outer/glob
   candidates and ordinary import completion after non-macro declarations.
2. Implement builtin macros and projected definition/call-site diagnostics.
   Extend the `$crate`/`local_inner_macros` and recursion-limit coverage below
   as those builtin paths are introduced.
3. Measure the full expansion path, finalize the PA8 handout/text vocabulary,
   add independently classified course fixtures, refresh receipts explicitly,
   validate debug/release and CI, then activate the milestone.
4. Continue through PA9 types/inference, PA10 typed body CFG, and PA11 actual
   Rust-to-LowIR execution using the pinned CPPGM backend.

The broad goal is still active. A backend smoke test on handwritten LowIR does
not establish Rust lowering, and neither PA8 nor the goal is complete yet.

## Completed early resolution integration

Source registration and newly generated syntax now feed persistent textual
scope links. Definitions link to predecessors; item and statement invocations
introduce output placeholders. Lookup stops at an unresolved predecessor unless
a nearer definition determines the name. Cached pending lookups retain their
actual dependency. Completion wakes only subscribers, without rescanning the
crate. The 10,000-link invariant bounds observed lookup work below 40,000 steps.

The early module/macro graph shares PA6's import dependency and glob-delta
engine. It supports qualified calls, aliases, module aliases, crate aliases,
reexports, privacy, macro_use modules and generated imports/exports. Plain
macro_rules definitions remain textual until explicitly imported or exported.
Generated binding ambiguity uses expansion provenance; successful calls retain
their chosen target and are checked again if a path dependency changes. Path
watchers are deduplicated and wake in deterministic order. Provisional lookup
errors can clear when stronger dependencies resolve.

Shared resolver changes preserve PA6's ordinary mode. Optional selected-path
traces and generated-binding conflict checks are enabled only for the expansion
import graph. The expansion report retains its macro path resolution facts.
The later mixed-site local/label implementation is described below.

`obj/pa8-work/resolution-probes/` retains pinned-rustc inputs, diagnostics and
classifications. In particular, a generated explicit import that conflicts with
a glob is ambiguous even inside that same expansion; a source explicit import
wins, and two bindings to the same target remain unambiguous. Tests cover both
orderings and an import that changes a previously expanded call's binding.

`path-validation.log` records all earlier fixtures and harness checks passing.
`path-lint.log` records strict Clippy and 98 audited files with zero findings.
The immutable-binary PA6 name-resolution comparison is retained in
`obj/perf/1791780190816987707/samples.json`: full output hashes match at all
three scales. At 64,000 items, A/B medians were 837.25/856.82 ms; the four A/A
observations ranged 747.41–847.37 ms. Keep the raw paired observations rather
than claiming a general speedup or no-regression guarantee.

## Opaque grammar consumers

The new consumer probes in `obj/pa8-work/consumer-probes/` distinguish successful
fragment forwarding from valid final syntax. Literal expressions and negated
literals can become patterns; arbitrary expressions, grouped expressions and
constant paths captured as expressions cannot. Captured paths remain paths
when used as patterns and can receive tuple/struct-pattern suffixes. They do not
become new identifier bindings. Opaque endpoints work in range patterns.

Path/type fragments used as attributes lower their already typed path segments
into the attribute forest. No source spelling is lexed or parsed to recover a
path. Identifier spans are exact; separator spans cover their existing source
gaps. Generic and non-path type fragments are rejected in this context. A path
followed by metadata arguments is parsed as one composed metadata fragment;
already captured metadata remains opaque when followed by matcher tokens.
Forwarded path types also work in trait bounds.

`consumer-validation.log` supersedes the preceding validation checkpoint:
123 Rust invariants, all 1,006 earlier fixtures, 14 harness regressions, strict
Clippy and the 98-file audit pass. No earlier oracle sidecars changed.
Full final-context coverage, mixed-site local/label resolution, builtin macros,
recursion-limit attributes and the PA8 text contract are still required.

Next ordinary-resolution work must preserve definition-site *binding versions*,
not just scope IDs: PA6 currently mutates a scope when a local is shadowed.
Macro definitions need snapshots of the visible local/label binding identities;
generated bindings and references need their expansion marks, while captured
syntax keeps its original context. Reuse the typed expansion and module facts
instead of rerunning expansion or reparsing source. Independently check local
shadowing, nested captures, labels and item call-site lookup before activation.


Release-profile Rust tests also pass. `reference-validation.log` records all
independent earlier-stage rustc checks and upstream provenance verification.
The authorized reference regeneration changes only `compiler_source_sha256`
in PA1–PA7 receipts; fixture manifests and oracle sidecar bytes are unchanged.

The ordinary AST comparison against PA7 is retained in
`obj/perf/1791780545141700626/samples.json`; complete output matches. At 64,000
statements A/B medians were 213.42/224.30 ms. A second comparison isolates the
consumer changes against the frozen post-path-integration binary in
`obj/perf/1791780649652558043/samples.json`: medians were 217.76/220.14 ms at
that scale. Lower-scale medians were higher, with substantial calibration
variation. These measurements do not establish a general no-regression claim.
The source files, immutable A/B binaries and every observation remain retained.


## Ordinary resolution and mixed-site locals

`resolve::analyze_expanded` consumes and completes the expansion report's
`resolution` graph. Module and macro IDs, flattened imports and textual import
sources are reused. Existing block/trait scopes are reparented beneath ordinary
function and condition scopes instead of rebuilding the early graph. Plain
macro_rules definitions are not republished as forward-visible path bindings.
A completed report can be passed through this entry point again without
changing identities. Generated item wrappers now visit their expanded bodies.

Hygienic local/parameter/label bindings use canonical name and context IDs.
Each macro definition records its lexical scope and visible binding version.
Local shadow histories retain earlier targets, with binary search for an older
snapshot. A single binding occupies the map entry directly; only shadowed names
allocate a history vector. Identifier/context pairs use one packed 64-bit key,
without widening temporary pattern records. Ordinary PA6 lookup bypasses
expansion provenance queries.

Lookup respects function/closure label barriers and nearer item scopes. Captured
identifiers retain caller identity; generated locals remain separate. Repeated
expansions retain independent local instances. Locals with the same spelling
but distinct contexts can coexist in one pattern or parameter list. Final
references retain canonical declaration targets, so the local-history scratch
can be released after resolution; later phases must consume those references.

`obj/pa8-work/hygiene-probes/` retains pinned rustc compile/runtime evidence for
local shadowing, captures, item precedence, labels, exported scope escapes and
receivers. `dev/tests/hygiene.rs` checks declaration identities, not just success,
and includes 2,000 shadows followed by 2,000 references to an early snapshot.
The macro-path test tables now also run ordinary resolution after expansion.

This work exposed an earlier receiver omission. PA6 now binds an explicit-lifetime
receiver and checks its lifetime, with positive/negative fixtures in
`pa6/tests/spec/names/receiver_*.rs`. The receiver AST retains the actual `self`
identifier span so a captured receiver keeps its own syntax context. Existing
AST text views and all previous oracle bytes remain unchanged.

`hygiene-compact-validation.log` records 133 Rust tests, all 1,008 active course
fixtures and strict Clippy passing; the 100-file audit reports zero findings.
`receiver-validation.log` records the new fixture checks, the 14 harness tests,
and upstream/Reference provenance verification.

CI's oracle-reproducibility pathspec now uses explicit Git globs for nested
fixture files. The earlier quoted `pa*/tests` pattern missed nested changes;
the corrected guard was independently observed returning failure for the
modified PA6 fixture manifest (`obj/pa8-work/oracle-guard-check.log`).


The ordinary-name performance investigation retains the initial regression in
`obj/perf/1791781865584839719/samples.json` and the compact-pattern followup in
`obj/perf/1791782070898545634/samples.json`. The latter 64,000-function A/B
medians were 902.99/975.96 ms. Hardware counters show approximately 0.3% more
retired instructions, with a larger increase in cycles; the cycle profile and
assembly comparison are retained under `obj/pa8-work/hygiene-*`. The changed
binary also has different register allocation/layout in unchanged parser code,
so the timing difference does not establish a lookup-algorithm regression.

A repeat pinned to CPU 0, using the same immutable binaries and input bytes,
is retained in `obj/perf/1791782496455053566/samples.json`. After A/A calibration
and three ABBA blocks, 4,000/16,000/64,000-function A/B medians were
86.18/87.17, 355.47/362.83 and 1404.88/1443.14 ms; full output hashes match.
The 64,000-function A/A observations ranged 1419.38–1438.42 ms. The host load
was variable. These observations narrow the measured difference to roughly
1–3% on that pinned run; they do not prove no regression. Full PA8 throughput
and further performance review remain part of activation.

`hygiene-release-validation.log` records the release-profile Rust invariants.
`hygiene-reference-validation.log` records all independent rustc checks,
14 harness tests, upstream provenance, and regenerated course references.
Only the two new PA6 receiver fixtures add oracle bytes; all earlier sidecars
are unchanged. The PA1–PA7 receipts refresh compiler source fingerprints.


## Introduction ancestry, imports and expansion attributes

Identifier syntax context and expansion introduction are now distinct facts.
A captured path, item, or entire macro call retains its identifier hygiene but
is introduced by the containing expansion. The worklist carries that
introduction through generated modules and syntax. Invocation parent links
record actual expansion ancestry. Sparse definition/import introduction maps
and parent facts remain in the resolution report for subsequent import checks;
ordinary PA6 leaves these tables empty.

Textual lookup now checks conflicting path candidates in inner lexical scopes,
and checks outer candidates when a generated textual definition would shadow
an earlier binding from outside its expansion. Candidates introduced together,
including nested expansions, remain compatible. Both calls and textual import
aliases subscribe to relevant path changes; a later generated import can
invalidate an earlier speculative selection without rescanning unrelated calls.
Pinned Reference guidance is in `names/name-resolution.md`, particularly
`path-vs-textual-macro` and `more-expanded-vs-outer`.

Opaque path/type captures in use declarations consume their parsed path facts.
Groups, globs and aliases are supported; appended identifiers, generic segments
and non-path types are rejected according to the pinned probes. `$crate` in a
use tree produces a typed crate-root segment, with the same placement checks as
ordinary rich paths. No capture is re-lexed to implement these consumers.

`local_inner_macros` qualifies literal unqualified macro names through the
defining crate during macro-path resolution. Captured identifiers, expressions,
token trees and already qualified calls retain their own lookup. Root
`recursion_limit` is read from configured attributes; the first value wins and
the default remains 128. The worklist honors the exact requested depth, with a
201-expansion successful chain tested under a limit of 256. Global work and
provenance bounds remain separate diagnosed resource limits.

Independent compile/runtime probes are retained under
`obj/pa8-work/introduction-probes/`, `local-inner-probes/` and `recursion-probes/`.
They include positive controls for ordinary function imports, generated modules,
macro aliases, captures and same-expansion shadowing. A `$crate` import probe
exposed the inner-path-versus-outer-textual ambiguity; its initial test assumption
was corrected to match rustc, and alias imports provide the positive control.

`ancestry-limits-validation.log` records 142 Rust tests, all 1,008 earlier course
fixtures, strict Clippy, 100 audited files with zero findings and 14 harness
tests. The earlier hygiene commit `e9c226a` passed GitHub CI.

The pinned CPU 10 ordinary-name comparison against the immutable hygiene binary
is in `obj/perf/1791783347942994750/samples.json`. Outputs match in full at every
scale. After A/A calibration and three ABBA blocks, 4,000/16,000/64,000-function
A/B medians were 56.19/51.70, 220.60/203.44 and 934.02/856.58 ms. The 64,000
A/A observations ranged 938.48–955.48 ms. This observed improvement is evidence
for these binaries and inputs, not a general throughput guarantee; full PA8
measurements still require the forthcoming expanded-crate CLI contract.


`ancestry-limits-release.log` also passes all 142 Rust tests in release mode.
`ancestry-limits-reference.log` records the independent earlier-stage rustc
checks, upstream/Reference provenance and explicit reference regeneration.
A structured comparison confirms only `compiler_source_sha256` changed in each
PA1–PA7 receipt; every earlier fixture and oracle byte is unchanged.


## Expansion-time outer-scope checks

Selected import/macro paths now record whether each binding was found lexically
or through a qualified path. Expansion-time lookup rejects an inner glob that
conflicts with an outer candidate, and rejects a more-expanded binding that
would hide an outer candidate from outside its expansion. Qualified paths and
ordinary expression/type references retain their separate lookup rules. Equal
targets and references inside the introducing expansion are positive controls.
Import completion runs these checks for every namespace and reuses its selected
path buffer across queries. A hidden glob contribution still wakes dependent
imports when its ambiguity evidence changes.

Four independently classified PA6 fixtures own the ordinary-import behavior:
`import_outer_conflict.rs`, `ordinary_inner_glob.rs`, `import_same_target.rs`
and `import_qualified_control.rs`. Their probes and the corresponding macro
cases are in `obj/pa8-work/outer-scope-probes/`. The emitted positive reference
for `ordinary_inner_glob` targets the inner module's constructor, while the
negative import reports `ambiguous import path binding` at its use path.

`outer-scope-final.log` passes 143 Rust tests, all 1,012 active course fixtures,
strict Clippy, the 100-file audit and 14 harness tests. `outer-scope-reference.log`
passes all independent rustc checks and regenerates receipts.
`outer-scope-release.log` passes all 143 Rust tests in release mode.
`outer-scope-full.log` also verifies upstream/Reference provenance.
All earlier oracle bytes remain unchanged; PA6 adds exactly these four cases.
The preceding commit `ff50a6c` passed GitHub CI.

The immutable import-alias benchmark is retained in
`obj/perf/1791783813276120270/samples.json`, with its generator at
`obj/pa8-work/measure-imports.py`. It freezes 4,000/16,000/64,000 independent
aliases, verifies full outputs, pins both binaries to CPU 10, calibrates A/A,
and retains three ABBA blocks. A/B medians are 18.21/18.63, 67.29/68.18 and
280.40/290.78 ms; 64,000-alias A/A observations ranged 274.57–285.52 ms.
These measurements show a roughly 1–4% cost on this import-heavy workload for
the added ambiguity checks, not a general no-regression claim.

Before activation, revisit early declaration registration for non-macro items
that can shadow a module used by expansion. The current early graph indexes
modules and macros; primary resolution indexes the other declarations. Builtins,
physical diagnostics, the expanded-crate text contract and its course suite
remain the next main work. PA8 and the source-to-LowIR goal remain incomplete.


## Typed builtin bootstrap

`cfg!` now uses an explicit builtin macro prelude, separate from crate item
bindings. Its result is a boolean literal node with a retained typed value;
no boolean text is parsed to recover that value. The configured predicate
implementation is shared with cfg attributes. Opaque metadata/path inputs are
consumed through the persistent registry and existing typed forest consumers.
The ordinary parser and builtin builder share the syntax arena's empty
constructor.

Expansion definitions distinguish declarative bodies from builtins. A builtin
has no invented source definition node or matcher rule. Builtin definitions are
materialized on demand; macro path registration carries the actual definition
index, including gaps from intervening builtin definitions. A regression test
first reproduced an out-of-bounds textual target when cfg expansion preceded a
generated macro definition. The explicit mapping fixes it, and an independent
rustc program compiles and runs the same sequence successfully.

Pinned probes are under `obj/pa8-work/builtin-probes/`. They establish that cfg
is callable with `no_implicit_prelude`, can be shadowed by a source textual
macro, is absent from the crate root, and is ambiguous in an unqualified import
because of its attribute identity. Captured metadata works; a nested macro in
the predicate's string position is rejected rather than eagerly expanded.
The `cfg_no_std` executable probe failed for missing panic/runtime support and
is not evidence of cfg rejection.

The same directory retains preliminary probes for subsequent builtins.
Stringify output is sensitive to token provenance: an opaque compact expression
and captured compact token sequence preserve `1+2` / `a+b`, while a literal
macro template produces `a + b`. Group spacing is not a blanket whitespace
collapse. Do not implement this by reparsing captured source or assuming all
builtins use eager argument expansion. Import aliases for stringify, concat,
line, column, file and module_path are independently accepted.

`builtin-full-validation.log` passes 147 Rust tests, all 1,012 course fixtures,
strict Clippy, 101 audited files with zero findings, and 14 harness tests.
The cfg builtin remains groundwork for planned PA8; other builtins and the
public expanded-crate text contract still need implementation and validation.


`builtin-reference-validation.log` passes independent earlier-stage rustc
checks and reference regeneration; only compiler source fingerprints change in
PA1–PA7 receipts. `builtin-release-validation.log` passes all 147 Rust tests in
release mode. No earlier fixture or oracle bytes changed.


The immutable CPU-10 ordinary-name comparison for the builtin bootstrap is
`obj/perf/1791784671136744000/samples.json`. Full outputs match. Following A/A
calibration and three ABBA blocks, A/B medians at 4,000/16,000/64,000 functions
were 51.21/51.59, 192.00/196.04 and 858.06/861.00 ms. The largest A/A samples
ranged 818.66–854.23 ms. These bounded observations accompany the change;
full expanded-crate throughput remains required before PA8 activation.

## Physical location builtins

`line!` and `column!` now construct typed `u32` literals. Their provenance walk
preserves a whole captured call's original location, but attributes a call
constructed from a captured identifier and template delimiters to its enclosing
invocation. Literal-template and nested-template calls follow that invocation
chain. The independently compiled/run cases in `builtin-probes/location-results.json`
cover these distinctions, direct calls, Unicode scalar columns and UTF-8 BOMs.
The same reference-implementation tests exercise LF and CRLF input.

Physical source positions build a line-start index on first demand. Generated
spelling images never become physical lines, and cloned physical module instances
retain the index. Invalid offsets inside UTF-8 scalars are diagnosed. Location
builtins reject arguments and do not invoke the syntax parser to recover values.

`location-full-validation.log` passes 150 Rust tests, all 1,012 active fixtures,
strict Clippy, the 102-file audit and 14 harness tests. Independent earlier-stage
checks and receipt regeneration are in `location-reference-validation.log`;
only compiler fingerprints changed, with every earlier oracle byte identical.
`location-release-validation.log` passes the release Rust tests.

The frozen CPU-10 ordinary-name comparison is
`obj/perf/1791785364032056514/samples.json`. All full outputs match. A/B medians
at 4,000/16,000/64,000 functions were 52.86/58.09, 208.64/229.93 and
883.12/952.25 ms; largest-input A/A samples ranged 820.82–879.01 ms.
This run observes an approximately 8–10% slowdown and does not establish a
no-regression result. The ordinary-name path does not demand a line index;
source container layout and binary layout remain possible contributors, not
measured explanations. Revisit alongside the forthcoming full expansion workload.

## Ordinary declarations during expansion

Expansion now shares the ordinary item-header indexer, registering item/member
namespaces, scopes and enum variants before attempting macro calls. Generic
parameters remain a later-resolution responsibility: pinned rustc accepts
`a::make!()` through an outer module even inside `fn f<a>()`, but ordinary
structs, enums and type aliases hide that module. Qualified paths, value locals
and nested item barriers have explicit positive controls. Generated headers
publish dependency events, so a late generated type can invalidate a previously
selected macro path. Final name resolution reuses those declaration identities
and scopes instead of creating a second graph or collecting headers again.

Nineteen pinned cases are retained in `obj/pa8-work/type-shadow-probes/`.
The additional cases cover enum-variant imports, glob ambiguity, a generated
shadow after a call, generic import rejection, generated function imports and
stable definition identities across the ordinary-resolution handoff.

These controls also exposed PA6's missing single-variant imports. A namespace
without a variant constructor or macro is an absent binding, not an associated
item import. Four independently classified fixtures now cover unit, tuple and
record variant imports and a missing-variant rejection. No earlier acceptance
contract is relaxed to accommodate a macro implementation limitation.

The earlier immutable item-indexer measurement (before the subsequent enum
lookup correction) is `obj/perf/1791785809293502448/samples.json`. CPU-10 A/B
medians were 89.17/88.76, 366.95/371.69 and 1505.43/1503.10 ms at
4,000/16,000/64,000 functions; largest A/A samples ranged 1475.27–1523.29 ms.
Full outputs matched. The host's absolute timings differ substantially from the
previous checkpoint, so compare paired observations only. Full expansion
performance still needs its public count/view interface.

Final validation passes 152 Rust tests, all 1,016 active course fixtures,
strict Clippy, the 102-file audit, 14 harness tests, pinned rustc checks,
upstream/Reference provenance and release tests. Evidence is in the
`early-items-*-validation.log` and `early-items-final-build.log` files.
Receipt review confirms only fingerprints and the four new PA6 fixtures changed;
all previously shipped fixture and oracle bytes remain identical.

The final-code immutable comparison is
`obj/perf/1791786099966592827/samples.json`. Full outputs match. Paired A/B
medians at the three scales are 55.31/55.82, 228.31/226.36, 950.25/954.99 ms;
largest A/A samples range 978.10–993.85 ms. These observations do not
replace full PA8 expansion throughput measurements.

## Expanded-crate driver and text view

The maintainer driver now supports `--emit-expanded-crate` and
`--count-expanded-crate`. Both expand the crate and finish ordinary-name
resolution over the retained graph. The view reports macro definitions,
invocations, parser callback counts, syntax contexts, introduction ancestry,
per-file configured syntax, origin ranges and final name facts. Source failures
produce no partial view. Builtins retain typed owned values and no invented
matcher rule. PA8's handout documents this prototype vocabulary; the assignment
remains planned until its remaining builtins, diagnostics and course suite are
complete.

The existing resolution renderer owns the composed view. PA5's syntax and
configuration writers now borrow either source-backed or stored literal arenas;
a caller-supplied literal writer reads the existing storage directly. No syntax
arena is cloned or reparsed to render an expanded file. The earlier text
contracts retain their existing entrypoints and byte vocabulary.

Three CLI integration tests check repeated-run determinism, count/full-view
agreement, opaque capture parser counts, owned builtin values, ordinary-name
completion and atomic failure output. The full validation in
`expanded-view-full-validation.log` passes 155 Rust tests, all 1,016 active
fixtures, strict Clippy, the 103-file audit and 14 harness tests. The preceding
item-registration commit `6d45e88` passed GitHub CI.

The new `make perf-expanded-crate` workload freezes independent functions with
a rejected first matcher rule and a duplicated opaque expression capture.
The initial CPU-10 A/A baseline is
`obj/perf/1791786417131657606/samples.json`; both frozen binaries and all full
output hashes agree. At 4,000/16,000/64,000 expansions, paired medians were
58.47/230.17/966.55 ms and peak RSS was 14,200/47,796/182,132 KiB. Largest-input
A/A observations ranged 975.63–1065.51 ms. Each scale reports exactly one retained
capture per invocation and one resolved return-type reference per generated
function. This establishes a scaling baseline, not an A/B regression claim.

`expanded-view-reference-validation.log` passes the independent rustc checks,
reference regeneration and harness checks. Only compiler source fingerprints
changed in PA1–PA7 receipts; all fixture and oracle bytes remain identical.
`expanded-view-release-validation.log` passes all 155 Rust tests in release mode.

## Indexed Unicode columns

Physical positions now use one lazily built index of line starts and cumulative
extra UTF-8 bytes at non-ASCII scalars. Each query uses binary search instead of
rescanning its entire line prefix. BOM handling, CRLF normalization, combining
scalars, tabs, supplementary characters and later-line BOM characters have
explicit tests. A single boxed cache keeps unused metadata small in ordinary
source containers; physical copies preserve the cache and generated images do
not change it.

The adversarial one-line Unicode/`column!` comparison is retained in
`obj/perf/1791786718350701061/samples.json`, generated by
`obj/pa8-work/measure-columns.py`. With frozen binaries, complete output equality,
CPU-10 affinity, A/A calibration and three ABBA blocks, A/B medians at
1,000/4,000/16,000 calls were 17.76/15.37, 81.52/52.32 and 546.80/172.35 ms.
The largest-input A/A samples were 652.10, 660.84, 577.70 and 580.28 ms.

The ordinary-name comparison against the pre-location cfg checkpoint is
`obj/perf/1791786839189813846/samples.json`: A/B medians were 53.05/53.96,
188.54/193.32 and 870.14/891.47 ms, with largest A/A range 878.03–913.00 ms.
The full expansion comparison against the new CLI baseline is
`obj/perf/1791786863281735054/samples.json`: 59.53/55.51, 250.54/238.47 and
1026.23/975.39 ms, with largest A/A range 1030.86–1057.93 ms. All full outputs
match. These observations show the long-line improvement and bounded costs on
these workloads; they are not a general compiler throughput guarantee.

`position-index-full-validation.log` passes 156 Rust tests, all 1,016 active
fixtures, strict Clippy, the 103-file audit and 14 harness tests. The corresponding
reference-validation and release-validation logs pass independent rustc checks,
receipt regeneration and release tests. Only compiler fingerprints changed in
receipts; no course fixture or oracle bytes changed. Commit `ce07ed0` passed CI.

## Physical diagnostics and expansion traces

Expansion and the ordinary-resolution handoff now project source errors through
retained origin records. Typed error frames retain macro names and physical
call/definition locations; their public display emits innermost-first notes.
Captured expressions retain the caller's location. Failed parse/configuration
of a new expansion maps its token offset before installation, and a failed
invocation still contributes a selected-definition frame. Builtin failures use
their argument offset when available. Newly loaded files can fail before a unit
exists; error projection handles that case and retains the introducing macro.
Projection and frame allocation run only on error paths. The error type also
implements the standard Display and Error traits.

Independent rustc JSON diagnostics in `obj/pa8-work/diagnostic-probes/` establish
physical locations for nested calls, captured expressions, generated definitions
and malformed output. Their primary offsets agree for those cases. The builtin
argument case intentionally locates the bad argument, while rustc highlights
the builtin call. RustGM does not promise rustc's diagnostic wording or notes.
Tests also cover separate-file definitions, uninstalled malformed modules and
atomic CLI output. The handout records the diagnostic vocabulary.

`diagnostic-full-validation.log` passes 159 Rust tests, all 1,016 active fixtures,
strict Clippy, the 104-file audit and 14 harness tests. Reference and release
validation logs pass independent checks, receipt regeneration and release tests.
Only compiler fingerprints changed in earlier receipts; no course fixture or
oracle bytes changed. The Unicode index checkpoint `08f851a` passed GitHub CI.

The frozen CPU-10 full-expansion comparison is
`obj/perf/1791787467770017233/samples.json`. Full output hashes agree. A/B medians
at 4,000/16,000/64,000 calls were 93.64/94.47, 228.71/228.86 and
999.74/983.45 ms; largest A/A samples ranged 969.66–979.38 ms. These paired
observations accompany the error-path change without asserting a general speedup.

## Requested source names and module identities

`file!` and `module_path!` now produce typed strings. The source provider keeps
requested paths separately from canonical cache identities, so aliases retain
their observable names and nested modules load beside the requested path. This
also fixes PA6 directory ownership through symlinks. Three independently checked
PA6 cases cover explicit aliases, implicit aliases to `mod.rs`, and an aliased
root file. Source instances still share parsed physical input where appropriate.

Module paths use the expansion's receiving module and configured crate name.
The driver accepts `--crate-name`; root string attributes, first-attribute
selection, conflict/malformed checks, filename defaults, stdin and Unicode names
have typed and CLI checks. Independent rustc observations live in
`obj/pa8-work/source-name-probes/`, including actual symlink packages and a
captured call moved into a generated child module.

The harness now supplies relative file arguments from the repository root,
records that cwd and permits explicit crate-name arguments. A relocation test
checks portability. Existing fixture and oracle bytes remain unchanged; only
three PA6 cases and their generated sidecars were added. Earlier receipts change
only compiler fingerprints; PA6 additionally records the new inputs and manifest.

`file-builtins-final-validation.log` passes 162 Rust tests, all 1,019 active
fixtures and 15 harness tests. The full validation log also passes strict Clippy
and the 104-file architecture audit; release and reference validation logs pass
release tests, independent rustc checks, regeneration and upstream checks.
The preceding diagnostic commit `b6c086f` passed GitHub CI.

The frozen CPU-10 ordinary-name comparison is
`obj/perf/1791789227393580160/samples.json`. Full outputs agree. A/B medians at
4,000/16,000/64,000 functions were 52.80/51.62, 283.41/338.91 and
876.78/880.90 ms. The middle scale's A/A samples span 209.26–346.44 ms, so those
observations do not establish a regression or improvement. Largest A/A samples
span 881.59–906.58 ms. The new file/module builtin workload has an A/A baseline
in `obj/perf/1791789253199624990/samples.json`: at 1,000/4,000/16,000 pairs,
A/B medians were 24.26/24.15, 88.73/90.84 and 388.56/398.58 ms. It freezes
identical binaries and verifies complete output hashes; it makes no A/B claim.
PA8 remains planned while other builtin behavior and its grading suite develop.

## Typed stringification and retained capture tokens

`stringify!` now walks classified tokens, opaque capture handles and explicit
spacing provenance. Demanded opaque tokens occupy one flat registry arena;
repeated/forwarded substitutions reuse the same range. Grammar-split boundary
tokens are retained with their typed categories. Ordinary source is never
relexed and opaque syntax is never reconstructed from rendered text.

The renderer uses explicit continuation stacks and a flat layout buffer.
Source/template spacing, identifier NFC, invisible empty visibility, delimiter
edges, 78-byte wrapping, brace indentation and direct versus captured doc
comments have permanent regressions. Template doc comments retain their typed
comment category until a consumer requires desugaring. Synthetic doc attributes
record spacing independently of their physical diagnostic origin. A 512-level
stringification case checks iterative traversal; work/frame bounds diagnose
excessive demands. The renderer does not expand argument macros or call the
fragment parser.

`obj/pa8-work/stringify-probes/matrix.py` checks 77 direct/template/capture,
Unicode, documentation, delimiter, long-layout and split-token cases against
pinned rustc runtime values; every value matches. The earlier small probe placed
stringification inside `println!`, which itself captured and desugared comments.
The expanded matrix intentionally checks functions returning strings directly
and found that distinction. Raw observations are in `matrix-results.json`.

`stringify-full-validation.log` passes 166 Rust tests, all 1,019 active fixtures,
strict Clippy, the 105-file audit and 15 harness tests. Reference/release logs
pass independent rustc checks, all receipt regeneration, upstream checks and
release tests. PA1–PA7 receipts change only compiler fingerprints; all prior
fixture and oracle bytes remain identical. Commit `f930ffc` passed GitHub CI.

The frozen CPU-10 full-expansion comparison is
`obj/perf/1791790578919403274/samples.json`. Its baseline is `f930ffc` with only
an unused `stringify` prelude entry added, so the planned view's name/definition
IDs agree. The exact compatibility patch is retained as
`obj/pa8-work/stringify-baseline-prelude.patch`; the workload never invokes it.
Full output hashes agree. At 4,000/16,000/64,000 captures, A/B medians were
55.29/56.64, 303.96/365.68 and 969.31/983.33 ms; peak RSS was
14,216/14,232, 48,268/48,424 and 183,304/185,552 KiB. Middle-scale A/A samples
were 221.81, 211.84, 224.37 and 207.35 ms, substantially below both paired
medians; the run contains time variation and does not isolate a throughput
regression. Largest A/A observations span 907.86–973.53 ms. The memory cost of
retained captures and spacing provenance is visible at the largest scale.

The new repeated-opaque `stringify` workload establishes an A/A baseline in
`obj/perf/1791790606716603990/samples.json`. At 1,000/4,000/16,000 invocations,
A/B medians were 24.20/24.09, 90.98/90.58 and 385.89/399.41 ms; largest A/A
samples span 369.34–425.25 ms. Both snapshots are identical; complete output
hashes, frozen inputs, affinity and every observation are retained. This is a
scaling baseline, not a speedup claim. PA8 still needs eager builtins and its
released course suite before activation.

## Eager argument worklist, concat and compile_error

Eager builtins now parse comma-separated arguments once into a detached typed
arena in the existing source unit. The argument roots keep source offsets,
configuration facts and literal storage. Nested calls use the existing macro
worklist and notify their waiting parent through an explicit dependency count;
generated definitions/imports wake those calls without reparsing arguments.
A completed builtin installs its typed result and retires argument nodes.
Ordinary expansion and eager argument retention share checked arena/literal
append logic. Unresolved dependency diagnostics select a failing child before
its waiting parent and retain the eager expansion ancestry.

`concat!` consumes expanded literal facts, including negative numeric literals;
`compile_error!` diagnoses one expanded string. Parentheses/blocks/constants
are not silently evaluated. Shadowing and macros that discard their input keep
their existing semantics. `concat_bytes!` was independently confirmed unstable
in the pinned compiler and is outside the stable profile. Remaining PA8 source
builtins include include/include_str/include_bytes and env; library-dependent
builtins require the later language/library hooks.

The independently declared 24-case acceptance/value matrix in
`obj/pa8-work/concat-probes/matrix.py` matches rustc throughout. It covers
numeric spellings, nested builtins, locations, late generated exports, opaque
captures, discarded arguments and invalid expression forms. A separate
cross-file program confirms an opaque capture retained in another source unit.
Permanent tests additionally exercise two delayed definitions, exactly one
argument parse per concat, retired temporary roots, capture reuse, recursive
failure, diagnostics and atomic CLI output.

`eager-full-validation.log` passes the then-current 170 Rust tests, all 1,019
active fixtures, strict Clippy, the 106-file audit and 15 harness tests.
Reference/release validation passes independent rustc checks, all receipt
regeneration, upstream checks and release tests. The additional cross-file test
brings the suite to 171; affected debug/release tests and final lint/audit/oracle
checks pass in the extra/final validation logs. Only compiler fingerprints
changed in PA1–PA7 receipts; fixture/oracle bytes remain unchanged. The preceding
stringification commit `7b4d558` passed GitHub CI.

The frozen CPU-10 full-expansion comparison is
`obj/perf/1791791539115112474/samples.json`. Baseline `7b4d558` has only the two
unused eager builtin names registered to preserve planned-view identities;
`obj/pa8-work/eager-baseline-prelude.patch` records that compatibility change.
Full output hashes agree. At 4,000/16,000/64,000 captures, A/B medians were
57.16/56.80, 229.85/236.35 and 1085.95/988.41 ms; peak RSS was
14,224/14,876, 48,616/49,448 and 185,440/188,940 KiB. Largest A/A samples span
961.24–978.51 ms, below the paired baseline median; this is not evidence for a
general speedup. The added worklist state costs about 3.5 MiB on that scale.

The new nested concat workload establishes an A/A scaling baseline at
`obj/perf/1791791569309680782/samples.json`. At 1,000/4,000/16,000 outer calls,
A/B medians were 39.55/40.21, 152.88/155.30 and 693.20/695.61 ms. Largest
A/A samples span 688.52–720.32 ms. Identical snapshots, full output hashes,
frozen inputs, affinity and every observation are retained. PA8 remains planned
until its remaining source builtins and nonempty released suite are ready.

## Explicit environment and byte-resource builtins

`env!`, `include_str!` and `include_bytes!` now consume eager string arguments
through the source provider. The library reads neither the process environment
nor the filesystem. Drivers supply raw resource bytes and Unicode environment
values; per-compilation caches retain both values and lookup failures. Relative
resource paths follow the invocation's physical expansion cause and requested
source name. Anonymous stdin requires an absolute path. Resource bytes do not
pass through Rust source normalization or create syntax units.

Typed tests cover BOM/CRLF/bare-CR preservation, arbitrary binary bytes, invalid
UTF-8, missing resources, cached reads, inactive code, eager argument validation,
custom absence diagnostics and non-Unicode provider errors. Child-process CLI
tests explicitly control the environment without mutating the test runner's
process. Cross-file probes confirm that both a template include and an opaque
include retained in a generated exported macro use the eventual root callsite
in those cases. An initially hypothesized definition-directory result was
rejected by the independent compiler and was never blessed as an oracle.

All 18 independently declared acceptance/value cases in
`obj/pa8-work/resource-probes/matrix.py` match pinned rustc. Physical-path probe
results are retained in `path-results.json`. The final suite has 176 Rust tests
and 1,019 active course fixtures. `resource-full-validation.log` passes those
before a test-provider field name triggered Clippy after its structure grew;
the field was renamed, and `resource-reference-validation.log` passes strict
Clippy, the 107-file audit, 15 harness tests, independent rustc, full reference
regeneration and upstream checks. Release tests and affected final debug tests
also pass. Earlier receipts change only compiler fingerprints; all PA1–PA7
fixture/oracle bytes remain identical. Commit `6c87353` passed GitHub CI.

The CPU-10 full-expansion comparison is
`obj/perf/1791792331054843996/samples.json`. Baseline `6c87353` only registers
the three unused resource builtin names for view compatibility; the exact patch
is `obj/pa8-work/resource-baseline-prelude.patch`. Full output hashes agree.
At 4,000/16,000/64,000 calls, A/B medians were 57.11/57.01, 232.32/232.06 and
985.99/984.24 ms; largest peak RSS was 188,824/189,132 KiB and A/A samples
spanned 971.82–983.27 ms. The nested-concat comparison in
`obj/perf/1791792358833885287/samples.json` also has identical full outputs:
38.19/39.46, 167.55/166.34 and 687.40/690.55 ms at 1,000/4,000/16,000 outer
calls; largest RSS was 108,732/108,800 KiB and A/A range 687.88–700.58 ms.
These are bounded-workload observations, not a general performance claim.

The new cached-resource workload has an A/A baseline in
`obj/perf/1791792378389779294/samples.json`. It freezes source and resource
bytes, explicitly records the supplied environment and verifies complete output
hashes. At 1,000/4,000/16,000 groups of three builtins, A/B medians were
32.96/32.71, 131.27/132.02 and 549.31/553.85 ms; largest A/A range was
541.55–572.03 ms. All snapshots, hashes and observations are retained.

`option_env!` is now explicitly assigned to PA26 because it needs the library's
nominal Option identity. The remaining PA8 source builtin is `include!`; then
its graded suite and contract can be assembled and activated. PA9–PA11 remain
required before the active source-to-LowIR goal is complete.
