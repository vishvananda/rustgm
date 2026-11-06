# Exact continuation after typed initializer checkpoint

Workspace: /mnt/vizier/rustgm-scratch/rustgm-pa29-work only. Branch
implement/source-selected-leaves-20261105. HEAD79dbce7235841665c7d7f845733b12ee42027648
merges qualified main4ab1926bb onto Source67d4154b6. All current consumer/API
changes remain WIP; do not call full suite qualified. No new worker/tree/cache,
backend/pin changes or main push. Parent owns integration/core/probes.

Durable earlier stashes remain:8cbcc509693e8024d3214e000e77a729679f3059,
24a81e3d555e6cd92c9b692f095cdea8f2c702dd and pre-main
6c03f451d0cd9e916738c4a6f30390dcce1658eb. Never drop by numeric stash index.

Actual implementation now in WIP:

* Types::destruction::Eligibility Body/Whole/OwnDrop; full(TypeId,purpose) claims;
  Copy/NoDrop Body-only; actual LangDrop scoped proof for OwnDrop. Shared full
  semantic field fold; selected owned coordinate/plan rows before visitor;
  physical cleanup/CFG/read/loan/Code independent. Seven historical cases still
  over-constrained by whole-type partial history (see below).
* Authenticated direct constructor Calls retain Candidate20B in paid Transfers;
  ALL candidates (even disappearing cleanup rows) validate public/opened
  signatures, full nominal/field arguments, actual member inventory and real
  late opening before eligibility. Actual generic/lifetime controls pass.
* State::decide in sema/nominals/traits/decide.rs shares actual caller index/fold,
  isolates each query Context, seeds its step count from owner budget before
  allocation and charges only delta. Proven evidence/equalities retained;
  Absent/debt-free Refuted separate; Deferred/conditional-negative/invalid/
  exhausted/provisional-sized are errors. Existing cdrop proof kernel uses it.
  State remains800B. New private ctqdc/ctopen compile, execution CI pending.
* Actual physical initialization is now a distinct unfinished Initial token in
  loans/templates/staged.rs and native check/cstage.rs. Owns SAME Facts, actual
  body borrow, parameters/builtins and Seal; no finish accepts it. begin then
  select_initial validates and moves SAME Seal to unchanged Prepared. One
  refinement/select. Facts::body returns stored lifetime. prepare/finish native
  wrappers preserve current future generated CopyView path.
* Batch::begin yields InitialRow; select consumes it to existing Row. No finish;
  dropping it leaves cursor/loan coverage incomplete. future_arena is still only
  final successful Batch::finish. All one/template/ordinary pipeline cdrop paths
  now expose a sema insertion boundary. No classifier requests/BAD bits yet.

Validation binds current sources in per-gate sources.json. Combined receipts:
obj/parallel/source-transport/va-cleanup/qualified-init-receipts.json.
API2 fmt/audit+all public/private strict metadataPASS123.33s. API3 focused
strictPASS83.89s. Proof1 seventeen ctseal/ctfix/ctown/ctleaf source testsPASS112.06s.
Init1 one elidable-lifetime lint; Init2 fixed strictPASS89.31s. Init3 one owner
pass-by-value lint; Init4 fixed strictPASS91.59s. Proof2 all same17PASS106.70s.
Measured symbolic Initial728/Prepared784, native Initial776/Prepared952,
InitialRow960/Row1008: established records did not grow. No new native/private
execution claim. No Source heavy job live at this checkpoint.

Actual existing positive progress (consumer6): prknow,prmove,tagmv,taglif,taggen,
tagshr,aryone,arybig,arysm. Nineteen frozen cleanup negatives pass. Remaining
Rust-positive required epart,epart2,snest,dwrap,dpass,dstrct,dowmv still fail at
over-strong whole-type partial history; dborrow's real loan cause is masked by
that same history, other five body causes have not run past it. Do not weaken
traits/loans or relabel these as negative parity. Code/CTFE physical activity
fence remains required until real scheduling/execution.

Pinned compiler laws and18 raw independent controls are now authoritative:
pa31/.../source-const-cleanup/histo.json predictionSHA194a91fc...; reference-v15
receiptSHA93755b85997ef0a6412e7ec0bbdb4c66d586d48ab197098560014234bde587a7.
All18 predicted results match, every negative E0493, no setup errors. Laws in
check_consts/resolver.rs+qualifs.rs: type possibly-qualified property first
real scoped needs_drop, then const Destruct; structural constructors propagate
operand VALUE qualification. Own Drop/union and unknown calls use TYPE property.
Projection filters projected TYPE then inherits ROOT-local qualification (deref
type-based). Assignment ORs and NEVER clears on even a full unqualified overwrite.
Whole-local move captures destination first and clears source only if unexposed.
Field move retains parent qualification. Raw/mutable/interior borrow sticky;
shared Freeze does not add it. StorageDead clears; joins OR. Tag tests cannot
clear sticky badness or erase sibling/fresh local history.

Root APPROVED concrete next implementation per source-const-qualify-design.md:
one BAD bit per existing paid Root, existing coordinate/tag/need plane for
actual selected cleanup; replace partial flag, no dense Body/Plan/Proved/Code
fields. Before flow: collect full semantic TypeId questions from actual body/
coordinate data under same sealed Initial, normal scoped NeedsDrop + typed
State::decide(Destruct) classifications NoDrop/Proven/MayQualify. Only completed
absence can be MayQualify, never errors. Retain successful evidence in same
actual Contract, recheck regions, revalidate body/store/defs/source/owner/env/
signature/Copy seal, then run existing single refinement/select. No runtime or
physical permission follows classifier answers. No-enum/ordinary allocation
fastpath required; prepay requests/maps/substitution/evidence/words/transfers.

Do NOT force the superseded two per-coordinate history-plane sketch: frozen
hcntp/hsubp/hrpre remain negative even with precise request; hcop constructed
W<T:Copy>0 vs hcpn unknown W<T:Copy>1 proves raw Need bits are insufficient.
Const Drop C must NOT itself contaminate roots/projections; actual scoped type
eligibility matters. Actual current selected leaves/OwnDrop still retain their
normal purpose-specific proof. Source precise policy may skip only genuine
non-access demand, not all physical conservative aggregate accesses.

Private foreign Selected swap: current Selected only stores BodyKey/source/
roots/rows, not body borrow. Must demonstrate actual replay-owned swap before
adding ownership state; Root prefers &'body Body not bare NonNull (valid interval).
No ownership witness added. Execution remains private-CI pending; no cold
monolithic link and no test-only public authority API. Keep this required.

Async worker c869bbb55 is committed/qualified, Root will integrate separately.
Exact union: cstage::finish unchanged7args delegates to finish_sealed returning
(Option<Plan>, SAME Seal); Completed Some(Box<Seal>) for actual poll path validates
while body borrowed, constCompletedNone unchanged. Our Initial→Prepared already
moves same seal. Preserve its new coroutines Permit Poll data and new callable
source/method joins; no source overlap beyond cstage/incremental final functions.

Resource policy stays2global heavy,1/tree,16GiB cap,150reserve,2Cargo/test threads,
<=300s total development batch incl admission; every fmt/audit/build/test/oracle
via resources.py/storage.py, wait0, timeout230 (tiny oracle90). No cold private/
release/full suite. Parent coordinates exclusive timings. Preserve every failed
receipt; no Source build/observer while another owned job is live.


## Scoped endpoint checkpoint, 2026-11-06

Fresh Initial API backup: stash631b2d3d329adcf748ae9ed3832647e946214dc5.
The actual scoped frontier is now connected through one-body, template Batch and
native const-original Initial owners. Queries resolve full semantic coordinates
and authenticate ALL constructor candidates before flow. Qualification holds the
actual body borrow, current arena witnesses and actual defining environment;
NoDrop/Destruct/MayQualify are distinct value answers. Only completed absence is
MayQualify. Genuine positive evidence is retained in the real Contract, regions
are rechecked, and the SAME existing Seal is revalidated before acceptance.
Asking/Answered/Applied rejects unfinished or repeated acceptance, even for empty
frontiers. Final Selected resolution reuses the authenticated full coordinates.
Successful exact TypeId/Purpose evidence survives subsequent owner preparation;
Copy/NoDrop remains no whole-type trait proof.

Real constructor factories were initially misclassified as source function
bodies: E::Good at5528 and Option::Some at6728 have actual member CFGs with
const_context=true and legitimately no FunctionHeader. Admission now keeps all
factories/non-Function owners on their existing sealed proof paths. No active
cleanup permission follows from skipping the new function-only query. Native
eligibility is only owner.is_some() AND body.const_context so c869's sealed
nonconst poll bodies keep ordinary physical selection. Initial refreshes its
builtin proof view after accepted region checking, preserving original Copy
identity in the SAME Seal. Final cstage finish remains untouched for c869 union.

Qualification: scope10 format/audit/strict PASS88.31s; proof4 all18 public tests
PASS126.96s (17 unchanged plus one test covering prknow/prmove/taggen/taglif).
Earlier failures are retained in qualified-scope-receipts.json: dependency/import
errors, one map_err mismatch, one type-complexity lint, one assigning_clones lint,
and actual factory/header failures. scope7 hit outer230s cleanup timeout after
recording those semantic failures; it is not a passed batch.
Measured scope7 records: Qualification56, Queries112, Qualif1, Partition456
(optional cold pointer +8); symbolic/native Prepared784/952, Initial728/776,
InitialRow960, State800 unchanged.

BAD/history semantics are NOT connected yet. The original seven positive gaps
remain; the new18-law candidate test has been authored but not observed at this
checkpoint. It must preserve all raw independent predictions/cause evidence.
BuiltinQuery storage/work accounting through scoped NeedsDrop remains a genuine
required borrowed-adapter join in the existing matcher; its cache is not currently
included in NeedsDrop retained_bytes. The no-enum hcop positive has not been
observed on candidate, and enum-only partition admission is retained. No claim
that every ordinary/no-enum constant path does zero extra indexing work. Actual
private hostile execution, broader scaling/native and real CTFE cleanup remain
required. Current active plan remains physically authoritative and Code is fenced.


## Root BAD implementation continuation, 2026-11-06

HEAD/branch unchanged79dbce723 / implement/source-selected-leaves-20261105.
All new producer/consumer changes remain uncommitted. Last durable stash of
qualified scoped-only endpoint: a0e02cf9852cca16af41fc78644617e8d22b8846.
No new private execution/native/CTFE claim. Root now main5f5918758 with c869;
our final cstage finish body is still available for SAME-seal union. Root owns
lower/proof select_cleanup; Source changes only the approved push_inner ordinary
call passing (input, requirements.as_ref()).

Actual source BAD+qualification-borrow bits are now wired through the ONE existing
initialization fixed point/replay. Root28, Node68, Row60, Term12, Queries136,
Partition480 measured in qualify-badsem1/selected.stdout; ordinary Prepared
records unchanged. Broader existing tag/alias exposure is kept separate. Sticky
BAD survives overwrite, only unborrowed whole moves clear source after the
pre-statement capture; fields retain parent BAD. Owned projected type filters
are compiled once per semantic coordinate from full answered types and parent
masks. qterm::Source is emitted by the existing typed constructor/value matcher;
all actual ctor candidates still authenticate before any flow. No second parser
or source authority. Cold Terms conservatively retain unknown unmaterialized
provenance; NoDrop type is not itself whole-type proof. NoDrop reference aliases
use a prepaid immutable single-store index/memo, not per-transfer local scans.

Selected now keeps explicit history leaves even when physical live_type=None;
actual access leaves still require physical live_type. History access distinguishes
actual partial roots using existing must-init facts from full contaminated values.
Precise policy cannot erase an active contaminated local. Physical Plan/CFG/read/
loan/Code fences remain independent. Targeted admission borrows actual
Requirements.values and validates all locals/values/projections extents under the
same Initial owner/arena. Actual requires-drop structural Aggregate/Repeat/Call
and now Read values seed only defining eligibility; no runtime/dropless broad seed.

qualify-bad3 static PASS124.48s. First full semantic gate86945 / qualify-badsem1
terminal semantic101147.34s. Original16 positives:15PASS, taglif regressed.
All7 earlier historical positives epart/epart2/snest/dwrap/dpass/dstrct/dowmv PASS.
All19 negative cleanup opponents PASS. Frozen18 now17/18 matching Rust: old four
false accepts hmraw/hrpre/hcntp/hsubp all correctly reject; hwrap/harray newlyPASS;
hcop remains Rust0 rejection. Five body causes passed; dbound actually has a
mistaken Clone test expectation (see below), not a compiler repair requirement.

qualify-qtrace1 typed observer PASS11.50s, complete new5 candidate collection
code101 .383s: rfield Rust0 accepted, mfield Rust1 wrongly accepted (HARD BLOCKER),
gcon/rlocal Rust0 wrongly rejected, gmiss correct negative. qscope.json predictions
and all5 independent Rust outcomes are frozen in reference-v16; rfield/mfield
exercise NoDrop-field vs MayQualify-field raw borrow before whole-root move.
Actual typed storage/value traces are retained in qtrace1/selected.stderr.

Typed cause corrections authored after qtrace1 (NOT semantically requalified yet):
1. Structural Reads must seed: S-param body has only S reads + raw field Address,
   so mfield otherwise has no enum/constructing value and no qualifier partition.
2. Unmaterialized immutable parameter reads use their genuine initial full-type
   qualification before the conservative Projected(root) fallback. hcop's T:Copy
   otherwise incorrectly became bad merely because its ROOT mask is true.
3. taglif/rlocal contain actual Reference reborrow through Dereference followed
   by Constrain. The existing pinned law propagates base reference value, not
   arbitrary referent badness. qterm expression dispatch now recognizes that exact
   typed reference-base chain and follows immutable reference alias initialization.
4. dbound.rs unchangedSHA6f27aae... actually calls need::<N>() with T:[const]
   Destruct. reference-v9 froze E0277 N:[const]Destruct absent. Candidate can't drop
   N is the correct explicit call debt; ctsel wrongly expected Clone. Corrected
   expectation without source change. Preserve negative and original evidence.

Latest static gate57163 / qualify-bad4: formatting/audit/typing passed; one routine
collapsible_if lint. Fixed that source-only. No next heavy gate yet. Root requested
exclusive AA/ABBA hold after57163 and Source acknowledged idle; DO NOT launch
until Root explicitly releases. Then run focused static qualify-bad5 and full ctsel
semantic retry (single suite_traits target) under unchanged guards. All7 tests now
collect qscope outcomes independently; none is relabeled a Rust-negative fence.

Still required: eliminate mfield hard false accept and hcop/taglif/gcon/rlocal gaps;
all unchanged cdrop/control/native gates; direct source/body/arena/permission/purpose
hostile evidence (private execution CI pending); borrowed BuiltinQuery work/storage
adapter (acknowledged real gap; no overlap edits by Root); no-enum/noise/shared
source/scaling bounds; actual original/runtime-instance witnesses and actual CTFE
cleanup scheduling/execution before Code. Source consumer is NOT integration ready.

## Root-local qualification checkpoint, 2026-11-06, after badsem2

The exclusive Root measurement completed and released its hold. Static bad5 passed
129.22s. Full semantic badsem2 (PTY89561) ended101 in125.20s: all original16
positives, scoped4,19 cleanup negatives and6 ordinary body causes passed. The four
previous false acceptances remain rejected. History17/18 leaves only Rust-positive
hcop rejected. All five qscope controls were observed independently: rfield,
rlocal and gmiss match; gcon remains a positive rejection, and mfield remains a
hard false acceptance. Never integrate while that false acceptance persists.

Portable backup obj/parallel/source-transport/va-cleanup/badsem2-backup binds HEAD,
tracked patch and all untracked bytes. Stashaf4409679f5105dcbc94cef5e8e8d93e57e467ca
was retained and reapplied after merging main9a04f7dc2. Current merge HEAD is
2a450feb0521547dfcf807d67b63d272ebcae9fa. Only incremental.rs import conflict needed
union: loaned module plus Completed/ConstInitial/with_const_original. The actual
c869 finish_sealed/SAME-Seal and generated CopyView law remain intact.

Typed observer qdiag1 passed110.55s; qdiag2 passed106.55s. Actual hcop/gcon body
parameter and caller subject are both TypeId73=Parameter14: no opening mismatch.
hcop's genuine Copy premise is Trait(CopyApp7), which the low-level builtin-only
NeedsDrop assumption query intentionally does not treat as builtin Copy. The
actual source-copy body producer already owns selected read decisions and debts.
Root approved optional O(1) lookup through that same Prepared with unchanged
body/key/source/instance/signatures/requirements-Arc/environment-Arc checks.
Some(true) filters only value qualification to NoDrop, never whole Destruct or
const execution. Missing required producer, stale/foreign owner and failure remain
errors. Existing demanded decision errors stay unchanged. New private optional
lookup opponents are authored; execution remains pending CI.

gcon's exact caller clause is MaybeConstTrait(DestructApp4). Qualification now
uses the pinned const-function Conditional query law, retains the genuine proof
in the actual Contract, and never converts it into Selected(Const). Execution
Prepared.effect is unchanged. History leaves retain their authenticated snapshot
access independently of a later absent physical cleanup node; ordinary selected
leaves still require physical access. This is the narrow mfield correction Root
approved, not read/loan/cleanup permission.

qualify-badsem3 is the current source-frozen format/audit/all8-ctsel semantic batch;
its terminal receipt, not this note, determines its outcome. Root owns the shared
paid builtin adapter in permissions.rs/instantiation.rs/radix_map. Its proposed
assumed_with_charged API is not yet consumed here; bounded destruction hook stays
Source-owned. The real accounting gap, hostile private execution, scaling/native
qualification and CTFE cleanup execution all remain required.

## Actual original-row admission repair

badsem3 passed format/audit and7/8 tests in112.30s: all18 history laws now match;
hcop and gcon are fixed, all previous positives/negatives/body causes preserved.
Only mfield still falsely accepted. Its actual ordinary lowering path passed
requirements=None from push_original, while the real original row remained in
program.drops.bodies. Thus its constructor-free S read never seeded a partition;
no qualification bits/snapshot existed. This was not an E/C/N classification
failure. Root approved a transient Originals owner carrying the SAME moved Body
and Requirements Vec payloads with exact count/extent preflight, paid reverse/pop,
prepaid old-live/new row-container overlap and reservation before take. Ordinary
no-drops remains None/no row allocation. Actual owned row now reaches push_inner.

A second nested production caller in demand/paid_owners was missed by the initial
nonrecursive search; corrected inventory and Root were informed. It now consumes
the same paired rows and appends each only after its actual retained parent/loan
proof. Its unimplemented source const value-history case has an explicit failure;
row presence grants no const permission. Parent region-group checks retain their
original program order; symmetric reversal happens only after their borrowed
input preparation, before pop. No Body/Builder field growth. New actual original
constructor-free row, missing/extra/extent tests are authored but private execution
remains CI pending, alongside existing foreign body/input/arena opponents.

bad6 stopped solely at repeated ordinary filename stem7; the module was renamed
cohesively origins.rs. bad7 reached4 missed paid_owners API compile sites and
stopped. bad8 fmt/audit/strict passed (16.75/2.78/67.41s), before the source-only
region-order refinement. No semantic retry after the row join yet. Root requested
an exclusive paid-query measurement window; Source is idle/HOLD until explicit
release. On release run format/audit and all8 ctsel semantic controls first, not
broader suites. mfield remains a hard blocker until that exact source rejects.

## First complete frozen semantic group and paid hook

badsem4 is authoritative PASS: fmt16.51s/audit2.76s/all8 ctsel105.52s.
Exact original mfield now rejects at file0:5632 with its actual const Destruct
source-definition debt; rfield accepts. All18 frozen history laws match, gcon and
hcop accept, all earlier positives/negatives/ordinary body causes remain intact.
Receipts and source fingerprints: obj/parallel/source-transport/va-cleanup/
qualify-badsem4/{receipt,sources}.json. Root28/Node68/Row60/Queries136/Partition480.
Source has not claimed private execution, general CTFE or whole library closure.

Adopted published Root c2b851a2e/8d0c2113b after portable badsem4-backup and retained
stashc38115b78cebb3700a5ba683d9eb01553a3c0e5d. Current merge2470bb2068a38ec62959faec5cc10f73bd1d97ee.
The exact same builtin Query now has a bounded NeedsDrop borrowed payer. One
builtin_bytes usize in existing cold optional Scope (Scope64, +8) retains paid
Box/Shapes/Matches/radix growth, using the same steps/limit; both Ok/Err keep
consumed costs. Existing own-state/work/custom allocation is counted separately
from matcher growth, exactly once. Warm unpaid/unavailable query fails BEFORE a
cached Ready result. Later unobserved ordinary growth marks a previously paid
counter unavailable (usize::MAX), preserving ordinary query results and denying
later unbilled bounded reuse. No dense record, second matcher or permission ledger.
New narrow cold/warm/partial-error/foreign/unpaid cost tests are authored.

cost1 fmt/audit passed, stopped at one combined101-line prepare_ordinary lint and
one private qcost mutable borrow. Both fixed. cost2 fmt/audit passed; all production
and private API typing passes, sole strict failure is public CPF collector missing
semicolon. Fixed source-only. Private execution still CI pending. Root requested
exclusive measurement hold after cost2; Source is idle/HOLD until release.

Before candidate observation froze qcopy.json and reference-v17. All4 Rust
predictions match: cpfld unknown W<T:Copy>{value:T} unused const parameter0;
cpmiss unbounded equivalent E0493; cpbad nonconst N sibling E0493; cpwhol explicit
whole W<T>:[const]Destruct with only T:Copy E0277. Candidate results are still
PENDING; the public test collects all4. If cpfld reveals a gap, Root approves
reusing actual source copy_decisions on the authenticated semantic question
frontier, retaining real scoped Copy proof only as NoDrop, then seeding the SAME
fresh NeedsDrop sparse state before any compound demand. No rewriting completed
parents, no original Copy carrier/loan Seal mutation, no Whole/OwnDrop shortcut.
First observe candidate; do not infer NoDrop from a missing read decision.

## Final semantic milestone, 2026-11-06

badsem6 preserved old8 but CPF still failed: no S-valued expression existed in
cpfld, only its real initialized Parameter local. Root approved StorageLive
parameter admission under exact authenticated Requirements.locals==true, reusing
same structural_type law and same single indexed scan. Existing ownership.rs
initializes Parameter/Variadic; paths.rs authenticates actual Variadic structural
preflight. Uninitialized Binding is not admitted. Root also approved replacing
Qualification's environment Arc with Arc of EXACT published defining State in the
same8B slot. State.checked still checks body/source/signatures/env; arenas/body
borrow remain unchanged. Old/unrefreshed state rejects, including empty answers;
normal mandatory refresh/Initial Seal remain the actual debt/authority boundary.

cost3 stopped at qualifier112-line lint and one private unnecessary-mut. Cohesive
Qualification::published constructor resolves function size and authenticates the
actual owner. badsem7 caught its missing mutable facts borrow before semantics;
fixed. badsem8 is authoritative PASS: fmt18.46/audit2.99/all9 tests148.86s. All27
new independent controls now match, along with original16/scoped4/19 cleanup
negatives/6 body causes. Five prior hard false acceptances remain rejected. CPF
positive accepts; missing Copy/nonconst sibling/explicit whole bound retain their
real distinct causes. Private exact state/empty/unpaid-debt tests compile; execution
is CI pending. No CTFE cleanup or conditional-parent history closure claim.

Portable badsem8-backup +stash689112af35cce7d434dc3c2dbd0bdbd1342db460 retain that
source. Adopted Root da066 then cbe6b25b2 with no source conflicts; current merge
f6b08c3496d53497e678c773783d212757cc51f1. No pending edits to Root sizing reuse or
pointer/Index consumer repairs. cost4 on this source passed fmt17.28/audit2.74 and
all-lib+tests/all-features strict metadata111.64s. All private/new APIs compile.
Final1 (89162) currently runs exact prior cleanup/source controls, all9, harness/
upstream and three genuine runtime native programs. Read its terminal receipt.
No other owned heavy job exists; same resource limits/guard always apply.

## Qualified finite handoff

Final1 preserved first two typed-cause expectation failures (prunk Body Destruct;
pract OwnDrop), and final2/3 preserved impro3 type-row advancement and egood's
actual acceptance. No source/oracle was changed. Prior Rust0 receipts and exact
hashes for egood/empty were checked before replacing the old named-gap assertion.
Final2/3/4 together pass44 regular public tests. Final4 also passes all10 ctsel,
harness/upstream and native3 actual programs (41/47/59). The bounded8 vs1048576
array work is exactly1328/1328. Field-fold growth28/124/508 over422 and demanded
body growth1473/5889/23553 over768 are retained; no throughput claim.

Measured Scope is64 (custom HashMap layout), +8 over56; corrected the unexecuted
private size assertion before final strict. Qualification56 and Origins48; other
reported sizes unchanged. strict1 final lib/suite/CLI all-feature Clippy passes
3.20s after fmt17.21/audit2.64; cost4 full lib+tests/all-feature metadata also passed.
All new private APIs/tests compile, execution remains CI pending. Formal handoff:
source-selected-cleanup-checkpoint.md and source-selected-cleanup-evidence.json.
No live Source job. Commit this coherent checkpoint, then follow Root review;
actual CTFE cleanup and conditional-parent value history remain active required work.
