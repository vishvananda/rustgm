# RustGM handoff — 2026-11-14

Continue the active goal: compile pinned original core, then alloc/std, completing
assignment components as their complete contracts qualify. Work as **one worker**.
**No original library artifact exists.** PA25–PA44 remain planned as whole PAs;
the released113-case PA25 const-machine component stays separate.

Latest continuation: the layout-budget repair is locally QUALIFIED. Baseline
1794638741351903699 reproduces the actual old failure; focused fixed
1794639072978956146 and native1794639354437875740 pass. Source inventory3957.
Shared headroom is restored, with150GiB reserve and16GiB B cap unchanged.
Full43561807969 passes every stage on preceding product source40d29b452.
Release22 still supplies the latest original-core frontier (completion layout
guard); no new original observation or library artifact yet. Read the final
"Layout work qualified; successor core attempt next" section before continuing.

## Workspace and authorization

- Main `/home/vishvananda/rustgm` is Git-only; never build there.
- Edit/build in B `/mnt/vizier/rustgm-scratch/rustgm-pa26-work`, branch
  `implement/scoped-inherent-20261110`. No agents or new worktrees.
- Coherent commits/pushes are authorized. Single-line subjects, no trailers.
  Publish `git push origin HEAD:main`, then fast-forward main.
- Backend stays v3,17b024f69. Do not alter active C++/v5rust checkouts or another
  checkout's outputs. No frontend delegation, invented library body or abort shim.
- Read AGENTS.md, spec.md, ROADMAP.md, owning handout and TESTING_AND_REFERENCES.md.
  Historical details in `git show 2072521b8:handoff.md` are superseded where below
  differs. Never restore its superseded private-port prototype.

## Previous core evidence: release15

Clean **candidate15-release** freezes2072521b8,3948 Rust source hashes and an
empty tracked patch. CLI SHA256
`700e4cb31ba1360f694ac6806b7e24b88fc2b814d6dedeee9de518c2c0d902b0`.
The coherent release build completed in7m22s under the unchanged resource caps.
Do not release-build per edit.

Its full unchanged120s **diagnostic** census clears candidate13's Alignment
constant-pattern error and reaches completion checking:

- all42310 declarations pass;
- all25858 checked bodies have zero failed/blocked functions;
- the same35120 item attempts as13 are reached, including the final initializer;
- the sole error is now **completion**, mem/mod.rs1421's SIZE default:
  `source computation requires its current initializer scope`;
- completion remains0; no complete census or original artifact exists.

The26.7341s diagnostic ran above the14/20 load gate, so it is semantic evidence,
not a controlled timing comparison. Waiting on the gate was delaying diagnosis;
its read-only wait was terminated after recording the above. A controlled
successor observation remains required. **Candidate13 is the last load-controlled
original observation**, not the latest semantic diagnostic.

`obj/parallel/capture-match/original-candidate15-diagnostic/` contains result,
progress, complete census, binding and postcheck. Postcheck verifies722 upstream
sources,375 opens, six configuration pins, all3948 current/producer compiler
hashes, empty patch, CLI hash and unchanged14500624 work/1GiB attempted/64MiB
retained/cap-lints allow. The old generic comparison helper incorrectly attributes
a completion error to the last initializer; use the stage-aware15 diagnostic
postcheck, which compares all reached items and the actual terminal phase.

Debugger evidence under `obj/parallel/initializer-completion/` identifies the
path: check_inline_bodies -> retain_anonymous -> build_anonymous -> promotion ->
Evaluator::associated -> source_computation. **Promotion of an original generic
anonymous template tries evaluating unresolved associated values without its
parked source caller.** No source trace patch is installed. First GDB line115
breakpoint moved to117 and did not fire; the subsequent function-entry trace
produced the useful stack. The extra anonymous trace logged only r8, not the full
NodeKey; do not treat it as a verified owner identity.

Frozen source reductions, all declared before oracle execution:

- `initializer-completion/source1/`:10 full sources,8 accept/2 reject. Independent
  Rust and release15 agree on all10 through the complete source pipeline. They
  cover unborrowed generic/inline/default/projected size and negative bounds/calls.
- `initializer-completion/source2/`:8 full sources,6 required positives/2 negatives.
  All independent Rust outcomes agree with declaration. Release15 reproduces the
  current scope error for borrowed generic inline size, actual const call and
  tuple. The ordinary borrowed-function control passes. A concrete inline borrow
  instead fails reentrant eligibility; a manual associated-constant borrow fails
  missing initializer. Both remain required positives, never inverted. The two
  unpaid-bound/nonconst-call opponents reject for their intended causes.
- Both batches reuse attributed source-div size declarations. Native values4/7
  are declared but unrun. They are not installed in a PA or integration root yet.
  Install actual-source reductions in PA31; ordinary reductions in the earliest
  owning PA. Pending valid boundaries must remain named even if a smaller contract
  is qualified first.

Next repair anonymous-template promotion using its real original/deferred owner,
without weakening evaluated instances, type bounds, constant eligibility or
caller proof. Add prepared/native/scaling controls, then a coherent clean pin and
unchanged full original observation. Continue toward genuine metadata/LowIR and
native original consumers after analysis clears; census alone never builds core.

## Published semantic checkpoints

- **ba6e731e2**, inferred closure inputs:75 outcomes in PA20/24/31,26 ordinary
  natives and two actual source Rust/C observers, all9345 active text cases plus
  separate113-case PA25 component.38 new independently reviewed triples leave
  all28260 older sidecars unchanged. [Details](docs/pa24-inferred-inputs.md).
  Candidate13 clears original26791–26796. Its CLI is
  a8307de89f5ba1bcfb6772e5dc193ff4de84bc7cbe50bcceee5974ae0b9cba67.
- **864ad9c6c**, actual structural constant patterns:19 prepared source outcomes
  (12accept/7reject), native1 0 1 0 1 0 0, real equality/marker identity and
  value-specific fields, modulo-regions classification with no trait permission.
  Literal versus captured derive allowance remains distinct.
  [Details](docs/pa31-structural-patterns.md). PA31 byte-bound packet and PA20's
  serialized ordinary controls preserve every input. No references changed.
- **c9641a67b**, completed implementation parent proof reuse, source-seal opponent,
  five intended layout guards and explicit constness matrix.
  [Details](docs/ci-parent-proof-repair.md). Existing private GAT5x bound unchanged.
- **2072521b8**, literal-pointer private adapters for source pattern expansion;
  actual Report/bootstrap mode supplied, every opponent and work bound unchanged.

Structural public receipts1794594933591102663 (7 exact),1794595182597516639
(final2),1794595309828174033 (native) pass strict selected/CLI Clippy, fmt/audit.
Upstream128+126,315 harness,487 integration roots/13 binaries and44-stage layout
pass. Width8/32/128 inference5024/6752/13664, canonical4047/8055/24087 and actual
trait queries16/64/256 are bounded; hash-checked replay is retained.

All48 A/A+two-ABBA source-pattern comparisons pass complete stdout/stderr equality
between same-profile wipclosure5 and clean candidate14-dev-fast. Load starts
13.15/18.58, ends12.84/18.24; no speed claim from tiny/noisy observations.
Raw: `obj/parallel/structural-constant-patterns/measure-candidate14/`.
The retained empty derive-list inputs still require work: Rust accepts8/32,
128 hits Rust's default recursion limit (setup failure), and release15 rejects
8/32 at vocabulary provenance validation. Earlier declaration/failures remain.

## Full CI and current follow-up

Last fully green remains43239321921 on a7ce2784a.
Manual43491074341 on c9641a67b stopped in private lint (literal-pointer adapters),
with no private execution. **Manual43492592267 on2072521b8 did execute**:

-3071 private pass,2 fail,3 ignored;
-all eight predecessor repairs pass, including source-seal and GAT scaling;
-all three new borrowed-input/source-Copy private opponents pass;
-12 public targets pass; frontend has one failure;
-course/native/debug/independent continuation did not run.

The failures share two causes: nonstaged source derive incorrectly requires a
staged API inventory (one private + frontend MaybeUninit packet), and an old
carrier-instance test expects a later generic Copy diagnostic after a precise
selected-callable guard now rejects first.

Current follow-up validates canonical macro identity, then requires API inventory
only for actually staged libraries; staged missing inventories still fail. Carrier
opponents now expect exact selected public/execution/full-instance/missing-owner
guards. No opponent is removed or production Copy permission changed. The Sources
comment also reflects actual positive implementation-parent reuse.

**1794599635678273601** passes3 exact wrapper/pattern/scaling tests, strict
production/CLI Clippy, full fmt/audit,176.29s. The final carrier diagnostic mapping
was corrected afterward by reading the exact guard order; no local private
harness is rebuilt. Publish this coherent follow-up and dispatch one fresh full
manual successor, justified by the actual failures. Record its exact head/run;
do not dispatch duplicates or full runs for each edit. Raw failed CI and focused
logs: `obj/parallel/structural-constant-patterns/`.

## Pins and storage

Keep current13/15 release pins, wipclosure5-dev-fast (same-profile source baseline)
and candidate14-dev-fast (source864ad9c6c/3948/empty patch,
CLI56485edcef6d8823074fd1a0cd153d0409282bd3bce9077a7e649174d4e0b1a1).
Candidate12-release remains present as the prior historical comparison.

Six obsolete pins wipclosure1–4 and wippattern1–2 were retired with literal marker
removal and scoped prune1794598398378001056, reclaiming0.58GiB. Producer patches,
inputs and raw evidence remain. make tidy1794598420127051960 completed. Current
pin.py count is3948; update it only for actual new Rust files. No14 release exists.

Shared controller:2 jobs,150GiB reserve,800% CPU,12GiB RAM,256 tasks; B absolute
cap16GiB, one B heavy job. Latest free space191GiB. Preserve reserve/caps; clean
this checkout's generated output first. No live build/test/debug/load wait remains
at this checkpoint. Inspect actual processes/controller before new work.

Every build/test/probe/benchmark uses wrappers; check free space before each batch:

```
python3 scripts/resources.py run --wait-seconds 0 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 \
  --prune-under obj/KNOWN -- COMMAND
```

Use `--no-prune` for build-only commands. Make targets own storage wrappers, so
wrap Make only in resources.py. Fastcheck supplies both guards; no outer wrapper:
`python3 scripts/fastcheck.py --max-output-gib 16 MODULE::test ...`.
Native selections require `--ignored --prune-under obj/KNOWN`.
Keep development batches under five minutes including admission; don't repeat
cold267s private link timeouts. CI owns full qualification. Wait for terminal
wrapper completion before another heavy job; a returned session ID is still live.

Measure immutable binaries/frozen inputs with A/A+paired ABBA and full output
comparison. Formal original observers retain120s, all722 bytes,375 opens, six pins,
limits/profile and no body filtering. Main runner15/postcheck scripts are under
capture-match/ and structural-constant-patterns/. Diagnostic observations and
controlled timings must stay distinguished. Never treat a pruned pin as available.
Retire markers literally, then explicitly scoped storage prune; no globs/xargs or
variable/command substitutions in deletion. Use make tidy between batches.

## Standing requirements

All retained requirements in `git show 03003a86f:handoff.md` and historical details
in `git show 2072521b8:handoff.md` remain binding, including:

- symbolic-index native companions and genuine source panic runtime;
- implicit-receiver Deref's two128-layer positives;
- const-adapter bootstrap/runtime positives;
- three tuple-factory cleanup-owner positives;
- general/nested layout, bool niche second unused-value and repr(Rust) tag work;
- authenticated original metadata/body/value transport and actual native consumers;
- PA25–PA44 planned suites fail rather than succeed empty.

Earlier bool/ZST/closure/reference/scaling inputs and unclosed positives remain in
owning packets and checkpoint docs. No frontend/body/library substitution is
permitted and no original-core artifact or whole planned PA is complete.

## Resumed after filesystem recovery: anonymous associated borrows

The temporary Btrfs read-only barrier was externally recovered; the volume is rw
and all28 original modified/fixture files matched the verified recovery archive.
No restoration or filesystem repair was performed by this worker. The archive
under main/obj/parallel/recovery-readonly-vizier-20261113T204218Z remains historical,
not a replacement for this newer state. Builds continue only in B.

Full43499374010 PASSED EVERY STAGE on exactc4d76b6af, including3073 private tests
in both release and debug, all public/course/native/independent and113-case PA25.
It is the latest fully green published baseline. It does not qualify the later
anonymous repair. No new manual full successor has been dispatched for that repair.

[Current checkpoint](docs/pa25-anonymous-associated-borrows.md) qualifies21 complete
prepared outcomes (five PA25 ordinary and16 PA31 source), one additional projected
source original, three ordinary native programs printing7 and a five-export actual
source observer printing4 8 4 4 8. All independently declared outcomes agree with
Rust. Both native tests and exact counters pass a hash-checked current-source/
harness replay. Public1794603811953549205 passes6 exact tests plus strict selected/
CLI Clippy, full fmt/audit,136.28s. All315 harness/488 roots/44-stage layout/
128+126 upstream checks now pass; no reference changes or backend contract change.

Inference3521/6545/18641 and canonical4473/6585/15033 at8/32/128 are linear;
original anonymous templates exactly8/32/128. All exact scaling inputs accept in
Rust. Frozen unborrowed inline controls plus arithmetic are under
initializer-completion/measure-inputs, declared before first compiler execution
for same-profile A/A+ABBA. Measurement itself is pending. Source packet retains
both required complete consumers (projected normalization and concrete inline
source promotion), never inverted. Ordinary inputs are now installed in PA25;
bootstrap3 accept/2 reject and all three actual natives pass.

Implementation uses the registered DeferredTemplate state, restores the lent
executor on every outcome, limits active anonymous service deferral to actual
source mode, and resolves closed expression implementations before promotion.
No new dense records, trait evidence, body substitution or compiler limit. The
original initializer-scope error is reduced and repaired locally; actual core
has NOT yet been observed with this code. There is no candidate16 yet.

Pin.py now asserts3949 for the real anon_borrows.rs integration root. After
publication, freeze one clean release16 and run the unchanged120s full original
observer/postcheck. Keep all722 sources,375 opens, six pins and compiler limits.
If host14/20 load prevents controlled observation, a separately labelled semantic
diagnostic may identify the next blocker; no controlled speed claim. Then continue
to actual original metadata/LowIR/native artifacts. No original artifact exists
and no whole planned assignment is complete. Source tree must stay frozen during
pinning/measurement. New full successor qualification remains required.

## Release16 original observation and implied-bound continuation

This section supersedes the preceding pending-release16/CI statements.
Full43509700804 PASSED EVERY STAGE on exact2b2332577, including release/debug,
all native/independent checks and the113-case PA25 component. B/main/origin main
were clean at that checkpoint. Only B builds; main remains Git-only. No agents.

Clean release16 (3949 Rust sources, empty patch) SHA256
edd069c6ae82b1ebe06f5bc06f4cd3819d577a496ab914471b0b91da9c8018b4
was built once in7m28s as the coherent profiling checkpoint. The separately
labelled unchanged120s original diagnostic checks42310 declarations/25858 bodies,
35120 item attempts, no failed/blocked functions. The initializer-scope cause is
gone; completion now fails with `required lifetime does not outlive its target`
at file227:103636, slice/iter.rs EOF. Completion0; no analysis/artifact claim.
Stage-aware postcheck verifies all722 upstream hashes,375 opens, six pins,
producer/current hashes and unchanged work/attempted/retained limits. Raw:
capture-match/original-candidate16-diagnostic/; scripts under initializer-completion/.
The27.4384s observation ran above14/20 load and establishes no speed comparison.
The pending A15release/B16release calibrated timing batch refused that same
unchanged load gate before observations; no passing timing receipt exists.

debug16c.gdb breaks the actual region error-conversion line and proves the stack
is check_original_child -> anonymous_needs_parent -> retain_anonymous during
complete_constants. The earlier Display breakpoint attempts did not intercept
the optimized/inlined formatting; their logs remain, with no stack claim.

inline-region-arrays/batch1 and batch2 contain20 accepted positive controls
with explicit T:a; none reproduce. Four rejected programs' type views also
accept because they do not perform complete ordinary-function borrow proof;
that is not a claimed full acceptance. Batch3 removes the redundant explicit
impl T:a while preserving the struct's declaration bound: all eight ordinary/
source, direct/macro, inherent/trait forms accept in Rust and fail release16
with the exact new core lifetime cause. This is the reduced missing implied
receiver/input well-formedness assumption.

The current implementation reuses regions::creator's existing implied-bound
cache for original anonymous headers, gated by the inherited universal prefix.
Both batch collect and incremental Collector pass that cache. No new dense
field, callable ABI, projection permission, local lifetime or constant value.
PA25/PA31 anonymous-implied-bounds each bind eleven complete outcomes (7accept,
4reject), including free/nested inputs, static/local escapes, unpaid static
type bounds and unrelated universal regions. All22 match Rust; both independent
Rust/C programs print7. Actual source MaybeUninit methods and real existing
panic_nounwind hook remain. Failed native consumer setup declarations and Rust
diagnostics are retained in native-declaration-v1/v2/v3 and oracle1/2/3; corrected
oracle4 passes, expectations never inverted. Focused RustGM qualification is in
progress in inline-region-arrays/fast1.log at that point; the following supersedes
that WIP statement. No new original observation of this repair yet.

Selected complete qualification is now22 outcomes (7accept/4reject per packet).
Each manifest additionally retains its twelfth, original native.rs borrowed-result
consumer as an unclosed positive. Rust/C7 passes for those original programs.
Callable anonymous results now use the existing requested-result transport and
complete reproof; this qualifies scalar results but retains the borrowed result's
unclosed enclosing-lifetime connection. No conditional debt is assumed. New
scalar-native.rs companions and exact u8 scalar-observer.c both independently
print7; original input/observer bytes remain in their manifests.

fast3 passes six exact tests (all22 selected complete outcomes, owner scaling,
earlier ordinary/source anonymous controls and MaybeUninit). The batch consumer
needed the correct parameterized template CFG and promotion preparation; fast4
1794607901095329290 passes it plus fullfmt/audit/selectedCLI strict Clippy in44.08s.
The source/harness-hash checked native-replay passes both new scalar programs,
all earlier anonymous source/bootstrap native programs and current scaling.
Inference528/2112/8448=66n; canonical80/320/1280=10n; anonymous templates=n at
8/32/128 owners. All four exact frozen timing inputs independently accept in Rust.
All failed fast/oracle gates remain with their real diagnostics; no expected
positive is inverted. Header/runtime borrowed-result work and original core
artifacts remain required; no whole planned PA or additional component release.

Current-source regular-replay also passes all five fast3 semantic controls
against the fast4 source map and harness SHA03657c5bfdec93620e10294a6477ed1177ae31836378a17d685533538842f8ce.
All315 harness/488roots/44stage+1unit/128upstream+126Reference checks pass in
harness-upstream.log. Controller was rechecked:2slots/150GiB/CPU800%/RAM12GiB/
tasks256, cap16GiB. The harness's printed "three slots" is a temporary test
controller, not the actual configuration. make tidy is the current cleanup;
no release17/compiler freeze exists yet. Pin.py now asserts3950 for the new
anon_borrows/implied.rs test submodule. Do not remove it to meet an old count.

## Release17 and late creator publication

Published d1e15f2ad20c991eb0f0b4dfcfea359d204d6375 is clean main/B/origin before
this cache repair. Manual full43517843369 is running that exact source; fast/lint/
architecture/harness pass, release/course running. Do not cancel or duplicate it.
Full43509700804 on2b remains the latest fully green checkpoint.

Clean candidate17-release:3950 Rust sources, empty patch,7m58s build, CLI SHA
f962818c1b6e295f5cfb0b16c3d2e208f439665d13278a8e578f1bb02eee5c69.
Unchanged120s diagnostic still checks42310 declarations/25858 bodies and35120
item attempts with no failed/blocked functions, then the SAME completion lifetime
error at227:103636. Completion0; no artifact. The stage-aware postcheck passes
722hashes/375opens/sixpins/current producer/limits and reports the previous site
is NOT absent.25.5293s above the load gate is not a speed comparison.

debug17a raw entry registers plus the unchanged expanded diagnostic identify
actual NodeKey227:9558 (register9559 is the NonZero representation). It is the
first Iter::next_chunk `[const { MaybeUninit::<&'a T>::uninit() }; N]` block.
debug17b breaks Creators::implied under that exact anonymous owner: owner26576
enters anonymous::header, then hits the actual `return Ok(None)` missing-signature
branch. The expanded resolution confirms definition26576 at227:9596 next_chunk.
Original expanded output is a diagnostic only (235MiB), not a library artifact.

Six earlier nominal-field/named header constants all accept and do not reproduce.
Four actual early function-signature parameter/result array lengths DO reproduce:
all Rust accept, all release17 reject the later method's implied bound. Raw
late-signatures/declaration.json froze outcomes before execution. They are now
late-parameter.rs/late-result.rs in both PA25/PA31 packets, with hashes and unchanged
positive outcomes. Packet now14 independent outcomes (10accept/4reject),13 selected
complete outcomes and one original borrowed-result native artifact still required.

The fix removes Creators.signatures' whole-function snapshot. Each call looks up
its owner's current function_indices row, excludes bodyless/authored-async owners
as before, and caches only successfully computed implied requirements. Missing
owners can appear later. No new dense field or whole-fact scan is introduced.
fast5-cache receipt1794609319805878213 passes7 exact tests in134.87s: all26 selected
outcomes, batch contracts, owner scaling, old anonymous source/bootstrap and two
actual const-closure execution/lifetime-opponent controls. Strict selected/CLI
Clippy, fullfmt/audit/metadata pass. Cache-native-replay PASSES all three native
test groups (both scalar programs, earlier source/bootstrap programs) and exact
66n/10n/n scaling against the current source map and harness hash. Source count
remains3950. Fresh test-upstream passes128+126, and make tidy completed with
record1794609565853113627. No live local build/test/debug process remains before
freezing the next compiler. Actual controller remains2slots/150GiB/16GiB cap.

Actual original observation of this cache repair and immutable paired timing are
pending. Frozen measure-inputs remain independently accepted; do not weaken the
14/20 load gate or change original120s/722/profile/limits to claim progress.
core-artifact.py is prepared for a genuine module-bodies-v13 metadata/LowIR attempt
only AFTER original completion clears; it has never been executed. No original
library artifact or whole planned PA exists. Both borrowed-result runtime positives
and all standing requirements remain required.

## Release18 and inferred local arrays

5e92161157e947d568bb795940dc1facaee3c86f is committed, pushed and main fast-forwarded.
The first SSH push failed public-key authentication; the credential-assisted retry
succeeded (Git still rewrote its transport to SSH). No credential/config changes.
Full43517843369 PASSED EVERY STAGE on exactd1e15f2ad, including release/debug,
native/course/independent and113-case component. It is now the latest full green.
One successor43523008382 was dispatched only after that completion and runs exact
5e9216115. Do not cancel or duplicate it; it does not include the graph WIP below.

Clean candidate18-release,3950 Rust sources/empty patch,8m08s build, SHA
66633ca4ca7da79bfe1d5941c3512119408031e4e5a43b27a9ac14bcaeac364e.
Unchanged120s diagnostic still42310 declarations/25858 bodies/35120 attempts,
zero failed/blocked functions and completion0. Same227:103636 lifetime message.
Postcheck verifies722/375/sixpins/current code+producer/limits, with prior site
NOT absent.42.4332s above load gate is not a speed comparison. debug18a confirms
the SAME first anonymous NodeKey227:9558, not IterMut or a later generated body.
No original metadata/LowIR/native artifact attempt has run.

local-inputs/ declares12 required positives before execution. All accept in Rust;
release18 rejects10 (ordinary/source unannotated local return, discard, wrapped,
redundant explicit bound and shared receiver). Both annotated-array controls pass.
The installed PA25/31 anonymous-array-outputs packets add two frozen opponents
each; all16 independent outcomes agree with Rust (outputs-oracle receipt).
Every input has a manifest/hash, with the exact original source wrapper prefix.

inspect-contract/ is a diagnostic driver built against the exact current release
RustGM library, not another frontend. Its retained contracts show T:a assumptions,
an extra inferred output slot b, actual scalar coercion a:b and WF T:b. The graph
collector rejects T:b before exported a:b can participate in its final reproof.
Driver hash/head binding and raw output remain; its generated executable was
pruned after success. Expanded17's235MiB diagnostic remains, with iter.syntax cutout.

Current WIP dev/src/types/region_graph/output.rs collects existing scalar interface
conditions, deferring non-interface obligations to its existing FULL solve after
adding exactly those conditions. It never drops the private obligation or invents
a new scalar debt. A private unit requires a real scalar debt and a rejecting
no-debt case; public source/bootstrap fixtures oppose static/private-local escape.
dev/tests/anon_borrows.rs adds one exact16-case consumer. No new Rust file/count
or dense field. fast6 failed only new-unit formatting before any compilation;
the exact formatting is corrected. fast7-outputs (session77756) currently runs
7 exact new/previous/closure/capture controls plus strict selectedCLI/fmt/audit.
Inspect terminal result before dependent work. New private unit awaits CI; do
not retry the known cold private build locally. No semantic result for WIP yet.

That fast7 pending statement is superseded: receipt1794610960930869698 passes
all7 exact new/previous/closure/capture tests, selected/CLI strict Clippy, fullfmt/
audit/metadata in158.08s. The16 complete outcomes include all four exact lifetime
opponents. fast8 failed only formatting in the added scaling test before compile;
corrected fast9 receipt1794611404797641881 passes that test/all checks in44.13s.
Three exact generated scaling inputs are installed and hashed in PA25's manifest.
At8/32/128 owners inference1784/7136/28544=223n, canonical152/608/2432=19n,
and real output ports=n. No proof is inferred merely from a zero template count.

outputs-native-replay passes all three actual earlier native groups plus these
counters against the current source map and harness SHA
fa28fb86f22f92806ea1da059d94e27753dcf86dd900524606fe7b40eb515676.
All19 exact new packet/scaling inputs independently match Rust with explicit
panic=abort (the earlier metadata-only oracle omitted that flag; corrected replay
records exact commands without changing outcomes). Fresh test-upstream128+126
passes. No backend/bridge change. New private graph unit still awaits CI.

Retired only superseded candidate12-release, candidate13-release,
candidate14-dev-fast and wipclosure5-dev-fast markers/executables. Their producer,
hashes, inputs/raw observations remain; retirement index is capture-match/
retired-before-output19.json. Scoped prune1794611800337068728 reclaimed0.54GiB.
Those executables are unavailable for new runs; rebuild/verify if ever needed.
Current comparison pins15/16/17/18 remain. make tidy completed with record
1794611969672383722. No local build/test/debug process remains before the next
compiler freeze. Actual controller remains2slots/150GiB/16GiB cap.

Scripts for observe/postcheck/measure-candidate18 and core-artifact18.py are ready.
The latter must wait for actual original completion. All paired timings remain
pending under14/20 load; keep unchanged original120s/profile/quotas and one B job.

## Release19 and the iterator loan frontier

Published65d222d279c5b294e8f2a4e7451d7ef21409a2d7 changes only scalar-output
condition collection/reproof plus its declared reductions, tests and docs.
The existing full solve still proves every private obligation after exactly the
actual scalar interface conditions are assumed. Main/B/origin match this head.
Full43523008382 PASSED EVERY STAGE on exact5e9216115. One successor43530351089
was dispatched only after that completion and runs exact65d222d27; fast, lint,
architecture and harness pass, release/course is running as of this checkpoint.
The new private region-graph unit is included there, not locally rebuilt.

Clean candidate19-release freezes3950 Rust source hashes and an empty patch.
CLI SHA2563abf902e7023be7a6541a79d7ffc9c563281130492473bac190caf5afab6e6c5;
release build7m13s. Unchanged120s original observation was admitted below the
14/20 load gate and checks42310 declarations/25858 bodies/35120 item attempts,
zero failed/blocked functions. The previous anonymous lifetime site IS ABSENT.
The sole completion error is now227:111229, "access conflicts with a live
reference loan"; completion0. Postcheck verifies722 sources/375 opens/six pins,
current and producer compiler hashes, and fixed14500624/1GiB/64MiB limits.
Host load rose after admission; there is no whole-core timing claim.
Raw evidence: capture-match/original-candidate19/ and inline-region-arrays/
postcheck-candidate19.log. No metadata/LowIR/native original attempt has run;
core-artifact19.py is ready only after original completion clears.

The unchanged expanded17 diagnostic maps111229 to `self` inside IterMut's
`into.write(unsafe { self.post_inc_start(1).as_mut() })` loop. Its anonymous
MaybeUninit array now gets past the earlier proof. Actual NonNull::as_mut has an
independent output lifetime and dereferences self.as_ptr(); as_ptr takes a Copy
wrapper by value. Read actual pinned ptr/non_null.rs and slice/iter/macros.rs.
Do not weaken real reference loans based solely on this original diagnostic.

All16 raw-loan inputs match independent Rust type outcomes; complete ordinary
nongeneric checks pass all six positives and reject both tied opponents. All16
generic-raw inputs likewise match independent Rust, then pass COMPLETE original
proofs/LowIR preparation for12 positives and reject four true tied opponents.
The type view alone accepts opponents and is not complete borrow evidence.
The small API diagnostic is built against the exact current release RustGM rlib;
it invokes RustGM's pipeline and lower::instances::build_roots_prepared, not any
external frontend. Raw scripts/results are under inline-region-arrays/
generic-raw-full/ and generic-raw-observed/. No source compiler fix exists yet.
The next frozen loop-raw packet tests repeated independent raw outputs, writes
and real tied-loop/static opponents. Inspect its terminal receipt/process before
new heavy work. Ordinary unsafe ownership belongs toPA27, actual source roles
toPA31; install a reproduced boundary there rather than moving it intoPA25.

All192 immutable output comparisons are now captured:15/16,16/17,17/18,18/19,
each4 inputs times4 A/A plus2 ABBA sequences. The16/17 raw batch stopped only
because its ordinary no-main control omitted the intended library flags. Preserve
that failure; measure-command-correction.json declares the exact lib identity
before rerunning only its missing12 observations. The qualified.json aggregate
references unchanged36 scaling observations plus corrected12 control. Every
stdout/stderr comparison is exact; measurement-summary.json independently checks
all192. A/A ratios1.05–1.63 and paired reversals support NO precise speed claim.
New-path bounded work223n inference/19n canonical/n actual ports remains stronger
scaling evidence. Preserve raw measurements; do not rerun passing comparisons
without invalidation. Pins15/16/17/18/19 remain;15–18 can be retired with explicit
scoped prune once no longer needed. Never claim new execution from a pruned pin.

The old active-local/live-CI statements above are historical. No release19 build
is still running. Update live-state.json as probes finish; keep one heavy B job,
2-slot shared controller,150GiB reserve and16GiB cap. Last free space166GiB.
All earlier unclosed positive native results, source/normalization/artifact
requirements, wholePA25–44 contracts and original transport gates remain open.


## Qualified loop replacement repair

This checkpoint adds a sparse index of loans rooted in actual temporary values.
A new definition first checks a shallow write to that temporary's own storage,
then retires the previous value's loan identities. Ancestor/input loans and all
lifetime, ownership, storage, alignment and constant-validity checks remain.
Ordinary and ordered-cleanup statements share the operation. Raw dereference
roots additionally carry no inherited reference-loan promise; references stored
after reconstruction retain ordinary reborrow checks. No quota, dense declaration
field, external frontend, backend/bridge or storage.py change.

The first loop reductions reproduce on release19. A safe callback minimum also
reproduces, so it belongs toPA16; generic unsafe wrapper reductions belong toPA27
and actual source-role counterparts toPA31. Frozen oracles qualify all31 initial
inputs, including three exact scaling inputs/two original native companions.
The two stronger native companions were declared later before their own first
execution. Outcome declarations, original failed setup bytes and raw logs stay
under inline-region-arrays/. The first no_core loop lacked bool Copy; corrected
v2 adds its actual implementation without changing any outcome.

Focused fast13 receipt1794616733767643096 passes six exact tests across three
targets in62.66s:25 complete outcomes, concrete scaling, old reference/reborrow,
raw-address and anonymous-array controls. All selected/CLI strict Clippy,
fullfmt/audit/metadata pass. Actual scalar output private unit also passed in
full43530351089:3074 private pass, zero fail, three ignored, all regular maintainer
targets pass. That CI then stopped at one old PA24 diagnostic-location difference,
so native/debug/independent continuation did not run. Latest full green remains
43523008382 on5e9216115; dispatch one successor after this coherent publication.

Native fast15 receipt1794616907890211530 passes all four Rust/C versus
RustGM/CPPGM/C programs in75.13s. Original companions output9; stronger C-owned
input programs output9 3 9, proving first3 is unchanged and second7 was mutated
through the second loop result. Raw stdout/stderr remains under
obj/parallel/loop-results-native/. The first native gate omitted the declared
Rust debug_assertions=false flag and could not link core pointer-check panic
symbols. Correct profile passes with no shim. No outcome was inverted.

Concrete scaling replay retains work[loans,points,steps]:8/176/6744,
32/704/26976,128/2816/107904, exactlyn/22n/843n. Its qualified ownership harness
is retained in dev-fast, SHA902a7e1b80465267fa5d3c15807fc79e8dee137af1003def8eb500a4669d5d93.
The first scaling test mistakenly selected an empty template-only inventory;
it failed0!=8 and was corrected to the actual concrete borrow proof, not weakened.
All raw failed gates10/11/12/14 remain, including the ordinary manual-Copy issue.

That original unbounded ordinary by-value-pointer.rs is STILL a required accepting
PA27 contract in its manifest. Bootstrap mode has no authored Clone/Copy traits
for that implementation and rejects before loan checking. Its input/acceptance
are unchanged. The genuine source-role counterpart passes all original proofs.
Selected packet coverage is8 safe +8 ordinary raw +9 source =25 outcomes; never
claim the additional ordinary boundary passed or silently replace it with T:Copy.

Scoped generation adds only eleven PA16 triples. Among28035 existing sidecars,
only PA24 inherited-input-regions/late-private-unrelated-result.rs.ref.stderr
changes168->192 with the identical lifetime cause and rejecting status. Independent
Rust still requires b:a. FullPA16 passes340/340; exact PA24 unrelated-result,
input-escape and projected controls pass3/3. All315 harness tests,489 integration
roots/13 binaries,44 stages/one component and128 upstream+126 Reference checks
pass. Full recipe/raw provenance: inline-region-arrays/loop-references/ and
loop-harness-upstream.log. Backend contract and113-case component are unchanged.

The compiler freeze scripts now assert3952 Rust files: new production
borrow/loans/replacements.rs and dev/tests/loop_results.rs are real additions.
Observe/postcheck/core-artifact20 scripts retain unchanged original120s,
722-source/six-pin/14500624-work/1GiB/64MiB conditions. The artifact script must
wait until analysis completion clears. Fresh same-profile19/20 timing inputs
(shared callback loops8/32/128 and the existing arithmetic control) are declared
under loop-measure-inputs/ before execution. Use unchanged14/20 load admission,
four A/A and two ABBA; preserve outputs/calibration and make no speed claim from
noisy ratios. Older192 output comparisons remain complete and valid.

Publish this coherent checkpoint, retire superseded15–18 pins with explicit scoped
prune while retaining current19 baseline, make tidy, then freeze clean release20.
No original verification of this new repair has run yet. Original core/alloc/std
artifacts, authenticated body/value transport, planned wholePA25–44 and every
standing valid requirement above remain open.

Final preparation: scaling postcheck verifies the exact qualified harness hash
and3951 unchanged production/ownership/test source hashes (the sole excluded
file is the frontend-only native consumer, absent from the ownership target).
All source inputs and n/22n/843n counters verify. Retired only candidate15–18
release pins, scoped prune1794617420959198993 reclaimed0.70GiB; retirement index
capture-match/retired-before-loop20.json preserves producer/hash bindings.
Those four executables are unavailable for new runs. Candidate19 remains pinned.
make tidy completed1794617520149629540. No local build/test/debug process remains
before publishing and freezing20. Actual caps stay2jobs/150GiB/16GiB B; the test
harness's printed three slots describes its temporary test controller only.

## Release20 verified: next is wide constant division

5e59877ca45c2add358bde7fe6888413285b6b18 is published and main/B/origin match.
Main's credential-assisted network fetch failed; it fetched the already pushed
B head locally and fast-forwarded, without building in main. One full successor
43537278071 was dispatched after43530351089 completed; exact5e59877ca, fast/lint/
architecture/harness pass, release/course running. Do not cancel or duplicate.

Clean candidate20-release freezes3952 Rust sources and empty patch,7m54s build,
CLI8ad062c2c202e2e0a3ab456df32ded891732921c0fc6ac0bab184138d94a3486.
Unchanged120s original DIAGNOSTIC (load28.56/27.00 above14/20) takes27.7973s,
checks42310 declarations/25858 bodies/35120 attempts, zero failed/blocked
functions. The previous iterator loan site IS ABSENT. New sole completion error:
file8:31479, num/f128.rs799, `wide floating constant arithmetic is not implemented`.
This is `const PIS_IN_180: f128 = 180.0 / consts::PI` inside to_degrees.
Completion0, no artifact. Stage-aware postcheck passes722/375/six pins/current
producer/unchanged limits; no speed claim. Raw: capture-match/
original-candidate20-diagnostic/ and inline-region-arrays/postcheck-candidate20.log.
core-artifact20.py is still unexecuted and must wait for completion.

Next reduction work reuses literal/decimal_rounding's bounded rational/ties-even
kernel for f16/f128 constant division. Source/runtime wide arithmetic, casts,
metadata and C ABI have distinct existing fences; the frozen backend is not
being extended. `sema/constants/arithmetic.rs::binary` currently rejects all wide
operations before matching; source f128/f16 degree/radian constants need division.
Exact FloatBits stay24bytes/alignment8, Scalar32; never round quad values viaf64.
The existing Unsigned limbs are bounded2048words and rational rounding already
handles subnormal ties. Consider moving only that shared numerical kernel and
limbs below both literal and constant consumers, leaving decimal scanning in
literal; preserve attribution/old literal outputs and existing private scratch
regressions. New ordinary CTFE belongsPA25, genuine source counterpartsPA31.
Declare outcomes before execution and use independent exported bytes/native C
observers, without requesting a native wide-float ABI. All earlier requirements
and ordinary manual Copy positive remain required. Current19/20 release pins
are available;15–18 were explicitly pruned. New19/20 paired timing remains pending
above unchanged load gate; older192 observations remain qualified.

## Exact wide constant division checkpoint

Full43537278071 PASSED EVERY STAGE on exact5e59877ca, including loop/private,
release/native/debug/independent/component checks. It is the latest full green.
No successor is dispatched until this coherent numerical checkpoint is published.

Current implementation adds ONLY f16/f128 constant division. The exact literal
rational kernel and bounded unsigned limbs now live below both consumers at
support/ieee_rounding.rs and support/ieee_rounding/unsigned.rs. Decimal scanning
stays in literal/decimal_rounding. Constant arithmetic consumes selected typed
FloatBits through sema/constants/interchange_division.rs. Binary scale remains
in the output exponent; early underflow/overflow avoids exponent-sized scratch
for opposite extremes. Nearest/ties-even, signed zeros, infinities and canonical
quiet NaNs are explicit. Other wide arithmetic/cast/pattern/wire/runtime/C-ABI
fences, backend v3, configuration truth and all quotas remain unchanged. No dense
record, extra primitive or FloatBits/Scalar alignment growth; no host f64 rounding.
The existing private long-decimal scratch test still uses its original path.

Frozen ordinary PA25 and actual source PA31 packets are installed under
 tests/spec/wide-constant-division/. Four full matrices independently observe
18+18+19+19=74 results, including direct and const-fn division plus actual source
Div UFCS calls. Finite/zero/infinity bits compare exactly; NaNs compare class,
never payload/sign. Four ordinary feature/type/const-call/integer-zero opponents
and two source role/const-implementation opponents reject. The source prefix is
unchanged source-div/foundation.rs plus genuine wide Copy/Clone/Neg roles and
primitive implementations. Source setup v1/v2 failures remain under obj/parallel/
wide-constant-division/: matching const_clone and Neg member stability were added
before corrected v3 replay. No outcome was inverted or original source changed.

Fast4 receipt1794620649316462287 PASSES5 exact tests/all74 bit observations,
six opponents, earlier loop/anonymous/literal controls and fullPA2 142/142 in
204.59s. All selected/CLI strict Clippy, fullfmt/audit/metadata pass. Fast5 native
receipt1794621066380636715 PASSES all four RustGM/CPPGM/C byte-image executions
in37.95s against independent Rust/C byte observations. Only the existing pointer/
integer ABI is requested; no wide runtime instruction, wide C ABI or panic shim.
Raw native candidate outputs stay in obj/parallel/wide-division-native/; independent
outputs in wide-constant-division/oracle-v3/. No old text reference changes.

Fast6 scaling receipt1794621384701268646 PASSES the exact8/32/128-initializer
programs in72.29s. Each output byte is checked. Independent Rust/C separately
matches all three frozen programs, including opposite exponent extremes. The
source/harness hash replay records[inference,canonical,bytes]82/11/128,
244/35/512,892/131/2048, exactly27n/4+28, n+3,16n. Raw current-harness observations
and binding: wide-constant-division/scaling-replay/. All315 harness/490roots/
13binaries/44stage+one component/128upstream+126Reference checks pass in
consistency.log. The new private scaled-ratio unit awaits CI; only numeric literal
separator spelling was corrected afterward. Do not rebuild the known cold private
harness locally. Raw failed fast1 audit naming and fast2/3 Clippy gates remain;
the helper ownership names/fixed slice iteration/test semicolon are corrected.

The source tree now has3956 Rust files, verified by enumeration: shared kernel,
constant division consumer, public division_images root and private scaled_ratios
unit are real additions, while unsigned limbs move without changing count.
Pin.py asserts3956. Frozen20/21 timing inputs are wide literal8/32/128 controls
plus an ordinary f64 arithmetic control; both must independently accept and
baseline20 must pass before comparisons. New divisions have separate bit/scaling
qualification since release20 rejects them. Same-profile immutable timing uses
four A/A plus two ABBA under unchanged14/20 gate; it is pending, never a speed claim.
Observe/postcheck/core-artifact21 scripts are ready. Original analysis must clear
before the genuine module-bodies-v13 artifact command; it remains unexecuted.
All722 upstream files/six pins/work14500624/1GiB/64MiB/120s stay unchanged.

One periodic guarded tidy-archive completed while idle; archive-before21.log
lists verified restore indices under obj/archives/. It touches only this B's
inactive scratch, preserves current19/20 pins and raw restored hashes. Restore
an archived path through its index if an earlier receipt needs its original input.
Finalize controls/tidy, publish this source, then freeze one clean release21 and
observe original core. Whole PA25–44, the113-case component's separate scope,
both borrowed-result native positives, ordinary manual Copy wrapper, original
artifacts and every earlier retained valid requirement remain open.

Final numerical preparation: all four frozen timing controls independently accept
and immutable baseline20 accepts their full type views (control-preflight receipt).
Periodic archive completed successfully; old inactive PID scratch was losslessly
compacted with restored-byte verification and restore indices in obj/archives/.
make tidy1794622629268106114 completed. No build/test/measurement/debug process
remains before publication and the release21 freeze. Reserve150GiB/2jobs/16GiB B
caps remain; latest shared free space155GiB. Baselines19/20 stay pinned. No original
artifact attempt or new original21 observation has run yet.

## Release21 ready; disk admission blocks verification

Published source is f5f3e29614ab1e7200a77556f684f0458984bebf. Its clean
candidate21-release build completed in7m59s, freezing3956 Rust source hashes and
an empty tracked patch. CLI SHA256:
`cfc8b0ee7382359754cf71d1215cd4cd3812bb67a37318bd979f24e5ab40d17a`.
Pin and original producer binding remain under obj/parallel/capture-match/
candidate21-release/. Current executable pins are19,20,21; do not rebuild21.
The handoff-only descendant does not change the compiler or six configuration
pins. Pending postcheck/artifact helpers explicitly require a clean descendant
whose sole changed tracked path is handoff.md, plus every original compiler hash.
They record current checkout and original producer heads separately.

One full successor43546470095 was dispatched after the previous full green
43537278071 completed. Exact f5f3e2961: fast PASSES; full qualification is live
in Release and course tests at this handoff. New private scaled-ratio execution
and all later full gates remain pending until actual CI output proves otherwise.
Do not dispatch or cancel a duplicate full run for this documentation descendant.

No candidate21 original census, postcheck or library artifact command has run.
The wrappers refused before launch five times; the immutable compiler remains
untested against original core. Refusal logs are inline-region-arrays/
observe-candidate21.log, observe-candidate21-retry.log and
observe-candidate21-admission3.log through admission5.log. The last refusal is
153.7GiB available versus154.4GiB required including unused output reservations.
Reserve150GiB, B cap16GiB and all CPU/RAM/task limits are unchanged. No local
build/test/observer job remains active. About1GiB additional shared headroom was
requested; do not clean another checkout or lower the reserve/refused cap.

Own-checkout cleanup first retired two idle incremental entries,147906560
unshared bytes, preserving newest warm sessions. Historical expanded17/stdout
was then losslessly compressed through the existing verified archive helper:
245784112 original bytes ->53486219 archive bytes, restored SHA256
f5b6fb7ad812f41193314f6211950c1329a64442d78b0bcb9d224402ea262659.
Read it with gzip; restore instructions/index are in
obj/cleanup/expanded17-archive-1794623986173222932.json. Current pins, original
inputs, receipts and raw observations remain. This historical diagnostic is
not a compiler input or library artifact. Further ordinary own-output cleanup
does not supply external headroom when the same absolute cap reserves its freed
space; no accounting or policy workaround was introduced.

Resume in B after checking live processes, CI, free space and the load gate:

```sh
python3 scripts/resources.py run --wait-seconds 0 --max-output-gib 16 -- /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --prune-under obj/parallel/capture-match/original-candidate21-diagnostic -- python3 obj/parallel/inline-region-arrays/observe-candidate21.py --diagnostic
python3 scripts/resources.py run --wait-seconds 0 --max-output-gib 16 -- /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --prune-under obj/parallel/capture-match/original-candidate21-diagnostic -- python3 obj/parallel/inline-region-arrays/postcheck-candidate21.py --diagnostic
```

Use the controlled variants without --diagnostic and their matching output root
if admission load is <=14/20. Do not widen that gate or wait indefinitely for
timing. The diagnostic still has the unchanged120s deadline,722 original sources,
375 opened sources, six pins,14500624work/1GiB attempted/64MiB retained. Do not
change tracked files while the observation/postcheck are live. Only if analysis
completion clears, run the prepared core-artifact21.py under the same guards;
it requires genuine original module-bodies-v13 metadata/LowIR before native
consumer qualification. Otherwise reduce the actual new original blocker in its
owning PA. Candidate20's f128 completion error remains the latest observed
frontier until21 actually executes.

Frozen19/20 loop and20/21 rounding comparisons remain pending above the unchanged
timing gate. All earlier192 output-equal comparisons and exact scaling evidence
stand, with no new speed claim. Whole PA25–44, the separate113-case const-machine
scope, both borrowed-result native positives, ordinary manual Copy wrapper and
all other retained requirements remain unchanged and incomplete. Live transient
status is obj/parallel/wide-constant-division/live-state.json.

## Release21 observed; exact products qualified

The external disk blocker has cleared: shared free space is now about201GiB,
with reserve150GiB and B cap16GiB unchanged. Full43546470095 PASSED EVERY STAGE
on exact f5f3e2961, including3075 private tests and the ratio boundary unit in
both release and debug lanes. Its full log is retained under
wide-constant-division/ci43546470095.log. No duplicate full successor was started.

The existing clean candidate21-release ran unchanged original core without a
rebuild. Diagnostic23.0756s, admission10.57/29.83 above14/20: no speed claim.
All42310 declarations,25858 bodies and35120 attempts pass without failed/blocked
functions. The old division site8:31479 IS ABSENT. New sole completion failure
is8:38864, num/f128.rs981: `const HI: f128 = f128::MAX * 0.5` in midpoint.
Completion0, no original artifact. Stage-aware postcheck verifies722 input hashes,
375 opens, six pins and all3956 producer/current Rust source hashes; current
054252674 is only a handoff descendant of the original f5f3e2961 producer. Empty
working patch and fixed120s/work14500624/1GiB attempted/64MiB retained verify.
Raw: capture-match/original-candidate21-diagnostic/, observe-candidate21-resumed.log
and postcheck-candidate21.log. Original-library-candidate21 remains unexecuted.

The [product checkpoint](docs/pa25-wide-constant-products.md) retains ordinary
PA25 and genuine source PA31 inputs under tests/spec/wide-constant-products/.
Four independent Rust/C matrices observe98 results, including both ties-even
directions, full products, special values, actual Mul const calls and the source
HI/MAX expression. Original Mul declaration/macro bytes and full midpoint cutouts
are attributed and hashed. Six independent guards reject; baseline21 reproduces
the unsupported arithmetic on every accepting matrix. Outcomes precede execution;
no valid positive was dropped or inverted. Three independent scaling programs
also pass before candidate execution.

Finite products use16 schoolbook limb steps, at most eight32-bit limbs, and the
existing bounded rational rounding kernel, preserving all226 f128 product bits.
Binary scale remains in the output exponent. Shared operator dispatch is renamed
sema/constants/interchange_arithmetic.rs; the integration root is now
dev/tests/interchange_images.rs, registered exactly once in frontend. There are
still3956 Rust files: these are semantic module renames, not additional records.
No dense field, layout/alignment, quota, literal bound, external frontend,
backend/bridge or storage.py change. Wide runtime/cast/pattern/wire/C-ABI fences
and the frozen backend/configuration truth stay intact.

Fastcheck1794633042585911598 passes four exact tests in134.60s: all172 old/new
matrix results,12 guards, both scaling packets and the literal control. FullPA2
remains142/142, all references unchanged. Strict selected/CLI Clippy, fullfmt,
architecture and metadata checks pass. Native1794633256850046765 passes all eight
RustGM/CPPGM/C byte observers in39.00s, using only the integer/pointer ABI.
The new private full-limb-carry/ties/extreme-product unit awaits CI; no cold local
private harness was rebuilt. Prior ratio private execution is proven by full CI.

Authenticated replay binds3956 compiler/test hashes and every packet source/bit
hash before/after the three exact tests. Product work[inference,canonical,bytes]
is81/11/128,246/35/512,906/131/2048, exactly55n/8+26,n+3,16n. Division controls
retain82/11/128,244/35/512,892/131/2048. All315 harness tests,490roots/13binaries,
44-stage/one-component layout and128 upstream+126 Reference checks pass.
Raw: wide-constant-products/fast1.log, fast2-native.log, consistency-v2.log and
scaling-replay-v2/. The first replay setup assumed an optional old bits_path
field and failed before tests; consistency.log remains. No semantic outcome changed.

Pending timing was also completed when load admitted:20/21 rounding48 complete
output equalities, and19/20 additional full-analysis type-view48 equalities, each
four A/A plus two ABBA per input. Raw calibration/commands/binary/input hashes
remain under wide-constant-division/measure-candidate21/ and inline-region-arrays/
measure-candidate20-typeviews/. Total historical output-equal observations are288.
Largest controls show possible roughly7–15% overhead; ordinary reversals and
small observations prevent a precise/whole-core performance claim. No speedup
is asserted. Original19/20 full-LowIR callback-export positives remain required:
baseline19 rejects exposed callable/reference scalar metadata before timing.
That failed attempt and original input declarations remain under measure-candidate20/
and loop-measure-inputs/. Additional type views do not qualify or replace those
artifacts; retain them in PA29's required transport boundary.

Observe/postcheck/core-artifact22 helpers are ready, preserving the original
inputs, six pins and fixed limits. Publish this coherent source, then dispatch
one full manual successor after confirming the previous full is terminal. Freeze
one clean release22 and run unchanged original analysis. Only after completion
clears, run genuine module-bodies-v13 metadata/LowIR and actual original native
consumers. A census or cutout is never an original library artifact. Whole PA25–44,
the separate113-case component and every retained positive remain incomplete.

Final product preparation: four frozen21/22 timing controls (existing divisions
8/32/128 and ordinary arithmetic) retain their independent source/byte evidence;
immutable21 accepts every exact planned command. Timing inputs and preflight are
under wide-constant-products/measure-inputs/ and timing-preflight/. New products
remain separately bit/native/scaling checked because21 rejects them. make tidy
1794633941285614191 completed; no local build/test/measurement remains. Current
19/20/21 pins are available. Publish, then freeze22 without further source edits.

## Release22 observed; layout work ownership WIP

Published main/B source baseline is40d29b45251e23988f9775fff3e72582b3c95e37.
Full43561807969 PASSED EVERY STAGE on that exact source:3076 private tests in
release/debug, including products_keep_every_carry_and_round_both_even_ties;
all native/course/independent/component gates pass. Full log is retained under
wide-constant-products/ci43561807969.log. No successor full CI is live or needed
for a handoff-only descendant. The current budget-ownership WIP is NOT in this CI.

Clean candidate22-release built in8m30s,3956 Rust hashes, empty tracked patch.
CLI8044e2678fa8b293ce477c3f4eaeda326021c4d9171d4daeb3cf9a635bd2bff8.
The unchanged120s original diagnostic takes38.0712s at load48.75/44.61, so no
speed claim. All42310 declarations/25858 bodies/35120 attempts have zero failed
or blocked functions. The old product site8:38864 IS ABSENT. Completion remains0:
sole budget record at file0:0, `source layout field preparation work budget
exceeded`. Stage-aware postcheck passes all722/375/six/current3956 bindings,
empty patch and unchanged14500624work/1GiB attempted/64MiB retained/120s.
Raw: capture-match/original-candidate22-diagnostic/, observe-candidate22.log,
postcheck-candidate22.log. No core-artifact22 or original native consumer ran.

Actual diagnostic: layout-field-budget/interactive22/gdb.log records the exact
optimized guard at prepare_layout+54, CLI above. Its compared register and
facts+0x1b20 memory both contain canonical_steps8047613; element TypeId687.
Stack: source_layout161 -> validate_ordinary_initializers -> complete_constants.
Assembly confirms the compare against unchanged1000000. The first source-line
breakpoint moved186->189 AFTER the guard and captured an earlier valid request;
debug22/ is NOT an exhaustion trace. The first disassembly.txt attempt failed
in GDB's Rust expression parser; the corrected interactive C-language disassembly
and exact instruction breakpoint supply the actual evidence. Never use current
WIP line numbers as the old producer's source; read source40d29b452 or restore
its bytes before repeating a source-bound probe.

BodyWork explicitly documents observations, not language facts. canonical_steps
sums inference/selection across the crate. Only layout_kernel and its
normalized_fields child compare this cumulative telemetry against MAX_COMPONENTS.
The WIP adds one Work counter to the once-per-provider Engine, retaining the
same1M limit across layout requests/waits/failures. Every layout delta still
increases canonical observations; no counter is reset or quota raised. Field
normalization, storage-state prepayment, readiness and ABI demand all share the
owner. Actual child failures retain their diagnostic while their failed work is
charged. Source caller, field/projection, closedness, lifetime and layout proofs
are unchanged. No dense AST/body/value field, wire, backend or storage.py change.

UNCOMMITTED WIP files are execution.rs, execution/layout_kernel.rs, its
normalized_fields.rs child, dev/tests/invariants/queried_storage.rs,
dev/tests/preparation_accounts.rs and frontend/mod.rs, plus
pa31/tests/spec/layout-work-owner/. The latter seven full programs preserve the
source-div size-only prefix. All independent Rust outcomes match: four source/C
programs output2 4 6 8 0 8 8; unsized/nonconst/unpaid-projection guards reject.
Public tests compare exact prepared/native-root programs after adding8047612
only to BodyWork observations; native companions and updated persistent/spent
work private guards are written, NOT compiled/executed. Inventory is now3957
Rust files. Pin.py still asserts the published3956: update only after qualification.

baseline-fast.log is an ADMISSION FAILURE, not a regression reproduction:
149.5GiB free was below150GiB before launching any compiler. Shared free space
has continued falling, about145GiB at this handoff. User was asked for at least
10GiB external headroom; no permission to clean another checkout or lower the
reserve/cap exists. Do not claim a failing or passing API regression yet. No
local build/test/probe/measurement remains active. Rustfmt editing and diff
whitespace checks are complete; all substantive fixed-code validation is pending.

The exact four-file implementation/unit delta is saved as
layout-field-budget/layout-owned-work.patch. baseline-switch.py verifies that
exact current diff and no staged changes, temporarily restores ONLY those four
original40d29b452 files, runs guarded baseline fastcheck, then restores the exact
WIP in finally and verifies restored hashes. New public tests/fixtures stay in
place. It accepts only a clean committed baseline or handoff-only descendant;
the script has NOT run yet. Once admission is possible:

1. Run that baseline helper and inspect actual test output. Refusal is not
   reproduction; a passing baseline means the new test must reach a later real
   layout query, not that the original failure should be ignored.
2. Run focused fixed preparation_accounts tests and relevant old source-layout,
   projection/constant controls under fastcheck, then the two ignored native
   observers with --prune-under obj/parallel/layout-work-native. Preserve current
   work limits, guards and independent declared outcomes. Do not cold-build the
   known slow private harness; new/updated private execution remains due in CI.
3. Verify source/fixture hashes, actual work/cold/warm/failure bounds, strict
   checks, harness/upstream and native outputs before publishing. Prefer a
   cached dev-fast original diagnostic if its effective cfg matches; no release
   build per ordinary edit, and never widen the original120s deadline or quotas.
4. After qualification update the exact compiler inventory3957, freeze the
   next coherent compiler and observe original core. Genuine metadata/LowIR and
   original native consumers still require analysis completion first.

Current executable pins are ONLY21/22. Completed19/20 and20/21 measurements
permit retiring19/20: scoped prune1794637011472478656 reclaimed0.35GiB. Their
producer/source/input/receipt hashes remain in retired-before-layout23.json;
those executables are unavailable for new runs. The original failed full-LowIR
callback exports remain required positives. All288 historical output-equality
observations stand;21/22 timings are prepared but pending above the unchanged
gate. Every earlier valid positive, whole-PA requirement and original artifact
gate is retained. Detailed transient state: layout-field-budget/plan.json.

## Layout work qualified; successor core attempt next

This supersedes the preceding WIP/admission notes. User restored shared headroom;
free160GiB observed, with unchanged150GiB reserve/16GiB cap. Work continues as
one worker in B, main Git-only. No agents, new trees or backend/storage.py changes.

Baseline1794638741351903699 reaches the accepting API regression and FAILS the
exact old source layout field-preparation guard. Ordinary3959->7683 succeeds;
high start8051571->8054410 fails. baseline-switch-latest.json verifies exact WIP
restoration. The first baseline-fast disk refusal and1794638580232389876 filename
prefix-audit failure did not reach the test. Root was renamed preparation_accounts
without changing fixture outcomes. Do not repeat the proven baseline.

Fixed1794639072978956146 passes five exact tests in154.65s: high observations,
three source guards, original Field defaults/projections, direct/source wide
constant calls and pointer-add contracts. Native1794639354437875740 passes two
tests/three actual programs in39.70s: both source size observers and original
Field output `field 4 23 29 37 41 47 31 1`. Strict selected/CLI Clippy, fullfmt,
architecture and metadata pass. Authenticated qualified-replay verifies3957
compiler/test hashes before/after and every original fixture. Preparation work
is3724+36n at0/8/32/128; high-final minus ordinary-final is exactly8047612.
Persistent provider Work keeps1M across waits/warm/failures, mirrors every delta
into canonical observations and changes no semantic permission. Updated private
spent-work tests and new warm-owner regression are due in CI; do not cold-build
that known slow harness. See docs/pa31-layout-work-owner.md for complete scope.

Independent Rust/C accepts four seven-size programs and rejects three guards.
Three later declaration controls preserve complete original size-only including
PtrWords, append8/32/128 identity functions and independently output8. Manifest
outcomes precede execution. All four planned CLI commands (three source types
plus ordinary LowIR) pass immutable22release and current dev-fast with complete
stdout/stderr equality, a correctness comparison across profiles, not timing.
The explicit current CLI build finished3m46s; it used wrapper-default incremental0
and unnecessarily rebuilt production code. For cached dev-fast builds use
pin.py or explicit CARGO_INCREMENTAL=1,CARGO_TARGET_DIR=obj/rust,jobs2.

Final consistency-final.log passes315 harness,491 roots/13 binaries,44 stages,
one component,128 upstream+126 Reference checks with the expanded manifest.
The previous full43561807969 is terminal success (all3076 private release/debug).
Push smoke43567667793 on handoff1649 also passes. No full successor yet.

Next: publish this coherent checkpoint and dispatch one fresh full manual CI;
update pin.py source inventory3957, freeze clean23release at this meaningful
algorithm checkpoint, and run unchanged original analysis/postcheck. Keep120s,
722/375/six,14500624work/1GiB attempted/64MiB retained. Only after analysis clears
run genuine module-bodies-v13 metadata/LowIR, then original native consumers.
Do not substitute a census or raise deadlines. Same-profile22/23 layout timing
and21/22 product timing remain due under14/20 gate. All288 earlier output-equal
observations stand with no speed claim. Only21/22 executable pins are currently
available;19/20 were retired, their original callback-export positives remain.
No original artifact exists, whole PA25–44 remain planned, the separate113-case
component and every earlier retained valid positive remain required.
