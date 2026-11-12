# RustGM handoff — 2026-11-12

The user requested this handoff and cleared the active goal. Implementation work
is stopped. Resume only on a new instruction. The original objective was to
compile the pinned original core, then alloc/std; **none of those library artifacts
exists yet**. This section supersedes every historical “current”, “NEXT”, CI and
pin-availability statement below. All older language/artifact requirements remain.

## Workspace and Git

- Main `/home/vishvananda/rustgm` is Git-only. Build and edit compiler code in B:
  `/mnt/vizier/rustgm-scratch/rustgm-pa26-work`.
- B branch: `implement/scoped-inherent-20261110`. Latest published implementation
  base is `03b2ad7e0d7cf77554bbd61ff1f22a9e05ea8098`; its compiler source is
  `08794b90955e7ba47674f582511b80c37ebc6152`. A documentation-only handoff commit
  may descend from that base.
- **The new helper-inference implementation is uncommitted in B.** It is not in
  main/origin and must not be treated as the fully qualified baseline. There are
  10 tracked modified files and 58 untracked fixture/manifest files, all listed
  and hashed in `obj/parallel/flatten-helper-inputs/handoff-state.json` in B.
- A complete restoration/review patch, including untracked fixtures, is saved as
  `obj/parallel/flatten-helper-inputs/handoff-wip.patch`. The work is already
  present in B; the patch is for restoration, not application on top of the WIP.
- Continue as one worker. Do not spawn agents or create another worktree. Do not
  modify the active C++/v5rust checkouts or another checkout's output.

## Fully qualified published baseline

[Full CI43157950706](https://github.com/vishvananda/rustgm/actions/runs/43157950706)
**passed every stage** on exact `03b2ad7e0d7cf77554bbd61ff1f22a9e05ea8098`:
fast, all-target lint/architecture/harness, host build graph, release/course,
native maintainer, broad native course, debug, independent Rust and PA25's
candidate contract. Job metadata is saved in
`obj/parallel/flatten-helper-inputs/full-ci-43157950706-jobs.json`.
Do not dispatch a replacement full run merely for this handoff documentation.

The immutable baseline compiler remains available at
`obj/parallel/filter-map-captures/candidate-dev-fast/rustgm` in B:
`623ff5bc248b5f8d722ade9e71cf2ed486328cc6b2271fa8a46b1aef3b220204`.
Its producer binds clean source08794,3907 Rust source hashes and an empty patch.
The published baseline includes the try-capture and computed-match fixes and
three independently reviewed PA21 inspection reference updates. See
[match evidence](docs/pa24-match-captures.md) and
[the PA21 reference review](docs/pa21-empty-binder-ci-review.md).

The baseline's last original-core observation used the unchanged120s deadline
and ended at121.082337s, with65 partial failure records/14,018 item attempts.
All15 required prior bodies, including filter_map_fold4634,
filter_map_try_fold4635 and find4648, were reached without their old failures.
Eleven earlier failures were absent when reached; no new failure appeared among
previously passing reached bodies. Partial counts are not throughput or a complete
census. Postcheck verified all722 source hashes,375 opened sources, six configuration
bindings,3907 current/immutable compiler Rust hashes and unchanged profile/limits.

The **first verified remaining blocker** is `FlattenCompat::iter_fold`,
`library/core/src/iter/adapters/flatten.rs:398`, function4718/body142:10360,
error142:10701: `self.iter.fold(acc, flatten(&mut fold))`; call inputs remain
unresolved. Raw evidence is `obj/parallel/filter-map-captures/original/` and
`original-comparison.json`; progress SHA is
`18a7eb53a2f813bc461bf482b21a203ce219cb0bc58e77adb38889ee0f4b991d`.
**The new WIP has not been reobserved on original core.**

## Current uncommitted helper-inference change

The reduced helper returns `impl FnMut(B,T)->B`, while its input mentions only
`T::IntoIter`. Explicitly naming T works in the baseline. The inferred direct,
named and free-applier forms fail. The parent fold's required callable inputs
must determine T from the published opaque bound; an associated result must not
be used to guess its inputs.

Seven production files change:

- `dev/src/sema/opaque.rs` adds an ephemeral `InferredCaptures` result, not a
  persistent field in the compiler's dense models.
- `dev/src/sema/opaque/captures.rs` maps formal type/const/lifetime captures into
  the owning inference graph. `opaque/refinement.rs` shares that mapping with
  existing trait-result refinement and accounts the region-fold work.
- `dev/src/sema/traits/results.rs` exposes the query helper;
  `results/opaque_bounds.rs` reuses the existing indexed declaration templates.
  A single compatible callable clause supplies input equations through the
  existing Probe and scoped predicate relation. It reads no hidden representation
  and caches no unresolved caller terms. The original selector still proves the
  full trait, formation and lifetime requirements.
- `dev/src/sema/traits/selection/search/callables.rs` invokes that contextual
  refinement and reserves opened regions. `search/inputs.rs` attempts it for
  opaque heads before projection readiness. Normalization/Unresolved from head
  inspection must continue to the existing normalizer; propagating that error
  was an early prototype regression and is fixed in the current files.

Tests change in `dev/tests/source_closures/parameter_projections.rs` and
`dev/tests/callback_provider.rs`; existing integration roots stay registered.
The shared exported-callback helper also serves the old match packet.

The managed WIP CLI is `obj/rust/dev-fast/rustgm`, SHA
`a08ab1a46475334d493514fa30ce2d2717f8c8aa6a382638f0239ce68221499b`.
It is **not an immutable pin**. There are still3907 development Rust files.
No backend, storage.py, resource-policy, original-input, configuration-pin or
compiler-limit change is part of this WIP.

## Qualified evidence and explicit pending contracts

All52 independently frozen Rust1.98 outcomes precede their candidate checks:
36 accept/16 reject, split PA24ordinary27 and PA31actual-source25. **46 complete
API outcomes qualify**:32 body/rule cases,12 width/noise cases and two real native
exports. Six whole contracts remain required, with their original outcomes:

- Ordinary and source implicit enclosing-return-opaque input context: one valid
  borrowed-return proxy and one escaping-borrow rejection per owner. The earlier
  unresolved-input error is not qualified as the intended lifetime rejection.
  The explicit borrowed-return twin passes; the explicit escaping twin rejects
  with an actual hidden-lifetime error.
- Two ordinary native inputs using nominal or `Self::Item` impl where-clauses
  stop at the existing callable-output conformance gate before the helper call.
  Both are independently valid and remain serialized positive requirements.
  The ordinary executable used for current qualification instead has a real
  provided fold body that executes one take and one callback. The source native
  input retains the original method implementation.

Owning packets: `pa24/tests/spec/flatten-helper-inputs/` and the PA31 counterpart.
Each has manifest, independent results, pending contracts and README. Ordinary
pending inputs are serialized where necessary to keep them out of active course
inventory; source files remain declared in the planned PA31 packet. No unsupported
positive is reclassified as a language rejection and no assignment is activated.

Latest exact fastcheck receipt `1794463509909508691` **passes3 tests/3 targets in
75.80s**, including32 complete rule cases, both real exported-call lowerings and
six existing RPIT identity/lifetime controls. Full formatting, architecture and
strict selected production/CLI/test Clippy pass. The earlier receipt
`1794461596019812828` passed the two complete rule/scaling groups and old match
lowerings, but was overall failed on the original ordinary native conformance
form; do not describe that earlier receipt as passed.

Closing review `obj/parallel/flatten-helper-inputs/handoff-review/receipt.json`
passes both exact ignored native groups (four actual programs,0.557s). New ordinary
and source programs print `flatten-helper 7 9 1234`, matching independent Rust/C:
Left and Right share Bucket as their associated result but perform their distinct
conversions and callbacks. Old ordinary/source match programs still print
`match-capture 7 9 12314`. Twelve work observations also pass in6.458s:

| Owner | Width8: inference/traits/canonical | Width32 | Width128 |
| --- | --- | --- | --- |
| PA24 | 23636/90/4302 | 90894/354/16902 | 359912/1410/67302 |
| PA31 | 33799/289/11611 | 128153/1129/41971 | 505555/4489/163411 |

Adding128 unrelated types changes no counter. These bounds make no optimized or
whole-core speed claim. Cleanup receipt is1794463736369907909. Earlier failed
compile/lint/prototype attempts and all original hypotheses remain in raw evidence;
the passing receipts, not those attempts, qualify the current implementation.

## Work still needed before integrating this WIP

**2026-11-12 source Copy capture update (supersedes the blocks below):**

- The first_size/other_size 5171/5172 blocker is fixed
  ([evidence](docs/pa24-source-copy-captures.md)). Closure capture
  classification now selects a source library's `Copy` trait when derives leave
  a capture non-Copy. It is a probe: no evidence or region equality is
  retained, and body Copy requirements still prove each copy.
  - Twelve declared reductions are in `pa31/tests/spec/source-copy-captures`,
    and the history is in `obj/parallel/step-size/core/declared.json`.
  - Exploratory pin `obj/parallel/step-size/wip1-dev-fast` (12358002) clears
    both bodies. The next first failure is fold 5290, zip.rs:275 (unresolved
    call inputs). `DisplayBuffer::new` 5862 ("repeated array element must
    implement Copy") likely shares the derive-only Copy cause.
- Clean pin `obj/parallel/step-size/candidate-dev-fast` (12358002, source
  d0e926179) passed its unchanged observation (46 records, first failure fold
  5290), the postcheck and an A/A plus ABBA measurement. It is now the
  comparison baseline. The 487e0881 and wip1 pins are retired; their producer
  evidence remains.
- Follow-on: repeated array operands use the same probe
  (`pa31/tests/spec/source-copy-repeats`, `obj/parallel/repeat-copy/`).
  Exploratory pin e18b4674 clears `DisplayBuffer::new` 5862 and six
  float-formatting bodies; the clean pin is pending.

**2026-11-12 rigid-header update:**

- The next_if_eq 5023 blocker is fixed
  ([evidence](docs/pa20-rigid-candidate-headers.md)). Trait selection's impl
  trial now refutes a header that needs a rigid normal form, already recorded by
  input normalization, to equal a parameter or a concrete head. Core's `FnPtr`
  `PartialEq` blanket was the ambiguous alternative.
  - Five PA20 and one PA24 triples are generated, and all 28,361 prior hashes
    are unchanged. Seven source-core reductions are in
    `pa31/tests/spec/rigid-candidate-headers`.
  - Exploratory pin `obj/parallel/peekable-eq/wip1-dev-fast` (34ce5ba3) clears
    next_if_eq and is_sorted. The next first failure is first_size 5171,
    step_by.rs:232, a quantified body requirement.
  - Reduction history, including rejected prelude drafts, is in
    `obj/parallel/peekable-eq/reduce/declared.json`.
- Clean pin `obj/parallel/peekable-eq/candidate-dev-fast` (487e0881, source
  ec763f142) passed its unchanged observation (28 records, first failure
  first_size 5171), the postcheck and an A/A plus ABBA measurement. It is now
  the comparison baseline. The 05289eb6 pin is retired; its producer evidence
  remains.
- Full 43196757856 on dd7d8e9d3 passed. It covers the helper-output and
  projected-header checkpoints.

**2026-11-12 later update (supersedes the blocks below):**

- Full 43177635950 passed every stage on main 774fd9dfe, which qualifies the
  const-machine release.
- The iter_try_fold helper-output fix is implemented and published on main
  ([evidence](docs/pa24-flatten-helper-outputs.md)). It covers 26 complete API
  outcomes from 27 declared contracts. Fifteen new PA24 triples are reviewed and
  generated.
- One borrowed-receiver executable stays required at the existing
  opaque-storage gate.
- Exploratory pin `obj/parallel/try-helper-outputs/wip2-dev-fast` (0665938a) is
  built from identical compiler source. It clears iter_try_fold and iter_try_rfold
  and four try_fold/try_rfold callers.
- The next core blocker is try_fold 4961 at map_while.rs:62: "invalid method
  candidate inference graph".
- Clean pin `obj/parallel/try-helper-outputs/candidate-dev-fast` (0665938a,
  `.keep-artifacts`, source 0af99f4a6) has passed measurement and the unchanged
  original observation with its postcheck. It supersedes a08ab1a4 as the
  comparison baseline. Full 43188445172 runs on 0af99f4a6.
- The next blocker, the six adapter `.into_try()` bodies on
  `impl<R: Try> ControlFlow<R, R::Output>`, is fixed and published
  ([evidence](docs/pa20-projected-inherent-headers.md)). Method applicability
  turns projected header leaves into projection-equality conditions, and owner
  binding defers the header to the body normalizer. Nine PA20 triples are
  generated, and all 28,333 prior hashes are unchanged.
  - Exploratory pin `obj/parallel/candidate-graph/wip3-dev-fast` clears all six
    bodies. The next first failure is next_if_eq 5023 at peekable.rs:318, a
    diagnostic-formatting fence.
  - The `unbound` rejection-cause contract is in that packet's pending.json.
  - Clean pin `obj/parallel/candidate-graph/candidate-dev-fast` (05289eb6, source
    b1063aab2) has passed the unchanged observation and its postcheck: 49 records,
    with all six bodies cleared. It is the current comparison baseline.
- Full 43188445172 (on 0af99f4a6) passed fast and all release integration
  suites. It stopped on the private `provisional_results_preserve_known_never…`
  delayed-Never `block` expectation. Rust gives E0308, so that expectation
  changed to "incompatible types", like `binding`. A fresh full successor
  qualifies both checkpoints.

**2026-11-12 update:** steps1–6 are complete; the checkpoint is ee8c11abb
([evidence](docs/pa24-flatten-helper-inputs.md)). Immutable pin
`obj/parallel/flatten-helper-inputs/candidate-dev-fast` (a08ab1a4, `.keep-artifacts`)
supersedes623 as the comparison baseline. The unchanged original observer clears
iter_fold; first remaining is iter_try_fold4720 flatten.rs:431 (projected helper
input `&mut Option<T::IntoIter>` needs T from the parent callable bound before
normalizing). Step7 published 0e74298f1; **full 43171710575 passed every stage**
(48m03s), qualifying a08ab1a4. The superseded 623 pin was retired and pruned
(producer/raw evidence remain). Successor full 43177635950 was dispatched at
07:45Z for the const-machine release (main 774fd9dfe); check it first, and do not
dispatch another while it runs. The text below describes the pre-commit state and
is retained as history.

**Next core blocker (diagnosed, not implemented):** iter_try_fold4720. Reductions
with declared Rust-valid outcomes are in `obj/parallel/try-helper-inputs/`
(`findings.md`, `declared*.json`, `inputs2-4`). Even without any projected input,
the helper fails whenever its own generic R occurs only in the opaque callable
output (`impl FnMut(B,T)->R`); a concrete output or explicit helper arguments pass.
`callables/output.rs::symbolic` exposes `<X as FnOnce<A>>::Output` only through
`match_signature`, which needs a closure/fn signature and returns Mismatch for a
declared opaque. `search/result_bounds.rs::derive_result` then sees an unresolved
subject. Planned fix: an output counterpart to `results/opaque_bounds.rs::infer_inputs`
using the unique indexed `templates.equalities(callable_output)` clause and the same
InferredCaptures mapping, returning the declared output term so `symbolic` exposes
it. This is forward normalization through the declaration, not inversion; the
selector still proves the full bound. Then re-observe flatten.rs:431.
An uncommitted experiment of exactly that (`infer-output-experiment.patch`, with
TRACE prints) reaches `symbolic` and finds the unique Output clause. But relating it
binds the helper's T, so `preserves_inputs` correctly refuses. The trait path then
binds T, yet the goal stays Deferred on the output-only capture. The real fix must
either export a Deferred trait decision's input equations so the output projection
retries with settled inputs, or carry the caller's required output into
`infer_inputs`. Sources were restored, and the managed CLI was rebuilt to exactly a08ab1a4.

**PA close-out:** const-machine is now **released**
([record](docs/pa25-const-machine-release.md)); it is the plan's first component
release. Its full-CI qualification is the successor after 43171710575, on the release
commit. Next candidates follow docs/assignment-order.md: operator-kernel, then
unsafe-places, c-abi and declarations. Each needs a declared unit manifest, reviewed
oracles and release checks; their implementation evidence already exists.

1. Inspect B's Git state and `handoff-state.json`; preserve the existing changes.
   The compiler and fixtures have not been committed or pushed.
2. Qualify the23 new ordinary course cases through the actual CLI library path,
   check their declared outcomes and successful stderr, and independently verify
   all object classifications. The root PA24 manifest currently declares684 cases
   (490accept/194reject), including16 accepted new cases and7 rejects. Fifteen
   new positives declare empty library objects; one is the real exported native
   program. **No new reference triples or reference batch have been generated.**
   API/native success alone does not establish CLI metadata publication.
3. Declare/review the new text contract before normal scoped oracle generation;
   preserve all old reference hashes and input bytes. Check relevant old accepting
   outputs because query interning/order and the shared opaque refinement changed.
   Never hand-edit references or bless an unsupported positive. The standard native
   checker expects warning sidecars: before first generation, use declared-warning
   checks and independent object checks, then normal generation and review.
4. Run the fixture-owning upstream/harness checks. Existing source formatting and
   selected strict Clippy already pass; repeat only checks invalidated by new work.
   No broad local private/release/all-target/native sweep is authorized per edit.
5. Once a coherent source checkpoint is committed, freeze the exact candidate,
   bind all3907 Rust hashes/empty patch, and compare immutable accepting controls
   with frozen inputs, A/A calibration and paired ABBA under exclusive admission.
   Use fully qualified623 as the available baseline. Retain honest counter/noise
   observations; this WIP has no immutable timing comparison yet.
6. Run a **new unchanged120s** original observer on that genuinely changed frozen
   candidate and perform the usual722/375/six/source/profile/limit postcheck and
   reached-body regression comparison. Reuse the pinned archive/manifest below;
   do not extend the deadline or rerun merely for a longer prefix. Until then,
   the verified original frontier remains iter_fold.
7. Publish only relevant qualified work. Full43157950706 passed source08794;
   it does not qualify this new compiler. A full successor is due after the new
   implementation publishes and a fresh no-active-full check. Do not dispatch or
   restart a full workflow for this handoff-only documentation.

Original archive:
`/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/original-core-inputs-88d9e12ae178fab0fb5cc050a94da85685d449ea`.
Manifest in B: `obj/parallel/census-progress/original-manifest.json`.
Prior observer/postcheck/compare scripts in `obj/parallel/filter-map-captures/`
are templates, not ready-made WIP observations; change their candidate bindings
and output root deliberately. All older PA20/25/26/30/31 artifact, import, depth,
coercion, const, cleanup and optimized-proof requirements below remain required.

## Resource and artifact state

No local compiler/test/resource job is active at this handoff. `make tidy` completed.
Current controller:2 shared jobs,150GiB reserve,16GiB output ceiling per run,
CPU800%,12GiB RAM,256 tasks; Cargo/test threads2. Check free space again before
resuming. Keep builds, logs, inputs and measurements in obj. Use `make fastcheck`
with exact selections; direct commands need the storage/resource wrappers. The
five-minute feedback budget includes admission. Preserve the warm dev-fast cache;
use the existing package environment/incremental settings when freezing a CLI.
Do not lower the reserve or duplicate strict Clippy with cargo check.

Current immutable623 is retained. Historical5d was retired **after** full431579
passed, cleanup1794464101014086908;67/6c were retired after publication/comparison,
cleanup1794459354367224635; e8 retired under1794457333921746162. Their raw evidence
and producers remain, but their executables are absent. Earlier81 and other retired
binaries also remain unavailable. Never treat a pruned path as available for a new
run. Preserve the older optimized21f requirement/pin from the historical work; its
exact executable path was not reverified here and must be located/hash-checked before
new optimized measurements. Do not remove pins protecting source/encoded inputs.

Journal: `/mnt/vizier/rustgm-scratch/integrator-20261109/pending_decisions.md`.
The B-local handoff-state/patch and current passing receipts are the restart anchors.

## Historical checkpoints and retained requirements

The following entries preserve history and outstanding contracts. Their older
“current”, “next”, active-goal, CI-status and binary-availability wording is
superseded by the current handoff above.

## Current checkpoint — computed match captures locally qualified

[Checkpoint/evidence](docs/pa24-match-captures.md):23 production lines reuse typed
Match decisions/arms for computed/local scrutinees, guards and arm values. Direct
captured-place pattern operations remain explicit required positives; no blanket
move/tag permission. No new engine/field/backend/storage/limit. CLI623ff5bc248b5f8d722ade9e71cf2ed486328cc6b2271fa8a46b1aef3b220204.
49independent Rust outcomes41accept8reject;40 complete API outcomes qualify,
20source. Nine entire positives required (4place contrasts perowner+ordinaryTry),
plus public-helper enum's CLI metadata artifact required despite actual API/native
success. Original public bytes serialized; private-helper variant independently
qualified. Public4/2targets129.39s receipt1794457800427825291; privatehelper1/31.93s
receipt1794458273040726694. Both fullfmt/audit/selected strictClippy. New ordinary/
source actual native7/9/12314 and oldTry7/43/23/123 PASS; complete move/Fn opponents
preserved. Width8/32/128 ordinary inference349/733/2269 canonical115/427/1675
traits0; source1466/1850/3386 canonical1521/1833/3081 traits5; onefield/noise128zero.
19newordinarytriples independently prechecked;15objects/14declaredempty verified
inbothcompilers, actualnative separatelyexecuted. Normal batch
1794458447436189585-55a59cd0e9eb.json SHA55a59cd0e9ebf33691542a946db284b8beefc141a1308d723db7eade2df8df0b;
all28145priorreferencehashesunchanged. RootPA24 661/474accept187reject/native396/
5codegen73object. Raw filter-map-captures/{review,native-review2,text-review2}.
InitialindependentmissingCopy ICE/nativewhitespace and firstCLIpublicenum+missing
not-yet-generated-warningrefs failures retained; none produced oracles or inverted
valid source outcomes. Final directpreverification+normalgeneration/textcheck PASS.
Upstream/harness/tidy PASS,28203currentreferencehashesunchanged. Clean source
08794b90955e7ba47674f582511b80c37ebc6152 freezes exact623 with3907Rust/empty patch.
Thirty exclusive A/A+twoABBA accepting controls preserve complete outputs vs67;
wall1.075/.995/.968 CPU1.078/.995/.968 AA.980/.999/.997, rawnoise/no speedclaim.
NEWunchanged120s original ends deadline121.082337s,65records/14018attempts,
lastfn8351 slice/mod.rs210:118749,line3197. Three filtermap bodies4634/4635/4648
(body141:1230/1514/5145) and12earlierrequired bodies clear. Eleven prior records
absentWHENREACHED; NOnewfailuresamongpreviouslypassingreached67bodies. FIRST
FlattenCompat::iter_fold4718/body142:10360,error142:10701,flatten.rs398:
self.iter.fold(acc,flatten(&mut fold)), callinputsremainunresolved. Postcheck
1794458868356791540 verifies722/375/six/current+immutable3907/profile/limits.
ProgressSHA18a7eb53a2f813bc461bf482b21a203ce219cb0bc58e77adb38889ee0f4b991d.
No completecensus/throughput/core/alloc/stdartifact. NEXT: publishobservations,
refreshfullstatus/noactivecheck afterreviewedreferencefollowup,
reduceactualnestedflatten return-callableinputTcorrelation ordinaryPA24/sourcePA31.
Full43154313816 TERMINALFAILURE EXACT01e7013/sourcec388: ALLRustregularPASS,
3065private0fail3ignored/linking285PASS. PA24text642/642; PA21text864/867 on3
inspection-onlyempty-binderproofrecords. Laternative/debug/independentSKIPPED.
[Review](docs/pa21-empty-binder-ci-review.md) verifies3originalbytes equalstable5d,
old5d matchesoldrefs, preceding67==current623 ALL3newoutputs. FouractualRust/CPP
runtimecomparisons+6intendednegativecompleteoutcomesPASS; nonemptybinderinspection
unchanged. Normal3tripleupdate/newbatch1794459158288820141-c8391c81a081,
28194otherreferencehashes/baseJSONunchanged. No compiler/input/outcomechanges.
Original623evidenceREUSED; no observerrerun forrefs/docs. NEXT publishreviewed
referencefollowup+ONEfull afterfreshnoactivecheck, thenreduceactualflatteninput.
Stable5d/current623/optimized21f
retained; preceding67pineligibleforretirementAFTERpublication+comparison. e8ABSENT
cleanup1794457333921746162, producer/rawkept. Allolderrequirements preserved.
Singleworker/no newtrees/backend/storage/limitsfixed.

## Current checkpoint — try-block captures clear filter_try_fold

[Checkpoint](docs/pa24-try-captures.md): productionONEline shares existing
Loop/TryBlock value-use traversal. No newengine/field/permission/backend/storage/
limit. CLI67f146ab1d137041423db053336ea8a4724269bb413dfdf97367ffceb108791e.
36independent Rust outcomes26accept10reject:PA24serialized14,PA30native1,
PA31source21. Source21completePASS; ordinary15remainREQUIRED atprovider/import
fences, includingtheirpositive/negativecontracts. RootPA24still642; no neworacles/
activation. CopyFn/mutableFnMut/ownedFnOnce retainoneactualfield. Typeinspection0
forrepeat-once/wildcard isNOTfalseaccept:completeownershiprejects1 asrequired.
Initialpublic1794453757639414841 PASS3/97.97s; final1794455612137610151
PASS3exact2targets43.27s fullfmt/audit/selectedstrictClippy. Native2groups2programs
PASS,newactualsource7/43/23/123matchesindependentordinary/sourceRust/C; oldByRef
23/1234controlPASS. Width8/32/128 inference5227/16411/61147,traits37/133/517,
canonical1400/1640/2600;onefield/noise128zero. Reviewfilter-try-capture/review/.

PrecedingFULL43145135573 exact8c231/sourcecf75 TERMINALFAILURE: ALLRustregular
PASS,private3065PASS0fail3ignored,linking285PASS. PA24text626/642 (13earliernegative
diagnostics+3equivalentptrreturnspills), laterstagesSKIPPED. All16inputbytes/status
outcomes fixed; allalreadychangedin6c. Exact3spillSSAequivalence toold5dIR proved;
pinnedRust16+actualRust/CPPnative3checked BEFOREnormalreferencegeneration.
Newbatch1794454815232195184-c67d8ff46d63.json;28096otherreferencehashes/baseJSON
unchanged. Rawby-ref-fold/ci-reference-review2/passed.json. No blindbless/inversion.

Upstream/harness/tidy allPASS;28145currentreferencehashesunchanged.
Source c388b27099c03f75e01adf830346fe3c2a04e3f6 freezes exact67 with3907
Rust hashes/empty patch. Thirty exclusive A/A+twoABBA controls retain complete
output equality vs e8; slower/noisy ratios preserved, no optimized/core-speed claim.
UNCHANGED120s original ends deadline121.087482s,78partialrecords/14320attempts,
lastfn8892,str/pattern.rs239:68090,line1816. Filter_try_fold4600/body140:3018 and
all11 earlier required bodies reached without failure records. Two old records
absent when reached; no new failures among previously passing reached e8 bodies.
First filter_map_fold4634,body141:1230/error141:1253,line39 at match in
iter/adapters/filter_map.rs. Postcheck1794457237662345758 verifies722/375/six/
3907/current+immutable/profile/limits unchanged. ProgressSHA
5f7da4e31911e13155c609d3ad4069120d7e1deadab565920678ec1ef44ba52d.
No complete census/throughput/original artifact. NEXT: publish+ONEfull after fresh
no-active-full check, reduce actual Match capture use through selected pattern
facts (discriminant reads differ from moves). Keep stable5d/optimized21f/current67;
e8 comparison pin eligible for retirement after publication. Duplicate6c
fixed-dev-fast/81 ABSENT; identicaltest-fix6c remains. All older requirements stay.
Single worker, no new tree, backend/storage/limits unchanged.

## Preceding checkpoint — wrapped fold premises

[Checkpoint/evidence](docs/pa24-wrapped-callback-premises.md): production2files
reuse scoped comparison to bind complete canonical projected inputs; blocked
result comparisons remain pending. Generic deferred equations/cycles unchanged.
CLIe8ca58dbc0bc0baab5dbf5c883dffd389d85a213f768445b6ff77d8e5747820f.
68independent Rust outcomes60accept8reject: PA24 29(3serialized),PA25 4,PA30 1,
PA31 34. Complete55qualify;4additionalsourceconsttypeoutcomes qualify. REQUIRED13
positive artifacts:3ordinary,4ordinaryconstimports,1ordinarynative,1source multiple,
4sourceconstownerlowering. Multiple-input negative hypothesis DISPROVED byRust;
acceptance required, originalbytes retained. No inversion/assignment activation.
Finalpublic1794450414769391285 PASS5exact3targets36.68s; linking
1794451117884602258 PASS2exact59.61s. Fullfmt/audit/selected strictClippy pass.
Native5groups/11programsPASS, newsource23/1234 matchesindependentRust/C. Twelve new
width/noise counters:ordinary2613/8065/29825 traits3;source4216/14156/53868 traits
25/73/265,methods0; noise128zero. Two oldfamilies exactunchanged. Review raw by-ref-fold/.
26newtriples generatedafter independentdeclaration+preverification, reviewed;
28065priorreferencehashes/35oldcallback/11oldneg outputs unchanged. Independent26
includes22declared emptyobjects/no runtimecomparison. PA24root642/459accept183reject,
native396runtime5codegen58object. Referencebatch1794450594955412258-3573a240e1c2.

Preceding FULL43138039057 exacteff52eb9d/source d198 is TERMINAL FAILURE: fast,
alltargetlint/harness/hostgraphPASS; private3064PASS1diagnostictestFAIL3ignored;
linking283PASS2diagnostictestsFAIL. LaterstagesSKIPPED. Previous2pending signature
invariants BOTH PASS. Four expecteddiagnostics repaired (including2cases in the
privatepacket); all22declarationcontrols/independentE0277/E0599 and2fresh-linktests
checked. Privateharness rerun/fullsuccessor PENDINGCI; no blindretry251scompile.

Upstream128/Reference126/harness315/468roots13bins44stages/tidy all PASS.
Sourcecf75c3f61e718dfcb0878f71a69577ecd7dbb0b5 committed clean; immutable
by-ref-fold/candidate-dev-fast CLIe8 binds3907Rust/emptytrackedpatch. Thirty
exclusive A/A+twoABBA accepting observations vs6c retain fulloutputequality;
nooptimized/neutrality/core-speedclaim. Rawmeasure/ retains noise/slowervalues.
UNCHANGED120s original ends121.563414s,61partialrecords/13261itemattempts,last
fn6964/file203:1097,fmt/builders.rsline44. ByRef fold4429/body134:1245 andrfold
4435/body134:2178 reachedwithNOrecords;5ArrayChunks+4regressions also clear.
44oldfailure recordsabsentwhenreached; NOnewfailures amongpreviouslypassing
reached6cbodies. Firstfilter_try_fold140:3088,line86, typedcaptureuseunsupported;
attemptedfn4600/body140:3018. Postcheck1794451995494437060 verifies722/375/six/
3907/current+immutable/profile/limits. ProgressSHA8bbdcf63f52c70ff87547e5a4b0c664d3319d71ab5658df44185134e2add06dc.
Partialcounts NOTthroughput/fullcensus. No core/alloc/stdartifact.
Published8c231518e/sourcecf75; FULL43145135573 runningEXACT8c231 afterfresh
noactivefullcheck. Fast/lint/harness/hostgraphPASS; release/course running. Never
restart for documentation descendants. Duplicatefixed6c executable retired
1794452534046951918; identicaltest-fix6c baseline stillavailable, producer/rawkept.
NEXT: reduce actualfilter typedcapture at try{acc}, ordinaryPA24/sourcePA31.
Collector::visit lacks Kind::TryBlock {body,gate}. No timeout/limitextension. Keep5d fullqualifiedbaseline,6c accepting
comparison andoptimized21f; intermediate81absent. Preserve every older requirement.
Singleworker/no newtrees/no storage/backend/limits changes; goalACTIVE.

## Preceding checkpoint — repaired projected callback results

[Checkpoint/evidence](docs/pa24-projected-callback-results.md) records84 independent
Rust outcomes: PA24ordinary35(26accept9reject), PA30native1required pending,
PA31source48(39accept9reject). All83supported complete API outcomes qualify;
source native23/12 matches independent Rust/C. Ordinary native bootstrap conformance,
eight serialized override contrasts and every older pending requirement stay required.
PA24 is correct earliest owner; PA20 manifest restored unchanged. No original artifact.

Intermediate local sourcef7e9c2814/immutable81a3c693 cleared reached ArrayChunks
failure records but exposed four regressions: Chain::last135:4953, FlattenCompat::last
142:17747, MapWindows::next_window148:3241, Option::into_flat_iter182:73756.
Published repair passes every reduced complete API regression and control.
It uses shared scoped pending-pair comparison (no inverse projection inference),
signature input equations only for unresolved projection roots, one recollected
normalization plan, and exact canonical selected-function identity after progress.
No blanket projection registration/new engine/dense fields/backend/storage/limits.

CURRENT qualified managedCLI6c0853071ccf7675c2f6c919eea99414c901f874ccb925279da1e0cc0defa620,
3907Rust+51tool/config bindings. Public1794444997306962028 PASS18exact/130.45s,
strictselectedprodCLItestClippy/fullfmt/audit. Same compiled frontend passes4native
/10actualprograms; sixoldregular groups inpublic. Rawphases4/native-review2.
Original widths8/32/128 counters unchanged[51800,208,2664,16]/[205897,832,10656,64]/
[822207,3328,42624,256]. Newmulti-input arity8/32/128 inference6071/12967/40807,
traits18/no methodwork. Noise128 addszero. Countercopy removed. Nooptimizedclaim.

35newPA24triples(17initial+18regression) reviewed; finalcompiler matches35new/11old
controls. Latestreviewpreserves27504prior sidecars/175trackedreceipts. Independent
new18library checks include14declared emptyobjects/no runtimecomparison. PA24root
616/437accept/179reject,396runtime5codegen36object. Alloldsourcebytes/checksums fixed.
Two exact private invariants remain pendingCI after fixed-budget combinedcompile
1794438008438968646 timedout before execution; no retry/limitraise.

Latestclean sourceD1980b3255cf607355b2a254dc83393463fb27b5 andimmutablefixed-dev-fast
CLI6c085307 bind3907Rust/empty patch. 30exclusive A/A+twoABBA observationsmatch
completeoutputs vs5d43; wall.998/.966/.960, CPU.993/.965/.961, AA.874/.914/.869;
nooptimized/neutrality/core-speedclaim. Old81candidate was retired after publication;
its raw observations and producer remain, but its executable is absent.

EXACT120s original6c observer ends121.415000s,111partialrecords/13562itemattempts,
lastfn7515 slice/sort/select.rs220:6377,line168. All5ArrayChunks and4regression
bodiesreachedwithoutfailure records; no newfailures amongpreviouslypassingreached
baselinebodies.12priorfailure recordsabsentwhenreached. FirststillByRefSized::fold
134:1322,line45, I::try_fold(self.0,init,NeverShortCircuit::wrap_mut_2(f)).0;
pairedrfold134:2256. Postcheck1794446887102513389 verifies722/375/six/current+
immutable3907/profile/limits. ProgressSHA
5c84a0a0e14f8553d7d7d3ecb665f84aa245c2b6a047e429c09348919a7dce42.
No completecensus/originalartifact, partialcountsNOTthroughput. Rawfixed-original/
andfixed-original-comparison.json. No rerunmerelyforlongerprefix/limitextension.

Published ec2d2a0b4/source d1980b325. Full43135131607 failed at all-target Clippy:
three suite_inference tests still passed an immutable Store to mutable refresh.
Only those three callers changed. Focused fastcheck1794448122967457238 passes all
three exact tests/fullfmt/audit/selected strict Clippy in42.77s. Production/CLI2246
hashes and both managed/immutable CLI6c085307 remain identical; original evidence
is not rerun for test-only changes. Later full CI stages were skipped, not passed.

NEXT: publish test-only correction, ONE replacement full after fresh no-active-full
check; then reduce actual ByRefSized wrappedtry_fold output before.0 consumer,
ordinaryPA24+sourcePA31 first. Other adapter projection/closure failures separate.
Prior full43081009696 ALLPASSED exact2c13e1cff/source5cd41810b, no docsrestart.
Keep5d43 until replacement full qualifies; keep optimized21f for its pending work.
Intermediate81 executable is absent. Singleworker/no
newtrees. Allpaid-alias/projected-premise/symbolic-index/depth128/factory/tuple/
bootstrap/import/transport/optimized requirements below remain. Goalactive.

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
Published D1 2c13e1cff; FULL43081009696 PASSED ALL STAGES EXACT2c13e1cff3fa3cca23410d35efab337621251aa2/source5cd41810b,
after fresh no-active-full check. Never restart for docs/input descendants.
Comparison0912 explicitly retired1794422858133137399 AFTER publication/comparisons:
executable ABSENT, producer/raw retained. Current5d43+optimized21f remain available.
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
