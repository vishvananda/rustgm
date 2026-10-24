# Parallel library bootstrap implementation

Use three GPT-6.1-Sol workers at high reasoning and one integrator. Each worker
owns a separate worktree and local branch. PA24 is published; main integrates
the remaining PA25–PA30 checkpoints and the prerequisites for original upstream
core, alloc and std. Do not copy dirty files between workers or reset their
unfinished work.
Workers own implementation, independently qualified tests and handout drafts.
The integrator owns shared interface decisions, integration, public pushes,
assignment activation and final text-oracle publication.

Use [the live library blocker tracker](library-build-blockers.md) for current
issue assignments and concrete closure tests. Workers may tackle multiple
independent blockers in their lanes while preserving a separate coherent handoff
for each. Current priority is the unchanged-core loading/expansion blocker:
the integrator owns scoped item/path/import/glob lookup, Worker C investigates
hygienic field/member consumers, Worker B owns projected Copy and early-impl
readiness, and Worker A owns complete generic source-body validation/transport.
These implementation priorities do not change assignment release ownership or
the shared resource limits below.

## Assignment ownership

| Lane | First assignment | Next assignment after integration | Contract |
| --- | --- | --- | --- |
| Integrator | PA24 closures | Integration and release qualification | Existing `docs/pa24-closures.md` |
| Worker A | PA25 constant evaluation | PA29 crates and metadata | [PA25/PA29](parallel/pa25-pa29.md) |
| Worker B | PA26 language/library hooks | PA30 foreign interfaces | [PA26/PA30](parallel/pa26-pa30.md) |
| Worker C | PA27 unsafe language | PA28 intrinsics and atomics | [PA27/PA28](parallel/pa27-pa28.md) |

Worker B's PA24 receiver-adapter slice is integrated and released. Its current
lane is PA26/PA30. Worker A retains anonymous instance identities and Worker C
retains unsafe capture replay, so changes to shared body enums must be coordinated
before editing them. The integrator owns combined qualification, reviewed text
oracles, assignment activation and public pushes to main.

The integrator's closed record, array and stored-value metadata checkpoints are
integrated on main. The isolated `rustgm-pa29-work` checkout remains available
for the next generic transport work, coordinated with Worker A's PA25 caller
transport. Generic body transport is still unimplemented; its
[owner/contract proposal](pa29-generic-transport-proposal.md) defines the shared
prerequisite and does not grant execution permission. Worker B may prepare
and independently qualify PA30 source fixtures while a coherent PA26 checkpoint
awaits integration. This preparation uses the same resource limits and does not
activate either assignment or change the backend contract.

The course release order remains cumulative. Future work can be developed
without marking an unfinished prerequisite or assignment active. A worker
moves to its queued assignment after the integrator accepts its checkpoint;
it does not wait for every unrelated handout to be published.

### Priority toward upstream libraries

The [library bootstrap plan](library-bootstrap-roadmap.md) makes PA31–PA36
required and moves async/full unwinding behind the first usable crate graph.
PA1–PA30 ownership and all retained valid-but-unsupported cases stay intact.
Finish current coherent proof, iterator and layout checkpoints, then prioritize:

- Worker A: PA25 proof closure, then PA29 generic/const body and contract transport.
- Worker B: PA26 semantics against source-declared identities, then PA31 bindings
  and the pinned core source path; retain PA30 ABI obligations and coordinate them.
- Worker C: PA27/PA28 validity, layout and intrinsic gaps selected by an exact
  pinned core/alloc inventory, with independent backend capability probes.
- Integrator: combined qualification, crate metadata integration, PA34 build and
  edition orchestration, publication and source/configuration provenance.

Current library factories are temporary scaffolding. Do not grow them into an
implementation of ordinary Option/Result methods, iterators, containers or other
upstream algorithms. Source-driven hooks and typed cross-crate transport are the
critical path. Record and retain remaining assignment requirements when switching
work; a checkpoint handoff does not activate an unfinished assignment. Keep the
three-job resource policy below unchanged and share immutable upstream sources.

Before central enum or pipeline changes, send the integrator the exact typed
interface and consumers. In particular coordinate closure/body readiness,
const/runtime operation permissions, language identities, callable ABI and
cross-crate identity remapping. Use local dense arena IDs internally; metadata
must define deterministic external identities and explicit import remapping.

## Resource controller

The controller is opt-in per local Git repository. Its configuration and
kernel lease files live in the shared Git common directory, so all registered
worktrees use the same limits. Hosted CI and unrelated clones retain the
ordinary storage guard unless explicitly configured.

Initial reviewed limits:

- Two heavy commands by default, with an explicitly enabled three-job trial;
  one command per worktree.
- Functional validation shares this budget with focused worker checks.
  Benchmarks use an exclusive lease, which excludes every other guarded heavy
  command. A waiting exclusive job blocks new ordinary admission.
- Each process tree has a systemd user scope: CPU quota 800% (eight CPU cores),
  MemoryMax 12 GiB, no swap and at most 256 tasks. Missing systemd limits fail
  the command; there is no unrestricted fallback in a configured repository.
- Cargo build jobs and Rust test threads are two. Course/native Python runners
  inherit a default of two jobs. Do not raise these settings in worker commands.
- Keep 150 GiB free on the filesystem. Admission additionally reserves unused
  worker allowance, conservatively accounting for active peers. A live disk
  monitor checks free space every 0.5 seconds and terminates the job's entire
  scope if the reserve is crossed, including pressure from unrelated projects.
- At most 50 GiB of allocated generated files per worker. Scans include Cargo
  caches and pinned artifacts, use bounded directory traversal and run repeatedly
  during commands. Crossing the limit stops the scope; this is monitored
  accounting, not an instantaneous filesystem quota. Disk checks still run while
  a scan is incomplete. Command completion also checks the final size.
- Workers account for their entire `obj/`. Main accounts for explicitly registered
  active output roots; historical evidence is outside that per-worker budget,
  but still reduces the global free-space reserve. Register any additional main
  output root before using it. New evidence should use `obj/parallel/`.

Do not delete another checkout's files or attempt to reclaim unrelated projects.
Keep only the baseline/candidate executables required by current measurements.
After diagnosing a failed batch, prune its generated objects/executables through
`storage.py`; retain inputs, diagnostics, receipts, hashes and raw observations.

Registration is an integrator operation, performed before workers start:

```sh
python3 scripts/resources.py register /absolute/worktree --name pa25
python3 scripts/resources.py status
```

The user authorized a three-job trial on 2026-10-21. At an idle boundary, after
updating the controller in every participating worktree, select it with
`python3 scripts/resources.py set-jobs 3`. Use `set-jobs 2` to restore the
default at another idle boundary. This changes no CPU, RAM, task, disk reserve,
worker quota, Cargo thread or exclusive-measurement limit. A command queued
under a changed policy fails before launching and must be retried.

The existing storage wrapper automatically joins the shared controller:

```sh
python3 scripts/storage.py run --no-prune -- cargo check --manifest-path dev/Cargo.toml --locked
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py run const_functions --release
```

Declare every disposable output root when selecting scoped cleanup. Existing
maintainer harnesses may write additional `obj/` directories; include them or
use ordinary cleanup. Cleanup is never run against another worktree.

For a complete Makefile target or an integration-only full/performance batch,
use an outer resource lease. Nested storage wrappers inherit it, but keep their
own cleanup and checkout lock:

```sh
python3 scripts/resources.py run -- make test-harness
python3 scripts/resources.py run -- python3 scripts/storage.py run --prune-under obj/parallel/release -- python3 obj/parallel/validate_release.py
python3 scripts/resources.py run --exclusive -- python3 scripts/storage.py run --prune-under obj/parallel/performance -- python3 obj/parallel/measure.py
```

Use `resources.py` rather than nesting `storage.py` around a Makefile that itself
uses storage wrappers. `resources.py` owns scheduling, limits and receipts;
`storage.py` additionally owns cleanup. Do not start background build/test jobs.
A process tree that outlives its launcher is terminated and reported as failure.
Nested leases must belong to a live ancestor and the same worktree; ordinary
leases cannot be upgraded to exclusive inside a running command.

The controller writes receipts in `obj/parallel/resources/`, including the
command, scope, limits' shared policy, exit status, error and measured usage.
Kernel locks own availability; stale JSON does not consume a slot. A cooperative
interrupt stops the scope before releasing the lease. If the controller is
forcibly killed, inspect its recorded systemd scope and stop any surviving job
before restarting; inherited lease descriptors are an additional safeguard,
not a substitute for checking the scope.

## Worker completion contract

Read `AGENTS.md`, `spec.md`, the owning handout and `TESTING_AND_REFERENCES.md`.
Use typed facts, bounded iterative algorithms and safe idiomatic Rust. Run
strict Clippy all/pedantic and the existing architecture/file audits. No Rust
frontend delegation; rustc is an independent oracle and CPPGM is only the
external LowIR backend. Do not modify the active CPPGM/v5rust checkouts.

Batch a small end-to-end feature, not unrelated infrastructure. Before generation,
declare positive/negative outcomes and qualify them independently. Run focused
source/native controls and scaling invariants during implementation; keep all
active text suites as a candidate regression check when the checkpoint is coherent.
Coordinate full suites and broad performance matrices with the integrator.
Do not mutate the source while a build or qualification is running.

Use all-target strict Clippy before code generation to catch type/lint failures
in both new tests and production code. A separate `cargo check` normally repeats
that work. Settle related test edits first, then use debug focused tests, batching
suite and exact private selections with `scripts/maintainer_suites.py batch`.
Keep release code generation for optimization-sensitive checks, performance and
the combined integration checkpoint. Workers do not each repeat the full release
suite; the integrator runs it once against the merged candidate. Local
`make test-maintainer` includes regular and native tests in the same release
build and performs one cleanup scan. Existing successful independent observations
can be cited for identical frozen inputs/toolchain/profile; candidate execution
and candidate-specific proofs must still be checked after integration. Do not
turn an object-only rustc observation into executable or runtime qualification.

For hot-path changes, apply the frontend performance precedent in `spec.md`:
identify repeated work removed, cache and ownership boundaries, and dimensions
that can multiply work. Include cold, warm and invalidated controls where
applicable, bounded work/storage counters and compact record sizes. Keep each
caller's proof authority separate from shared structure. Faster wall times alone
do not establish a better complexity bound.

Commit coherent checkpoints locally and report commit/source hashes, exact
passed/failed/pending checks, evidence locations, interface changes and remaining
boundaries. No worker pushes branches/main, changes assignment activation,
auto-blesses outputs or hand-edits references. The integrator runs the required
combined release checks before publishing to main. A resource interruption is
an infrastructure failure, never a language rejection or passing test.
