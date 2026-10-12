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
