# RustGM handoff — 2026-11-13

Continue the active goal: compile pinned original core, then alloc/std, completing
assignment components as their complete contracts qualify. Work as **one worker**.
**No original library artifact exists.** PA25–PA44 remain planned as whole PAs;
the released113-case PA25 const-machine component stays separate.

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

## Current core evidence and next blocker

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
