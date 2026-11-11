# RustGM single-worker continuation — 2026-11-11

This current section supersedes historical snapshots below. The user asked for
one worker: do not spawn agents or create worktrees. Use existing B
`/mnt/vizier/rustgm-scratch/rustgm-pa26-work`; main is Git-only. Goal: original
core, then alloc/std. No original library artifact exists.

## Current clone checkpoint — local qualification complete

[Source Deref proof ownership](docs/pa26-deref-clone.md) fixes the reduced
RefCell::clone failure: a bound-selected Clone member cannot impose caller-only
proof on its implicit Deref call. The existing sparse index retains the selected
member, and actual protocol calls use normal complete application selection,
including when both calls name the same member with different Self types.
The preceding full43059556041 finished FAILURE on exact eae11a06a/source655:
one unchanged result-self-context negative was accepted. General early result
hints now require the unresolved result to reach actual formal inputs; the exact
old regression and valid shared-Self controls pass. Backend/storage/limits fixed.

New packets own40 independently checked outcomes: PA26ordinary13, PA30native2,
PA31source25. All25 source typing/24 complete prepared-root outcomes qualify;
one valid borrowed-reference return keeps its required artifact pending in
PA31 pending.json. Ordinary15 including native2 remain required behind imports.
New actual source native programs print deref-clone23/123 and deref-member23/12.
Final public15/121.07s, native4/83.66s, scaling3/63.24s pass selected strict
Clippy/fullfmt/audit. Width8/32/128 inference6492/23580/91932, traits55/199/775,
method steps43/probes1; noise128 changes no counters. All57 old bytes unchanged,
all40 manifests/independent outcomes, upstream128/Reference126/harness315/468roots
pass. No positive inversion, oracle generation or whole-assignment activation.

ImmutableCLI5d434e3a2eee340faa9172c9e70d6af18b63ecfe4d8251fcaa90451be728ee8c
at Bobj/parallel/refcell-clone/candidate-dev-fast binds clean source5cd41810b,
empty patch/all3906Rust hashes. Thirty exclusive A/A+twoABBA accepting controls
match complete outputs versus0912; pairedwall1.015/.957/.941, CPU1.015/.957/.941,
AAwall.647/.849/.871. Slower/noisy retained; no optimized/neutrality/core-speed claim.

EXACT120s original observer ends121.159034s,123partialrecords/13,428itemattempts,
lastfunction7265,file202:105885,line2987 fmt/mod.rs. ActualRefCell::clone
function3770/body112:49632 and later cell bodies reached; NOfile112failure records.
Clone/replace/clone_from/comparison failures absent in this reached prefix.
FIRST REMAINING ArrayChunks::next,iter/adapters/array_chunks.rsfile133:2226,line66,
self.try_for_each(ControlFlow::Break).break_value(), receiver has no inherent
nominal method. Later adapter projection failures stay separate. ProgressSHA
56159896464f2c90c80b1826b3e1636876870f20a591de499ede030834a80745.
Postcheck1794422695891811776 verifies722/375opens/six/immutable/current3906/profile/
limits. NOcompletecensus/originalartifact; partialcounts NOTthroughput. NEXT reduce
actual first adapter call; do not extend120s or repeat for longer prefix alone.
Full semantic successor due after publication and fresh no-active-full check.
Keep all requirements below. Smaller bounded reservations11–12GiB fit approved16;
resource reserve150GiB and compilerbudgets unchanged.

## Published callback predecessor

[Formation versus invocation](docs/pa25-callback-formation.md), source5798bd7f1,
clears the reached LazyCell<T>::default initializer failure. Function-value
creation pays ordinary bounds; actual calls retain conditional const obligations
using existing typed clauses/bindings/selection. No new proof engine or permission,
backend/storage/limits changes. Canonical FunctionItem arguments reuse the existing
known_arguments API. Required const-trait members retain actual-call checking.

Earliest PA19=3/24=2/25=37/31=4 packets freeze46 independent Rust object outcomes,
37accept/9reject. All46 declared type phases and44 complete LowIR outcomes pass.
Two paid alias/reference full LowIR positives remain REQUIRED: typing succeeds,
existing checked-body literal-callee identity gate still fails. No positive was
inverted. Twelve creation/paid-invocation width8/32/128+noise128 bounded-work checks
pass. Three new real Rust/CPPGM executions check23/23/41; old priority and actual
destructor execute. Public1794402541806766803 passes6exact/206.71s; native
1794402928426840181 passes3exact/75.08s. Selected strict Clippy/fullfmt/audit pass.
Exactly5 reviewed new PA19/24 triples pass; all5,461 old oracle/receipt bytes stay
unchanged. Upstream128/Reference126/harness315/468roots13bins44stages pass.

Immutable CLI05cc3029540dc069cf360821f180521d74278c5f5986a8e8e69255dc7955def5
at Bobj/parallel/lazy-default/candidate-dev-fast/rustgm binds source5798,empty
tracked patch/3903Rust hashes. Thirty exclusive A/A+twoABBA observations match
complete outputs on ordinary import,accepted paid calls128,actual priorityLowIR;
wall.965/1.018/.994,CPU.990/1.026/.961,AAwall1.026/1.019/.693. Slower/noisy values
remain; no neutrality/optimized/core-speed claim. Historical05cc is now retired after successor publication; current0912 and
olderoptimized21f remain available. Comparisonf46 was retired AFTER
replacement publication with explicit scoped prune1794405662867398530. Its
executable is absent, as are94bd/00d2/9109/e7; raw observations/producer remain.

## Prior original-core observation

Unchanged120s observer FAILS deadline121.668676s:118 partial records through
function6558,range/iter.rsfile195:3846,node310. LazyCell default is reached at
function3647,file113:12543 with no file113 failure. First now **RefCell::replace**,
cell.rsfile112:35607,line1026, incompatible types at
mem::replace(&mut self.borrow_mut(), t). Subsequent clone/clone_from/partial_cmp/
cmp records remain separate. Reduce the actual mutable-borrow wrapper coercion;
do not infer a shared cloning cause without a contrast. Earlier Cell::replace
shorthand identified only the module. ProgressSHA972dee75d1fc8bcc9cfd77fc38928e798f07ba823cc9f20587fb53b12975d177.
Postcheck1794405273338140972 verifies722hashes/375opens/sixbindings/immutable
compiler/current3903/profile/limits. No complete census, comparable throughput or
original artifact. Never raise the120s window or budgets to hide a blocker.

Prior [method-inference repair](docs/pa20-post-inference-properties.md),sourcef93,
passed EVERY full43019302026 stage on exactbc401434ac3a525781359f6d194a65cf5811aa6f.
Its two generic projected-premise positives remain required in PA20 pending.json
with complete frozen bytes/outcomes. Full callback43034322443 PASSED every stage on EXACT357d0bc5e188e3e8af4da4c264c28f2e3b918d26
containing source5798, dispatched after checking no active manual/scheduled full.
Never cancel/restart for documentation/input descendants. The LazyCell stash was
restored, verified and dropped; no stash restoration remains to do.

Machine record: docs/pa25-callback-formation-evidence.json. Raw evidence:
Bobj/parallel/lazy-default. Checkpoint published and comparison pin retired. The newer original observation and source coercions are below. Keep the paid
alias2, projected-premise2 and every retained requirement below positive/required.

## Current source reference coercion checkpoint — original observed

[Qualified source coercions](docs/pa26-borrow-coercions.md) now use actual selected
Deref/DerefMut calls, preserving original expression operations in a separate
sparse conversion plan. Existing access-call lowering executes once; actual
returned-reference ancestry supplies lifetime proof. A known result hint reaches
call inputs; an incompatible pointee destination cannot rewrite a constructor.
Built-in reference/unsize conversion precedes Deref exploration. An independently
accepted builtin-before-deref regression (baseline05cc0/localWIP1) proves no unused
conditional debt may remain. No new proof engine/global cache/denseNodeFacts,
Frame,Declarations orCallTypes field; backend/storage/limits are unchanged.

Packets now own57 independent outcomes43accept14reject: PA26ordinary16(11/5),
PA30native2(2/0), PA31source39(30/9). All28 original bytes/outcomes unchanged,
38constDeref prefixes and one originalUnsize prefix exact. All39 complete source
checked-body/prepared-root outcomes pass; the2loan negatives typecheck before
actual body rejection. Ordinary18/imports still prevent candidate qualification;
import errors qualify no language rejection. Ordinary2native remain REQUIRED.
Newsource4real Rust/CPPGM executions:23/41/1342,guard23/41/1345,const41 twice.
Oldreceiver23/41 and3callback native executions pass. Six accepted width8/32/128
noise0/128 have inference3653/13253/51653,trait56/224/896,methodwork0; all noise
counters identical. Temporary immutable counter harness retired; raw/hash kept.

FINALPUBLIC1794409791154946495 PASS15exact2targets104.59s includes newsource,
oldreceiver,callback/15adapter/unusedchild,PA18factory contexts and oldsource
Unsize/upcasts. FINALNATIVE1794410023347977560 PASS3exact38.29s runs4newactual
source programs plus oldcontrols. StrictselectedprodCLItestsClippy/fullfmt/audit
PASS. Phases1794410310412904864 verifies57CLItype observations/all3905Rusthashes/
source39API qualification/native4; checks-final upstream128/Reference126/harness315/
468roots13bins44stages andall57 independent identities PASS. No course oracles,
assignment activation or original artifact. The fresh original observation is below.

Draft24 author predictions were checked independently. Two no-result-hint cases
and6unhinted scale drafts are genuineRustE0308, not positives; exact bytes/raw
remain. Separately declared explicit-type6scales accept. Corrected ordinary/source
prefix opponents have independently required1; no Rust-valid outcome inverted.
Alloriginal28 outcomes remain fixed. Machine docs bind the review and failed
exploratory receipts without calling failed batches passed.

SOURCE7c7b1b132 is committed, immutableCLI2b7a8d17e768a1d60303dce88283f30e8f4096c8e6000eb34f976b790bb9ef48,
empty patch/all3905 Rust hashes. Thirty exclusive A/A+twoABBA observations retain
complete outputs on import/builtin-before-Deref/priorityLowIR. Wall1.044/.998/1.029,
CPU1.043/.999/1.029,AAwall.850/1.084/.739; slower/noisy values remain. No optimized,
neutrality or core throughput claim. Its first120s original observer ends122.069393s
with0records/noitems. Four bounded90s phase snapshots vary between declaration
passes and bodies, so no repeatable regression or cause is established. The
non-equivalent hint-revert is not adopted or qualifying.

Diagnostic successors53e2509c1/6556387ca add typed implementation progress and
correct all item events to existing diagnostic provenance, retaining ASTidentity
separately. Four exact checks1794414341537683152/42.85s, strictselectedClippy,
fullfmt/audit pass; final census tables are byte-identical with/without progress.
Current immutableCLI0912e08f677a2880a62b42bef9c5baf3dcd253ceec5e5d805b22a873e34a58a8
at Bobj/parallel/refcell-coercion/projected-progress-dev-fast binds6556387ca,
empty patch/all3905 Rust hashes. Managed dev-fast CLI now matches this source.

EXACT120s original observer ends121.226504s,141partialrecords/14,554itemattempts,
lastfunction9261,file244:58785,line998 unicode/unicode_data.rs. Actual
RefCell::replace(function3753,body112:35560) and clone_from(function3771,
body112:49846) are reached with no prior failure records. FIRST REMAINING is
RefCell::clone,cell.rs112:49655,line1449:
RefCell::new(self.borrow().clone()), trait selection requires an unambiguous
assumption/implementation. partial_cmp/cmp remain separate. ProgressSHA
1444d340ebf3b801acb112bdb9c3369326fbe53138c102c7aace338c83ef5bcf.
Postcheck1794414625686080575 verifies all722/375opens/sixbindings/immutable/current
3905 hashes/profile/limits unchanged. No complete census/original artifact;
partial counts are not comparable throughput. Do not repeat for a longer prefix
or extend limits. Raw new observation: Bobj/parallel/refcell-coercion/projected-progress-original.

PUBLISHED source+diagnostics+D1eae11a06a; B/main/origin clean at publication.
Full43059556041 finished FAILURE on EXACTeae11a06ad731c3ea4d618095daa43a5b1025bb8,
containing source6556387ca; the current clone checkpoint above repairs its sole
unchanged result-self-context regression. Dispatch followed a fresh no-active-full check.
Never cancel/restart it for docs/input descendants. Prior callback43034322443
PASSED every stage on EXACT357d0bc5e/source5798.

NEXT reduce real clone expression using actual ordinary Clone caller debt and
actual Deref selection; do not assume a shared comparison cause without contrasts.
Current0912 and olderoptimized21f are available. After replacement publication,
comparison05cc, historical2b7, diagnosticd77/NONQUAL4f were explicitly unpinned
and pruned1794414887003043616; all four executables are ABSENT. Producers/raw
observations/hashes remain. Do not use those old paths as available binaries.
All prior requirements below remain required, including ordinary18/native2,
paid-alias2/projected-premise2/symbolic-index native2/depth128/factory/tuple/
bootstrap/transport/optimized gates. No backend/storage/source-pin/limit change.

## Retained requirements and resource rules

- [Symbolic source indexing](docs/pa31-symbolic-index.md) retains both new native
  companions as REQUIRED POSITIVES pending genuine source panic signature/caller/
  runtime routing. Both real Rust/C programs print41/23/47/23/41/23/23/47; source
  analysis and CTFE47 pass. Keep the hook and guard. Existing7 native indexing
  controls passed at their checkpoint. No candidate-native parity for these2.
- The [implicit receiver checkpoint](docs/pa26-implicit-deref-methods.md) retains
  full source/generic/region/const/stability/capture/borrow proofs and one operand
  evaluation. Its PA31 Deref packet42 has40 qualified finite contracts and2 valid
  128-layer positives pending inference sharing. Canonical work at8/32 is linear;
  total inference is quadratic, noise128unchanged. No budget raise or negative
  language qualification. PA26's7 ordinary requirements still need bootstrap
  imports. Operation16/MethodsCall20/Plan64/Calls8/Step40/coldselection12 remain.
- Keep optimized nested-proof A/A+ABBA/release-stack checks and older optimized21f
  pinned; avoid cold private/release links in the local five-minute loop.
- Prior d1d55648b+27cef3f64 const-adapter scoped defining proofs/local-child
  completion remain qualified. Its PA25 packet10 and PA31 packet31 retain four
  ordinary bootstrap positives and three source runtime/CTFE execution positives.
  Runtime-self/runtime-forward typecheck but LowIR still stops at callable owner;
  CTFE-self still stops at formal order preparation. General conditional function
  items, bootstrap/imports and authenticated transport remain required.
- Three older tuple-factory cleanup-owner cases remain required. Whole PA25–PA44
  stay planned; earlier finite releases retain their recorded scope. Genuine
  source panic runtime routing is required for the two new native artifacts and
  final core artifacts. No frontend/body/abort substitution is authorized.
- Main never builds. Single worker, existing B, no new worktrees/agents/CPPGM/v5
  edits. Backend remains17b024f69. Every heavy command uses storage/resource guards,
  jobs2,16GiB output cap,150GiB reserve and <=300s feedback. Never raise the120s
  observer window or compiler limits. Preserve input/raw receipts; retire pins
  explicitly and prune scoped output only at an idle boundary.

[PA19implementedtraitscope](docs/pa19-implemented-trait-scope.md) remains published
with34newoutcomes/25native/all1233texts andzeroexistingreferencechanges. The prior
[pending-joincheckpoint](docs/pa20-pending-join-inputs.md) retains its separate
1.266x tinyunoptimizeddirectcontrol limitation. Broadoptimizedqualification remains
pending; no originalcore/alloc/stdartifact exists.

[Incremental census progress](docs/census-progress.md) preserves diagnostics when
the observer times out. Full joined diagnostic CI42820032217 on df8507cd8 now
PASSED every stage. Its semantic successor is due after publication; wait for
any active full run to terminate before dispatching another.

[Failed alias-work accounting](docs/pa21-failed-alias-work.md), compiler67b277950 /
CLI8a653e24, repairs the one real regression in full42805641905. Joined successor
42813128062 on2a75c680b now PASSED every fast/release/course/native/debug/independent
stage. The diagnostic successor is due after publication; wait for any active full
run to terminate before dispatching another.

The [source Unsize upcast checkpoint](docs/pa31-source-object-upcasts.md),
compiler6d3b6d424/CLIec0e1273, supports non-quantified intrinsic object upcasts
with all original equality/marker/lifetime/formation debts. Ten new PA31 outcomes,
seven private controls, a width8/32/128+8192-type noise invariant, three paired
Rust/CPPGM native observers and30 unchanged-path observations pass. Harness314,
upstream and selected strict checks pass. Two custom-auto object forms remain
required; no oracle or backend contract changed. This addresses a shared cause
of17 retained body fences; no complete new census or original artifact is claimed.
Incremental progress above now supplies retained partial diagnostics. No further
observer-window increase is authorized; joined full42813128062 passed all stages.

The [source-probe/body-session checkpoint](docs/pa26-body-place-session.md)
clears the cumulative method-probe guard and a newly exposed stale indexing
place-offset panic. Compiler6c4ae3d04/CLI61c983f6 has passing focused private,
production and21 native program comparisons,314 harness tests and complete-output
paired controls. Full predecessor42800653535 succeeded every stage.

Original722 source hashes/profile/compiler limits stay fixed. New core census
attempts time out at120.97/170.75s without a usable table; these are infrastructure
failures. An interrupted115s immutable snapshot is in body projection
normalization/trait candidate selection; it does not establish a bottleneck.
The last completed table remains119 body failures then method-budget record120
in flatten.rs, TSV e21ad8822d098081b987f654afc5fcfc122cb54f49b9c992029e01030f0b4d4e.
Use its repeated Unsize/range/receiver/projection causes for owning reductions
while completing bounded discovery. Observer --observer-seconds defaults120,
allows5..170 and records the window; hard190 worker/300 local feedback limits
remain. Do not raise the observation window again to hide a timeout. No artifact.

Seven new PA21 scoped lookup outcomes and three PA24 width128 callable inputs
are independently checked.12 focused checks and8 final affected checks pass,
including all three real f01 CI scaling failures. All867 PA21/493 PA24 text
outcomes and three new native executable comparisons pass. Existing oracles
are unchanged; one new Split oracle's private IDs were checked against identical
complete LowIR before regeneration.127 frozen outcomes change only the six
declared PA31 accepts. Thirty A/A+ABBA observations retain complete ordinary
lookup LowIR and74 ordinary outcomes, with no speed claim. Native receipt is
obj/native/pa24/selected/1794303683196999869; current evidence is under
obj/parallel/scoped-lookup, producer candidate2-dev-fast/producer.json.

Earlier path through the declaration frontier:

The [PA28 multiplication correction](docs/pa28-multiply-overflow.md), compiler
541972e4a, clears overflowing_mul after [overflow query](docs/pa31-overflow-query.md)
and [static knowledge](docs/pa31-static-value-knowledge.md) cleared pow and
checked_ilog. The actual source intrinsic returns target-width wrapped bits and
an overflow flag without panic. Genuine Copy/source/prototype/call/const proof
remains. Native narrow multiplication reuses the unchanged widened kernel;
128-bit flags use unsigned magnitudes and a nonzero denominator, covering zeros
and both MIN/-1 orders. Ordinary checked128 guards remain unchanged.

PA28 freezes6 source outcomes and156 independently verified native pairs across
all12 integer types. Compiler knowledge/configuration query policies stayPA31;
typed identity alone does not determine the owning assignment. Production
1794291492523956888 passes3 groups; native1794291677746131719 passes2 groups/five
real programs; private1794291757965593895 passes10 guards. Strict selected/private
Clippy, format, audit,313 harness/467 roots/44 stages/upstream128/Reference126 pass.
Thirty A/A+twoABBA observations preserve complete ordinary multiplication LowIR
at8/32 functions and74 ordinary outcomes, without a speed claim.

Current immutable source-bound dev-fast CLI is
`bf168bc7dd54bd213780b32fd17ce0a700237220d4a34c22c7b2da26b898ccdd`.
It opens375 unchanged sources from722 and checks42310 declarations, none
failed/blocked, bodies0, in79.45s. Next local_annotations record is slice/mod.rs
line2251, byte82861, split's Split::new: quantified clause requires a scoped
consumer. TSV SHA
`9a585c80afd2f737e5359956b85c9be14097442b91abd2bc667dbde5896645e2`.
Inputs/profile/limits remain unchanged; this is discovery, not an original artifact.
This reaches the previously retained Split::new/f01 boundary. The
[PA21 universal-call checkpoint](docs/pa21-universal-call-consumer.md), compiler
ffa610c52, now closes f01 before widening lookup. Nine independent PA21 outcomes,
ten private controls, the existing higher-ranked LowIR group and scoped course
references pass. Actual caller inputs stay distinct from selected scoped debts;
ordinary CallTypes remains728 bytes. Current immutable CLI77ddb733 binds the
committed compiler and repeats the exact original/rooted f01 and valid twins.
Its then-current original722-input census retains the byte-identical9a585c80 Split
table,375 opens/42310 declarations/0 bodies,70.49s. Fifty calibrated A/A+ABBA
observations preserve complete changed-path and74 ordinary outcomes, with the
additional closed-call proof cost recorded and no optimized throughput claim.

Full predecessor42754442914, slice42760129302, combined query42765929708 and
multiplication42773069633 all succeeded in every stage. Full f01 42782379010
failed one private captured-vtable and two public closure-context work guards;
all three now pass locally after retaining computed duties with their owning
consumers and limiting early checks to ordinary trait selections. Later full
stages were skipped. Joined full42793756438 and query-budget42800653535 passed every stage;
source-probe/body-session successor is due.
Wait for an active full dispatch to terminate before dispatching another;
push-fast is separate.

## Next work, one worker

Current single-worker continuation, 2026-11-10 14:06 UTC:

- Main is published7db87a632 (compiler a5c7c21e0). New full CI42826374117 is
  active; fast passed, qualification was in release/course tests at14:03.
  Do not dispatch another full while it is active. Predecessor42820032217 passed.
- Work remains in B, no workers/new trees. Current immutable CLI6923e768 is
  obj/parallel/inclusive-calls/candidate-dev-fast/rustgm. The comparison792f38b9
  remains in census-progress; optimized nested baseline21f66 remains pinned.
  Older alias8a653e24 and upcastec0e1273 binaries were retired/pruned with
  record1794317914176088955. Preserve backend pins and old pins that protect
  source/encoded input bytes rather than obsolete executable comparisons.
- Next projection investigation has NO compiler edit. All exploratory inputs,
  declared hypotheses and raw observations are under obj/parallel/projection-casts.
  Ordinary direct/generic-equality scalar casts, primitive/overloaded indexing,
  source Index casts, correctly supplied standalone closures and known-input
  three-level array/slice/selector forwarding all pass baseline6923. Do not
  rerun those controls. Missing callable/const-stability/Eq provided-member
  prerequisites and genuinely ambiguous uncalled closures are retained failed
  hypotheses, not evidence of the core bug.
- The next probe is probe9.py, plan probe9-plan.json: an inferred forwarding
  closure is followed by a real f(1usize) call; typed, generic-parent and local
  constant companions are declared Rust-positive. This should test whether a
  future caller supplies the argument type after early cast preparation.
  It has NOT executed: probe9.log and probe9-retry.log are admission failures.
  baseline9/ should not exist until the guarded driver actually launches.
- Storage is currently the blocker for that probe. At14:05,156.5 GiB free vs
  159.6 required with the unchanged16 GiB cap/150 GiB reserve. Routine tidy
  removed7 tiny artifacts (1794319267274421841); tidy-archive was refused at
  154.3/154.6. make tidy-build completed (1794319379538845190), retiring Cargo
  cache output while preserving pins/evidence. A same-cap retry still refused;
  shared/cache accounting means this cleanup increased unused cap reservations.
  Do not lower caps/reserves or clean another checkout. Wait for external free
  space before retrying; this turn's resource blockage is the first blocked audit,
  with substantial completed work earlier in the same turn. Goal stays active.
- At admission, retry exactly one heavy command with a fresh log:
  python3 scripts/resources.py run --max-output-gib 16 --
  python3 scripts/storage.py run --prune-under obj/parallel/projection-casts/baseline9
  -- python3 obj/parallel/projection-casts/probe9.py
  Inspect every outcome before
  assigning the reducer to a PA or changing cast preparation. Keep source frozen
  until the wrapper exits. The first core record remains slow.rs POWERS[n] as
  usize; prepare_pointer_cast and the function-pointer cast guard both inspect
  source heads before final inference. This is a hypothesis, not a proven cause.


1. Use retained incremental progress to batch reached failures by shared cause:
   Projection demand, unresolved call inputs and receiver lookup lead the list.
   The first retained expression is POWERS[n] as usize in dec2flt/slow.rs.
   Freeze independently checked reductions in the earliest owning PA.
2. Inspect the joined successor full CI when available and repair actual failures;
   preserve pending optimized nested-proof measurements and all remaining debts.
3. HR transport may start now that record variants landed. Its approved notice
   and existing scratch tree remain available; no HR implementation exists.
4. Client exports remain incomplete: source-client availability and an actual
   three-crate Rust witness are required. Backend v3 remains frozen.

## Evidence, measurements and tools

- Durable decisions: /mnt/vizier/rustgm-scratch/integrator-20261109/pending_decisions.md.
- Current compiler/test source hashes are bound in pa26-work's
  obj/parallel/inclusive-calls/candidate-dev-fast/producer.json. Current receipts are
  under obj/parallel/inclusive-calls; diagnostic receipts are
  under obj/parallel/census-progress; semantic receipts are
  under obj/parallel/alias-query-work and obj/parallel/source-upcast; body-session receipts are under
  obj/parallel/place-session and obj/parallel/source-probe-budget; scoped lookup is
  under obj/parallel/scoped-lookup; f01 receipts and
  f01 evidence are under obj/parallel/scoped-inherent. Multiplication receipts and
  census are under obj/parallel/multiply-overflow; prior query evidence is under
  obj/parallel/overflow-query, and static-knowledge evidence is
  under obj/parallel/static-knowledge, slice evidence is under
  obj/parallel/radix-slice, and joined proof/layout evidence is
  under obj/parallel/reuse-ci and obj/parallel/atomic-layout. All temporary production/test diagnostics are gone.
- Current multiplication observations retain30 full-output-equal A/A+ABBA rows:
  ordinary LowIR at8/32 functions plus74 controls, ratios0.937/1.088/1.003,
  A/A0.693/1.134/1.019; no timing claim. Prior overflow ordinary-name/74 controls
  retain20 full-output-equal A/A+ABBA
  observations, ratios0.883/1.007, A/A0.910/0.985, without a speed claim. Earlier
  static-knowledge ordinary-name/74 A/A+ABBA retains20 full-output-equal observations,
  ratios0.956/1.001, A/A0.733/1.154, with no timing claim. Earlier strict-str/74
  control A/A+ABBA retains20 full-output-equal observations,
  ratios0.966/1.003, with no timing conclusion. The earlier receiver benchmark preserves30 accepted same-profile A/A+ABBA observations
  with identical complete LowIR and a real exported return7/calling entry point.
  Dev-fast ratios at8/16/32 are0.815/0.651/0.437;74 ordinary inputs retain10
  complete-output-equal observations, ratio0.991. No optimized speed claim.
  The first emit-types comparison stops on intentionally removed private type
  inventory rows; the second LowIR setup omitted main and is disqualified.
  Both failed/setup observations remain retained. No oracle was regenerated.
- Never build in /home/vishvananda/rustgm. Use existing registered scratch trees,
  storage/resource wrappers, one heavy command at a time, explicit16GiB output
  caps and the150GiB reserve. Do not lower reserves for admission. Use guarded
  cleanup at idle boundaries; keep only current immutable comparison pins.
- Standing rm rule: literal, inspected absolute paths only; bulk cleanup goes
  through storage.py/make tidy. Commits have single-line subjects and no trailers.
  Do not modify cppgm-extended or v5rust, or raise the pinned v3 backend.
- resume_record_variants.py retains explicit restart stages and source-hash
  joins. Old joined_d7.sh/fresh.py are historical runners, not the next sequence.

---

## Historical multi-lane snapshot (superseded state; retained restart details)

# RustGM handoff — 2026-11-09

The goal is unchanged: **complete all assignment requirements to build the
original std crates**, starting with core. Work follows the
[core census and transport plan](docs/core-census-plan.md) and
[docs/parallel-work.md](docs/parallel-work.md). No original core, alloc or std
artifact exists yet. Don't infer assignment completion from a census count or
a focused checkpoint.

Standing user instructions:
- The integrator makes decisions without blocking on the user and records
  their tradeoffs.
- Commits use the repository's single-line subjects with no trailers (no
  Co-Authored-By).
- Never build in `/home/vishvananda/rustgm`; build only in the
  `/mnt/vizier/rustgm-scratch` worktrees.
- Don't create more worktrees.
- Don't edit the cppgm-extended or v5rust checkouts.
- The backend stays pinned to v3 (`17b024f69`). Only the user raises backend
  extensions.
- `rm` takes only literal, inspected absolute paths: no variable or command
  substitution, globs, `xargs rm`, `find -delete` or `-exec rm`. Bulk cleanup
  goes through `storage.py prune` or `make tidy`. Repeat this rule in every
  subagent brief.

## Main and CI

The last compiler commit on `origin/main` is **`198fdea9b`**. The handoff
commits after it change only `handoff.md`. Landed in this session, newest
first:

| Commit(s) | Item |
| --- | --- |
| `375c125e7`, `f500e9692`, `198fdea9b` | Lane A census batch 21 (empty callable binders in method applicability, D1 dedupe through the lookup-local `ScopeConditions` reader); Lane C's (ii) rulings docs and associated-unsafe functions under version 13 |
| `d9f6ad4f6`..`65670125b` | Lane B item B, negative premises in nested trait proofs |
| `a6be80084` | Lane C (ii), zero-slot nominal fn fields |
| `0bcba9e3f` | batch 20 |
| `20e079fa3`, `fbe18b8a8` | F target features and the stream witness |
| `99f79b8d0`, `c8c2a8eda` | R2 pointer calls |
| `4df942467` | batch 19 |
| `b2a14ea76` | F0 |

Full CI is workflow `381630191`, dispatched with
`gh workflow run 381630191 --ref main`.
- A push runs only the fast job; a dispatch adds qualification.
- A new dispatch cancels the one in progress, so wait for it to finish.
- Passed: 42659856520 (`0bcba9e3f`), 42668881267 (`a6be80084`) and
  42679123457 (`65670125b`).
- **42689087036 (`198fdea9b`) was still running at handoff.** Check it first:
  `gh run view 42689087036 --json status,conclusion,jobs`.

Census 24 used batch 21's pin `88273b11` at
`rustgm-pa29-work/obj/parallel/census-batch21/qualified-1/pin/`, which is the
current census baseline.
- It checked 42,310 declarations, with none failed or blocked.
- **It found one record:** `local_annotations` at
  `iter/traits/iterator.rs:698`, in `Intersperse::new`, failing with
  "candidate properties require associated type normalization".
- Batch 22 targets that record.
- The next known blocker after it is `slice/mod.rs:2251` `Split::new`, with
  "quantified clause requires a scoped consumer". That is batch 21's
  bound-region fence, and the provisional lead for batch 23 (A+O).

## Integrator state and procedure

**The durable integrator kit is `/mnt/vizier/rustgm-scratch/integrator-20261109/`.**
- `pending_decisions.md`: the append-only log of every ruling and its
  tradeoffs this session. Read it before ruling on anything in flight.
- `joined_common.sh`, `joined_d7.sh` and `admit_retry.sh`: join runners.
- `tools/`: `fresh.py` dedups selections across stages, `verify_receipts.py`,
  `warm_fastcheck.py` and `build_cli.py`.
- `keep_both.py`: doc conflict resolution. Use the default, ours-first, for
  `core-census-plan.md`, and `theirs` for `library-build-blockers.md`.
- `d7-private-a0.args` and `joined_d7.out`.

Join pattern:
1. Cherry-pick the lane's commits onto main in a temporary
   `integrate/<item>` branch in the main checkout. Resolve doc conflicts
   keep-both, and verify that the +/- lines match the lane's commits.
2. In the lane's worktree, check that the tree is clean, then write
   `obj/parallel/integrator-hold`.
3. The script switches to a temporary branch at the joined sha, runs the
   stages and switches back.
4. Watch for `rc=[1-9]|abort|never admitted|fatal|switched back`.
5. If everything passes, fast-forward main, push, delete the temporary
   branch, release the hold with `rm` on its literal path, and dispatch full
   CI.

Rules for joins:
- **Join one at a time.** Machine load is about 20, and the 300 s
  fastcheck budget includes admission.
- Put the pa31 stages last, and split pa31-other into its a/b/c receipts.
- `fresh.py` marks tests as seen when it selects them, so a stage that fails
  hides its tests from later stages. Rerun that stage's exact args afterwards.
- Before a join, check that every selected test still exists at the joined
  sha. Pre-seed SEEN with tests that were removed or renamed.

Rules for outcomes and flips:
- No accepted→rejected flip where rustc accepts.
- No rejected→accepted flip without rustc returning 0 and the integrator
  approving it by name.
- Outcomes are declared before runs. A failed prediction is a stop and report;
  an amendment is declared before it is rerun.
- Version 13 additions go only under `body_nodes()`, with presence derived.
  Existing packets stay byte-identical, and older readers reject by name.
- Amendment numbering: amendments already landed keep their numbers. A rebased
  item takes the next free numbers, with truthful timestamps and a
  `renumbered` field.

## Lanes

Each lane is a background agent with a worktree under
`/mnt/vizier/rustgm-scratch/`. All four were wound down at a safe point; each
wrote a restart note. **No integrator hold is active.**

| Lane | Worktree, branch, HEAD | Item and state | Restart note |
| --- | --- | --- | --- |
| A | `rustgm-pa29-work`, `implement/census-batch22-20261109`, `198fdea9b`, clean | Census batch 22. Notice `obj/parallel/census-batch22/drafts/notice.txt` (f76db5c9) is approved: P1, which leaves an unresolved projection in `types/infer/properties.rs` UNKNOWN. Base pin `pins/base-198fdea9b` (259d0ebe). T1 is done and reverted: rustc matched every row, and P1's premise is confirmed. **The four T1 departures (`probes/p1/observed.txt`, a7be5dd8) are ruled below; relay them to Lane A.** Next: P1 candidate build and probe run (which confirms h09's text), then fixtures, code, invariants, survey, qualification, census 25. Retire the T1 pin 2f9dc9bb. | `obj/parallel/census-batch22/handoff.md` |
| B | `rustgm-pa26-work`, `implement/nested-proof-reuse-20261109`, `b621764b0` | Reuse item, which makes 2^d − 1 nested normalizations linear. Draft 3 of `obj/parallel/lane-b-reuse-design-notice.md` (020e6c9e) is approved. Committed, not on main: the behavior-preserving preparatory commit `b621764b0`, which adds the nesting level and `Caller::root`. Its focused fastcheck passed (`1794254952115849884`). **Its byte-identity gates (74 twins, survey, golden table) have not run.** A shared-storage admission refusal stopped them at 239.0 GiB free against 241.7 GiB required. A dev-fast CLI build for `obj/lane-b10/prep-dev-fast/` was still finishing at handoff, so verify its hash before using it. Kept pins: `obj/lane-b10/base-dev-fast` (43818e28) and `base-release` (21f66e51). Four notice edits are still owed (listed in the note). Next: the gates (`gates-retry.sh prep <HEAD>`), then checkpoint 2 (ledger in `infer::Context` and the depth budget at L_nest = 93), then checkpoint 3 (reuse). **Checkpoints 2 and 3 land together.** Rebase onto main after (i) lands. | `obj/parallel/lane-b-reuse-handoff.md` |
| C | `rustgm-async-work`, `implement/guarded-methods-20261109`, `b3a89c643` | Higher-ranked fn pointers (the HR item). Notice `obj/parallel/hr-callables/notice.md` is approved, with rulings folded in. **Waits for (i) to land, then branches from main.** No code and no pins yet. | `obj/parallel/hr-callables/handoff.md` |
| D | `rustgm-backend-work`, `implement/record-variants-20261109`, `5e5665580` | Item (i), record variants: `7047a05d9` (i128 slot control) and `5e5665580`, rebased onto `198fdea9b` and union-merged with Lane C's associated movers. **Ready, but its join is incomplete.** Keep pins `base-198fdea9b` (0680f6ad) and `cand4` (0b096be3) until (i) is on main. Next item: client exports. It has an unsent draft, `obj/parallel/client-exports/notice-draft-20261109.md` (d1771b19), which still lacks its fixture plan, invariants and survey; the rulings it needs are listed below. | `obj/parallel/record-variants/handoff.md` |

## Rulings to relay, and rulings still open

**Lane A, batch 22: ruled at wind-down; send these on resume.**
1. h06 stays rejected, with its observed base text "no applicable trait
   method for this prefix".
2. h07 is already accepted on the base through `Make::new`.
   - It becomes an accepted control that must stay accepted, and leaves the
     known rejects-valid list.
   - Drop the error-propagation blockers entry for the `?` that loses the
     whole lookup, because nothing witnesses it. Record nothing without a
     witness.
3. Correct D0's wording: the projection reaches demand through its Sized
   builtin, while Clone is a polled trait condition. P1 is unchanged.
4. h13, a method call on an unresolved receiver, becomes invariant (i)'s
   method-call shape.

**Lane D, client exports: open.** Lane D's probes found:
- Both fences sit in the source client's legacy interface (SCHEMA 1), and a
  source client cannot publish a version 13 packet.
- Any Rust consumer of a source client first stops at "imported API
  availability is unknown in its metadata profile".
- The pinned backend passes `__m128` in xmm0 and exits 42 natively.
- rustc 1.98 accepts every probe.

The draft recommends B1 (a pointer node) plus B2 (a source-owned nominal
reference) in the legacy interface. It needs rulings on:
- A versus B;
- B2's identity scheme and consumer;
- whether the availability error is fixed first or as a follow-up.

The fixture plan, invariants and survey come after those rulings.

## Next integrator actions, in order

1. **Check CI 42689087036.** If it failed, investigate before anything else.
   **Check shared storage:** `/mnt/vizier` had 239 GiB free at handoff, just
   under the 241.7 GiB the resource controller required for Lane B's gate.
   Run `make tidy`, then `make tidy-archive`, at idle boundaries in each
   worktree before the next heavy runs. Don't lower any reserve.
2. **Rerun the (i) join.**
   - The first attempt reached `private-a2`. Every stage passed except
     `private-a0`, which was refused with "Another protected RustGM command
     is running; cleanup refused" because Lane D ran probes during the hold.
     That was not a test failure.
   - Rerun from the start: write the hold in backend-work, then delete only
     the literal `/mnt/vizier/rustgm-scratch/integrator-20261109/joined_d7.seen`
     if it exists. The script recreates SEEN.
   - Run `/mnt/vizier/rustgm-scratch/integrator-20261109/joined_d7.sh` in the
     background with a watcher.
   - If it passes: main now carries handoff-only commits on top of
     `198fdea9b`, so fast-forwarding to `5e5665580` is no longer possible.
     Instead, cherry-pick `7047a05d9` and `5e5665580` onto main. Check that
     `git diff 5e5665580 HEAD` shows only `handoff.md`, then push, dispatch
     CI and release the hold. Then tell Lane D, and tell Lane C to start HR from main.
   - Then Lane D retires `base-198fdea9b` and `cand4`.
3. **Lane B:** rebase onto main after (i) lands. Join only when checkpoints
   2 and 3 are both complete.
4. **Lane A:** continue batch 22. Join it, then read census 25's first record
   to confirm or replace the A+O lead.
5. **Lane D:** rule on the client-export notice.
6. **Cleanup:** many old `integrate/joined-*` branches remain in the shared
   repository. Delete them, and each lane's pre-rebase backup branches, once
   their items are on main.

## Recorded debt and follow-ups (not queued unless noted)

**Functions at their line limit.** The next edit that adds lines to any of
these must factor first:
- `declaration_conditions` (99/100);
- `select_projection` and `prove_canonical_trait`.

**Candidate fixes:**
- D3: lift the closed consequences of an empty binder in
  `elaborate_trait_bounds`.
- P2n: normalize projections in properties, after Lane B's reuse item lands.
- h07: a lookup error that kills the whole lookup instead of filtering the
  candidate.
- Body inference's Cursor owners leave Gap B open: bounded, but not summed.
- A frontend worker thread with a larger stack would lift the depth limit
  L_d from 31 toward rustc's 64. This is a `dev/bin` change.
- A canonical-selection rejects-valid at d ≥ 2:
  `Field<U>(for<'a> fn(P^d<&'a U>), u8)`.

**Known rejects-valid recorded with Lane C's items:**
- C, early-region and default-body fences;
- passing a `&mut` parameter on into a call;
- fn-pointer impl Self types;
- fn types as application type arguments;
- generic aliases;
- promoted `&5`.

**For the user:** the alignment of a LowIR i128 slot is unspecified in
`shared/lowir.md`. 16 bytes is observed under v3. It is recorded in the
blockers doc, with no frontend workaround.

**Other queued items:**
- f01, then A+O;
- f03a, low priority;
- `SliceIndex` needs const trait impls;
- the fmt chain still needs HR, the transmute intrinsic transport and the
  client-export fences.

## Resource rules (summary; AGENTS.md is authoritative)

- **Storage:** each worktree has a 16 GiB absolute output cap. Run
  `make tidy`, then `make tidy-archive`, then `make tidy-build` if needed.
  Never lower caps or reserves.
- **Fastcheck:** run it through the wrappers, within its 300 s budget. Don't
  wrap it in `timeout`.
- **Pins:** keep only current pins with `.keep-artifacts`. To retire one,
  remove its file by literal path, then run a scoped
  `storage.py prune --apply --prune-under`.
- **Code:** safe, idiomatic Rust. Clippy all and pedantic warnings are
  errors. No production unwrap, expect, panic or unsafe. No blanket lint
  allowances, and no mechanical file splitting.
