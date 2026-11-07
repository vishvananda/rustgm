# RustGM handoff — 2026-11-07

The goal is active again: **complete all assignment requirements to build the
original std crates**. Execution follows the
[core census and transport plan](docs/core-census-plan.md), adopted 2026-11-07.
The backend stays frozen. Lanes: A (Source tree) LIB-193 attribution and repair;
B (Root tree) analysis-only census mode; C (Async tree) branching body
transport; D (next free lane) target-unit executor, Rust 2015/2018 and runtime
shims. The const-callable WIP stays isolated on its branch. Commits use the
repository's single-line subjects without trailers.

## Published checkpoint and assignment status

Main compiler checkpoint: `fb2acd69c292469d01596ef274906e69f888ccfa`.
The subsequent handoff-only commit changes no compiler or qualification inputs.

- `18fad82a8`: actual Field OFFSET/default/layout execution and typed projection
  dependency/body-proof publication repairs.
- `6b994c816`: joined real pointer guarantees/native fallbacks and captured const
  execution through shared/mutable/owned environments and existing components.
- `09ca0fd48`: cold trait/foreign prototypes use actual bound header owners.
- `567ab37b5`: actual staged/ordinary deprecation release semantics; inactive
  attributes emit no event or expectation fulfillment; malformed staged headers
  still reject before use and independently of caps.
- `fb2acd69c`: combined qualification, specification and LIB193 tracker update.

PA1–20 remain implemented-draft while release records are reconciled; PA21–24
are implemented for their documented course profiles. **PA25–44 remain incomplete
as whole assignments.** The dependency plan still has37 units:10 implemented
finite profiles,15 partial,12 missing; all37 component publications are pending.
No original core, alloc or std artifact exists. Do not infer assignment completion
from a finite checkpoint or a successful semantic view.

Read `AGENTS.md`, `spec.md`, `ROADMAP.md`, `assignment-plan.json`,
`docs/assignment-order.md`, `docs/library-build-blockers.md` and the owning handout.
The authoritative latest checkpoint is
`docs/parallel/cold-deprecation-main-join.{md,json}`. Older frontier paragraphs
are historical; use the tracker's first entry and newest evidence.

## Current core frontier: LIB193

Current qualified immutable compiler:

```
/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/cold-deprecation-main-join-v1/pin/rustgm
SHA256 48a0b12a2578d1753a8d8d0f094e72b9bd129d3a015c0fc3fba8761a0c7f788c
```

Its actual unchanged-source observation clears LIB192 and reports:

```
ERROR: input: source header contract query work exceeds profile limit
```

It takes19.6071s, opens375 original sources, preserves all722 input hashes, all
six current configuration-source pins, profile/options and limits. **722 is the
manifest's checked input count, not722 successfully compiled files.** This is
an unlocated guard failure, not an artifact, complete body checking or coverage
percentage. The last profile remains Rust1.98/edition2024/x86-64 Linux/panic-abort,
explicit cap-lints=allow, source-contract work14500624/attempted1GiB/retained64MiB.
Final uncapped qualification remains required. No resource limit was raised.

Root observation folder:
`/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/cold-deprecation-main-join-v1/`.
It contains the receipt, producer/manifest bindings, raw diagnostics/opened
sources and frozen pin. Do not repeat this known failure without a real repair
or specifically justified new diagnostic.

Source's separate analysis-only diagnostic at clean97574, **without cold44f0**,
identifies the last crossing charge during target-feature collection:

```
steps=1000002 delta=2 bytes=14686030
declarations=26322 definitions=26564 owners=24723
caller=src/sema/target_features.rs:38:5
charge_query -> reserve_source_contracts -> target_features::pay
 -> parsing::parse -> FeatureSet::insert -> parsing::collect
 -> Checker::from_facts -> Checker::new -> declarations::analyze_with_executor
```

This crosses the existing local1M work limit; storage is below16,000,000 bytes.
It does not establish how prior aggregate work was distributed, prove quadratic
behavior or identify the current combined compiler's exact caller. **Lane A:
attribute actual shared-budget contributions and collector work, then repair per
the plan's LIB-193 section.** Preserve sticky failures and genuine
source/target permissions. Replace the header ledger's borrowed fixed constants with the
input-proportional bound of the amended spec budget rule; never tune charges,
skip checks or raise a factor to absorb repeated work.

## Qualification and CI

Final same-snapshot Root receipts under
`/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/qualification/fast/`:

| Receipt | Actual passing scope | Seconds |
| --- | --- | --- |
| `1794021721253894418` | Strict cfg(test) lint/link +4 private cold-owner/replay/scaling/transport controls |218.33|
| `1794022014299758566` |23 regular cold/deprecation/pointer/closure/overflow controls |204.54|
| `1794022233450994874` |6 actual native tests including both new families and pointer/closure regressions |34.31|

Format, architecture and strict selected/private Clippy pass. Harness282,
460 registered roots, upstream127/Reference126 pass. Main matches all3717
qualified input hashes and73 new fixture/reference hashes. Native streams include
`cold-foreign 28 36 2`, `future-deprecation 23`, genuine pointer fallback variants,
and `closure-source 23 47 23 23 47`.

Earlier6b integration also executes all3 new sealed closure-record adversaries
and the five-fault source Copy overlay control in private1794020317010221757;
28 regular and6 native controls pass. Field's complete native output is
`field 4 23 29 37 41 47 31 1`. All raw measurements and setup failures are retained;
the calibrated ordinary/creation controls establish no general speed change.

Full CI **42333296811** targets older main6b994c816, not the final cold/deprecation
checkpoint. At the last poll its fast job, lint/architecture/harness, independent
host graph, release and course stages passed; native qualification was running.
The preceding push fast run42333274746 passed. New final-main fast CI starts on
push. On resume inspect actual status/logs first:

```sh
cd /home/vishvananda/rustgm
gh run view 42333296811 --json status,conclusion,jobs
gh run list --limit 5 --json databaseId,headSha,status,conclusion,event
```

Keep an admitted run intact. Fetch terminal failure logs and resolve real
regressions. After this full run is terminal, dispatch one full successor for
current main if still needed; do not cancel it by immediately redispatching:
`gh workflow run check.yml --ref main`. Focused passing evidence permits main
integration, but is not full release qualification.

## Restartable worktrees

Main checkout is `/home/vishvananda/rustgm`. All build work stays on btrfs.
Git refs are shared; inspect status from each repository root before mutations.

**Root integration tree:** `/mnt/vizier/rustgm-scratch/rustgm-pa26-work`, branch
`implement/forced-indirect-abi-lib111-20261030`. Qualified source checkpoint is
`cb12ec17e`; qualification/spec/tracker commit is `f9d7df50d`, cherry-equivalent
to mainfb2acd69c. Preserve the warm `obj/rust/dev-fast` and
`obj/rust/private-lint/dev-fast` caches. There is no live Root validation handle
after wrap-up. Merge current main at an idle boundary before new source edits.

**Source worker:** `/mnt/vizier/rustgm-scratch/rustgm-pa29-work`, branch
`implement/source-future-deprecation-20261107`, clean HEAD
`97574aca92ed3a52d0206daeca188ad691b3cb71`. Its qualified changes are integrated.
Detailed restart note:
`obj/parallel/source-transport/header-query-budget/handoff.md`.
Diagnostic evidence is `header-query-budget/diagnostic1/`, including the saved
instrumentation patch, raw backtrace, immutable binary/binding and restored.json.
Temporary production instrumentation was removed; handle4349 terminated normally.
Diagnostic SHA isf56f18d54efd3916eb245cdb04363ddb0e7620dd7914574f7d0a23bc51a67aa6.
**Its mutable obj/rust/dev-fast/rustgm is diagnostic, not qualified.** Qualified
Source dfbd113e is frozen under future-deprecation/final1/rustgm. Preserve existing
Source cache flags (nonincremental plus its type-size diagnostic flags); do not
create a replacement cache. Resume budget attribution against the correct
integrated baseline, preserving diagnostic scope differences.

**Const-callable worker:** `/mnt/vizier/rustgm-scratch/rustgm-async-work`, branch
`implement/const-closures-20261107`, clean HEAD
`c9b968a756d910edfd829935b7f862460b7188f4`; read its root `handoff.md`.
Reference-only458ad516e independently qualifies generic/conditional Rust
metadata/object/native `conditions 23 23 47`; ordinary Flag rejects E0277.
Source WIP0f156fe84 (33 files) is **UNQUALIFIED and NOT ON MAIN**: no candidate
compile/public/native/metadata gate ran, and its last format predates final edits.
Do not blindly cherry-pick it into main or infer support from reference outcomes.

That WIP shares inherited argument reconstruction, folds genuine defining clauses
with a paid Bound, retains scoped conditional children, and introduces the approved
Dispatcher::const_callable port. Its actual member/full arguments/source/effect,
unused-argument mutation, region/debt publication, borrow/API correctness, formatting
and predicted width assertions still need review and validation. Callee clauses
must never become caller premises. Generic-F receiver adaptation, prove_adapter,
bootstrap, binders/anonymous owners, complete capture destruction and metadata
transport remain required. Merge current main safely before its first coherent
focused gate; the worker note gives exact commands and limitations.

The existing `/root/source_transport` and `/root/async_signatures` workers are
idle, restartable with follow-up tasks after an explicit resume. No new agents,
trees or caches are needed.

## Resource, cleanup and verification rules

- Keep2 heavy jobs globally,1 per tree,16GiB absolute output cap per tree,
  150GiB filesystem reserve; check space before work. No root-drive builds.
- Use existing guarded cached dev-fast checks, <=300s including admission,
  jobs2. No local release/full sweep per edit. Root uses incremental1; preserve
  each other tree's established flags. Full longer checks belong in CI.
- Keep exact selections and authoritative process handles; observation timeout
  is not terminal. Do not mutate compiler/fixtures during live qualification.
- Safe idiomatic Rust, Clippy all/pedantic -Dwarnings; syntax-aware file/function,
  nesting and phase audits apply. No production unwrap/expect/panic/unsafe/env.
- Parse once, consume typed facts, share indexed work and preserve exact caller
  obligations. Measure algorithm checkpoints with immutable A/B, A/A+ABBA,
  frozen inputs and retained observations; Python timeout polling quantizes short
  wall samples, so use blocking waitpid inside an outer guarded deadline.
- Backend pin staysb7c6b8e72595a524e1b3f147212d69c02b257dd8. No backend/codegen
  changes were made. Do not edit cppgm-extended or v5rust during RustGM work.

Root retired7 superseded compiler pins (~0.63GiB unshared blocks) and retains
only current48a0b12a and predecessor987578c2 for this series. Retirement receipt:
`obj/parallel/cold-deprecation-main-join-v1/pin-retirement.json`. Historical Field
and older baseline binaries are pruned; their receipts/hashes remain. Do not claim
those executables are still available for a new measurement.

Root also completed Source's released5bd header-pin retirement (2 binaries,
~0.18GiB), preserving evidence/currentdfbd/b800/diagnosticf56. Receipt:
`/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/source-transport/pin-retirement/header-5bd/receipt.json`.
This supersedes the Source worker note's earlier cleanup-pending paragraph.
Keep existing warm caches for restart and continue scoped cleanup at idle
boundaries; never remove another active tree's artifacts.
