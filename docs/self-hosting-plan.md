# Self-hosting and assignment close-out plan

Adopted 2026-11-15 by the user. This is the current execution path. It
supersedes the execution order and process in the
[core census plan](core-census-plan.md) and the single-worker, strict-only loop
in the 2026-11-15 handoff. Source pins, Rust semantics, the frozen CPPGM backend
boundary and the resource/storage guards are unchanged.

## Why the plan changed

All of original core type-checks: 42,310 declarations and 25,858 bodies with
zero census records in 22 s. The artifact path after type checking was still a
first-error loop. Candidates 27 to 41 cleared 14 blockers in 14 hours, one per
clean release build, and 12 of them came from the const-call authentication
layer in `dev/src/body/constants/`. From 11-08 to 11-15 the tree gained 30K
compiler lines against 729K fixture and reference lines. Nothing downstream of
core (alloc, std, consumer instantiation, the backend at scale) was exercised.

## Adopted decisions

1. **Bring-up profile.** An explicit, unqualified profile may produce
   artifacts. A body that fails after type checking is recorded and emitted as
   a trapping stub, and the artifact is stamped unqualified. Strict mode stays
   the default and remains the only release gate. A bring-up artifact never
   closes a requirement.
2. **Ordinary profile frozen at PA24.** The synthetic bootstrap library gains
   no new behavior. Fixes for PA25 and later declare source-profile outcomes
   only. Existing ordinary gaps listed in earlier handoffs stop being work
   items; they are superseded by real core.
3. **Finite handouts for PA25–PA31.** Each assignment's contract is its
   published manifest plus an explicit deferred list. Everything the
   self-hosting corpus needs stays required through PA32–PA36, PA44 and PA45.
4. **Three lanes.** Frontier, runtime probes plus consumer suites, and
   publication. They use the three existing worktrees under
   `/mnt/vizier/rustgm-scratch`. No new worktrees. The shared controller runs
   three heavy jobs.
5. **PA45 self-hosting is the required capstone.**

## Phase 0: make the frontier visible

- Extend the keep-going census through the artifact pipeline: CFG build,
  promotion, borrow and destruction, lowering, and metadata encoding. One run
  lists every failure, grouped by message.
- Build a demand list with rustc as oracle (`-Zprint-mono-items` on the pinned
  toolchain) for RustGM itself and for the milestone consumers. It names the
  core, alloc and std instances self-hosting needs.
- Observe with an optimized incremental build. Clean pins and postchecks are
  for milestones only.

## Phase 1: artifacts before strictness

- Implement the bring-up profile and produce unqualified core metadata and
  LowIR.
- Fix census records in batches by cause. Bodies on the demand list go first.
- Confirm per layer whether a check is rejection-only before making it
  advisory. A check that produces facts lowering consumes stays a hard error
  and gets a stub instead.

## Phase 2: self-hosting ladder

| Rung | Exit test |
| --- | --- |
| M1 | core plus a `no_std` consumer (Option, iterators, fmt) runs natively |
| M2 | alloc plus a Vec/String/BTreeMap/Rc/Arc consumer |
| M3 | std (rlib, abort) plus a println/fs/env/HashMap consumer |
| M4 | unicode-ident, tinyvec, unicode-normalization |
| M5 | Stage 1 (RustGM built by RustGM) passes the committed PA1–PA24 oracles through `RUSTGM_TEST_APP` |
| M6 | Stage 2 output is byte-identical to stage 1 |
| M7 | Strict ratchet: zero bring-up records for core, alloc, std and the compiler |

Self-hosting does not need procedural macros, unwinding or async. It does need
atomics beyond U32, SIMD lowering for hashbrown, allocator/start/TLS shims and
a backend that handles the LowIR volume. The runtime lane probes these now.

## Process

- One qualification and one tracker row per batch by cause. No per-fix
  checkpoint document, ROADMAP paragraph or handoff rewrite.
- Per fix: a minimal positive and negative in the source profile. Add a native
  observer only when code generation changes, and a scaling run only when a
  loop or ledger changes.
- Outcomes are still declared before any oracle is generated, and references
  are never hand-edited.
- Full CI runs nightly and at milestones. ROADMAP and the handoff change at
  milestones.

## Lanes

| Lane | Worktree | Work |
| --- | --- | --- |
| Frontier | `rustgm-pa26-work` | Pipeline census, bring-up profile, batched fixes, M1 to M7 |
| Runtime | `rustgm-backend-work` | Demand list, consumer programs with rustc-verified output, backend and runtime probes |
| Publication | `rustgm-async-work` | Publish the nine implemented units, reconcile PA1–PA20 records, finite handouts, PA45 brief |

## Assignment close-out

- **Package what exists:** the nine implemented, unpublished units and the
  PA1–PA20 release records.
- **Falls out of the ladder:** PA32 (M1), PA33 (M2), PA34/35 (executor and
  crates), PA36 (M3), PA44 and PA45 (M5 to M7).
- **Independent feature work:** PA37/38 after std; PA39 to PA43 in parallel
  when a lane is free.
