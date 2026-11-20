# RustGM handoff

## Pause boundary · 2026-11-20

The strict build of original Rust Core reaches the metadata export of
`ChangeOutputType<T, V>` in `core/src/ops/try_trait.rs:389`. The last
integrated compiler checkpoint is `1c0022201` on local `main`, with the
strict replay recorded at `7851ebfd9`. The two-projection alias change is
preserved in the frontier worktree and as a patch. The next original Core
replay follows the focused alias check. Earlier handoffs and their receipts
are in `git show 7851ebfd9:handoff.md`.
[Self-hosting plan](docs/self-hosting-plan.md) · [Saved alias patch](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/strict-compiler-unit-graph/change-output-type-pause.patch)

| Worktree | State at the boundary |
| --- | --- |
| `/home/vishvananda/rustgm` | Local integration branch `main`. This handoff is the only change since `7851ebfd9`. |
| `/mnt/vizier/rustgm-scratch/rustgm-pa26-work` | `lane/strict-trait-constant-defaults-20261120` at `6c9972cd8`, with the unintegrated alias change. |
| `/mnt/vizier/rustgm-scratch/rustgm-pa29-work` | `lane/const-panic-marker-publication-20261120` at `0a8667768`, the isolated marker candidate. The performance worker was stopped. The earlier trait-header worker was absent after the restart and its experiments remain parked. |

## Self-hosting

- The strict executor has authenticated inputs for 22 planned target
  commands. The opt-level 1 observer on the unchanged Rust 1.98.0 Core
  sources at `88d9e12ae178fab0fb5cc050a94da85685d449ea` cleared foreign
  `VTable` and reached the alias gate in 37.92 seconds. Core did not
  publish an accepted strict artifact. Strict alloc, std and the compiler
  dependencies still follow. Stage 1, its PA1–PA24 run, the byte-identical
  Stage 2 rebuild and PA45 remain open. [Strict Core replay](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/strict-compiler-unit-graph/core-vtable-optimized-v1/target-units/receipt.json)
- Local `main` contains the Pointee language-role family, Thin trait alias
  transport, one-formal projected module alias transport and opaque foreign
  type transport. Fresh clients passed the relevant metadata round trips.
  The foreign type regression also verifies thin-pointer metadata, declaration
  identity and rejection by value. [VTable focused receipt](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/qualification/fast/1795193543534400841/receipt.json)
- The prior shared bring-up graph compiled and ran the original Core,
  locked `unicode-ident` and a native `no_std` scalar sweep. It recorded
  zero skips, 49 unproven obligations and eight waivers. Separate bring-up
  runs emitted Core plus alloc, and Vec and String matched their native
  oracles. Those runs use the bring-up profile. [Historical receipts and remaining consumers](docs/self-hosting-plan.md)

### Resume the alias checkpoint

The frozen reduction contains the crate-private alias, a public equivalent,
a closed alias control, a valid concrete use and a missing implementation.
Pinned rustc accepts the valid cases and rejects the missing implementation.
The trace shows two ordered root normalization requirements, first
`T::Residual` and then `<T::Residual as Residual<V>>::TryType`. The existing
packet reconstructs one root. The draft adds a sparse v14 metadata trailer
for the ordered roots, keeps the compact path for older packets and retains
the actual alias visibility. It also adds a fresh consumer check for a
successful projection, a missing trait proof and an inaccessible alias.
[Reduction](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/strict-compiler-unit-graph/core-change-output-alias-reduction-v1/result.json) · [Proof trace](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/strict-compiler-unit-graph/core-change-output-alias-proof-observation-v1/gdb.stdout)

The pause check passed assignment metadata validation, formatting, the
architecture audit and strict Clippy. The dev-fast build reached the
five-minute validation deadline before any of the three selected tests
could run. Resume by rerunning the same focused gate from the frontier
worktree. The receipt records the exact command, three test selectors,
stages and timeout. The compiler patch remains uncommitted.
[Focused check receipt](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/qualification/fast/1795194953336135589/receipt.json)

After the focused check, replay the frozen alias cases and compare the
closed control's metadata bytes and the missing-proof diagnostic. Then
commit the qualified change in the frontier worktree, pin an updated
optimized observer and use
`obj/parallel/strict-compiler-unit-graph/replay_core_step.py` in the
frontier worktree on the same unchanged strict Core input. The previous
strict replay used the observer pin in `core-vtable-optimized-v1/` under that graph
directory. The earlier shared bring-up replay can still use
`--bringup-guest NAME[@EDITION]=ROOT.rs` per dependency in order, with
each guest keeping its own crate identity, visibility and extern prelude.
[Validation and resource rules](docs/parallel-work.md) · [Test selection](TESTING_AND_REFERENCES.md)

## Performance

The saved optimized shared bring-up comparison is **42.775 s RustGM vs
37.014 s rustc**, a **15.6% RustGM slowdown** by median. The paired blocks
varied and the build scopes differ. RustGM used the joint source graph
with the waivers above. Rustc also freshly built `compiler_builtins` and its
build script. On three smaller native controls, RustGM was 1.81–2.01×
faster than pinned rustc and 4–7% slower than the rebuilt October RustGM
compiler. The early 4× workload is unavailable for exact replay. The 37.92 s
strict failure above is a frontier observation. [Paired comparison](/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/nostd-compile-time/paired/receipt.json) · [Attribution and controls](/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/performance-gap/report.md)

- **Where time goes.** Body semantics accounts for 57.8% of sampled
  RustGM cycles on the frozen graph, declarations 16.1%, and demanded
  instance preparation and proof 10.5%. Trait selection appears in 46.0%
  of body-phase samples, inference in 28.9%, and substitution or interning
  in 15.6%. Those call-chain shares overlap. Macro matching appears in
  about 0.23% of all samples. Destruction-state resizing takes 15.7% of
  instance-preparation samples. The sample profile attributes RustGM's
  work. The measured RustGM–rustc gap still has no per-feature allocation.
  [Profile](docs/self-hosting-plan.md#performance-work-from-the-agent-audits)
- **What landed.** Bounded child-proof reuse and its early source guard
  are on `main` at `9143861f0` and `049db339d`. Separate trait-header
  experiments are parked. Inference work, including the first-root visit
  reuse at `8ee6cea47`, and body-phase reuse of validated caller-tracking
  witnesses at `f5e3fbdcf` are also on `main`. The isolated const-panic
  marker candidate `0a8667768` retired 0.0213% fewer graph instructions
  and 1.28% fewer in the focused checked-absence control. The wall-time
  effect is unresolved within A/A spread. [Marker A/A and ABBA results](/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/const-panic-marker-publication/result.md)
- **Next performance experiment.** A capped replay of 746 frozen inputs
  matched archived output and found 46,584 repeated CFG owner validations
  across 5,505 owners, reading 1,579,269 attribute witnesses and 190,498
  tree rows. After the package shell moves from semantic analysis into
  lowering, the same immutable producer and source units account for
  42,027 reads and 94.5% of the observed validation work. The next step
  is to split those reads by template, concrete and late execution
  builders, then test carrying a borrowed declaration proof through the
  observed lowering path. Preserve the current owner checks and logical
  debits, including the first failed debit when an aggregate preview
  refuses the query. The read-only reference is
  `~/work/private/v5literal/checkout` at `a59264bd`. No candidate or
  speed claim exists for this census. Use frozen inputs, immutable
  binaries, A/A calibration, paired ABBA and exact output comparisons.
  [Owner census and budget controls](/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/prepared-owner-audit/observations.md)
