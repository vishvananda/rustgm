# Library build blocker tracker

Updated 2026-10-24. This is the work queue for compiling the original pinned
`core`, `alloc` and `std`, not a count of completed internal checkpoints.
PA1–PA24 remain implemented drafts; PA25–PA44 remain planned. No additional
assignment or whole upstream library has been completed.

The source pin is Rust `88d9e12ae178fab0fb5cc050a94da85685d449ea`, edition 2024,
x86-64 Linux, panic abort. Use the existing immutable shared source tree and
record target configuration, compiler identity and opened-file hashes on each
whole-crate attempt. Never patch upstream source to move a blocker.

## Observable crate milestones

| Milestone | Current result | Next acceptance evidence |
| --- | --- | --- |
| Original core loading and expansion | The qualified cfg_select compiler passes its declaration at line 235 and stops at the `format_args` builtin attribute at line 994. The same four unchanged original files are opened. It is integrated locally as `4ad4db87`; combined qualification remains pending. | Implement authentic formatting support (LIB-017), qualify the integrated projection-opening repair (LIB-016) and combined compiler and rerun the unchanged root. |
| Original core semantic checking and body export | Not reached end to end. | Check every required declaration/body and produce the real crate artifact. |
| Original core object and consumer | No successful build. | Link and execute an independent consumer using its actual generic implementations. |
| Original alloc and consumer | Not reached; depends on core and cross-crate transport. | Build unchanged alloc and execute allocation, reallocation and destruction. |
| Original std and consumer | Not reached; depends on the preceding crate graph and platform contracts. | Build unchanged std and execute a default-std consumer. |

The preceding published probe uses immutable compiler `9793c30f2d29a99196687032f88960c317c28c47a3f4f6973ba681bb09698ac4`
from integration `1df52349` (production `91113764`). It exits 1 with
`ERROR: file 3: 50: source builtin macro expansion is not implemented`.
The declaration is `core_panic`; no invocation has yet been expanded. The
four opened original files are core's root, `prelude/mod.rs`, `prelude/v1.rs`
and `macros/mod.rs`. All match the pinned source inventory. The retained
`rustgm-validation-work/obj/parallel/whole-core-blockers-v3/` record binds the
compiler, target/edition configuration, source inventory and actual open trace.
Its result SHA-256 is `b2f6bb683c0c6800b0dc2e89fa46d616bbdc925e66a444829c02cb7f7b2529dc`.

The preceding probe's `expected path segment` error at byte 6527 is retained in
`library-stability-whole-core-frontier-v2/`; it no longer describes the fresh
frontier. Parsing that file now succeeds, but neither probe completes core.
The preceding integration passed 3,605 release/native tests, 8,382 active text
tests and all 60 functional qualification stages. The newer
[lookup/input/panic checkpoint](library-lookup-panic-checkpoint.md) passes
3,644 release/native tests (1,721 fresh plus 1,923 authenticated retained),
8,382 text cases and 65 functional stages. Neither count releases a PA.

The newest published unchanged-core observation uses the qualified integrated compiler
`20eb54f33961841b6e496bec52dae3a41f51e10ee97dbdb4b03de68d8e728134`,
source `6bba6589`, in the validation worktree's
`obj/parallel/whole-core-blockers-v5/`. It exits 1 at file 3, byte 1250,
with `internal macro allowance feature is not implemented`. This is the
`panic_internals` allowance on `assert_eq`, line 41. The same four original files
were opened; all hashes and configuration remain unchanged. Its result SHA-256
is `b333b6b448ed489df47726a86aac71040c8025d3ad32e664b2ce0dcd9a8c940a`.
The preceding worker-only result at `2c31704b` is retained separately.
Passing the prior declaration advances this specific crate boundary; it does not
complete expansion, semantic checking, an artifact, or an assignment.

The allowance worker subsequently qualified source `222b637a` (production
`45c9a757`) with 1,393 nonignored private tests, 43 frontend contexts and 34
native programs. Its immutable CLI
`2a8bf052b0d0d77d5d19ba740c7d8e1c105544f3a64efe45d741e50a6f48554f`
advances the same unchanged core root to line 170, byte 6523:
`scoped declarative macro hygiene is not implemented` on `pub macro assert_matches`.
The four opened files and configuration are unchanged. Evidence is in the
worker's `obj/parallel/pa31/complete-allowances-whole-core-v1/`, result SHA-256
`fb12fac6b2860c8572417c4e3df13919e4d54ac7ef335ee8d7441571a137c502`.
The combined candidate now passes functional qualification and confirms this
frontier independently; one projection-opening growth failure remains under repair. The [allowance handoff](pa31-internal-allowances-handoff.md)
records source/native evidence, retained failures, measured costs and the
foreign API-use boundary.

## Work since the last whole-core probe

The unchanged-core stopping point is still `format_args` at line 994; the
following component results do not establish a newer whole-crate result.

- Canonical impl-member matching at `9b6665a6` passes 1,464 nonignored private
  tests, 405 affected integration cases and 109 native test functions. Its
  runtime layout is 240 bytes for the associated inventory and 7,032 bytes for
  declarations. PA20 exposed two diagnostic-location regressions: the message
  was correct, but referred to the impl instead of the extra constant. Recovery
  `51acf093`, integrated as `e4fc9eea`, preserves the actual member's location
  and passes formatting, audit and strict Clippy; fresh text qualification is
  in progress. Original text references are unchanged.
- A's imported source-owner/header checkpoint `74319e07` passes static checks
  and all 1,445 nonignored private tests, including seventeen required controls.
  Complete original-body reconstruction and fresh imported proof remain in
  progress. Public source metadata and client admission remain closed; there
  is no successful artifact/consumer transaction yet.
- B is implementing explicit implementation-validation readiness for LIB-004,
  preserving the canonical conformance checks and shared constant ownership.
  Its independent corpus has 32 metadata/object and eleven native observations;
  these qualify the contract, not the unfinished RustGM implementation.
- C's formatting implementation has reached static qualification. Its latest
  completed attempt type-checks production but fails two test-style Clippy
  checks. Recovery `fbef5a69` fixes those expressions and is entering a fresh
  gate; no candidate runtime test or newer core probe has passed. Literal
  formatting is the first slice; nonempty arguments remain required.

The integrator's native batch exceeded its 8 GiB worktree cap because completed
test groups retained executables until batch end. The reserve remained intact;
after diagnosis, cleanup reclaimed 6.41 GiB. Successful targets now clean their
disposable outputs between groups. The shared runner change passes its nine
focused harness tests and the full harness. Combined release/performance and
public integration remain pending.

## Prioritized issues and assignment ownership

"Owner PA" identifies the compiler/course responsibility. "Test PA" is the
earliest assignment owning **all** features in the complete reproduction. A
source-library reproduction using `no_core`, lang items or compiler-private
attributes belongs in PA31 even when its underlying algorithm has an earlier
owner. Add an earlier-PA regression too when the failure can be reproduced
without those later features. Do not move fixtures mechanically by subsystem.

| ID | Priority and evidence | Blocking issue | Owner PA / test PA | Worker | State / dependencies |
| --- | --- | --- | --- | --- | --- |
| LIB-001 | P0; preceding whole-core parse blocker removed, scoped semantics still required | Scoped declarative macro publication, item/path/import/glob lookup and definition-site visibility | PA31 internal feature; PA8 shared machinery / PA31, plus applicable PA7–PA8 regressions | C + integrator | Contextual lookup is qualified at `6bba6589`. Scoped declaration publication passes component qualification at `f2b8a493` and is integrated locally as `68a30f1b`; invocation, export and complete hygiene remain explicitly unsupported. |
| LIB-002 | P0; prerequisite of LIB-001 | Hygienic field, method and associated-member identity; captured names and container-relative privacy | PA31; PA12/PA19 shared consumers / PA31, with earlier ordinary controls | C + integrator | Field storage/consumers at `b937b6da` remain unqualified and excluded from the merge. Enum-alias controls pass combined functional qualification at `5602dc27`. Inherent methods and trait-method scope discovery pass component qualification through `956c9904`, integrated locally as `708ee191`; Canonical impl-member conformance has private/integration/native evidence at `9b6665a6`; diagnostic recovery `51acf093` is in text qualification. Scoped invocation and combined qualification remain required. |
| LIB-003 | P1; actual reduced source failure | Projected Copy proof sees one lifetime premise in inconsistent declaration/body namespaces | PA21 scoped proof + PA31 source const-trait defaults / PA31; add PA21 control where expressible | B | The actual instance-scope carrier and fresh Copy reproof pass component qualification at `c92e2c9b`: 1,275 unique private tests and 54 native programs, including real nonempty alias scopes. Combined functional recovery at `5602dc27` passes, including all ten previously failing projection cases. Actual subject growth 32→128 exceeds the declared bound (10.04×); LIB-016 owns the structural-opening repair. |
| LIB-004 | P1; actual reduced source failure, readiness cause observed directly | Early const read treats source `u8: Copy` as absent although its impl is present | PA20 implementation readiness + PA25 early CTFE / PA31 source-binding reproduction | B, coordinated with C | A bounded actual-execution trace confirms two fresh queries see a PositiveHeader before implementation validation and incorrectly report absence. B has resumed the preserved unqualified readiness draft `df9758de` after qualifying LIB-016. Shared per-owner conformance must retain the integrator’s canonical member mapping. The design separates readiness from negative selection and evaluates indexed per-owner validation before a suspension mechanism. |
| LIB-005 | P1; independently qualified reduced program | Anonymous constant-expression `const_eval_select` ownership and cache validation | PA25 execution + PA28 intrinsic semantics + PA31 source identity / PA31 | C | Source-ready at local `379006ad`, unqualified; preserve handoff while C prioritizes LIB-013. |
| LIB-006 | P1; core-to-consumer prerequisite | Complete substituted signatures, operation types and required inputs for real exported generic originals | PA29 / PA29 data invariants; PA31 source-provider fixtures | A | Complete operation-derived input/signature validation passes the combined checkpoint, including all 11 private controls and actual complete-source A/B output equality. Executable artifact/import proof is still required. |
| LIB-007 | P1; core-to-consumer prerequisite | Deterministic generic/source metadata, authenticated remapping and fresh imported body proof | PA29 + PA31 source-role admission / PA29 ordinary multi-crate tests and PA31 source-provider tests | A + integrator | Source-owner/header checkpoint `74319e07` passes format, audit, strict Clippy and all 1,445 nonignored private tests. Actual imported declaration readiness and original-body reconstruction are in progress. Public serialization, attachment and source-client admission remain closed until complete fresh proof and the artifact/native transaction pass. |
| LIB-008 | P1; next whole-crate acceptance gate | Close remaining active core source requirements and emit a real core artifact | PA32, with each failure assigned back to its language owner / PA32 original crate plus reduced owning-PA cases | Integrator | Blocked on LIB-001/002; discover concrete additional issues from unchanged-root runs, not guesses. |
| LIB-009 | P2; crate-graph prerequisite | Source dependency identity, edition/build configuration and reproducible build orchestration | PA34 / PA34 | Integrator | Partial interfaces exist; complete original graph qualification remains required. |
| LIB-010 | P2; not yet reached as a whole crate | Original alloc, allocator/runtime contracts and collection consumers | PA33; PA27/PA28 operation prerequisites / PA33 plus reduced owning-PA cases | Unassigned until core advances | Blocked on LIB-007/008/009; split observed alloc failures into concrete issues when reached. |
| LIB-011 | P2; not yet reached as a whole crate | Original std, platform linkage and abort-profile consumers | PA36; PA30/PA34 platform/build prerequisites / PA36 plus reduced owning-PA cases | Unassigned until prerequisites advance | Blocked on the original core/alloc graph; no unsupported percentage-complete estimate. |
| LIB-012 | P2; externally useful acceptance | Original no-default-features serde_core, then alloc/std serde_json graphs | PA35 / PA35 | Integrator | Run the core-only graph as soon as LIB-007/008/009 permit it; do not wait for std. |
| LIB-013 | P0; prior unchanged-core blocker advanced | Source-declared `core_panic` builtin and edition-dependent delegation with authentic defining-crate/token provenance | PA31 source builtin contract; PA8 expansion machinery / PA31, plus applicable PA8 ordinary controls | C + integrator | Local 2021/2024 source delegation and exact warm spelling guards pass combined qualification. The qualified unchanged-core probe advances to LIB-014. Foreign macro transport and Rust 2018 remain required; full LIB-013 is not closed. |
| LIB-014 | P0; previous unchanged-core failure at `assert_eq`, line 41 | Macro-specific internal unstable feature allowances, beginning with actual `panic_internals` | PA31 source/compiler policy; PA8 token provenance / PA31 with applicable PA8 ordinary controls | C + integrator | Complete local allowance lists and provenance pass combined functional qualification and move original core to LIB-001. The allowance growth controls pass; foreign API/const consumers remain required. |
| LIB-015 | Component fixed; unchanged-core boundary advanced | Authenticated source-declared builtin `cfg_select`, complete predicate/branch parsing and selected syntax | PA31 source/compiler role; PA7/PA8 shared cfg/expansion machinery / PA31 plus earlier ordinary controls where expressible | C + integrator | Qualified source `d8ed7286` passes 1,449 private tests, twelve frontend cases and 51 native programs, with all static checks. The unchanged-core probe reaches LIB-017. Exact compiler files are integrated locally as `4ad4db87`; combined release/performance and foreign macro transport remain pending. |
| LIB-016 | Component fixed; combined qualification pending | Repeated full caller-environment opening and premise-index construction across independent projections | PA21 projection/formation machinery; PA31 complete source regression / PA31 plus earlier ordinary controls | B + integrator | Mapped-view source `1b198324` passes static checks, 1,445 private tests, thirteen frontend cases, 31 native programs and the unchanged nine-row performance gate. All nine output pairs and sixteen native contexts pass with no triggers; subject growth is 3.732× against the 6× limit. Integrated locally as `5a5e9d51`; combined release/performance remains required. |
| LIB-017 | P0; qualified component's unchanged-core failure at `format_args`, line 994 | Source-declared formatting builtins, diagnostic/allowance identity, argument parsing and construction using the actual source formatting roles | PA26 formatting semantics + PA31 compiler/source bindings / PA31 complete providers, with earlier ordinary controls where expressible | C | Independent declaration and complete literal-provider contracts at `cb6cc8b8` retain 224 actual observations and 34 matched native programs, including diagnosed original failures. The RustGM declaration/literal-call production type-checks; recovery `fbef5a69` addresses two test-style lints before candidate execution. No qualified formatting support or newer core frontier is claimed. Full argument construction, obligations, evaluation/lifetimes and const permissions remain required. |

The combined functional recovery is recorded in
[the alias/allowance checkpoint](library-alias-allowances-checkpoint.md):
2,084 fresh release/native passes, 1,609 authenticated retained passes and all
8,382 active text cases. Its unchanged-core probe at `5602dc27` confirms the
line-170 boundary with the same source/configuration and four opened files.
Those performance measurements recorded one growth failure (LIB-016). Its
qualified repair is now integrated locally; public integration awaits combined
qualification of the coherent compiler.

The focused LIB-016 successor at `f96ec23a` passes 1,439 nonignored private
tests, thirteen frontend cases and 31 native programs. Its fixed optimized
comparison preserves all nine equal-output pairs and sixteen native executions,
but still fails subject growth: 48,267,926 to 441,032,521 instructions for
32 to 128 subjects (9.137×). The previous compiler takes 550,420,452 instructions
at 128 subjects, so this is about 20% less work without resolving the scaling
problem. Evidence is in B's
`obj/parallel/pa31/projection-opening-optimized-v2/`; analysis SHA-256 is
`49c92d4bd53c839c192a08970cd6d9b8f50af4e4bed1f831ad18d3dc556aa78f`.
The bounded observation on the same source's qualified debug test records
1,046 successful exact-owner comparisons and no foreign-owner cache misses;
867 queries use genuinely captured caller regions. Further work targets that
structural remapping, preserving fresh proof for every caller. This performance
result does not advance the unchanged-core source boundary.

The [mapped-view successor](pa31-projection-opening-sharing.md) fixes that measured
failure without changing its inputs or limit. Complete-source subject counts
8/32/128 use 9,086,952 / 27,625,883 / 103,096,127 instructions. The largest case
previously used 550,420,452 instructions: 81.27% of that work was removed, and paired
wall time fell 75.56%. Subject growth is 3.040× / 3.732×; site growth is
1.879× / 2.868×, all below 6×. Ordinary Copy and macro instruction controls
change by -0.163% and -0.018%. All nine output pairs and sixteen native contexts
pass, with no instruction, RSS or growth triggers. The actual source consumers
entered the mapped path 867 times, all successfully; that bounded observation
is separate from the optimized measurements. Hot layouts remain unchanged and
cold allocation costs are documented. B's `captured-opening-optimized-v1`
completion SHA-256 is
`5aa3265a232ed91a7f550a9bc77c19c9128ad35ac3803587baa1faf5c4ca64d2`.
The exact reviewed implementation is integrated locally as `5a5e9d51`; the
preceding failed measurements remain retained. This fixes a prerequisite and
does not produce a core artifact or advance its latest line-994 source boundary.

The separately qualified scoped-declaration component uses worker CLI
`1c2298f1ac0ac64e30e3ae6db4ecf5432e86219a69adb64c0ede50747ad2a29f`.
It reaches file 3, byte 8704, the `rustc_builtin_macro` attribute at line 234
on `cfg_select` (declaration line 235). The compiler exits 1 without an
infrastructure failure; the four original source hashes remain unchanged.
Evidence is in the C worktree's
`obj/parallel/pa31/scoped-declarations-whole-core-v3/`, result SHA-256
`69f63ebdab2a9b0a7a5d244fca0b3f413c468f54836a9a6e62af3acf20a21d8c`.
The final component gate passed twelve fresh frontend tests and 46 fresh native
programs, retaining 1,437 private passes by exact unchanged source and receipt
hashes. Formatting, architecture/file audit, strict Clippy, harness and upstream
checks passed. The preceding failed test-contract checks and provisional probe
remain preserved in the [component handoff](pa31-scoped-declarations-handoff.md).
The exact reviewed source is integrated locally as `68a30f1b`; combined release
and performance qualification remain pending. No core artifact has been built.

The [qualified cfg_select checkpoint](pa31-cfg-select-progress.md) includes
canonical source ownership, complete parsing of every branch in its actual
fragment category, and configuration/installation of the selected syntax.
Its 61-source corpus and original scoped provider retain 206 independent
observations, including 41 native executions. The final candidate gate passes
1,449 private cases, twelve frontend cases and 51 native programs. Its exact
compiler files are integrated locally as `4ad4db87`. Combined release and paired
performance remain pending; general scoped invocation and foreign transport
remain open.

The original provisional observation (`ac5aaebb`, CLI `687cf42c` prefix) at
`obj/parallel/pa31/cfg-select-provisional-core-v1/` is retained with its failed
component gate: 1,446 private passes, ten of twelve frontend passes, no native
phase. The diagnostic assertion and overly narrow ordinary-Clone profile were
fixed without changing the required fixtures. The provisional result is now
superseded by the successfully qualified component's unchanged-core probe:

- Source `d8ed72868bb73ccd8fd481720ed0f1f5cdb64144`, CLI SHA-256
  `0836f6af2d4ad8dfb58d88475d486bca99d223ba614e22627587ee7cbf31f33d`.
- C worktree `obj/parallel/pa31/cfg-select-whole-core-v2/`, result SHA-256
  `05b59bbd32e145fb92d534f21eaa705a6975fd3cceeccb87e0b30238c3e0d6b5`.
- Compiler exit 1, no infrastructure failure, `format_args` builtin attribute
  at line 994, byte 35014. All four opened original files still match their pin.

LIB-002's [inherent-method component](pa31-inherent-member-contexts.md) now
passes 1,443 private cases, 28 affected integration cases, seven native test
functions and all 8,382 active text cases. The final `8e798fcd` recovery preserves
both the declaration-record layout and the committed private-receiver diagnostic.
It is integrated locally through `abae2094`. Seven scoped trait-method cases in
`pa31/tests/spec/declarative-macros/trait-members/` have 24 independent observations,
including five native programs. They establish discovery and identifier-context
rules, not candidate support. The subsequent trait-discovery component below
implements the scope walk; associated-member conformance, scoped invocation and
the complete source artifact transaction remain open.
No core artifact or additional assignment has been completed.

The [trait-discovery component](pa31-trait-discovery.md) at `956c9904`, integrated
locally as `708ee191`, passes format, the 2,327-file audit and strict Clippy,
1,460 private cases, 202 affected integration cases, twenty native test functions
and all 8,382 active text cases. Eleven independent scope programs establish
actual block-boundary and method-context rules; their 34 observations preserve
two rejected initial hypotheses. These component checks do not enable general
scoped invocation or advance the latest observed core boundary beyond line 994.
The next conformance corpus independently passes sixteen metadata/object checks
and four native programs across eight sources. It requires impl identifiers to
map to actual trait members after hygiene adjustment and rejects duplicate
canonical-member mappings; candidate conformance changes remain pending.

The integrated source `df2c48cf` passes fresh format, the 2,325-file audit,
all-target/all-feature strict Clippy, all 183 harness tests, 377 registered
maintainer roots and upstream provenance verification (24 fixtures and 126
Reference files). Evidence is in
`obj/parallel/cfg-members-integrated-static-v1/`; completion SHA-256 is
`2ac0af9765916861d0da067926fc46605939f4875805486cab07ae7a71a090ce`.
This validates the combined wiring and retains each component's functional
evidence; it is not a combined release or performance pass. Guard
`4cebb57ecc814be3ae0d3988a9ca0258` finished with status 0/error null and cleanup
`1792863734641562721` completed without new disposable outputs. The two diagnosed
failed inherent-method runs were separately pruned after recording all 334
native observations: cleanup `1792863758154341210` removed 668 generated files,
1.40 GiB of unshared blocks, while preserving sources, diagnostics and hashes.

The first LIB-007 executable transaction uses a complete smaller provider with
ordinary sizing/Clone/Copy/receiver declarations and one generic free function.
Thirteen independent rustc observations establish source acceptance, native
consumer answer `7`, and the expected double-move errors without Copy impls.
The source profile is defined by supported semantics, never the fixture's name,
arity or body count. Renamed, reordered and multiple-function controls are
required. Every actual provided body must cross the boundary and prove again.
The larger three-body const-trait/default provider remains a required successor;
this first transaction does not claim to build core or finish PA29.

The contextual lookup, input-plan and local panic checkpoints are integrated
and qualified together; their exact fresh/retained counts and measured costs
are in the linked checkpoint. Worker A's intermediate column codec passes
component checks; its actual artifact/import execution remains disabled and
unqualified. B's actual alias-scope recovery, C's complete allowances and the
enum-alias correction pass combined functional checks; the measured projection
opening growth failure remains under repair. Public and worker-only core observations remain separate above.

## Reproductions, tests and closure criteria

Paths below include unpublished worker fixtures. Their presence in a working
branch is not a passing test or a released assignment contract.

- **LIB-001:** `pa31/tests/spec/declarative-macros/` contains predeclared source
  contracts and independent observations for concise/braced declarations,
  forward/imported aliases, repeated introduced items, mixed transparency,
  captured path segments and privacy. Maintainer suites `declarative_macros`,
  `hygiene`, `textual_scopes` and the PA7/PA8 suites retain legacy behavior.
  Close only after the real positive/negative source corpus and native answers
  pass, including exact-context import wakeups, then rerun original core.
- **LIB-002:** extend that PA31 fixture cluster with predeclared captured/literal
  field names, repeated opaque containers, method/associated-name collisions
  and private-member opponents. These new cases are **required, not yet
  qualified**. Reuse PA12 aggregate and PA19 method controls. Close only when
  actual member selection and privacy consume the shared typed context rule;
  a context-aware storage table alone is insufficient.
- **LIB-003:** retain
  `pa31/tests/spec/trait-default-effects/projections/provider-moved.rs` and
  `provider-partial-container.rs`, their public-context counterparts and
  independent contracts. Add actual body-alias/foreign-environment tests,
  independent `'a`/`'b`, complete trait/family arguments, nested binders,
  unresolved inference and failed-budget controls. The private
  `body::defining_cleanups` and trait-selection tests must pass alongside the
  retained native contexts. Do not fix ambiguity by merging arbitrary matches.
- **LIB-004:** keep the unchanged
  `pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs` and
  its independent Rust acceptance. The bounded `read_frontier` invariant now
  observes a canonical `u8` read and false Copy decision despite the actual impl.
  The retained 54-event trace resolves all ten required breakpoints: both exact
  queries use the same environment, no reuse cache, and an actual positive header;
  no validation completion occurs before the error. Pending header completion is
  already finished, so waiting on that dependency alone cannot fix the bug. Add
  validated-implementation dependencies and cold/warm/declaration-order controls. Close
  only with correct dependency resumption and a successful complete program;
  returning Deferred forever is not a fix.
- **LIB-005:** `anonymous-empty-selector.rs` and its C consumer already have
  independent positive results. The local successor adds four private owner,
  cached-result and foreign-Code controls, reuses lifetime/readiness checks,
  and declares a 64th native context. Those successor checks remain unrun.
  This does not close LIB-004 or general reference/conditional callback support.
- **LIB-006:** use the actual
  `pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs`
  collector and private `qualified_source_rows`, `transport_domains` and
  `regional_forest` controls. Add same-arity wrong signatures, changed reference
  types, wrong binder relocation, omitted inputs and unused-body failures.
  Require complete operation-derived coverage, atomic failure and independent
  width/depth/shared-DAG work and storage bounds.
- **LIB-007:** extend PA29's multi-crate contracts and the PA31 source-header
  transport family. Require complete provider → artifact → fresh consumer
  checking → native execution, input reordering, duplicate/missing dependencies,
  tampered owners/roles, const permissions, lifetime/loan/cleanup proof and
  deterministic text views. Support indexes or serialized bytes alone do not
  close this issue. Exported macro provenance is a separately required subtask.
- **LIB-008–012:** each actual build attempt supplies a versioned source/config/
  dependency lock, exact diagnostic, opened-source inventory and artifact/native
  result. Add a reduced owning-PA fixture for every newly reached failure while
  retaining the unchanged whole-crate acceptance test. An earlier parser error
  must not make a later semantic negative appear to pass.
- **LIB-013:** `pa31/tests/spec/source-panic-macros/contract.json` (original source
  checkpoint `af49643a`) declares ten single-source cases in three
  editions and nine imported-wrapper contexts. These source-builtin controls cover real
  declaration identity, aliases, call-site edition, unchanged argument tokens,
  definition-crate delegation, and forged/missing/duplicate claims. Independently
  qualify source and native outcomes before generating text. Keep the original
  core root as acceptance evidence; a declaration-only reducer cannot qualify
  actual panic delegation or a whole-core build.

- **LIB-014:** retain the unchanged core `assert_eq` declaration and add a
  reduced PA31 source-macro allowance family. Independently declare and qualify
  actual `panic_internals` and `core_intrinsics` use, caller versus definition
  literals, captured-token opponents, missing/foreign/disabled owners, repeated
  names and bounded work. A parsed allow-list alone cannot qualify feature use;
  real stability/feature consumers must check the authenticated expansion grant.

Every completed fix includes its owning handout update, independently declared
positive/negative outcomes, relevant maintainer tests, and manifest-discovered
text-oracle tests when that assignment's contract is published. Generate and
review oracle text; never hand-edit it or accept an empty/planned suite as a pass.
Text fixtures belong in the earliest PA owning all features, and private test
roots must remain registered in the existing maintainer suite groups.

## Work and qualification rules

Keep the three existing GPT-6.1-Sol/high workers. A worker can carry multiple
independent issues, but keeps coherent commits and an explicit handoff for each.
Agree shared APIs before concurrent edits; use separate worktrees. Current
primary lanes are A: LIB-006/007; B: LIB-016, then the preserved LIB-004
readiness draft; C: LIB-017 formatting, with scoped invocation, the LIB-002 field
checkpoint and LIB-005 still required; integrator: LIB-002 method/associated-member
consumers, LIB-008/009/012 and combined qualification.

Source work proceeds concurrently. Heavy jobs use the existing 150-GiB reserve,
50-GiB/worktree ceiling and CPU/RAM limits. After the qualified schema-2 tooling
is installed at an idle boundary, admit up to three jobs with explicit smaller
absolute caps and measured headroom. Keep measurements exclusive.
Use focused batched checks during a fix and one combined release gate
for a coherent integration. Reuse unchanged qualification and independent
observations with exact source/tool bindings; retain failures and clean outputs.

Update this tracker at each handoff/integration and each original crate attempt.
Report three separate outcomes: **issue fixed**, **crate boundary advanced**,
and **assignment released**. Closing an issue requires its complete reproduction
and guards to pass. Advancing a crate requires a fresh unchanged-crate result.
Releasing a PA requires its entire cumulative contract; no partial activation.
