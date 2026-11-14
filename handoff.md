# RustGM handoff — 2026-11-14

Continue the active goal: compile pinned original core, then alloc/std, completing
assignment components as their complete contracts qualify. Work as **one worker**.
**No original library artifact exists.** PA25–PA44 remain planned as whole PAs;
the released113-case PA25 const-machine component stays separate.

Latest continuation: clean release19 on65d222d27 clears the anonymous-array
lifetime error and reaches IterMut::next_chunk's loan conflict at227:111229.
The next source checkpoint now repairs fresh temporary identities in loops and
raw loan roots, qualifying25 complete PA16/27/31 outcomes, concrete n/22n/843n
loan/point/work scaling and four actual native comparisons. FullPA16 is340/340;
only one independently reviewed PA24 stderr location changes among28035 old
sidecars. Read the final "Qualified loop replacement repair" section first.
Freeze the next clean compiler and observe unchanged original core. There is
still no original library artifact. The ordinary unbounded manual Copy-wrapper
positive and both earlier borrowed-result native consumers remain required.

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
