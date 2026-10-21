# Parallel PA24–PA30 implementation

Use three GPT-6.1-Sol workers at high reasoning and one integrator. Each worker
owns a separate worktree and local branch. Main retains the unfinished PA24
snapshot; do not copy its dirty files into unrelated workers or reset them.
Workers own implementation, independently qualified tests and handout drafts.
The integrator owns shared interface decisions, integration, public pushes,
assignment activation and final text-oracle publication.

## Assignment ownership

| Lane | First assignment | Next assignment after integration | Contract |
| --- | --- | --- | --- |
| Integrator | PA24 closures | Integration and release qualification | Existing `docs/pa24-closures.md` |
| Worker A | PA25 constant evaluation | PA29 crates and metadata | [PA25/PA29](parallel/pa25-pa29.md) |
| Worker B | PA26 language/library hooks | PA30 foreign interfaces | [PA26/PA30](parallel/pa26-pa30.md) |
| Worker C | PA27 unsafe language | PA28 intrinsics and atomics | [PA27/PA28](parallel/pa27-pa28.md) |

The course release order remains cumulative. Future work can be developed
without marking an unfinished prerequisite or assignment active. A worker
moves to its queued assignment after the integrator accepts its checkpoint;
it does not wait for every unrelated handout to be published.

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

- At most two heavy commands across all worktrees, one command per worktree.
- Functional validation shares the two-command budget with focused worker checks.
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

The existing storage wrapper automatically joins the shared controller:

```sh
python3 scripts/storage.py run --no-prune -- cargo check --manifest-path dev/Cargo.toml --locked
python3 scripts/storage.py run --prune-under obj/parallel/pa25 -- cargo test --release --manifest-path dev/Cargo.toml --test constant_execution
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

Commit coherent checkpoints locally and report commit/source hashes, exact
passed/failed/pending checks, evidence locations, interface changes and remaining
boundaries. No worker pushes branches/main, changes assignment activation,
auto-blesses outputs or hand-edits references. The integrator runs the required
combined release checks before publishing to main. A resource interruption is
an infrastructure failure, never a language rejection or passing test.
