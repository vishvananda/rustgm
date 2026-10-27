# Library bootstrap execution plan, 2026-10-26

The objective remains every assignment requirement needed to compile original
pinned core, alloc and std and execute their consumers. This revises execution
order and internal preparation policy. Source pins, Rust semantics, assignment
completion criteria and the frozen CPPGM backend boundary remain unchanged.

## Assessment

The architecture is producing useful components, but the unit of work has been
too small and too internal. Readiness records, source-only adapters and separate
checkpoint documents accumulate faster than executable consumers. This is now
the main scheduling problem. More workers or fewer semantic tests alone will
not solve the joins between those components.

The combined compiler `9315d746` executes direct source Thin, PhantomData, final
trait methods and full-attribute Location derives, and authenticates source
derive helpers and try-block syntax. Original core passes VTable, Share::share,
Default and the try blocks, then stops at the const closure in
`ops/try_trait.rs:415`. It takes 1.976 seconds,
verifies all 722 source inputs and opens 96 files. That is neither 96 compiled
modules nor a completion percentage; there is still no core artifact. Broader
alias/Pointee semantics and actual library body/metadata joins remain required.

The largest bottleneck is the implementation joins. The integrator has now
merged qualified actual imported names/receiver shapes and Pointee consumers
at 4a23c7297. Complete PA29 member conformance, the public encoded Provided
contract and demanded original/instance execution remain unfinished. Private
Ready states do not establish an artifact or an imported source call.

Recent work exposed restrictions beyond the language requirement: eager imported
body-environment reconstruction, exact-original-signature recovery at transmute,
and speculative DynMetadata field snapshots. The latter two have already been
removed from their qualified paths. Pinned source and independent observations
must determine the obligations of each actual consumer.

Validation also needs consolidation. The earlier combined nineteen-control check
took 221.05 seconds: private Clippy 63.63 seconds, build 127.38 seconds, and
selected execution 0.31 seconds. Removing more test cases saves little here.
The previous 32-control check took 150.13 seconds with 0.215 seconds of execution.
Static checks and compilation dominate.
The latest combined twenty-one-control check takes 266.90 seconds: build 135.53,
production/CLI Clippy 44.47, private Clippy 58.06 and test execution 0.22 seconds.
It qualifies three joined consumers in one build; native controls and the core
observer then reuse that binary. Continue this consolidation within the unchanged
five-minute development limit rather than repeatedly compiling isolated helpers.
The push workflow now runs nineteen shared source/CLI smoke controls through
the same fastcheck runner. Scheduled 40297261891 previously exposed ordinary
bootstrap diagnostic regressions and stale assertions. Full qualification
remains in scheduled/manual CI; full run 40403702497 on f1f08f203 is green.

The newer full run `40368263999` took 14m15s and failed: 1,779 private tests passed
in 2.26s; two publication-replay helpers omitted newly required completed callee
facts. Three memory controls retained obsolete numeric-pointer diagnostic text,
and one CLI test hit a pipe race when an invalid option closed stdin early.
These are actionable regression controls, not a reason to repeat full release
qualification on each edit. Native/debug/independent steps were skipped, so this
run does not qualify them. The earlier successful-step evidence remains scoped.

The successor full run 40377221175 passed release/course checks but failed one
native frontend group: the cross-edition never-fallback helper appends a parsed
donor Unit after expansion without producing its lint-owner records. The error is
`active lint source owner has no produced record`. The fixture is now corrected
through the existing provenance-template parser, retaining only the actual
macro definition, and both ordinary LowIR and native controls pass. Production
source validation is unchanged. Debug/independent
steps were skipped. This is a concrete example of useful regression coverage
arriving too late in the current CI arrangement.

## Scheduling decisions

The review below makes these decisions operational. Existing source checkpoints
remain evidence; they do not create another scheduling queue.

1. Give each lane one next executable acceptance target and its exact missing
   joins. Internal components remain commits within that task, rather than new
   open-ended milestones. After two coherent checkpoints without an advanced
   source/artifact consumer, review whether the work is needed for that consumer
   before adding another preparation API. Do not discard required work.
2. Complete the actual supported source path before broadening it. Replace a
   blanket feature fence only with complete checking of the supported forms;
   keep explicit diagnostics for other unsupported declarations and uses.
   Core's actual Thin alias must not wait for unrelated const/object-alias
   features. All required unused declarations still undergo their real checks.
3. Keep an implementation queue and a separate PA release queue. A missing broad
   PA25–PA30 corner case does not prevent work on core, but no partial assignment
   becomes active. At a consumer milestone, reconcile the owning PA's remaining
   behavior, tests/oracles and publication checks; run its release gate when
   that checklist is actually empty. Avoid treating every historical checkpoint
   document's old remaining-work paragraph as a current extra obligation.
4. Keep three workers and the existing three-heavy-job limit. Shared AST/type or
   metadata interfaces get one owner and one coherent handoff; successors merge
   that base rather than developing competing exhaustive walkers. Integration
   gets the next heavy slot when qualified branches are waiting. Adding workers
   before these joins close would increase build and merge contention.
5. Inspect the alloc/std graph now, without waiting for core or launching target
   builds that cannot succeed. Freeze selected manifests, dependency sources,
   build-script inputs/outputs and required runtime symbols. Original alloc
   depends on compiler_builtins; original std has platform crates, std_detect,
   panic_abort, unwind and other dependencies even in an abort-oriented plan.
   Resolve the actual selected graph instead of inferring it from the panic
   setting. PA34 orchestration and PA30 ABI/backend probes can use small real
   providers before the original libraries compile.
6. Broaden blocker discovery using existing phase views, configured source
   inventories and independent source reducers. Keep observed failures separate
   from suspected requirements. Do not bypass a failing original module or count
   a reducer as a whole-crate pass. Refresh original core after relevant changes;
   its current 1.873-second attempt is cheap compared with a compiler rebuild.

The first sequence is: actual imported provided-method execution, genuine
Thin/Pointee/Foreign/NonNull consumers and false-assert execution; then complete
original core checking/export and an independently compiled consumer; then
original alloc with allocation/drop consumers; then the selected std graph and
platform consumers. The first three component targets proceed in parallel.
Begin core-only real-crate acceptance immediately when its dependency graph
works. Do not insert a broad assignment-release or performance campaign between
these executable milestones.

## Execution review and next decisions

The latest review still finds implementation joins and repeated compilation to
be the limiting work. It does not justify more concurrent compiler workers.
The four most recent standard receipts in each active checkout include repeated
static or assertion failures before a consumer advances. For example, B's
qualified Pattern checkpoint spent 80.63 seconds in private Clippy and 130.00
seconds building, versus 0.17 seconds executing its selected tests. C's Foreign
attempt spent 75.69/145.06 seconds in those compilation steps and 0.06 seconds
executing: five semantic groups passed and the ordinary feature-retention group
found a real production bug. These checks are useful; shrinking their test
selection further would not recover meaningful time.

Full CI [40382875468](https://github.com/vishvananda/rustgm/actions/runs/40382875468)
on b6acf4f1a found one stale Pointee diagnostic assertion: 1,787 ordinary private
tests passed, one failed and two were ignored in 2.38 seconds. All thirteen
ordinary integration binaries passed; the combined release step still failed,
so native/debug/independent qualification did not run. C's Foreign successor
27b1712dd now passes that assertion and the affected feature-retention group in
112.86 seconds, with five unchanged source-mode groups retained from the earlier
attempt. Root integration and a production CLI/core observation remain pending.
The earlier never-edition repair remains
locally qualified, not newly native-qualified by this failed run.

Apply the following rules at the next handoffs:

- Keep one active consumer per worker and at most one qualified handoff waiting
  for integration. A dependent successor can continue in source, but the
  integrator takes a queued shared base before starting another unrelated
  compiler task. Finish the integrator's current Thin join, then reserve that
  lane for integration, actual crate observations and release preparation.
- Integrate independently useful Foreign/NonNull changes when their controls
  pass. Do not wait for the entire Thin or panic chain before observing original
  core again. Loading and semantic milestones remain separate; an earlier
  phase can discover the next independent blocker without claiming a compile.
- Keep a real failing consumer in each lane from the start. Internal header or
  proof tests accompany it, but cannot replace it. After two preparation
  checkpoints without progress through that consumer, review and remove
  duplicated adapters or unnecessary prerequisites before adding more states.
  This is a design review trigger, not permission to skip Rust obligations.
- A continues installed member/body checking. A also specifies the exact
  Provided namespace/advisory/defaultness wire rows in the existing milestone.
  After Foreign integration and the next core observation, C can implement that
  codec independently if the newly observed core blocker is already owned.
  Keep codec ownership separate from A's semantic tables; do not introduce
  another metadata owner, proof token or handoff format. This overlaps two
  serial PA29 requirements using the existing three workers.
- At a coherent integration that already builds the private executable, run
  its ordinary tests when they fit the remaining five-minute budget. The
  existing dev-fast complete run took 35.39 seconds; budget that observation,
  not the 2.38-second release timing. Otherwise select a shared smoke set from
  that same binary. Do not rebuild a release executable for this purpose.
  Include the actual changed declaration family and promote escaped regressions
  into that selection. Private and public/CLI controls share one standard batch
  where needed; avoid repeating static work just to change selectors.
- Use `make smoke` for shared push coverage through the existing runner. Its
  nineteen exact tests cover source roles/Pointee, reached const execution and
  ownership failures, encoded generic/source imports, required local trait impls,
  frontend-to-LowIR execution
  and ordinary lexer cases. The local check passes in 34.63 seconds including
  strict Clippy, formatting, global audit and both integration binaries. Keep
  the existing five-minute limit and ordinary exact selection for changed paths.
- Update the live tracker and the owning handout at a consumer milestone.
  Preserve raw evidence in obj and existing checkpoint records, but stop
  creating a separate narrative document for every private adapter. Historical
  remaining-work paragraphs are not independently authoritative backlogs.

No completion multiplier or calendar estimate is established by these changes.
Measure the next three integrated consumers: elapsed time to acceptance,
compilation/validation time, queued handoff age and the actual original-crate
phase reached. More workers become useful only when the tracker has another
independent observed blocker with disjoint interfaces and no integration queue.
The three-job limit, CPU/RAM caps, scoped cleanup and 150-GiB reserve stay in force.

## Assignment closure without blocking the library path

PA25 through PA31 currently have no published tests/manifest.json. They also
retain real semantic gaps; generating a manifest alone would not complete them.
At each consumer exit, reconcile the owning assignment's existing declared
outcomes against the current implementation into four categories: supported
behavior, independently rejected input, Rust-valid unsupported behavior, and
missing oracle/release evidence. Retain the original expected outcomes and
source bytes. Resolve historical superseded gaps rather than counting them twice.

The [tracker's closure queue](library-build-blockers.md#assignment-closure-queue)
owns the next review for each PA. Reuse existing independent inputs for reviewed
text references. Keep an unfinished assignment planned and its ordinary target
failing; do not activate partial contracts. Publication preparation can overlap
implementation, while the complete cumulative release gate runs only when the
actual behavior and required reference inventory are ready. This keeps broad
assignment closure finite without making it a prerequisite for testing core.

## Reviewed imported-body policy

The defining crate still checks every required source declaration and body
before export. Metadata retains the complete source family, defaults and
contracts. Validate decoded structural identities, headers and visibility.
Prepare an imported body's defining environment and immutable original proof
on its first actual demand. Unrequested bodies remain explicitly Pending,
never absent, Ready or proved. Reuse a completed original only under the same
authenticated immutable body, Store and environment. Every actual instance
still pays its ordinary obligations and ownership/lifetime duties. Serialized
proof tokens cannot pay these debts.

Separate genuine declaration conformance from body readiness. A borrowed
declaration-header view may carry canonical origin, signature, universal scheme,
complete declaration inputs and authenticated body presence. It carries no
invented body Environment or execution permission. Original/copy/materialized
and execution paths retain the real body-context API after demand.

The same demand rule applies to expensive imported header/role elaboration.
Validate the complete structural, identity, signature, visibility and
impl/coherence inventory at its actual boundary. Leave unrelated elaboration
Pending only when it cannot affect lookup, coherence or the selected proof.
Use one validated immutable owner with indexed queries instead of adding
equivalent provenance wrappers or replaying whole-package validation on each
query. This does not remove required original/instance checks, accept missing
metadata as absence, or serialize a paid proof.

Reuse the actual collector's namespace columns, resolved ExportBindings and
existing namespace remapping/pending-binding producer before final resolution.
Names alone do not establish visibility or reexports. An unpaid sidecar can
stage data internally; final acceptance needs encoded/reloaded metadata and a
fresh consumer. An out-of-band collector receipt is not wire authority.

Complete const/default-body and broader lifetime/alias/nominal transport remain
required for PA29 and the upstream graph. Unrequested imported default bodies
cease to block the first ordinary provided-method execution.

## Work and exit evidence

| Owner | Immediate work | Next execution evidence |
| --- | --- | --- |
| A, PA29 | Finish selected declaration/member joins, demanded defining environment and the explicit public Provided codec | Encode/reload the complete provider, resolve actual Holder calls in a fresh consumer, prove original and fresh instances, then native carry/borrow output 7. Unrequested defaults remain present/Pending; qualify their demanded cases afterwards. |
| B, PA25–28/31 | Join Pattern/NonNull, PhantomData, caller_location/track_caller and real panic source | Actual NonNull operations, then real Location/PanicInfo/handler and a false assertion reaching Abort. Add nonempty formatting against actual source constructors and methods. |
| C, PA22/30/31 | Finish foreign opaque VTable declarations (LIB-032) on the shared Pattern/Pointee base | Advance unchanged-core loading past VTable; combine genuine Thin use with the integrator's alias work. Unknown Foreign value layout must coexist with correct thin pointer metadata. |
| Integrator, PA19–21/29/31/32 | Repair regressions, integrate shared interfaces, close aliases needed by actual Thin and maintain the original-source frontier | Validate unused declaration obligations as well as actual uses; refresh core after relevant changes, then produce the real core artifact and separately compiled consumer. |

Keep three implementation workers. Their tasks follow independent observed
blockers. The integrator reserves time for the actual upstream path and joins.
Do not create another helper-only milestone when related consumer work can be
qualified together.

The next scheduling boundary is a working consumer, not another readiness flag.
A retains the real no-import source caller and corrected `no_core` extern-prelude
lookup. C retains dynamic lifetime opponents through the existing body-region
consumer, since successful type analysis alone has not proved regions.
B preserves original NonNull attributes,
but documentary/lint claims must not acquire invented semantic prerequisites.
Shared source components may move between workers before the whole lane passes,
with failed/pending checks retained; only the combined real consumers qualify
their integration. Do not create extra build trees for that handoff.

B's simplification follows pinned fmt/rt.rs and const-eval validity/call code:
pointer formation and actual call ABI checks are distinct. Preserve original
canonical function identity, destination type and real unsafe/call obligations.
Do not invent numeric targets or runtime traps. SourceSelect stays unchanged.

C's automatic metadata rules apply only to a genuinely selected builtin source.
Pinned observations show an explicit caller Pointee assumption without equality
can keep Metadata opaque under Sized. A changed unresolved tail returns its
genuine Metadata projection to the existing normalizer stack. Only unchanged
unknown Self asks the real scoped Sized child. Forming DynMetadata does not
consume its fields/layout; all required nominal definitions still undergo
ordinary defining-crate checking before crate success.

## Crate-first priority

Use unchanged upstream source and actual artifact consumers to choose work.
Static source search currently finds Thin as the written trait-alias declaration
in core/alloc/std; this is not an expanded-graph census. Prioritize its general
bound/equality semantics before unrelated alias corners. Preserve every broader
requirement and honest unsupported diagnostic. Replace a blanket fence only
when the replacement validates the actual program's declarations and uses.

Track loading/expansion, declaration/body checking, artifact/object emission and
native consumers separately. Existing phase views may expose independent
blockers; a successful parse is not a compile. Repeat original-core attempts
after relevant behavior changes, rather than every helper. Keep source/config
identity and the reached phase in each observation.

Inspect alloc/std dependencies and backend operations during core work. Prepare
build/edition/runtime interfaces against small genuine consumers. Active SIMD,
target features, f16/f128 and assembly cannot be waived by later PA ownership.
Record actual typed operations and backend capabilities before proposing any
separately reviewed backend change.

Develop the necessary portions of PA25–PA34 in dependency order rather than
waiting for each broad assignment to release. Keep a separate closure checklist
for every owning PA. A first working core path does not release an unfinished
assignment; conversely, unrelated assignment closure must not delay testing that
path. Reassess the allocation of three workers whenever a lane is waiting on a
shared interface, rather than adding another competing implementation.

After core, build original alloc and allocation/destruction consumers, then std
and platform consumers. Exercise the pinned core-only crate graph as soon as
core and separate compilation permit it. No patched sources, synthetic library
algorithms, rustc-built target libraries or false cfg substitutions qualify.

## Validation and integration

- Use cached dev-fast builds and coherent batches of exact controls within five
  minutes. Keep formatting, strict Clippy and placement audits. Settle interfaces
  and avoid near-limit functions before freezing a candidate. Do not duplicate
  Clippy with another full check command.
- Select private and public consumer controls together before freezing an
  integration, using the existing runner's mixed selections. The latest separate
  221.05/150.55-second batches total 371.60 seconds; they are not a five-minute
  end-to-end integration. Avoid repeated static setup and builds of the same
  source. Retain worker checks when their exact inputs remain valid, and measure
  the combined critical path before claiming a faster integration. Do not add
  another test runner or drop strict Clippy to hide that cost.
- Run the real selected source/native acceptance when behavior changes. Build a
  CLI only when invoked. Retain exact unaffected receipts and independent source
  observations with provenance; source/LowIR equality can retain unchanged native
  results. Never relabel retained execution as fresh.
- At integration, a relevant broad CI failure can justify reusing the already
  built private executable for its ordinary tests: the latest release private
  run executed in 2.26s. The corresponding repaired dev-fast executable now
  passes all 1,782 ordinary private tests in 35.39s, without rebuilding. Keep
  this check within the same five-minute budget. This adds no release build,
  native sweep or per-edit full-inventory
  obligation; do not spend 14 minutes in CI to discover another cheap local
  regression in an executable already available.
- Run the shared `make smoke` selection at push/integration: source hooks, CTFE,
  source diagnostics, generic imports and lowering, alongside lexer cases. Full
  release/text/native/debug/independent checks remain scheduled/manual;
  investigate failures and promote root causes into smoke.
- Extend one reusable qualification runner for actual consumer needs instead of
  inventing a workflow/receipt format for each helper. Preserve failures, hashes
  and exact counts. Consolidate helper documentation at semantic milestones.
- Split frontend/LowIR preparation from backend execution in native-only tests
  when that exposes a cheap independent regression. Keep native acceptance as
  well. The cross-edition donor failure is the first concrete candidate. Reuse
  built integration executables at coherent joins rather than requesting a
  separate release build to discover frontend-only failures.
- Check uncertain language restrictions against the pin before building a new
  proof mechanism. The alias-default observation retained two wrong rejection
  predictions: a default violating a bound inherited through another alias is
  accepted when unused. Written declaration bounds still require validation.
  Preserve the difference between language validity, representation validity,
  optional optimization and lint-only attributes.
- Keep changed-path algorithmic scaling and work/storage invariants. Measure
  representation changes and complete semantic checkpoints. Broad ABBA/RSS and
  release matrices run at meaningful qualification boundaries.
- Preserve three-job/CPU/RAM/thread caps and the 150-GiB reserve. Keep one useful
  incremental cache per active worktree, clean disposable outputs, retire obsolete
  pins and losslessly archive historical evidence. Completed observations do not
  need their executables forever.

Report actual crate/artifact/native milestones and remaining joins. Do not derive
a finish date or percentage from helper/test/opened-file counts. A dependable
estimate first needs the complete selected core frontend blocker census and an
actual imported generic consumer.

## Follow-through at this checkpoint

The integrator has repaired the CI replay helpers and early-usage pipe handling,
and updated the four stale numeric-pointer assertions without changing their
rejection outcomes. The current code passes all 1,782 ordinary private tests
in 35.39s using the existing dev-fast executable. Four affected memory/CLI
integration groups also pass; two unchanged memory groups retain their actual
passing streams from the initial batch, whose later assertion failure is kept.
The corrected option/CLI batch passes in 31.47s, including strict production/CLI
Clippy, formatting and architecture checks. Private strict Clippy also passes.

Unused direct alias defaults now reach the existing default proof instead of
being hidden by the early blanket execution fence. The fence remains at the
end of declaration checking. Five frozen inputs and ten independent metadata
observations distinguish written from inherited default bounds; the original
two wrong predictions remain recorded. No complete alias execution, assignment
release or original library artifact is claimed.

The shared push smoke now uses the existing fastcheck runner through `make smoke`;
the workflow uses that same target and passes push CI.
A reusable source-observation runner remains separate work. New custom gate wrappers
have already caused avoidable serialization/launcher failures, so workers now
use standard exact private selections and share reviewed source dependencies.

The subsequent local integration 4a23c7297 combines A's qualified names/receiver
shapes with C's qualified source Pointee consumers. Nineteen exact private tests
and private strict Clippy/global audits pass in 221.05 seconds. Three ordinary
CLI/frontend controls then pass with production/CLI strict Clippy in 150.55
seconds. These are two separately scoped development batches, not one combined
five-minute run or a full native qualification. Receipts are
`obj/qualification/fast/1793039582331443219/receipt.json` and
`obj/qualification/fast/1793040263229452879/receipt.json` in the integrator tree.
Full CI's native donor-record failure is subsequently repaired and verified by
ordinary and native focused checks; see the
[never-fallback record](pa34-never-editions.md). The execution review above
records the subsequent full run's distinct Pointee assertion failure and its
qualified worker correction. Previously skipped full native/debug/independent
steps still need qualification. No original-core advance is claimed for these
joins.
