# RustGM handoff — 2026-11-16

## Current plan

The user adopted the [self-hosting plan](docs/self-hosting-plan.md) on
2026-11-15. Where older sections below conflict with that plan, the plan wins:

- Three lanes and agents are allowed, on the three existing worktrees.
- An unqualified bring-up profile may produce artifacts; strict mode is still
  the default and the only release gate.
- The ordinary bootstrap profile is frozen at PA24. The "ordinary" items under
  "Requirements still open" are no longer work items.
- Work is ordered by what the self-hosting ladder demands (M1 core and a no_std
  consumer, M2 alloc, M3 std, M4 the three dependencies, M5 stage 1, M6 stage 2,
  M7 strict ratchet), not by the next strict fence.
- Open design calls are decided by the implementer and reported afterwards;
  destructive or outward-facing actions still need the user.

## State on 2026-11-16

Latest compiler checkpoint: `b9c0a9a8b`, integrating all three lanes and the
capture-Deref fix. Final local validation: 3,112 private tests pass (3 ignored),
2,778 regular tests across all 13 suites pass, source/native promotion and capture
controls pass, and the backend handshake is checked. Full qualification:
[43965840797](https://github.com/vishvananda/rustgm/actions/runs/43965840797) (dispatched; pending).
The previous full green qualification was on `e31379b29`
([43925422216](https://github.com/vishvananda/rustgm/actions/runs/43925422216)).
Nothing here is a qualification of PA25–PA45: strict mode still stops at its
first fence, and those stages stay planned.

**M1 is reached under the bring-up profile.** All 18 staged programs
(`pa32/tests/spec/core-consumers/`) behave as
`pa32/tests/core-consumer-outcomes.json` declares. 16 exit 0 with rustc's exact
stdout; `panic-index` and `panic-overflow` abort after rustc's stdout. Each was
compiled together with real Rust 1.98 `core` in one session, built by the pinned
backend, linked with libc, and run three times in a fresh empty directory with
an empty environment. That full 18-program observation used a 12-row skip list
(`rustgm-pa26-work/obj/parallel/frontier3/core.skip.m1-12.tsv`):
- `mem/drop_guard.rs` `new` and `dismiss`;
- `num/niche_types.rs` `default`;
- nine "qualification has no exact type request" rows: `slice` `split_off_first`,
  `split_off_first_mut`, `split_off_last` and `split_off_last_mut`; `ascii_char`
  `make_uppercase` and `make_lowercase`; `socket_addr` `set_ip`; `fmt` `width`
  and `precision`.

About 50 bodies per program are still unproven (`bringup-unproven`), so this is
bring-up evidence only.

The latest compiler removes all nine qualification rows: core now needs only
`DropGuard::new`, `DropGuard::dismiss` and `niche_types::default` (3 rows).
`atomics`, `fmt-floats` and `slices` compile with that list and each matches its
declared status, stderr and stdout three times in fresh empty directories with
an empty environment. This is a focused replay, not a new 18-program receipt.

**M2 remains open.** The repeated real-alloc census compiles with 37 alloc rows
(down from 44), plus the 3 core rows, and 55 unproven bodies. Cleared alloc rows:
B-tree `replace` and `get_or_insert_with`, linked-list `clone`, and the four
`raw_waker`/`local_raw_waker`/`clone_waker` rows. Vec, String and Box-to-dyn still
abort in their consumer stubs; none counts as a passing consumer.

Evidence is in the frontier worktree under `obj/parallel/resume-20261116/`:
`alloc/c14/`, `alloc-consumers/`, `core-consumers-fixed/`, `core-runs/`,
`private-final.log` and `integration/`. The current optimized observation binary
is `bin/rustgm-fixed`, SHA-256
`1a1ef1735a300b3c3c73e53c21c101973edd87e32392064177c6dae3fab36701`.
It is not release or performance qualification evidence.

**Landed on main on 2026-11-15/16:**
- **Backend:** the CPPGM pin moved to v0.9.3 (in-process library interface 4).
  The optional in-process bridge crate `dev/backend-bridge` exists; the text path
  stays the default.
- **Guest crates:** `--bringup-guest`, and `bringup_linked.py --guest
  --dependency NAME=ROOT.rs`.
- **alloc:** its 16 crate-level stoppers are fixed, including coherence items
  13–16. The bring-up census now names every alloc body that fails to type and
  stubs it, keeping its checked signature.
- **Allocation:** `#[global_allocator]`, `#[alloc_error_handler]`, rustc's
  internal-symbol names and the allocator shim (`dev/src/lower/allocator_shim.rs`).
- **SIMD:** 16-byte vectors (LowIR `v128`, nine `simd_*` intrinsics).
  `simd_splat`, `mem::zeroed` and `type_name` are evaluated in constants. This
  removed the 1,043 stdarch skip rows.
- **M1 fixes:** the closure-family, struct-update and coercion-storage fixes.

**Timing (one rough data point, 2026-11-16):**

| Compiling core plus `fmt-floats` from source | Wall time |
| --- | --- |
| RustGM (release frontend at 952dfae84) | ≈24.6 s |
| rustc | ≈17.7 s |

Both ran on effectively one core. Because RustGM cannot yet publish core's
metadata, every program recompiles core, so per program RustGM is ≈120× slower
than rustc over a prebuilt core (0.2 s). Script and records:
`/mnt/vizier/rustgm-scratch/integrator-20261116/scripts/timing.py` and
`rustgm-backend-work/obj/parallel/timing-20261116/`.

## Integrated lane batch and remaining drafts

All three lane branches are aligned with main. Their pre-integration heads are
retained as `checkpoint/publication-before-integration-20261116` and
`checkpoint/runtime-before-integration-20261116`; the WIP branches remain intact.
The former unpublished lane heads are integrated in `b9c0a9a8b`:

- Frontier: exact store-destination qualification, plus capture boundaries for
  source `Deref`/`DerefMut`; four accepted native programs and two loan opponents.
- Publication: owning Box moves/drop order, trait-object/slice unsizing,
  `Option<u8>` tag decoding, nested Box cleanup and marked const-call promotion.
  Promotion has a separate authenticated entry for closed generic callees;
  ordinary generic roots and conditional/intrinsic/drop-glue fences remain.
  A real-core atomics regression exposed the missing generic-callee case; it is
  fixed in the promotion commit, with six source outcomes and two native controls.
- Runtime: delayed nominal requirements, declaration-time lexical module/crate
  identity, per-crate deprecation policy and TLS globals in text and ABI-4 bridge
  paths. Two-thread TLS output and fixed/PIC object equality pass.

Only two frontier drafts from `wip/frontier-20261116` (`5d0c550fc`) remain:
closure `T: 'a` output requirements (three region/relations files) and empty
callable binders (`member_conditions.rs`). The capture part of that draft has
landed; do not replay its three closure/mutation files over the new Box logic.
The publication promotion WIP and runtime row-10 WIP have been superseded.
Precise Box capture paths remain outside the current Box fixture coverage.

The original reductions and std census remain in `obj/parallel/frontier3/`.
Superseded compiler pins were retired; historical binary paths below may be
pruned. Rebuild or restore and verify a recorded hash before a new run.

## Integration procedure

Each step of the 2026-11-16 integration ran in a lane worktree while its agent
was idle:
1. Rebase the lane onto main.
2. Run the whole private binary (3,112 passing tests and 3 ignored). In the worktree,
   with `W` its absolute path:

   ```sh
   python3 scripts/resources.py run --wait-seconds 1800 --max-output-gib 24 -- \
     /usr/bin/python3 scripts/storage.py run --max-output-gib 24 --no-prune -- \
     env CARGO_INCREMENTAL=1 CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=$W/obj/rust/private-lint \
     RUSTC_WORKSPACE_WRAPPER=/usr/local/bin/clippy-driver CLIPPY_ARGS=-Dwarnings \
     cargo test --manifest-path dev/Cargo.toml --locked --profile dev-fast --lib
   ```
3. Run every root of **all 13** suites with
   `/mnt/vizier/rustgm-scratch/integrator-20261116/scripts/suite_check.sh WORKTREE LOGDIR closures constants declarations frontend inference linking lowering memory ownership patterns projections regions traits`.
   It drops roots that hold only ignored tests and retries admission refusals.
4. Run `make test-upstream` and the script tests. Add `make test-backend` when
   LowIR or the bridge changes.
5. In `/home/vishvananda/rustgm`, fast-forward and push:
   `git fetch -q WORKTREE BRANCH && git merge --ff-only FETCH_HEAD`, then
   `git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push https://github.com/vishvananda/rustgm.git main:main`
   and `git update-ref refs/remotes/origin/main main`.
6. Dispatch qualification with `gh workflow run check.yml --ref main`.

Running only the suites a change seemed to affect let three failures through to
qualification on 2026-11-16 (size pins, linking tests, a memory-suite
regression). Every commit must pass on its own: fold test repairs into the
commit that needs them. The lane briefs used this session are in
`/mnt/vizier/rustgm-scratch/integrator-20261116/briefs/`.

## Next work, by lane

- **Frontier:** validate the remaining closure-output and vacuous-binder drafts
  one at a time against `obj/parallel/frontier3/{r1,guard}/declared.json`.
  Eleven alloc lifetime rows remain. `guard/consumed.rs` still has an unproven
  const Drop. Core `niche_types::default` needs wider integer niche ABI support;
  `dev/tests/invariants/integer_niches.rs` pins its current rejection.
- **Publication (M2):** start from `obj/parallel/resume-20261116/alloc/c14/skip.tsv`.
  Four generic aggregate rows remain (Box drop, two B-tree node methods and
  VecDeque Drain drop), plus three ambiguous trait obligations and other named
  rows. Vec still stops at "no applicable method for this receiver"; String at
  a trait obligation and an associated-type selection; Box-to-dyn at "captured
  object target lacks checked conversion entry evidence". Their actual census
  and native failures are in `alloc-consumers/`; no consumer is counted as passed.
- **Runtime (M3):** TLS is validated. Next are gap rows 11 (linkage/sections),
  2 (`lang_start`/`Termination`), 12 (float math), and the real HashMap probe's
  unattributed trait selection. Std guest census is still blocked by guest
  editions: libc's 2021 extern blocks are parsed in the session's 2024 edition.

Lane briefs may change; milestone order stays M2 before M3/M4 and self-hosting.

### Bring-up profile

`--bringup-skip-list FILE` selects it; a library also needs a crate
disambiguator ending in `_bringup`. Everything below is refused without it.

| Mechanism | Reported as |
| --- | --- |
| A listed function becomes a trapping stub; every stage sets aside the failures it can attribute to a function (typing failures too, keeping the checked signature) and names them | `bringup-blocker<TAB>path<TAB>offset<TAB>name<TAB>message` |
| A failure with no owning function | `bringup-unattributed<TAB>message` |
| A crate-level publication fence is passed over; metadata is withheld when it cannot be produced | `bringup-waived<TAB>fence` |
| A constant that cannot be evaluated belongs to the body that demands it | blocker for that body |
| Compiler-inserted checks keep the course abort although core installs panic hooks | waived fence |
| A bodiless intrinsic without run-time lowering becomes a trapping shim; one with a fallback body runs it | waived fence naming the intrinsic |
| A body that fails only a loan, lifetime or const-cleanup proof keeps its code (the failing lifetime requirement is left out and the proof retried) | `bringup-unproven<TAB>path<TAB>offset<TAB>name<TAB>message` |

`scripts/bringup_census.py` grows a skip list until a crate compiles and writes
`summary.tsv`, `waived.tsv`, `unproven.tsv` and `records.json`. A consumer's waived intrinsics
are its remaining work list.

`--bringup-guest NAME=ROOT.rs` (repeatable, in dependency order, over a
`--source-core` session) resolves a further crate inside the same session. A
guest has its own `crate` and `$crate` root, restricted visibility, exported
macros, extern prelude (the host and the guests before it) and the prelude its
root attributes select; lang items, feature gates, coherence and the edition
stay the session's. Coherence knowability is session-wide too: every source
trait counts as owned, so a goal rustc leaves ambiguous from `alloc` because
only upstream core could implement it (`[Local]: Iterator`) is decided as if
one crate owned the trait. Its files are keyed `NAME:PATH` in skip lists and
`bringup-*` lines. The last guest is the program: under `#![no_main]` its
exported C `main` is the entry point. `scripts/bringup_linked.py --guest`
passes a consumer this way instead of mounting it in a copy of core, after
each `--dependency NAME=ROOT.rs` guest (such as alloc). A guest
session publishes no metadata. When a crate declares `#![needs_allocator]`
(alloc), the program gets rustc's allocator shim and must have a
`#[global_allocator]`. There is no runtime start or `std` prelude yet.

```sh
# A lane worktree. Optimized incremental compiler; never release evidence.
make observe && cp obj/rust/observe/rustgm obj/parallel/LANE/bin/rustgm-observe

# M1 consumers, end to end (census, backend, run, compare).
python3 scripts/resources.py run --wait-seconds 1800 --max-output-gib 24 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 24 --no-prune -- \
  python3 scripts/bringup_linked.py \
  --compiler obj/parallel/LANE/bin/rustgm-observe \
  --library /mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/original-core-inputs-88d9e12ae178fab0fb5cc050a94da85685d449ea/library \
  --skip-list /mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/frontier3/core.skip.m1-12.tsv \
  --work obj/parallel/LANE/linked pa32/tests/spec/core-consumers/*.rs
```

`bringup_linked.py` counts only exit 0 as a pass. A declared-outcome checker
(status, stderr and stdout, three runs) is
`rustgm-backend-work/obj/parallel/m1-verify/check_declared.py`; edit its `ROOT`
and `WORK` paths before using it elsewhere.

Detailed earlier handoffs are retained in Git: `git show 0031a83d4:handoff.md`.
The 2026-11-15 state of this file is `git show e31379b29:handoff.md`.

## Workspace, authorization and resources

- Build only in the three worktrees under `/mnt/vizier/rustgm-scratch`:
  `rustgm-pa26-work` (frontier), `rustgm-backend-work` (runtime) and
  `rustgm-async-work` (publication). Main `/home/vishvananda/rustgm` is Git-only.
- No new worktrees. Coherent relevant commits/pushes are authorized;
  use single-line subjects without trailers.
- CPPGM backend: release v0.9.3, `41e06a23e`, of `vishvananda/cppgm` (the
  renamed cppgm-extended; the driver is `cppgm`). Install it in a worktree
  with `python3 scripts/setup_backend.py`; a `backend.local.json` from v0.9.1
  or earlier no longer works. The release's in-process LowIR library is pinned
  at interface version 4, which adds record types to the boundary facts of
  version 3, and has a handshake. The optional bridge crate
  `dev/backend-bridge` (FFI in `src/ffi.rs` only) states RustGM's LowIR to it;
  the text path stays the default
  ([checkpoint](docs/backend-current-checkpoint.md)). A LowIR change must keep
  the bridge in step (state the form or refuse it) and pass `make test-backend`.
- Do not modify C++/v5rust checkouts or another checkout's outputs. Rustc is a
  test oracle, never a frontend substitute.
- Read AGENTS.md, spec.md, ROADMAP.md, the owning handout and
  TESTING_AND_REFERENCES.md. `docs/parallel-work.md` contains historical worker
  assignments; the self-hosting plan's three lanes take precedence.
- Shared controller: 3 heavy jobs globally, 150 GiB free-space reserve, CPU 800%,
  RAM 12 GiB, tasks 256. Lanes use a 24 GiB output cap per worktree (each
  worktree includes its caches and pins). Admission requires the reserve plus
  the job's own unused cap plus **every running peer's full cap**
  (`scripts/resource_runner.py:181`). At this wrap-up about 185 GiB is free
  after generated-output cleanup. Let the controller admit jobs; three configured
  slots do not guarantee that three simultaneous reservations fit. `/mnt/vizier` is shared with the user's other
  projects: never clean outside RustGM's worktrees, and do not lower reserves.
- Every build/test/frontend/debugger probe/measurement uses resource and storage
  guards. `make fastcheck` supplies its own guards and has a 300s feedback budget.
  Wait for a wrapper's actual terminal result before editing or starting the next
  heavy command. Do not change tracked files during builds or validation.
- Never use variable or command substitution in an `rm` command; write literal,
  inspected paths. When stopping a long run, also stop every wait loop started
  for it.

```sh
# Direct validation/probe; declare all disposable roots, or omit --prune-under
# for scattered outputs and let storage perform its full scan.
python3 scripts/resources.py run --wait-seconds 1800 --max-output-gib 24 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 24 \
  --prune-under obj/KNOWN_PARENT -- COMMAND

# Build-only: use --no-prune, never combine it with --prune-under.
python3 scripts/resources.py run --wait-seconds 1800 --max-output-gib 24 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 24 --no-prune -- COMMAND

# Focused selection syntax: exact libtest paths root::module::test or a bare
# root; a prefix matches nothing. A root holding only ignored tests needs
# --ignored.
make fastcheck FAST_ARGS='--max-output-gib 24 root::test --prune-under obj/KNOWN_PARENT'
```

Fastcheck selections cover only what they name. Before publishing, run the
whole private binary and all suites (see "Integration procedure"). Do not run
broad release/native matrices per edit. Use `make tidy` between batches; it
keeps pinned (`.keep-artifacts`) directories.

The sections from here on describe the strict frontier as it stood before
the plan changed (2026-11-15 and earlier). They are history, except
"Requirements still open", which is the strict ratchet's list.

## Latest genuine original-core attempt: candidate41

Clean immutable release pin:
`obj/parallel/capture-match/candidate41-release/`.
Producer is exact product `2e25fdba4`, with 3967 compiler/test Rust hashes and an
empty tracked patch. Optimized build took 7m48s. CLI SHA-256:
`1dd358581384e4783248db169ec99cc722c23626609d117d277afffd7a495fdf`.

The genuine original v14 library attempt **clears the PartialOrd const closure
calls in slice/cmp.rs**, then rejects:

```text
ERROR: file 210: 176765: marked function-call promotion requires its actual evaluation profile
```

File210 is `library/core/src/slice/mod.rs`, line4667, the `as_simd` body. That body
contains `assert_eq!(size_of::<Simd<T, LANES>>(), size_of::<[T; LANES]>())`.
The exact marked callee has NOT been identified; size_of is a lead, not a verified
callee identity.

Result: status 1, no timeout, 20.600939011899754s, peak child RSS 772712 KiB.
No `core.rgm`; `core.lowir` exists with zero bytes. This is diagnostic progress,
not an artifact, native-consumer or performance qualification.

Raw evidence under `obj/parallel/closure-members/`:

- `build41-release.log`, `core-artifact41.py/.log`,
  `artifact-postcheck41.py/.log`;
- `original-library-candidate41/{declaration,result,postcheck}.json`,
  `whole-core.openat`, `stdout`, `stderr`.

Postcheck41 passed: 722 original source hashes, 375 source opens, six reviewed
configuration bindings, all 3967 compiler hashes, clean producer and the fixed 120s
observer deadline. Unchanged source-contract limits: 14500624 work,
1073741824 attempted bytes,67108864 retained bytes; lint cap allow; profile
`module-body-operations-v14`.

Original input tree:
`obj/parallel/original-core-inputs-88d9e12ae178fab0fb5cc050a94da85685d449ea/`.
Reviewed manifest: `obj/parallel/body-operations/original-manifest.json`, SHA-256
`9d14519e7f61352c1ed126d1a4e1590fcbb99aa9debcaafa3e54f3d08e750381`.
The older census manifest remains immutable. The sole reviewed configuration
rebinding is dev/bin/crates.rs; see body-operations/config-rebinding.json and
cli-config-rebinding.patch. Do not overwrite old manifests or loosen the profile.
The file map is capture-match/original-candidate28-diagnostic/progress.tsv.

## Next blocker: actual marked-call promotion policy

There are **no production edits or new fixtures for this frontier**.

`dev/src/body/promote/queries.rs` treats a nonconstructor Rvalue::Call as follows:
call `require_call_promotion_profile(callee)`, then return false (unpromoted).
`dev/src/body/promote/values.rs` currently errors for ANY KnownPresent
`Evaluator::promotable_contract` result. This is an explicit incomplete boundary,
not proof that the original program is invalid.

Relevant existing contracts and APIs:

- `dev/src/sema/constants/values.rs`: Evaluator::promotable_contract, Overflow and
  Failure. Promotion value arithmetic wraps while original CFG overflow assertions
  remain; required constant evaluation still diagnoses overflow.
- `dev/src/body/promote/{queries,values,session}.rs`: demand DAG, constructor-only
  call folding and shared evaluator preparation. The call helper currently handles
  constructors, not arbitrary marked function execution.
- `dev/tests/invariants/promotable_headers.rs` and
  `pa31/tests/spec/source-promotable/`: source marker ownership, required CTFE,
  ordinary runtime calls and the valid marked borrow/profile requirement.
  The borrow-required private guard currently records this unfinished fence.

A guarded read-only GDB run of frozen41 has finished successfully as a probe
(the compiler inferior still rejects core). Evidence is under
`obj/parallel/promoted-calls/`: trace.gdb, trace.py, trace-command.json,
trace-command.stdout/.stderr, trace.log and original-trace/.
It confirms require_call_promotion_profile -> dependencies -> demand -> collect ->
analyze_templates_reusing. The binary's limited debug information reports
"No arguments"; **no DefId/TypeId or exact marked callee was observed**.
No compiler/test/probe remains running locally.

Recommended continuation:

1. Identify the actual marked source callee and its complete instance/profile.
   Use a bounded probe; the existing stack-only GDB trace cannot name it.
2. Read the source promotable promise and evaluator execution/overflow policies.
   Declare paired PA25 ordinary and PA31 source outcomes before execution.
3. Preserve runtime effects, overflow checks, failure classification, safety,
   borrow/storage/lifetime and actual const Code requirements. Do not simply
   remove the guard, ignore rustc_promotable or grant static storage from a marker.
   Existing valid borrow/profile positives remain accepting requirements.
4. Qualify a coherent change with focused controls, actual observers and bounded
   work. Keep generated output in obj. Run test-upstream after fixture changes;
   run test-backend only if the bridge/backend contract changes.
5. After publication, freeze one clean optimized successor using
   obj/parallel/capture-match/pin.py (currently asserts 3967), then repeat the same
   original artifact attempt/postcheck. Do not use a dev-fast whole-core timeout
   or a census as a substitute for an actual library artifact.

## CI

- **Entirely green:** [43669171589](https://github.com/vishvananda/rustgm/actions/runs/43669171589),
  exact 73eec244c. Strict all-target/private Clippy, architecture/harness,
  release/course, maintainer native, broad native course, debug, independent Rust
  and PA25 component stages all pass. 3088 private release tests pass in 4.77s;
  debug 3088 in 45.91s. Raw: closure-members/ci43669171589.log.
- **One successor queued at wrap-up:**
  [43672166930](https://github.com/vishvananda/rustgm/actions/runs/43672166930),
  exact 2e25fdba4. This covers the member-call change and expanded private guard.
  Check its actual status on resumption. Do not duplicate, cancel or restart it
  for documentation. No full-success claim is made for 2e25fdba4 yet.
- Earlier full 43665225882 failed one malformed-signature corruption guard after
  3086 private passes. Product 73eec restores an explicit error for non-function
  Call signatures; the assertion was unchanged and the green full verifies it.
  Earlier compile/semicolon CI repairs are also superseded by this green run.

## Completed checkpoints and focused evidence

Detailed implementation, declared predictions, limitations and raw paths are in
these documents. Whole assignments were not released by these finite checkpoints.

| Checkpoint | Verified result | Details |
| --- | --- | --- |
| 2e25fdba4: const closure members | Six older unannotated const closure cases now pass prepared lowering; bytes/predictions unchanged. New 12-case PA25/31 packets: 10 source outcomes qualified, 2 source and 12 ordinary gaps explicit. | [docs/pa25-closure-members.md](docs/pa25-closure-members.md) |
| 73eec244c: captured generic const calls | Actual closure owner, wrapper and complete enclosing arguments; 14/25 source outcomes qualify; runtime 41/59. | [docs/pa25-captured-const-calls.md](docs/pa25-captured-const-calls.md) |
| b3ab20376: implicit generic result fields | Exact implicit protocol call results prepare existing storage;17 source outcomes and ordinary no-question control qualify; borrowed native 41/59/23. | [docs/pa20-implicit-result-fields.md](docs/pa20-implicit-result-fields.md) |
| 638369a03: unsafe const members | Exact discharged call safety retained; 34 ordinary/source outcomes qualify; runtime and simple CTFE 41/59. | [docs/pa25-unsafe-const-members.md](docs/pa25-unsafe-const-members.md) |
| 694738c4e: const Try calls | Existing complete protocol plans authenticate implicit calls and effects; runtime 41/59/23. | [docs/pa25-const-try-calls.md](docs/pa25-const-try-calls.md) |
| 05d1beedd: borrowed callables | Actual source reference path, capability, final receiver borrow and formal bindings retained. | [docs/pa25-borrowed-callables.md](docs/pa25-borrowed-callables.md) |
| ead92a4cd and predecessors | Actual Fn role/tuple ABI; projected const parent promises; exact generic/default/supertrait premises. | [docs/pa25-explicit-call-members.md](docs/pa25-explicit-call-members.md), [docs/pa25-projected-const-members.md](docs/pa25-projected-const-members.md), [docs/pa25-default-generic-subjects.md](docs/pa25-default-generic-subjects.md), [docs/pa25-generic-const-calls.md](docs/pa25-generic-const-calls.md), [docs/pa25-default-supertraits.md](docs/pa25-default-supertraits.md) |

Latest member checkpoint evidence under obj/parallel/closure-members/:
fast2=1794712084032770967, six exact selections, 53.15s, strict selected/production/CLI
lint, full format and architecture pass. Native 1794712252305956183 passes three
groups/four programs, 41.09s: new Compare and captured controls print 41/59, two ordinary
unannotated closure programs. Independent Rust runtime/CTFE both print 41/59.
Replay authenticates3967 hashes and 24 CLI attempts (10 qualified/14 explicit gaps).
At 8/32/128 closures, inference/canonical work is 6952/1425, 27736/4325, 110872/15901.
315 harness tests,497 integration roots/13 binaries,44 stages/one component,
128 upstream fixtures and126 Reference files pass. No speed claim.

PA20's new nominal-residuals/without-question.rs primary is codegen-only, explicitly
not a runtime feature consumer. One declared scoped reference batch and three new
sidecars were independently checked; all 4878 prior PA20 oracle files are unchanged.
Current main manifest: 1627 cases: 614 semantics/1013 LowIR; 608 runtime executions
unchanged. Never hand-edit or broadly regenerate references to hide failures.

## Pins, cleanup and pending measurements

Available current executables:

- candidate41-release above (clean original-core/compiler pin).
- obj/parallel/closure-members/replay/wip-rustgm, SHA-256
  `5bbc391f93e44f44d66ab288e85d29d63b8f3e05e40c1dd99192843fa9a08994`.
  This qualified dev-fast replay records its tracked patch/prepublication head;
  it is not a clean release pin.
- **Pending performance baselines 21–24: preserve their executable pins.**

Candidates25–40 and superseded nominal/captured/unsafe/Try/borrowed/explicit/
projected replay executables have been retired and are unavailable for new runs.
Historical receipts and hashes are retained; do not mistake them for available
binaries. Retirement records are in the owning obj/parallel/*/retired-pins.json,
most recently closure-members/retired-pins.json. Older operational/archive details
remain in the historical Git handoff. Never clean another checkout.

Pending performance comparisons use immutable candidates 21/22, 22/23 and 23/24:
wide-constant-products/measure-candidate22.py,
layout-field-budget/measure-candidate23.py,
intrinsic-completion/measure-candidate24.py. Preserve their producer manifests,
inputs, actual executables and .keep-artifacts files. Use four A/A plus two paired
ABBA runs, equal outputs and the unchanged 14/20 load gate. Historical 288 equal
observations do not qualify current speed ratios. Do not wait indefinitely or
relax admission; no current speed improvement is claimed.

Remove a superseded pin only after qualification of its replacement, then
explicitly prune its own directory with storage.py prune --apply --prune-under.
Successful wrappers prune generated scratch; failed artifacts remain for diagnosis.
No storage.py implementation changes were made in this continuation.

## Requirements still open

The six unannotated const closure prepared-lowering cases are completed. The
following requirements, and any other unclosed positives in their owning docs,
remain binding; accepting predictions must never become infrastructure rejects.

- Genuine original core v13 AND v14 metadata, complete LowIR and native consumers;
  original 19/20 full LowIR callback exports (type views do not replace artifacts).
- Source Future/state/poll/drop/Pin/Output/cancellation/recursive cleanup/native
  execution and ordinary authored async; four transported Future headers grant
  no executable Future Code.
- Const-adapter bootstrap/runtime/CTFE/factory consumers; three tuple-factory
  cleanup-owner positives; anonymous associated-borrow/implied-bound, projected
  inline normalization and concrete inline source-promotion consumers.
- Layout/bool niche/repr(Rust) tag and closure cases; symbolic Index native plus
  real source panic routing; two implicit-Deref 128-layer accepting inputs.
- Earlier const payment: anonymous/implied-borrow consumers, range contains,
  projected constructor/aggregate, explicit-call open signature/lifetime/argument,
  borrowed callable and Try aggregate CTFE. Simple unsafe-member CTFE does not
  close these. New captured local/consuming cleanup and opaque CTFE remain open.
- closure-members/default-owner.rs still needs the exact const-default Self
  premise continuation. closure-members/native-ctfe.rs still needs opaque-result
  callable typing. All 12 ordinary member-packet import attempts and all 25 ordinary
  captured-packet attempts remain unqualified at Destruct imports.
- Ordinary bootstrap gaps retained in earlier packets: const-Deref 11,
  default-supertraits Destruct 2, generic-const-calls Destruct 15,
  default-generic-subjects Bound 3, projected original-helper imports,
  explicit-call-members 15, borrowed-callables 17, const-Try 18 and ordinary nominal
  question-mark 16/provider. Independently rejecting inputs masked by these gaps
  are also unqualified, not semantic rejection evidence.
- Ordinary PA27 manual Copy wrapper valid bootstrap input without adding T:Copy;
  three original unused-definition projected-Copy negatives must eventually reject.
- Two initializer-storage named mutable-static direct/raw positives; ordinary
  nominal-pointer legacy metadata separate from its in-memory native behavior.
- Three conditional-member-calls const-Deref metadata positives:
  shared-field.rs, mutable-field.rs, direct-method.rs. v14 clears declaration
  encoding but still meets the source-role first-family fence; the actual first
  role has not been identified. Do not assume DropGlue or DerefTarget.
- Runtime wide casts/operations/patterns, metadata/wire/C ABI and all other named
  boundaries in owning packets. Mixed declaration/body v14 extensions do not
  replace the required genuine original v13 transport/artifact contracts.

Earlier retained requirements in Git handoffs 03003a86f and 2072521b8 remain binding
where not expressly completed in later checkpoint docs. Planned suites must fail,
not pass empty. Infrastructure exits, timeouts and panics never qualify a reject.
