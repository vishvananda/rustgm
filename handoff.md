# RustGM self-hosting handoff

## Resume point — 2026-11-20

`aeab57f68` records the lifetime census and `0caa1751d` the latest strict
unit-graph replay. The combined graph has authenticated inputs for all
22 planned target commands. The strict optimized compiler reaches
`core` publication after 36.80 seconds without emitting an accepted
artifact. Its first failure is the provided
`Lemire::MAX_EXPONENT_FAST_PATH` constant.
[Strict graph evidence](docs/self-hosting-plan.md#L1176)

The self-hosting worktree is
`/mnt/vizier/rustgm-scratch/rustgm-pa26-work` on
`lane/strict-trait-constant-defaults-20261120`. The required-constant
checkpoint `7a5329550` is on `main`. A fresh RustGM client imports
`Table<T>::ROW`, returns `19` and `513` for its two implementations and
rejects a missing or mistyped override. Four focused integration tests
passed. `4520c8716` repairs the private decoder call. The private
harness reached its build deadline before execution. Frontier commit
`d7a9ddcc8` publishes and byte-round-trips an actual default and
override with five passing focused tests. `7839da8d0` keeps the default
under its trait owner and passes four focused checks. `dc9017c11`
re-proves imported constant links using the actual declaration kind
and canonical constant contract. Two focused checks pass. A fresh
RustGM client with the same provider reached the old function-only
original-body preflight. `5bb85597e` reconstructs and checks
the initializer under the constant's actual owner. A fresh import-only
client emits `return u8 4`, and a local override emits `return u8 21`.
The inherited default reaches evaluation and fails with `selected constant
has no initializer`. The division control exposed a synthesized-constructor
check ordered after function-header lookup. The fix and expanded consumer
regression pass all three focused checks. `c3311cc45` executes imported
constant originals through the checked interpreter. Fresh clients emit
`8` for the inherited default, `9` for the provider's override and `21`
for a local override. `89fd654d0` publishes and re-proves const `Sub` under
its actual language role. A fresh client returns `2` from the transported
`f64` subtraction and both prior controls pass. The frozen reduced Core
provider now reaches original-value encoding and requires the open
supertrait constant row. The first row candidate passes four focused
integration checks, formatting, architecture and strict Clippy. The frozen
reduced Core provider now publishes 12,929 bytes of metadata. Its fresh
native client matches pinned Rust on `10 22 27 91`. `75e9fa917`
re-proves each reconstructed open trait read. The focused consumer
accepts the actual bound and rejects a packet whose same-typed operand
requires an unrelated trait, even when the source function is unused.
The seven trait-constant commits are on local `main` through
`d4f8759f6`. The compiler, tests and scripts match the qualified
frontier checkout byte for byte. The final proof-aware compiler
replays the frozen reduced Core metadata, emits byte-identical client
LowIR and matches pinned Rust's native output.
The authenticated original Core unit timed out after 180 seconds on
the unoptimized dev-fast binary, before publishing any output. The
optimized build expired at the five-minute limit before producing a
replacement executable. The unoptimized 215-second replay also
expired before publication.
Stage 1 and Stage 2 are pending.
[Strict Core evidence](docs/self-hosting-plan.md#L979)

Frontier cleanup retired seven superseded executables with 0.86 GiB of
unshared blocks. The performance worktree retired eight with 1.36 GiB of
accounted blocks. Both retained the current compiler pins, warm dev-fast
caches and cleanup receipts. Free space later recovered above the
150 GiB reserve. The PA45 comparator checks each executable against
the native-link receipt for its exact strict target build. Fourteen
focused comparator and linker tests pass, including stale and missing
link records and a different linker pin.

The performance worktree is
`/mnt/vizier/rustgm-scratch/rustgm-pa29-work` on
`lane/lifetime-inspection-tables-20261120` at `e781afa746`.
The singleton candidate remains pinned at `8fbf96d4a`. The first
local-annotation census identifies 631,869 inert visits out of 970,361
active visits (65.12%). The frozen graph and 12 focused controls preserve
output bytes, and the focused outcomes agree with pinned Rust. The
producer shadow matches the original traversal order and predicts all
31 tested traversal failure sites. References written during the pass
create 7,547 events at later positions. All 64 events missed by the
publication hooks are Core pattern endpoints. The compact representation
would still use at least 9.54 MB of payload for this pass. The flat `Kind`
index shadow matches 1,468,549 lookups and uses 3 MiB less logical index
capacity. The worker is measuring a separate change that removes singleton
lifetime argument allocations. Exact parity passed on the frozen 746-input
graph, 21 focused outcomes, four fresh staged outcomes and 12 real budget
controls. DHAT counts 2,060 fewer blocks at the target allocation site
on the 256-call singleton control, and that control retires 0.733% fewer
instructions. The zero and three-lifetime controls preserve allocation
counts. The paired whole-graph instruction difference falls inside A/A
noise, and the two elapsed pairs disagree. The candidate stays isolated.
The subsequent generic-call census finds 55,367 repeated small-map
preparations and zero reusable completed substitutions on the frozen
graph. The worker deferred that change. The lifetime-inspection copy
elimination is on local `main` through `52c394788`. Four focused tests
and eight independent output and diagnostic comparisons passed. The
full graph reached its 235-second development-binary timeout before
emitting LowIR. Whole-graph candidate parity and timing are pending.

Lifetime candidates `72264239b` and `4757cac6d` stay isolated. The first
retires 0.253% more original-graph instructions. The second retires 0.102%
fewer instructions and takes longer in both elapsed-time pairs. Bounded
child-proof reuse `9143861f0` and its early source guard `049db339d` are
on `main`. The separate trait-header experiments are parked.
[Performance evidence](docs/self-hosting-plan.md#local-annotation-traversal)

## Historical bring-up checkpoint — 2026-11-19

The compiler checkpoint on `main` is
`314833ada04e6a859ae5a26598d2a35c0ab2af48`. The original Rust 1.98.0
`core + alloc` graph emits LowIR, a bring-up metadata stamp and a native
link manifest with **zero skip rows**. ThinBox's inline constant and Vec's
`from_iter` inline constant were the final two skips. The original Linux
`std` graph also uses an empty skip list. It stops in `quota_v2` during
lifetime checking, before emitting LowIR. Stage 1 is pending.
[Self-hosting plan](docs/self-hosting-plan.md) · [PA45 contract](pa45/README.md)

## Verified progress and remaining milestones

| Milestone | Evidence and remaining work |
| --- | --- |
| M1 · original `core` | Atomics, slices, integers and intrinsic operations matched the native oracle in all three runs on zero-skip checkpoints. All 18 consumers passed on an older 12-skip profile. Complete the current zero-skip retest for the remaining cases. |
| M2 · original `alloc` | The graph emitted 667,928 bytes of LowIR, a 55-byte bring-up stamp and a 52-byte link manifest. All three are byte-identical across the three latest checkpoints. Vec and String matched their native oracles in three runs each with zero skips. Box-to-dyn reaches a missing concrete callable or checked template in `Box::call_once`. B-tree `descend` retains an unmapped lifetime. Rc and Arc fail call syntax through `Deref` to `dyn Fn`. |
| M3 · original `std` | The pinned Linux guest graph reaches `std:sys/thread/unix.rs`, `quota_v2`. Build the abort-profile rlib and run the native println, fs, env and HashMap consumer after the source graph emits. |
| M4 · RustGM dependencies | Build the locked `unicode-ident`, `tinyvec` and `unicode-normalization` sources. |
| M5 · stage 1 | Use the pinned rustc-built stage 0 and CPPGM to build the unmodified RustGM CLI and library. Run every committed PA1–PA24 text oracle against stage 1 through `RUSTGM_TEST_APP`. |
| M6 · stage 2 | Rebuild the same graph using stage 1. Compare each crate's LowIR and metadata and the linked compiler byte for byte against stage 1. |
| M7 · strict build | Build both stages in strict mode with zero bring-up records and zero trapping stubs across the libraries, dependencies and compiler. |

The successful `core + alloc` replay still records 54 unproven obligations
and 20 waivers. The failing `std` replay records 79 unproven obligations and
8 waivers before it stops. The observed `core + alloc` LowIR SHA-256 is
`a9864c51434c40abd255ae27a8af8729b50cfc84fec07d78422af1c729907910`.

Three integrated checkpoints moved the original `std` frontier from
`rustc-demangle` into `std`. `127a1b6e4` preserves the parent place's lifetime
aliases in closure capture storage. `05e86ea45` excludes a closure's private
inherited slots from returned opaque captures while preserving the proof of
its stored borrows. `314833ada` collects deferred nested byte-pattern work in
one shared batch and clears the `quota` failure. All three passed focused
gates. A separate opaque-lifetime control still needs implicit shortening
from `'b: 'a` when the opaque result captures only `'a`. The explicit typed
shortening works.

## Current compiler blocker

The unchanged original `quota_v2(group_path: PathBuf) -> usize` declares no
lifetime parameters. Its declaration begins at byte 32345. The failure is
`invalid lifetime constraint identity` at byte 32387, the start of its body.
A guarded debugger trace shows the error arises while applying an ordinary
equality in the original body's borrow-region proof.

The diagnostic replay traced that equality to
`sema::bodies::field_storage::Batch::publish`. A projected field presented to
storage normalization already carries a bound lifetime at depth 0, slot 0.
Normalization opens it as fresh region 84 and publishes the equality between
the two in the body's root scope. The root binder has width zero. The two
other observed applications of the same projection carry existing body
regions 45 and 46. Their equalities preserve those identities.

The failing storage root is `TypeId(360402)` with nominal owner `ItemId(52206)`
in this replay. Its raw field is `TypeId(36676)`, a projection with member
`ItemId(8765)` and the single lifetime argument `RegionId(2147483648)`.
Those IDs are local to this input graph. Trace that root back through
`storage_roots` and field instantiation to the source occurrence. Determine
whether the caller passed a provisional signature type as storage or the
instantiation lost the caller's lifetime. Repair the producer and preserve the
binder check. A preliminary pattern-family reducer reaches a different
outlives diagnostic. An independently qualified reduction for the exact
failure remains open.
[Storage roots and proof publication](dev/src/sema/bodies/field_storage.rs) ·
[Field dependency traversal](dev/src/sema/stored_fields/dependencies.rs) ·
[Region proof](dev/src/borrow/regions/check.rs)

The diagnostic patches were restored after the replays. Their diffs, raw
stderr and commands are preserved under `obj/`. Use an independent pinned
Rustc positive and a genuine lifetime-escape control for the exact repair.
Run focused RustGM checks, then replay original `core + alloc` and Linux
`std` with the same empty skip list. Continue from the next concrete failure
or from the emitted artifacts.

## Performance audit

Both workers completed their source audits against `707b5b9d`. The frontend
worker also traced the historical macro budget symptom. No optimization
patches or timing runs came from these audits.

| Area | Finding and first measurement |
| --- | --- |
| Macro matcher scratch | Three generation-tagged vectors shrink and regrow between arms. A wide arm that fails on its first token can clear its unused states on every short invocation, outside the current matcher step count. Retain the initialized length and vary wide-arm size and invocation count independently. [Matcher](dev/src/expand/macros/matcher.rs#L194) |
| Textual macro names | Distinct cold names rescan the same lexical scope links and retain each visited lookup pair. Measure visits and retained rows as both scope depth and distinct names grow. Preserve shadowing, hygiene and pending expansion output. [Textual lookup](dev/src/expand/textual.rs#L158) |
| Derive literal grants | Each qualifying literal revalidates the derive receipt's full generated origin interval. Count interval visits per receipt and retain the exact per-use token and feature checks when reusing a verified receipt. [Literal grants](dev/src/resolve/expansion/derive_lists/literal_grants.rs#L132) · [Origin validation](dev/src/resolve/expansion/derive_lists/vocabulary.rs#L31) |
| Trait coherence | Open impls sharing an outer constructor queue all pairs even when distinct nested rigid constructors rule out overlap. A historical core census recorded 20,986 rigid rejections out of 22,895 pairs. Count candidates before instantiation and retain wildcard cases when testing a nested index. [Pair registration](dev/src/sema/traits/implementations.rs#L484) · [Core census](docs/core-census-plan.md#L733) |
| Borrow dataflow | Dense live-value, cleanup and loan bitsets scale with tracked entities times CFG blocks. Sequential short-lived borrows can keep the active set sparse while both dimensions grow. Measure touched words and component resets with sparse and dense controls. [Value liveness](dev/src/borrow/liveness.rs#L347) · [Loan propagation](dev/src/borrow/loans.rs#L504) |
| Body outlives proofs | The cache stores exact source and target pairs. Successive targets along one chain can repeat the same traversal. Use the declaration verification scaling case as a model and preserve first-failure order. [Region graph](dev/src/types/region_graph.rs#L338) · [Scaling control](dev/tests/invariants/input_contracts/answers.rs#L67) |

The audit also identified repeated formatting provenance and header lookups,
logarithmic charging for a single immediate expansion-parent read, nested
configuration subtree rescans and repeated file-module ancestor scans.
The retained audit note records the source sites and proposed work counters.

On `05e86ea45`, the old explicit macro work cap of 14,500,624 produced
23 macro-ledger diagnostics. A second replay with the same compiler,
inputs, empty skip list and storage caps cleared all 23 by using the existing
finite source-proportional work allowance. The 1 GiB attempted storage cap
and 64 MiB retained storage cap stayed in force. That replay reached the
earlier `quota` body blocker. Keep the source-proportional allowance for the
current frontier. Performance changes need bounded-work controls, immutable
binaries, frozen inputs, A/A calibration and paired ABBA runs with output
equality. [Repository validation rules](AGENTS.md)

## Evidence and resume procedure

Run builds and probes in
`/mnt/vizier/rustgm-scratch/rustgm-pa26-work` on
`lane/frontier-20261115`. Use `/home/vishvananda/rustgm` for integration.
Evidence paths in this table are under the frontier worktree's
`obj/parallel/` unless a path starts with `obj/qualification/`.

| Path | Contents |
| --- | --- |
| `resume-20261118-patterns/compiler/pin.json` | Immutable optimized observation compiler for `314833ada`, SHA-256 `4026873ead8ef8a079fd38d4beced870b26c48d6bb6e485cd316faae3ce91ab9`. |
| `resume-20261118-patterns/{alloc-empty,std-alloc-empty}/receipt.json` | Successful original core plus alloc artifacts and failing original Linux std replay. The same directories hold the exact commands, environment, skip files and artifact digests. |
| `resume-20261119-quota-projected-field/observation/{receipt.json,stderr,inputs.json}` | Latest storage trace and exact local type identities. The diagnostic patch is `region-identity.patch` in its parent directory. |
| `resume-20261119-quota-producer-site/observation/stderr` | Producer backtrace through field-storage proof publication. |
| `resume-20261118-quota-lifetimes/identity-stack/{stdout,stderr}` | Guarded debugger stack at the invalid region lookup. |
| `resume-20261118-captures/performance-audit-707b5b9d.txt` | Both performance audits and the macro budget follow-up. |
| `resume-20261118-opaque/{std-alloc-empty,work-budget/replay}/receipt.json` | Macro work-cap comparison on the same binary and graph. |
| `resume-20261118{,-saturation,-cstr-cast}/` | Original library receipts, reductions and native per-case evidence, including the last two removed skips and remaining alloc consumers. |
| `obj/qualification/fast/{1795040551872768887,1795041966611535046,1795042991123269179}/receipt.json` | Focused passing gates for capture storage, opaque captures and nested deferred patterns. |

Replay from the saved commands and environment in a fresh directory. The
original sources are pinned to Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. The saved `std` command records
the frozen guest roots and their editions. `--bringup-guest NAME[@EDITION]=ROOT.rs`
is repeatable in dependency order in a `--source-core` bring-up session.
Guests have their own crate roots, visibility and extern preludes. The
`NAME:PATH` keys identify their source files in skip lists and diagnostics.
For native bring-up consumers, `scripts/bringup_linked.py --guest` adds the
consumer after its dependency guests. The stamp emitted by this profile
carries bring-up evidence. The self-hosting milestones still require usable
crate transport and strict artifacts.

Use the existing three worktrees and shared controller. Check free space
before each build, validation batch or probe. Preserve the 150 GiB disk
reserve and 24 GiB frontier output cap. Wait for the resource wrapper's
terminal result before editing sources or starting another heavy command.
Use `make fastcheck` with exact selectors for development. The optimized
`observe` binary supports whole-graph observations. Use the owning handouts'
qualification procedure for release and performance evidence.
[Parallel work and resource rules](docs/parallel-work.md) ·
[Tests and references](TESTING_AND_REFERENCES.md)

The previous handoff with historical commands and earlier checkpoints is
available through `git show 314833ada:handoff.md`.
