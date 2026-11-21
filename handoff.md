# RustGM handoff

## Stop boundary · 2026-11-21

All work was stopped on 2026-11-21 at about 14:55 UTC at the user's request.
No build, test, census or measurement is running and the resource controller
has no active or queued job. The last compiler change on local `main` is
`affa1c42e`, and this handoff is the only commit after it. Local `main` is
more than 330 commits ahead of `origin/main`. Nothing has been pushed, and
nothing is to be pushed without the user asking.

The [acceleration plan](docs/acceleration-plan.md) is still the order of
work. This file says how far each part got, what each worktree holds and how
to resume. The previous handoff, with the parked strict alias checkpoint, is
`git show 320400216:handoff.md`.

| Goal of the plan | State |
| --- | --- |
| Faster validation loop | Done. Every planned step except the batch runner, which was deferred with numbers. |
| Frontend performance | Core graph 66.59 → 54.14 billion instructions (−18.7%), joint 97.97 → 83.08 billion (−15.2%), byte-identical output. rustc's front end needs 34.34 billion on the same core sources. |
| Self-hosting | The compiler's own source type-checks in the joint graph except for 82 stubbed functions (425 at the first census). No LowIR has been produced for it yet. Stage 1, stage 2 and the strict ratchet are not started. |

`main` is green: `make integration-check` passes all 48 parts on `affa1c42e`.
[Receipt](/mnt/vizier/rustgm-scratch/rustgm-backend-work/obj/qualification/integration/1795269580117000327/receipt.json)

## Worktrees and unintegrated work

Every tree is clean. Uncommitted work was committed as it stood on `wip/`
branches, and those commits have not passed any check.

| Worktree | Checked out | Holds |
| --- | --- | --- |
| `/home/vishvananda/rustgm` | `main` | Git-only integration checkout. Never build here. |
| `/mnt/vizier/rustgm-scratch/rustgm-pa26-work` (self-hosting A) | `wip/selfhost-5-20261121` `004ba3141` | One WIP commit on `affa1c42e`, **never compiled**: the dispatch fix that unblocks the census, and an `Option::len` discriminant fix. `lane/selfhost-5-20261121` equals `main`. |
| `/mnt/vizier/rustgm-scratch/rustgm-backend-work` (self-hosting B) | `wip/selfhost-b-2-20261121` `bfb2dfb3d` | One WIP commit on `affa1c42e`, tried only on an observe build: the last 4 closure identity rows, six PA24 fixtures without references. `lane/selfhost-b-2-20261121` equals `main`. |
| `/mnt/vizier/rustgm-scratch/rustgm-async-work` (runtime) | `lane/runtime-2-20261121` `bbc5105fc` | Seven commits on `affa1c42e`, no integration check yet. The three standard-library consumers pass on this branch. |
| `/mnt/vizier/rustgm-scratch/rustgm-pa29-work` (performance) | `lane/perf-3-20261121` `2c98f57b3` | Two commits on `affa1c42e`, measured before the rebase and not compiled since; no integration check yet. Also `wip/perf-3-formation` `61c38722e` (based on `7ea299f5d`), written but never compiled. |

Each lane's handover note says what proves each commit, what is owed, which
pinned directories to keep or prune, and the exact next command:
`/mnt/vizier/rustgm-scratch/integrator-20261120/handover/{selfhost,selfhost-b,runtime,perf}.md`.

## Resuming

In this order, because each step unblocks the next:

1. **Runtime branch.** Run `make integration-check
   INTEGRATION_ARGS='--max-output-gib 40'` on `bbc5105fc`, then
   `make test-upstream` (a PA24 fixture was added), then fast-forward `main`.
   This puts on `main` the caller-transport fix behind `fs`, the backend
   command limit scaled by input size, the remaining whole-graph guards and
   the stage-1 driver `scripts/stage1_run.py`. Lane A can then drop its five
   "nested output transport" rows.
2. **Performance branch.** Take a baseline for the new `main`, confirm the
   two commits' deltas and byte-identical output against it, run the
   integration check on `2c98f57b3`, then fast-forward. Both commits are
   small.
3. **Lane B's WIP.** Build it, record rustc's verdict for the six fixtures
   before generating references, fastcheck with `--pa pa24`, reword the commit,
   integrate.
4. **Lane A's WIP.** Build it, split it into its two families, generate the
   PA20 fixture's references only on a compiler that passes it, fastcheck each,
   then run one census pair from
   `obj/parallel/stage1-census/selfhost5/bringup-16/skip.tsv`. That pair
   should reach the instance stage again and shows whether lowering starts.
5. Then relaunch the lanes from their briefs (`briefs/selfhost-5.md`,
   `selfhost-b-2.md`, `runtime-2.md`, `perf-3.md` in the kit below) and
   handover notes. Start fresh agents: every lane agent's context was large at
   the stop.

The session that ran this work used four background lane agents on the Opus
model and one integrator. `briefs/common.md` holds the rules given to every
lane.

## Integration procedure

1. Rebase the lane onto `main` in its own worktree.
2. `make integration-check INTEGRATION_ARGS='--max-output-gib 40'` must exit
   zero after the lane's last change: gates, build, the private unit binary,
   13 harness suites and the PA1–PA30 course text, 48 parts. It takes 18–30
   minutes on the shared host. A lane's own receipt is accepted when it is on
   exactly the rebased tip (`state: passed`, 48 parts, `failing: []`, written
   after the last commit, clean tree).
3. Read every changed reference. A moved rejection site is accepted only where
   rustc reports it; nothing is blessed to hide a difference.
4. `git merge --ff-only` from `/home/vishvananda/rustgm`, then tell the other
   lanes what landed.

Standing rules beyond `AGENTS.md`: commits carry no trailers; `rm` never takes
a variable or command substitution, only literal inspected paths (give this to
every agent: one broke it once on its own scratch file); a controller slot is
held only while a job runs, about 15 minutes per lease; the controller stays at
two heavy jobs; the backend pin moves only when the user asks.

## Validation loop · complete

- A fastcheck with nothing to do takes about 9 s (was 47–92 s). A one-module
  library edit reaches its tests in about 63 s (was about 87 s). Gates are
  skipped by fingerprint and run beside the compile; Clippy runs beside the
  rustc build.
- `make integration-check` is the one command before `main` moves. The
  harness suites inside it went from 534 s to 172 s, and the 11,452 course
  text cases take 47–90 s.
- Decided with measurements, recorded in the plan: no crate split (a mean
  saving of 10–12 s against 10,545 path rewrites), the 12 GiB job cap is
  unchanged, and the fixture batch runner is deferred because course text is
  about 5% of a run.

## Performance

**On `main`.** Sealed tables are read without revalidation, and closed trait
goals are answered from one crate-wide answer table
(`dev/src/sema/traits/selection/search/answers.rs`) after their first search.
The table answers 87.5% of core lookups and 81.0% of joint lookups. It holds
its own reference to every promise set a key names, records nothing from an
unsealed, provisional, cyclic or deferred state, and `--self-check` re-runs
the search on every hit. Both self-hosting lanes ran the whole-source census
on top of it and saw no difference other than time.

**Measured, not on `main`.** `lane/perf-3-20261121`: reason counters for goals
the table passes over (−0.04%), and a generation counter in place of the
per-lookup catalog comparison (−0.13%). `wip/perf-3-formation` would lift the
restriction behind 99% of the `uncaptured` fallbacks (73,483 core lookups); it
is unbuilt.

**Where the time goes now** (core graph, cycles, at `65bbbe7f4`): trait
selection and normalization 35%, body typing outside selection 22%, loading
and expansion 10%, declarations 9%, borrow and region proofs 8%. Hash tables
are 15.8% of cycles and the allocator 14.4%. The inference context's `known`
map is regrown from empty for every trial. The full table is in
`handover/perf.md`, section A3.3.

**Against rustc.** On the same frozen core sources, rustc 1.98.0 with
`--emit=metadata -Zthreads=1` retires 34.34 billion instructions; RustGM
retires 54.14 billion, 1.58 times as many. RustGM's count includes instance
demand and LowIR emission (about 9% of its cycles), which rustc does not do
there. The wall-time comparison in the previous handoff has not been repeated.

**The largest known cost is outside the benchmark set.** One whole-source
census iteration takes 490–540 s and 4.4 GB when it reaches the instance
stage. That stage is about 60% of the time and carries nearly all of 163–175 s
of system time. RustGM still proves regions, loans and ownership again for
every instance (`dev/src/lower/instances/proof.rs`), which contradicts the
plan's decision that proofs run once per template. A profile of that stage
was running when work stopped and has to be rerun
(`BENCH/selfhost_run.py … perf`, `handover/perf.md` A3.2).

**Next, in order.**
1. Per-template proofs. The design is written in `handover/perf.md` A3.7 and
   **has not been reviewed**. Its first step is a counting probe that changes
   no behaviour: every instance whose proof fails where its template passed,
   and the time each instance analysis takes, on core, joint, the whole-source
   graph and every course suite. Build that probe, then decide the rest from
   its result. Self-hosting lane A stays out of `lower/instances/proof*`
   meanwhile.
2. Table coverage: finish the formation change; for the 74,320 `unsealed`
   lookups either validate impl headers before `validate_signatures` or seal
   per impl.
3. Pre-reject and one-way matcher for lookups that still search, and a pooled
   frame context.

Decided with numbers and not to be reopened without new data: no caller-keyed
cache (under 1% of core), and no undo-log inference yet (context copying is
3.2% of cycles).

## Self-hosting

**What the census is.** One bring-up session compiles core, alloc, std, the
vendored crates, a frozen copy of RustGM's own source (taken at `320400216`,
with the edits listed in its `step-overs.tsv`) and the CLI as one joint graph.
Functions that fail are stubbed through a skip list and the loop repeats. The
command, guests and flags are in `handover/selfhost.md`.

**Skip list.** 425 rows at the first census, 82 now
(`selfhost5/bringup-16`, on `main` `affa1c42e`): typing 43, instances 17,
cfg 5, destruction 4, ordinary 4, lifetimes 3, stopped 3, copies 1,
constants 1, opaques 1. The 17 instance rows were not re-verified, because
that iteration stopped earlier. Lane B's WIP removes 4 more typing rows.

**Why there is no LowIR yet.** The census reached the whole instance demand
at `7ea299f5d`, then lane B's typing fixes removed 88 rows and all 63 stub
cascade rows at `cfg`. It now stops at the opaques stage on
`sema/declaration_inputs.rs` `own`, whose `impl Trait` result comes from
`Inputs::iter`. That function fails with "invalid lifetime constraint
identity": a trait method passed as a value (`.map(Input::from)`) is copied at
fresh lifetimes inside a generic callee, and those copies are never resolved
to the selected implementation. The fix is lane A's WIP commit
(`dev/src/body/dispatch/region_copies.rs`). Lane B's error of the same name is
a different cause, in `sema/closures/matching.rs`.

**After that**, by lane:
- **A** (`briefs/selfhost-5.md`): the instance stage's own defects (callable
  condition without a source slot 3, aggregate fields needing normalization
  2 + 1, instance premise 2, promoted call 1, conversion adapter 1), the
  residue after typing, then the 32 proofs rustc makes and RustGM lacks.
- **B** (`briefs/selfhost-b-2.md`): access plan 8, field access 3,
  incompatible types 2, the `clause` row, two lifetime defects, then array
  lengths read before implementation validation (step-overs SO-5 with 22
  sites, SO-7 and SO-2). The `accepts_charged` row at `cfg` is a function that
  now types and joins lane A's "generic aggregate requires instantiated
  fields" family; it is not a regression.
- **Runtime** (`briefs/runtime-2.md`): a reached stub calls `abort()` and
  dies with empty standard error. It must name the function before stage 1 is
  debugged. Then the census on its tip, and stage 1 when LowIR exists.

**Unproven obligations and waivers.** 106 and 19 at the last iteration that
reached instances. Of 99 classified, 32 are proofs RustGM lacks and 67 are
artifacts of proving instances and nested closures per instance; about 58 of
those should go with per-template proofs. The classification is
`rustgm-pa26-work/obj/parallel/stage1-census/selfhost4/bringup-13/unproven-classes.md`.

**Output path.** A synthetic 398 MB LowIR file with 349K functions is emitted
in 21 s and compiled by the backend at `-O0` in 21 s; every executable ran.
Lowering guards on `main` have whole-graph headroom. On the runtime branch,
`fs`, `env-args`, `io-stdin` and nine other consumers pass all three native
runs with no skip rows; `float-math` and `threads` do not, and the compiler
needs neither.

**Not started.** Stage 1 link and run (M5), stage 2 (M6), the strict ratchet
(M7), the strict-export census (`briefs/export-census.md`), a fresh snapshot
of the compiler source for the census, and the close-out of PA31 and
PA37–PA45. PA25–PA30 were closed at finite contracts on 2026-11-20.

## Known defects and debts

- **Wrong code found by the census, fixed in `7ea299f5d`:** a hidden type
  naming an overridden default method ran the default. Expect more of this
  kind as the census reaches further; report any at once.
- **`make test-backend` fails at `--test-bridge` on `main`** since
  `3ef6b79e9`: `dev/backend-bridge/bin/inprocess.rs` `mod emission` takes 3
  arguments and `dev/bin/modules.rs` passes 4. Reported by the runtime lane and
  not otherwise verified. The bridge is optional and the text path is the
  default.
- **No fixture for `f8624a6cb`** (a cleanup's field coordinate read from
  normalized storage). It reproduces only with the real `Cow`, `ToOwned` and
  `String` across crates of the joint graph.
- **Layout.** 25 of 26 probed sizes differ from rustc's. The size pins are
  `cfg(test)` and enforced by the audit.
- **Parked.** The strict alias patch and all strict metadata-transport fixes,
  until the export census sizes them.
- **Housekeeping.** About 40 `lane/*` branches are fully contained in `main`,
  and `wip/perf-*-pre-*` are safety copies of rebased work; none has been
  deleted. Each handover lists pins to prune. 226 GiB is free.

## Where things are

- Kit: `/mnt/vizier/rustgm-scratch/integrator-20261120/` with `briefs/` (every
  assignment given), `handover/` (each lane's note), `research/` (the five
  audits behind the plan) and `scripts/`.
- Performance harness, baselines and profiles:
  `/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/instruction-bench/`.
  There is no baseline for `affa1c42e` yet; build one before measuring.
- Census directories: `rustgm-pa26-work/obj/parallel/stage1-census/` (lane A)
  and `rustgm-backend-work/obj/parallel/stage1-census-b/` (lane B's copy).
- Runtime reductions, consumers and the stage-1 proofs:
  `rustgm-async-work/obj/parallel/runtime/`.
