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

1. Complete the grammar consumers for forwarded classes and continuations.
   Examples include expression fragments used in patterns, type/path fragments
   forwarded to metadata, and composed fragments containing nested opaque input.
2. Integrate the retained macro graph with ordinary name resolution. PA6's
   declaration pass still treats macro_rules names as ordinary direct bindings;
   it must consume the expansion result without reintroducing duplicate or
   forward-visible textual names. Keep import targets and module scopes typed.
3. Consume recorded expansion contexts for mixed-site hygiene, `$crate`, and
   local/label definition-site visibility. Context IDs and token provenance alone
   are not a hygiene implementation. Qualified `$crate` syntax still needs the
   applicable path consumers and crate visibility checks.
4. Add the specified builtin macros, source diagnostics/provenance projection,
   recursion-limit attributes, and independently classified course fixtures.
5. Measure the full expansion path, finalize the PA8 handout/text vocabulary,
   refresh receipts explicitly, validate debug/release and CI, then activate and
   push the milestone. Continue through PA9 types/inference, PA10 typed body CFG,
   and PA11 actual Rust-to-LowIR execution using the pinned CPPGM backend.

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
This is not yet the mixed-site local-variable and label hygiene implementation.

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
