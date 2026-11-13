# RustGM handoff — 2026-11-13

The goal is to compile the pinned original core, then alloc/std, completing
and releasing assignment components as their contracts qualify. Continue as one
worker. **No original library artifact exists yet.** Part A's zero-sized niche
companion change is published and clears core's actual `Result<(), Repr>` size
assertion. Candidate13 also clears all six borrowed-closure failures/blocked
bodies. Its next actual failure is Alignment's structural constant pattern at
core/src/mem/alignment.rs:29. This is still an incomplete analysis-only census.

This supersedes earlier stop instructions. Historical checkpoints and retained
requirements are in Git (`git show 03003a86f:handoff.md`); every retained requirement
there still holds (see [Standing requirements](#standing-requirements)).

## Workspace and Git

- The main checkout `/home/vishvananda/rustgm` is Git-only and never builds.
  Edit and build in B, `/mnt/vizier/rustgm-scratch/rustgm-pa26-work`, branch
  `implement/scoped-inherent-20261110`. Publish with `git push origin HEAD:main`.
- Part A's published implementation is `34a28702e`, descending from handoff
  e6610fae3. The clean release candidate11 binds that source. Documentation-only
  descendants may follow without rebuilding it.
- Push CI is bounded smoke/harness/upstream, not a full qualification. The latest
  verified full success is [43239321921](https://github.com/vishvananda/rustgm/actions/runs/43239321921)
  on exact a7ce2784a; fast and qualification both pass. Current candidate11 has
  focused, course and original-census evidence; full successor qualification is
  pending and owned by scheduled CI. Do not dispatch a manual full per edit.
- The old uncommitted part-A patch is integrated. Its raw restoration patch and
  earlier failed receipts remain under `obj/parallel/zst-niche/`; do not reapply it.
- Bool's implementation and current input contracts are qualified; see
  [the checkpoint](docs/pa31-bool-niches.md). There are21 active PA31 cases,
  three native observers and one required nested-niche positive in pending.json.
  Independent Rust22/22, API21/21,18 private controls and current native programs
  pass. Full9307 course references are unchanged. No new oracle is generated.
- Clean source8950ad99c freezes candidate12, CLIbca8b9e1,3945 source hashes and
  an empty patch. Original observation/postcheck/comparison and48 exclusive
  A/A+ABBA controls pass; actual7631 clears with no reached-body regression.
- Commits use the repository's single-line subject with no trailers.
- The backend stays pinned at v3 (17b024f69). Don't modify the C++ or v5rust
  checkouts or other checkouts' outputs. Work as one worker, with no new
  worktrees.

## Current checkpoint: inferred input proof identities

Published implementation ba6e731e2 (origin/main and B) qualifies the inferred
closure input repair. The historical private-port draft below is superseded;
never restore it. Details: [checkpoint](docs/pa24-inferred-inputs.md).

- All75 declared outcomes pass: PA20 ten reference-tail cases, PA24 twenty-eight
  inherited-input cases, PA31 thirty-seven actual-source cases. All inputs are
  manifest-bound.26 ordinary native programs and two actual Rust/C consumers
  (six exports) pass; outputs are13/29/0/13/13/1 and13/13/13/29/29/29.
- Public receipts1794586372014375955 and1794587214204194259 pass31+5 exact
  selections. Native1794586605899163087 and1794589096793583294 pass the actual
  consumers and captured-drop control. No temporary trace code remains.
- Marker scaling8/32/128 has inference641/1625/5561 and canonical2329/6193/21649,
  exactly linear for this family. Raw hash-checked counters are retained.
- Exactly38 new PA20/24 reference triples were independently declared, generated
  and reviewed. All28,260 existing sidecars are unchanged. All9345 active PA1–24
  comparisons and separate113-case released PA25 component pass. Upstream,
  315 harness tests,486 integration roots and44-stage/component layout pass.
- New private opponents cover changed public/inherited regions, unpaid port
  debt and changed source Copy instance/body/environment identity. None executed:
  cold strict lint/link repeatedly exceeded the bounded267s compiler window.
  Do not describe intended selections as passing or repeat cold bounded builds.

Clean immutable candidate13-release binds ba6e731e2,3946 Rust sources, an empty
patch and CLI SHA256a8307de89f5ba1bcfb6772e5dc193ff4de84bc7cbe50bcceee5974ae0b9cba67.
The initial270s release attempt timed out. A formal unchanged-source build then
completed in7m08s under the same16GiB cap/150GiB reserve. This is a profiling
checkpoint, not permission to rebuild release per edit.

All48 frozen A/A+two-ABBA comparisons against candidate12 pass complete stdout/
stderr equality at widths8/32/128 plus an ordinary control. Tiny noisy timings
make no speed claim. Raw data: obj/parallel/closure-return-inputs/measure-candidate13/.
The successor original observation and all postchecks passed; see Core census.

Known full-CI baseline failures remain real work. Full43415545351 on bd0d003b4
had3062 private passes,8 failures and3 ignored, while all13 public targets passed.
The failures are: source_seal_precedes_declared_specialization_and_required_stays_unpaid;
format recipe Output layout472->496; derive Unit688->712; gated Syntax312->336;
two Methods layout guards312->320; the Const-to-Conditional proof matrix; and
completed-parent GAT proof scaling (width8 work1866,32 work18114, quadratic).
Inspect the intentional cold storage additions before updating layout guards;
preserve the source-seal opponent and scaling bound. Raw failed CI and triage:
obj/parallel/closure-return-inputs/ci-43415545351-failed.log and ci-baseline-triage.json.
Push fast43470491776 on exact ba6e731e2 passed; qualification was skipped by event.
The most recent known full success remains43239321921, not the current checkpoint.

Next: reduce actual Alignment constant-pattern failure, preserving derive_const,
real source trait roles and independent structural-pattern opponents; repair the
known full-CI proof/scaling failures coherently. Appropriate ordinary reductions
belong in their earliest owning PA; actual core/language-item reductions in PA31.
No original library artifact or planned whole assignment is complete.

## Current B continuation: structural constant patterns

Source-pattern repair864ad9c6c is published on origin/main and both checkouts. Candidate13 remains the
last original-core observation; no successor core run/artifact exists yet.
See [the checkpoint](docs/pa31-structural-patterns.md) for the exact contract.

Source admission now uses actual PartialEq/StructuralPartialEq, visits only
selected variant fields, and checks applicability modulo regions. This classifier
grants no trait/call evidence; private witnesses become allocator gaps, preserving
actual constant/initializer/borrow proof. One predicted lifetime rejection was
disproved by Rust and corrected to a required positive, with history retained.
Actual type-bound opponents remain enforced. Standalone structural headers use
the existing cold role ledger. Local derive lists retain their real API policy;
the actual name token distinguishes literal allowance from captured meta input.
A temporary per-owner index is prepaid and built once, without dense record growth.

19 source outcomes (12 accept/7 reject), full prepared originals, and three actual
native exports pass. Outputs match independent Rust/C:1 0 1 0 1 0 0. Public receipt
1794594933591102663 passes7 exact existing/new controls;1794595182597516639 passes
final2 source/scaling controls;1794595309828174033 passes finalnative. Strict
selected/CLI Clippy, format/audit, upstream128+126,315harness,487roots/13binaries,
44-stage/component layout pass. No reference changed. Source/classifier widths
8/32/128 give inference5024/6752/13664, canonical4047/8055/24087 and actual trait
queries16/64/256; hash-checked replay is under structural-constant-patterns/.

Owning PA31 packet has all19 byte-bound inputs/C observer. PA20's four ordinary
controls are serialized in their own manifest, following pending-input precedent;
no orphan .rs or expected-positive inversion. Empty derive-list benchmark input
is independently accepted at8 but baseline-rejected by the existing vocabulary
introduction/edition guard; larger widths/current candidate are unrun. All bytes
and required positives remain serialized in PA31's pending_inputs, not counted.

Immutable exploratory wippattern2-dev-fast (3948 source hashes/dirty patch) is
CLI56485edcef6d8823074fd1a0cd153d0409282bd3bce9077a7e649174d4e0b1a1.
Wippattern1=753f5cd6 is older, with a known captured-attribute gate bug; do not use
it as the final candidate. Pin.py now asserts3948, for the two real new Rust files.
All four nonempty paired-derive8/32/128+ordinary measurement inputs independently
accept and have byte-identical outputs between wipclosure5 and wippattern2.
A/A+ABBA timings await load14/20; last actual host load42.88/57.73. Keep both
comparison binaries until observation, and retain12/13 release baselines.
Raw failed setups/fastchecks and pin patches remain under
obj/parallel/structural-constant-patterns/; no temporary trace code exists.

Next finish calibrated observations when admitted, publish the source checkpoint,
repair known predecessor CI failures and build a clean release at a coherent
checkpoint before the unchanged120s full original observer. Avoid unoptimized
full-core timeout repeats; no compiler/profile/limit/upstream-byte change is
permitted. Then continue to an actual core artifact, not merely a clean census.

CI investigation (not yet changed or verified): dd984cad9 now gives original
closures their creator signature's implied b:a premise. That may make seals.rs's
old "unpaid" fixture already paid. Preserve the opponent by explicitly removing
only the exact exported relation for the manual and Declared comparison, and
assert absence; do not weaken proof or waive failure. The GAT scaling failure
appears to involve implementation-only normalization children skipping completed
parent proof lookup. Preserve Sources/Scope/positive-source distinctions; keep
the5x scaling guard. Five stale layout tests and the Const->Conditional matrix
also remain, as listed above. No new private tests have executed locally.

## Current B: predecessor CI repair awaiting private execution

The coherent eight-failure repair is in B; [details](docs/ci-parent-proof-repair.md).
Only production change: implementation-first normalization can reuse an actual
positive Implementation parent from this same query/caller scope. Caller-only,
absent/builtin sources cannot cross modes; existing witness/scope/provisional
cache guards remain. The GAT private5x assertion is unchanged.

Five exact layout guards now account for the already intentional per-Syntax
pending-tail Vec and cold Methods smart-name box: Syntax336/Unit712/Output496/
Methods320, keeping Node28/36 and total Declarations budgets. The nine-case
constness matrix retains ordinary-evidence rejection and permits the already
qualified Const->Conditional pair. seals.rs now asserts/removes the exact creator
implied b:a relation, using that narrowed contract for both Required rejection
and Declared acceptance; no production seal permission is relaxed.

Public1794597008570879431 passes8 exact tests across projections/traits/frontend,
strict selected/CLI Clippy, full format and audit,185.43s despite host pressure.
All eight private repairs and preceding borrowed-input private opponents still
need execution. Do not call them passing. A single coherent full manual successor
is next after publication; avoid another cold local link timeout.

Clean candidate14-dev-fast now freezes source864ad9c6c,3948 hashes/empty patch,
CLI56485edcef6d8823074fd1a0cd153d0409282bd3bce9077a7e649174d4e0b1a1.
The CI patch is later and is not included in that pin. No14 release or original
observation exists. Calibrated observations remain refused by the14/20 host-load
gate; last observed42.88/57.73. Source/checkpoint docs and packet metadata hashes
were separately rechecked after documentation edits. Current original evidence
remains13, stopped at Alignment; no original library artifact exists.

## Historical private-port draft (superseded by resumed checkpoint)

Published compiler source remains8950ad99c; later commits record documentation.
B now has six modified compiler files carrying a
**prototype**; no implementation from this continuation has been committed.
`obj/parallel/closure-return-inputs/private-port-wip.patch` and
`private-port-wip-state.json` preserve the full patch and current source hashes.
Do not describe this prototype as qualified or as clearing original core.

The bounded diagnostic traces establish a duplicate-universe import. In both
actual26791/26792 and the ordinary `(|&s|s)(x)` reduction, a parent's canonical
bound slot enters a closure input hole as a new child universal beside its
existing inherited alias. The output closing map correctly refuses that new
slot. This is an identity transport defect; keep the lifetime boundary check.

The first actual fix adds nested-body scheduler presence to `shape_equation`'s
existing alias preparation in `dev/src/sema/bodies.rs` (three added lines).
Exploratory pin `wipclosure1-dev-fast/rustgm`, SHA5f9764341b069902d1e90bd92c3f70b5a3a9dc56fdc2fcf3d8e6d327608b2752,
binds that patch alone against814f17db3 with3945 source hashes.
Its `wip1-ordinary/results.json` checks the unchanged eleven executable ordinary2
inputs: all eight positives type; seven lower. The generic pattern now reaches
the already recorded private-input-port instantiation gap, "closure signature
lifetime interface differs from its body". All three opponents still reject
prepared lowering through ordinary lifetime proof. Typing alone accepts those
opponents and does not constitute full body proof.

The remaining five modified files draft the generic port repair. Noncapturing
signatures with no public late binder match their instantiated body roots using
the existing typed closure matcher. The inherited prefix must agree exactly;
private slot connections, repeated-region equalities and projection debts enter
the existing instance contract. Captured execution still requires its retained
conditions. Argument substitution and signature normalization are shared with
captured execution. This draft has **not compiled, run formatting/Clippy/audit,
or run any regression**. Public late binders mixed with private ports remain a
required case to investigate, rather than a solved contract.

All temporary TEMPTRACE printing and selected-body scheduler edits were restored
before the three-line fix. Trace1's unchanged120s full observer timed out before
the affected bodies; trace2's explicitly filtered diagnostic reached both in
87.94s. Filtering was diagnostic only, never full-census qualification. Its
original source722/open375/config bindings remain unchanged. Trace1/2 pins are
retired (cleanup1794566414828881647); their producers, patches and raw observations
remain. Current original census is still candidate12, below.

Inputs frozen before any run:

- ordinary2: eleven executable programs, already independently accepted/run or
  rejected by Rust1.98.0; wip1 results as above.
- ordinary3: twelve additional generic/port/projection/nesting/opponent programs;
  `declared-ordinary3.json` and exact `ordinary3/` bytes. **No oracle/candidate run.**
- source1: thirteen actual-language-item source reductions, including original
  `str/pattern.rs` macro structure and a scalar transposition with three real C
  consumers. `declared-source1.json` binds every source and the C observer;
  declared native stdout13/29/0 then13/13/1. **No oracle/candidate/native run.**

These raw inputs have not yet been installed in an assignment. After independent
qualification, ordinary reductions belong in PA24's manifest/references and
actual language-item/core reductions in a PA31 packet manifest. Keep the original
positive outcomes and failed setup history. Select both existing
`source_closures::specializations::generic_closure_programs_have_checked_specializations`
and `inherited_lifetime_frames_keep_late_closure_inputs_distinct`, plus existing
input/creator/lifetime proof controls, before a new clean pin and unchanged120s
**full** original observer. Actual source exports/native and bounded scaling
remain required; no new core artifact is implied by this draft.

Storage currently prevents validation: the shared volume fell below the unchanged
150GiB reserve (about143GiB free on the third recheck). Other project builds are consuming it. The
user has an asynchronous request to free about10GiB elsewhere. `make tidy`
(1794566729541174850) and `make tidy-build` (1794566809851110788) completed; the
rebuildable Cargo cache is gone, both current immutable pins remain. Resource
admission refused the subsequent build-only attempts, including smaller actual
absolute caps, before Cargo; **wipclosure2 does not exist**. The guarded archive
attempt was also refused before mutation, so no closed root was archived by it.
Do not lower the reserve or touch another checkout. Resume checks after headroom
returns. After the same storage barrier persisted for three consecutive goal turns, the
goal was marked **blocked** through update_goal. No original core artifact exists
and no PA is newly complete. There is no live RustGM process to wait on. The
required external change is roughly15GiB of shared-volume headroom; then the user
can resume the goal. The objective and preserved uncommitted work are unchanged.

## Core census

Candidate13-release (clean ba6e731e2, CLIa8307de8) is current. The unchanged120s
exclusive observer finishes in23.8284s under identical compiler limits/profile:

- all42310 declarations pass;
- 35120 item attempts check25858 bodies, with no failed/blocked functions;
- actual26791 into_searcher,26792 is_contained_in and26793–26796 all clear;
- no previously passing reached body fails;
- one crate-category error stops initializer39106: Alignment's matches!(a,
  Alignment::MIN), alignment.rs:29, "constant pattern type must implement PartialEq";
- completion remains0: no complete census or original artifact.

Postcheck verifies722 upstream sources,375 opens, six configuration bindings,
3946 current/producer compiler hashes, empty tracked patch and unchanged limits.
Observation/comparison/postcheck: obj/parallel/capture-match/original-candidate13*,
obj/parallel/closure-return-inputs/postcheck-candidate13.py.

Alignment and its field enum use actual derive_const(Clone, PartialEq, Eq).
Source derive expansions are completed, while the constant-pattern path still
queries the ordinary equality-derive table; diagnose the missing typed proof
connection without trusting attribute spelling or accepting manual equality as
structural matching. No fix/reduction for this next blocker is installed yet.

A core build additionally requires completion, genuine original metadata/body
transport and native consumers, and backend support beyond v3's initial ymm/zmm/
kreg assembly profile. Passing the analysis-only census does not complete PA32.

## Completed: zero-sized niche companions (part A)

[Implementation and qualification](docs/pa31-zero-sized-niche-companions.md).
Source nullable eligibility now admits two single-field variants when one is
zero-sized and no more aligned than the other's existing carrier. Layout chooses
the actual orientation per instantiation; constant reads, construction, initializers
and imported-header re-derivation preserve those same variant identities.

Eleven declared PA31 outcomes agree with Rust. Fifteen private controls pass in
overall-timeout receipt1794558954797138354; ten fresh integration tests pass in
1794559299273190847 after fixing the ABI test's missing no_mangle exports. Every
active text case passes (9307/9307); upstream and315 Python harness tests,485
integration roots and44-stage layout checks pass. No references are changed.
Forty-eight frozen exclusive A/A+ABBA observations at widths8/32/128 and an ordinary
control preserve complete outputs. Short timings/noise make no speed claim.
Source34a28702e freezes release CLI58d7312a with3945 source hashes and an empty patch.

The larger tagged fallback remains I64, wider scalar patterns retain their
unsupported-profile error, alignment may not increase, and the declared orientation
is tried first. Course eligibility is unchanged. Raw evidence is
`obj/parallel/zst-niche/`; the actual core assertion is now verified clear.

## Completed: bool niche (part B)

[Implementation and qualification](docs/pa31-bool-niches.md). `Carrier::Boolean`
provides empty_bits=2 and byte storage. An explicit bool_niche flag admits it
only for source enum policies and validated imported source field groups. Both
cache query orders preserve the course layout; standalone transparent-wrapper
ABI keeps its existing profile. CTFE, validity, transmutability, construction,
matching, cleanup, initialization and zero-data classification consume the actual
carrier value. No backend or compiler-limit change is made.

Current API receipt1794562814860019382 checks all21 outcomes; native
1794562983528985835 passes three programs, including false/true destructor counts
`0:0 1:0 2:1 2:1`. Four unchanged cell programs retain passing native evidence.
Initial11 integrations pass1794561838421695979,18 private controls pass
1794562152284043406. All9307 active references reproduce with CLIf042c27d,73.58s.
Upstream/harness/485root/44-stage checks pass. Only test expectations/declared
cleanup inputs changed after the initial run; production hashes and CLI match.

Twenty-two independently declared outcomes agree with Rust. Batch1's four rustc
ICEs and later incomplete no_core prefixes remain setup failures, not language
rejections. The exact21 inputs and three C observers are bound by the owning
packet manifest. pending.json retains the Rust-positive second unused-value
contract; f042c27d still rejects it at its size assertion. No unsupported positive
is inverted and PA31 stays planned.

Candidate12's clean source8950ad99c/CLIbca8b9e1 clears actual7631. Its
postcheck and reached-body comparison pass, as do48 immutable A/A+ABBA
observations with complete output equality. Short timings make no speed claim.

Next reduce actual26791/26792. Eleven frozen ordinary controls and their
independent Rust/baseline11 observations are in `obj/parallel/closure-return-inputs/`.
Inferred immediate/named/reference identity closures reproduce the output-lifetime
error; annotated/contextual twins pass typing. The first LowIR probe omitted main, and the library retry supplied artifact
flags to a typing view, producing usage status2; neither qualifies prepared
bodies. ordinary2-observed/results.json now uses executable consumers: independent
Rust accepts/runs eight positives and rejects three opponents. Baseline12 rejects
five inferred positives at typing; three annotated/contextual twins lower. All
three intended negatives reject in typing or prepared lowering, including the
static escape and annotated-elided input. Keep those phase distinctions.
The diagnostic capture and current draft are recorded above; do not repeat the
selected-body trace as a full observation.
Do not relax all output lifetimes: annotated elided returns and local/static
escapes must still reject. Remove temporary trace code before qualification.

A separate source-layout deviation remains: repr(Rust)'s fallback I64 tag differs
from Rust's smallest fitting tag. Result<(),()> is8 versus1, and ControlFlow<u8>
is16 versus2. General scalar/multifield and further unused-value nesting remain
required; course outputs must not be moved without a reviewed contract.

## Recent checkpoints

- 34a28702e, zero-sized niche companions:15 private/10 integration controls,
  all9307 course cases, upstream/harness and48 output-equal calibrated observations.
  Release candidate11 clears actual25396 without a reached-body regression.
- 99a409d03, body projection normalization
  ([evidence](docs/pa31-body-projection-normalization.md)): 17 cases, and six
  independently reviewed references. Candidate9 cleared `first_set` and both
  `try_from` bodies.
- 534f3cc67, global clause premises
  ([evidence](docs/pa31-global-clause-premises.md)): a normalizer's concrete
  premise selects implementations before a caller's global where clause
  (rust-lang/rust#50825), with 25 cases. Candidate10 cleared `div` 36996 and
  nine `div` and `rem` bodies.
  - Focused receipt 1794556164673085248 passed, and the frozen CLI passed all
    9,307 active text cases.
  - Recorded tradeoff: implementation-first frames skip the goal cache. 256
    clause-free `select` copies exhaust the crate-wide source method selection
    budget at function 222 instead of 229, and candidate9 already exhausts it
    at 128 copies.
- 0ceecbfe6 and 03003a86f, the candidate9 and candidate10 observations.

## Pins, artifacts and storage

- `obj/parallel/capture-match/candidate13-release` is current; candidate12-release
  is the retained immutable comparison baseline.
- Candidate10 and wipniche1/2 were retired after publication of11; cleanup
 1794560721836625856. Their producer/raw files remain, their binaries are absent.
- Candidate11-release and wipbool1-dev-fast were retired after publication of12;
  cleanup1794564483313447992. Their producers and raw observations remain.
  Wipclosure1–5 are superseded exploratory pins; retire them with scoped pruning
  after preserving producer patches and receipts.
  The managed Cargo cache/CLI has since been retired for space; rebuild before use.
- Current pinning asserts3946 development Rust files, including creator_inputs.rs.
  Update the count only for actual new files, never by dropping tests.
- Last free space219GiB; storage admission works. Controller2jobs/150GiB reserve/CPU800%/RAM12GiB/
 256tasks and the16GiB output cap remain unchanged. Storage's40GiB local default
  does not reduce that shared reserve. Check free space before every batch.

## Procedures

- Wrap every build, test, probe and benchmark:
  `python3 scripts/resources.py run --max-output-gib 16 -- /usr/bin/python3 scripts/storage.py run --no-prune --max-output-gib 16 -- CMD`.
  The wrapper chdirs to the repository root, so use repo-relative paths. Its
  "resources: waiting" lines can appear in stdout.
- Fastcheck: `python3 scripts/fastcheck.py --max-output-gib 16 [--pa paN]
  [--lib-case full::path] MODULE[::test] …`.
  - Selections name the test module, such as `global_clause_premises::case`,
    not the `suite_*` target.
  - Never pass `--pa pa41` or wrap fastcheck in `timeout`.
  - The architecture audit rejects 7 or more files sharing a filename stem, and
    Clippy rejects more than 3 bools in a struct.
- Declare outcomes before the first rustc or RustGM run of a new fixture.
  Record malformed batches and mispredictions.
  - rustc oracle: `obj/parallel/simd-divrem/oracle.sh BATCH OUT`.
  - RustGM: `obj/parallel/simd-divrem/rustgm.sh CLI BATCH OUT`.
  - The no_core prefix needs `legacy_receiver`, `clone_fn`, `freeze`, and
    `destruct` as `Destruct: PointeeSized`, plus `Neg` impls for negated
    literals.
- Packets are byte-for-byte copies of declared batch files, with `cases.tsv`,
  `outcomes.json` (sha256, declared, rustc, base, rustgm), a README and a
  `pa31/tests/README.md` entry. `obj/parallel/simd-divrem/make-packet.py` is the
  latest builder.
- Reference updates need a mechanical check, a base reproduction, an
  independent review and then scoped `course.py STAGE --bless --case PATH`.
- Core observation:
  1. Pin a clean head with `pin.py candidateN release`.
  2. Derive the observer with `sed 's/candidate12/candidateN/g' observe-original-candidate12.py`.
  3. Wait with `wait-load2.py 14 20 21600`, then run
     `scripts/resources.py run --exclusive … --prune-under obj/parallel/capture-match/original-candidateN -- python3 obj/parallel/capture-match/observe-original-candidateN.py`.
  4. Compare with `compare-original.py candidateN obj/parallel/capture-match/original-candidate(N-1)`.

  Keep the 120s observer window and all compiler limits. Don't build or clean
  during an exclusive measurement.
- Retire a pin by removing its literal `.keep-artifacts` path, then
  `python3 scripts/storage.py prune --apply --prune-under DIR`. Never use
  variable or command substitution, `xargs`, `find -delete` or globs with `rm`.

## Standing requirements

Everything under "Retained requirements and resource rules" and "Historical
checkpoints" in `git show 03003a86f:handoff.md` remains in force. That
includes:

- the symbolic-index native companions;
- the implicit-receiver Deref packet's two 128-layer positives;
- the const-adapter bootstrap and runtime positives;
- the three tuple-factory cleanup-owner cases;
- genuine source panic runtime routing for native and core artifacts;
- PA25-PA44 remain planned assignments that must fail, not pass empty.

No frontend, body or abort substitution is authorized.
