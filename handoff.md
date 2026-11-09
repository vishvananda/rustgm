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
