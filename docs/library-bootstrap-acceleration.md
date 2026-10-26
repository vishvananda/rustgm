# Library bootstrap execution plan, 2026-10-26

The objective remains every assignment requirement needed to compile original
pinned core, alloc and std and execute their consumers. This revises execution
order and internal preparation policy. Source pins, Rust semantics, assignment
completion criteria and the frozen CPPGM backend boundary remain unchanged.

## Assessment

Useful prerequisites are accumulating faster than end-to-end crate evidence.
Public compiler 637f5678 includes source Abort, source transmute, const formatter
pointer erasure, alias clauses/naming, shared imported mapping and a Pointee
declaration producer. There is still no core artifact. The latest qualified
original-core observation passes Thin syntax and stops at the foreign
`type VTable;` declaration in ptr/metadata.rs:170. It takes 1.873 seconds, verifies
all 722 source inputs and opens 65 files. That is neither 65 compiled modules nor
a completion percentage. Full alias/Pointee semantics remain separate blockers.

The largest bottleneck is the implementation joins. PA29 still lacks genuine
imported namespace installation before final name resolution, complete member
conformance and demanded original/instance execution. Private Ready states do
not establish an encoded artifact or a named imported source call.

Some internal restrictions also exceed the language requirement: eager imported
body-environment reconstruction, exact-original-signature recovery at transmute,
and speculative DynMetadata field snapshots. Pinned source and independent
observations determine the obligations of each actual consumer.

Validation also needs consolidation. Run 40347068121 takes 150.13 seconds;
its 32 tests execute in 0.215 seconds. Static checks and compilation dominate.
The normal push workflow currently selects lexer tests. Scheduled 40297261891
later exposed ordinary bootstrap diagnostic regressions and stale assertions.
A green narrow push is not complete qualification.

The newer full run `40368263999` took 14m15s and failed: 1,779 private tests passed
in 2.26s; two publication-replay helpers omitted newly required completed callee
facts. Three memory controls retained obsolete numeric-pointer diagnostic text,
and one CLI test hit a pipe race when an invalid option closed stdin early.
These are actionable regression controls, not a reason to repeat full release
qualification on each edit. Native/debug/independent steps were skipped, so this
run does not qualify them. The earlier successful-step evidence remains scoped.

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
| A, PA29 | Checked trait/impl/member declarations, genuine namespace installation and demanded body lookup | Encode/reload the complete provider, resolve actual Holder calls in a fresh consumer, prove original and fresh instances, then native carry/borrow output 7. Unrequested defaults remain present/Pending; qualify their demanded cases afterwards. |
| B, PA25–28/31 | Actual Pattern/NonNull source declarations, transparent storage and pointer validity; formatter erasure is integrated | Unchanged NonNull operations, followed by real Location/PanicInfo/handler and assertion/formatting consumers. Reuse C's shared metadata classifier rather than building another aggregate graph. |
| C, PA22/30/31 | Finish actual Pointee consumers, then foreign opaque VTable declarations (LIB-032) | Real source unit/length/tail/Dyn projections with equality/opacity and actual region checking, then unchanged-core loading past VTable. Foreign metadata does not imply known size. |
| Integrator, PA19–21/29/31/32 | Repair regressions, integrate shared interfaces, close aliases needed by actual Thin and maintain the original-source frontier | Validate unused declaration obligations as well as actual uses; refresh core after relevant changes, then produce the real core artifact and separately compiled consumer. |

Keep three implementation workers. Their tasks follow independent observed
blockers. The integrator reserves time for the actual upstream path and joins.
Do not create another helper-only milestone when related consumer work can be
qualified together.

The next scheduling boundary is a working consumer, not another readiness flag.
A keeps the real no-import source caller while correcting `no_core` extern-prelude
lookup; explicit imports cannot hide that defect. C checks dynamic lifetime
opponents through the existing body-region consumer, since successful type
analysis alone has not proved regions. B preserves original NonNull attributes,
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
- Add a small shared ordinary-program smoke set to push/integration checks:
  runtime hooks, CTFE, source diagnostics, generic imports and lowering. Lexer-only
  push coverage is insufficient. Full release/text/native/debug/independent checks
  remain scheduled/manual; investigate failures and promote root causes into smoke.
- Extend one reusable qualification runner for actual consumer needs instead of
  inventing a workflow/receipt format for each helper. Preserve failures, hashes
  and exact counts. Consolidate helper documentation at semantic milestones.
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

The shared push smoke and a reusable source-observation runner still need
implementation. The current push workflow continues to select lexer tests;
publishing its workflow change requires credentials with workflow-write scope.
Local work uses the existing standard fastcheck runner. New custom gate wrappers
have already caused avoidable serialization/launcher failures, so workers now
use standard exact private selections and share reviewed source dependencies.
