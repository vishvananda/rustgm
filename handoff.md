# RustGM handoff — 2026-11-15

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

## State on 2026-11-15

**Real core compiles in the bring-up profile, and a real no_std consumer
compiles with it, passes the pinned backend, links with libc and runs** until
it reaches an intrinsic that has no run-time lowering yet. Nothing here is a
qualification: strict mode still stops at the first fence, and whole PA25–PA44
remain planned.

- **Core alone:** the census stubs 1,130 functions
  (`obj/parallel/bringup/core/core.skip.tsv`, summary in `summary.tsv`); about
  1,045 are x86 SIMD functions whose constants need intrinsics that compile-time
  evaluation lacks. Publication passes over nine metadata fences (`waived.tsv`)
  and writes no usable metadata. A source library lowers nothing itself.
- **Linked consumer:** `scripts/bringup_linked.py` mounts a consumer as a
  private module of a writable copy of core, so one session compiles both and
  no metadata is needed. This is how every result below was produced.
- **M1 consumers:** all 17 staged programs
  (`pa32/tests/spec/core-consumers/`) were run this way; **none passes yet**.
  Six compile, pass the backend, link and run until their first unlowered
  intrinsic: `atomics`, `chars-strs`, `duration-net`, `integers`, `panic-index`,
  `panic-overflow`. Eleven stop while the program itself is type-checked or its
  constants are evaluated, before any function can be stubbed (table below).

### Bring-up profile

`--bringup-skip-list FILE` selects it; a library also needs a crate
disambiguator ending in `_bringup`. Everything below is refused without it.

| Mechanism | Reported as |
| --- | --- |
| A listed function becomes a trapping stub; every stage sets aside the failures it can attribute to a function and names them | `bringup-blocker<TAB>path<TAB>offset<TAB>name<TAB>message` |
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
stay the session's. Its files are keyed `NAME:PATH` in skip lists and
`bringup-*` lines. The last guest is the program: under `#![no_main]` its
exported C `main` is the entry point. `scripts/bringup_linked.py --guest`
passes a consumer this way instead of mounting it in a copy of core. A guest
session publishes no metadata, and there is no allocator shim (a
`#![needs_allocator]` guest is reported as waived), `#[global_allocator]`,
runtime start or `std` prelude yet.

```sh
# Frontier worktree. Optimized incremental compiler; never release evidence.
make observe && cp obj/rust/observe/rustgm obj/parallel/bringup/bin/rustgm-observe

# One or more M1 consumers, end to end (census, backend, run, compare).
python3 scripts/resources.py run --wait-seconds 900 --max-output-gib 24 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 24 \
  --prune-under obj/parallel/bringup/linked-work -- \
  python3 scripts/bringup_linked.py \
  --compiler obj/parallel/bringup/bin/rustgm-observe \
  --library obj/parallel/original-core-inputs-88d9e12ae178fab0fb5cc050a94da85685d449ea/library \
  --skip-list obj/parallel/bringup/core/core.skip.tsv \
  --work obj/parallel/bringup/linked-work \
  pa32/tests/spec/core-consumers/integers.rs
```

The frontier lane uses a justified 24 GiB output cap. Core's own census command
is in `obj/parallel/bringup/core/records.json`.

### Next work, in blocking order

1. **Run-time lowering of the waived intrinsics.** Programs that waive each:
   `offset` 6; `unreachable`, `ptr_offset_from_unsigned`, `exact_div`,
   `cttz_nonzero`, `ctpop`, `add_with_overflow` 4 each; `black_box` 2; then
   `read_via_copy`, `slice_get_unchecked`, `saturating_add/sub`, `ctlz`,
   `ctlz_nonzero`, `cttz`, `bswap`, `bitreverse` and the sixteen `atomic_*`
   operations. The hook is `Lower::trapped_call`
   (`dev/src/lower/function/trapped_calls.rs`): add an unqualified lowering
   there, reported as its own waiver, before the trap shim.
2. **Frontend failures on consumer programs.** These stop the whole session
   and are ordinary strict-profile bugs; each program is its reproducer.

   | Program | Failure |
   | --- | --- |
   | `conversions`, `fmt-floats`, `option-result`, `sorting` | `constant execution reached panic` (core file 64, offset 3755) |
   | `consts-statics` | `source intrinsic constant execution is not implemented` (core file 12) |
   | `cells-mem` | `associated type projection requires normalization` |
   | `fmt-integers` | `incompatible types` at `f.debug_map().entry(&"k", &1)` |
   | `iterators` | `trait obligation requires an unambiguous matching assumption or implementation` |
   | `patterns-ranges` | `assignment requires a place` |
   | `slices` | `selected implementation is missing the associated type` |
   | `traits-generics` | `dyn supertrait edge requires a trait bound` |

   Failures of these stages are not yet attributed to a function, so the
   census cannot stub around them (`dev/bin/modules.rs` attributes lowering
   failures only).
3. **Stubbed core functions that programs reach.** Joined with the rustc demand
   list, only 36 of core's 1,130 stubs are reached by the M1 programs and 35 by
   the compiler's own build. By cause: const `Destruct` cleanup proofs
   (`Iterator::try_fold`, `fold`, `reduce`, `ControlFlow::break_value`,
   `Option::insert`); borrow and lifetime proofs that reject valid code
   (`quicksort`, `fmt::write`, `Chain::next`, `Flatten::next`, `str::split`);
   and `opaque storage requires a checked defining type` in `Map::fold`, which
   every program adds to its own list.
4. `#![no_main]` with a C `main` (the driver rewrites the entry meanwhile).
5. Metadata export, for the strict ratchet only: the linked route needs none.

### Lanes

- **Frontier** (`rustgm-pa26-work`, `lane/frontier-20261115`): the bring-up
  profile, the census and the fixes above.
- **Runtime** (`rustgm-backend-work`, `lane/runtime-20261115`): finished its
  first assignment. `docs/self-hosting-runtime.md` holds the rustc-oracle demand
  list, 47 staged consumer programs for M1 to M4, the run-time gap table with
  nine open decisions, and the backend scale probe (4.2 million functions and
  1.6 GB of LowIR pass at `-O0`; one self-hosting unit is estimated at 280 to
  415 MB).
- **Publication** (`rustgm-async-work`, `lane/publication-20261115`): eight
  component units are released (2,078 cases). Open: `provided-profile` cannot be
  released yet; PA7 and PA11 could be flipped to implemented; the handout totals
  of PA8, PA10, PA12, PA13 and PA15 to PA18 are stale.

Sections from "Latest genuine original-core attempt" onward describe the strict
frontier as it stood before the plan changed. They are history, except
"Requirements still open", which is the strict ratchet's list.

Detailed earlier handoffs are retained in Git: `git show 0031a83d4:handoff.md`.

## Workspace, authorization and resources

- Build only in the three worktrees under `/mnt/vizier/rustgm-scratch`:
  `rustgm-pa26-work` (frontier), `rustgm-backend-work` (runtime) and
  `rustgm-async-work` (publication). Main `/home/vishvananda/rustgm` is Git-only.
- No new worktrees. Coherent relevant commits/pushes are authorized;
  use single-line subjects without trailers.
- CPPGM backend: release v0.9.1, `c9b05c4c0`, of `vishvananda/cppgm` (the
  renamed cppgm-extended; the driver is `cppgm`). Install it in a worktree
  with `python3 scripts/setup_backend.py`; a `backend.local.json` from v0.9.0
  or earlier no longer works. The release's in-process LowIR library is pinned
  at interface version 3, which carries the boundary facts that RustGM's
  writer states, and has a handshake. Nothing uses it yet. The user approved
  an in-process FFI bridge on 2026-11-15: it is the next backend step, in its
  own crate outside the frontend library
  ([checkpoint](docs/backend-current-checkpoint.md)).
- Do not modify C++/v5rust checkouts or another checkout's outputs. Rustc is a
  test oracle, never a frontend substitute.
- Read AGENTS.md, spec.md, ROADMAP.md, the owning handout and
  TESTING_AND_REFERENCES.md. `docs/parallel-work.md` contains historical worker
  assignments; the self-hosting plan's three lanes take precedence.
- Shared controller: 3 heavy jobs globally, 150 GiB free-space reserve, CPU 800%,
  RAM 12 GiB, tasks 256. Output cap 16 GiB per worktree; one heavy job per worktree.
  Shared free space was about 278 GiB at wrap-up. The former headroom blocker is
  resolved. Do not lower reserves or clean other checkouts.
- Every build/test/frontend/debugger probe/measurement uses resource and storage
  guards. `make fastcheck` supplies its own guards and has a 300s feedback budget.
  Wait for a wrapper's actual terminal result before editing or starting the next
  heavy command. Do not change tracked files during builds or validation.

```sh
# Direct validation/probe; declare all disposable roots, or omit --prune-under
# for scattered outputs and let storage perform its full scan.
python3 scripts/resources.py run --wait-seconds 0 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 \
  --prune-under obj/KNOWN_PARENT -- COMMAND

# Build-only: use --no-prune, never combine it with --prune-under.
python3 scripts/resources.py run --wait-seconds 0 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --no-prune -- COMMAND

# Focused selection syntax: root::test, not --case.
make fastcheck FAST_ARGS='root::test --prune-under obj/KNOWN_PARENT'
```

Do not cold-build the private unit harness locally: prior attempts exceed the
feedback budget. Use focused public controls and let full CI execute private
invariants. Private fast selections lint/link once with clippy-driver; do not
repeat that compilation using cargo test --lib. Do not run broad release/native
matrices per edit. Use `make tidy` between batches, preserving active pins.

Publish from B, then synchronize main using Git only:

```sh
git -c credential.helper= -c 'credential.helper=!gh auth git-credential' \
  push https://github.com/vishvananda/rustgm.git HEAD:main
# In /home/vishvananda/rustgm:
git fetch /mnt/vizier/rustgm-scratch/rustgm-pa26-work HEAD:refs/remotes/origin/main
git merge --ff-only --quiet origin/main
```

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
