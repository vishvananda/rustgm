# RustGM single-worker continuation — 2026-11-10

This current section supersedes the historical multi-lane snapshot below.
The user explicitly asked for one worker. Do not spawn workers or create worktrees.
The goal remains original core, then alloc/std, with every owning assignment
requirement retained. No original core/alloc/std artifact exists.

## Published and qualified

- Record variants and P1 are published. Full predecessor CI42689087036,
  record-variant CI42701179456 and P1 CI42708775327 succeeded.
- Nested-proof prep/accounting/reuse landed together through77e6b407e.
  [Its initial checkpoint](docs/parallel/nested-proof-reuse.md) records130
  passing private cases, identical8356-row golden table and74/336 complete
  outcome comparisons. PA31 has eight frozen projected-size source reductions.
  Normalization count is linear; total work remains quadratic. Optimized
  A/A+ABBA and release-stack checks remain pending outside the local five-minute
  feedback loop; the old release build took7m20s.
- Full77e CI42736874497 failed one Evidence/Selection136-byte guard (actual144)
  and three actual receiver128 tests with the projection work budget. Other3039
  private and545 frontend cases passed. Later native/debug/independent steps
  were skipped. Do not describe that run as qualified.
- CI repair748ef1aaa retains cold completion detail in the old padded flag word,
  preserves the136-byte guard, and contracts actual selected Deref suffixes
  inside their original proof query. Whole-parent lifetime evidence remains
  mandatory; const/scoped/provisional/equality-dependent cases fall back.
- Projected-layout correction5334dd135 follows the repair. It consumes shared
  typed stored fields, independently discharges a fresh caller's lifetimes and
  charges completed graph/caller/nested work between producer steps. PA31's two
  independently checked alignment reductions freeze the original atomic blocker.
- [The joined checkpoint](docs/parallel/nested-proof-ci-repair.md) passes151
  exact private tests (1794282814262496754),6 production tests including all3
  original CI failures (1794282980906312191), and1 real native nested-borrow
  observer (1794283232090614989; stdout23). Strict selected/private Clippy,
  formatting, architecture,313 harness tests/465 roots/44 stages,128 upstream
  fixtures and126 Reference files pass. The PA31 dereference packet now lists39
  cases, including independently checked static/unpaid lifetime controls.
  Full42754442914 passed every stage. Wait for an existing full dispatch to
  terminate before dispatching another; push-fast uses separate concurrency.

## Current original-core frontier

[Actual nominal source Unsize](docs/pa31-nominal-unsize.md), compilerc31afe8d7 /
CLI00b2af11, qualifies PA31's original nominal-tail fence. Completed existing
storage-tail rules establish eligibility; unchanged arguments use bulk equality.
The actual final field pays source Unsize plus both endpoint formation/lifetime
proofs. Field terms reuse the existing body argument environment, preserving
unresolved elements/type/const inputs. The constructor is shared with body field
access, retaining its owner check, cache and projection continuation. No hot
fields, global scans/cache, compiler limits, LowIR or backend change.

PA31 nominal-unsize manifest owns46 outcomes (33positive13negative), independently
object-checked;31 Rust-native/CPPGM observers return9 23. Source-faithful bytes and
old nominal-unsize-required are unchanged; the latter now passes complete LowIR.
Initial44-case candidatef22/5e passed6exact188.67s but two added independent element-
inference cases failed. Finalc31 sharedFields fixes those, not a new language fence.
public2 receipt1794355091892551059 passes8exact196.49s:46completeLowIR/requiredold,
8-counter width8/32/128noise128, oldsourcearray/object/upcast/erasure plus ordinary
PA22/23nominal41 and oldarray28. Native1794355334645613177 passes3groups65.37s:
31new+oldsourceupcast+oldnominaltransfer/cleanup. Controls1794355520793219579 passes
3groups41.37s including all3 required tuple-factory cleanup-owner boundaries.
Strict selectedprodCLItestClippy/fullfmt/arch pass.315harness/468roots13bins44stages/
upstream128/Reference126 pass1794355460066739241. No oracle/source-pin changes.
Private/full successor remains pending; no cold private/release/all-target retry.

Current immutable00b2af11b5d3e0396043305fbf4164ed5ea9bdf89ba7a3d16561f9fb367a486e
at Bobj/parallel/nominal-unsize/candidate2-dev-fast/rustgm binds committed
c31afe8d78f3e6487b3c4524574dd40a6c341f4e/emptydiff3892Rusthashes. Initial5e pin
retired after replacement by1794355873725556381 (metadata/failed inference evidence
retained); tidy1794355873834134744 removed82 disposable outputs. Comparison2cab was retired after publication and the input-only probes by
scoped1794356864064214354; metadata and observations remain, its binary does not.
Keep current00b and optimized21f for the older pending measurements.
Forty exclusive A/A+twoABBA observations preserve complete ordinaryimport/array
LowIR/sourceknownrepr+74survey; ratios.979/1.084/.959/.997, noisy A/A,
no speed/neutrality claim. Raw evidence under obj/parallel/nominal-unsize/.

Original120s observation still FAILS deadline121.1401s,220failure records/3743item
attempts through function7809,file226:27629,node2318 in slice/index.rs. All722
hashes/sixbindings/profile/limits fixed. Partial progress SHA
6feb6dc8314b77a07327749901e41fb58afffa72d08c9dcc6e703177279e3341.
No as_request or nominal-tail fence remains. FirstIndexRange::fold in
ops/index_range.rs file93 byte4858, known tuple/struct required for
self.try_fold(init,NeverShortCircuit::wrap_mut_2(f)).0; rfold same at6101.
Adapter source in ops/try_trait.rs file95 returns impl [const]FnMut(...) -> Self
with const move closure; wrap_mut_1/2 themselves have invalid canonical selection.
Next freeze ordinary drive(wrap(f)).0 / generic associated callable-result twins
before compiler edits; keep actual const/sourceFnMut/Try roles if necessary.
Determine earliest PA from reduction (ordinary closure PA24 candidate, source
protocol PA31 if essential), preserve actual source counterpart. Do not turn
valid unsupported cases into negatives. Different census prefixes give no speed,
body-count or completion claim. No original core/alloc/std artifact.

Next input-only discovery is complete at obj/parallel/callback-result/.
Probe1eight+probe2seven+probe3six inputs independently Rust1.98object0, with exact
frozen2cab/00b comparisons. Generic FnOnce/FnMut/Fn -> (T,) forwarded through a
function drive<R,F:Fn...()->R>(f:F)->R then .0 fails known-tuple/struct in both
binaries. Opaque Self/named/free wrapped callbacks fail too; explicit result or
explicit R type passes. Direct opaque invocation separately needs known function
type. General custom Make<Output=Wrap<T>>, including generic/opaque/inherited-
output, all pass; function-pointer forwarding and scalar-result cast pass.
Thus earliest owning reduction is PA24 (call-trait result inference); nominal,
opaque, const and actual source roles are unnecessary to reproduce it. Keep
actual original const/sourceFnMut/Try counterparts required. No PA24 packet or
kernel edit yet. Read complete_field in dev/src/sema/bodies/aggregates.rs and
callable-result projection advancement; do not invent a tuple arity from .0 or
infer equality from output layout. Source-compatible constraints must settle.

Full42918510671on216364b72 and full42925327481oncba471220 terminalSUCCESS every
fast/release/course/native/debug/independent/PA25 gate. Pushfast42924479783oncbaPASS.
Published nominal-source checkpoint4a9409a58f41712581476c352747920490c5bd2c; newfull
42930363606 on that exact source/fixture snapshot is ACTIVE, dispatched only after
confirming predecessor terminal/no active full. Pushfast42930175480 separate.
Do not cancel or redispatch this full run for later documentation-only updates.
Three tuple-factory owners and optimized nested-proof A/A/stack remain required.
Singleworker existing B; mainGitonly, no new trees/agents or edits to active CPP/v5.
Whole PA31 remains planned; core compilation remains incomplete.

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
