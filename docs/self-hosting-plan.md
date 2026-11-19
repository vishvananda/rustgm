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

Until metadata publication is complete, the crates above core join core's
session as guest crates (`--bringup-guest NAME[@EDITION]=ROOT.rs`, bring-up profile
only): each is resolved as a crate of its own within one compilation, so
alloc, std and third-party sources need no textual mounting. `handoff.md`
describes the option and its limits.
An omitted guest edition inherits the host's; an explicit edition follows the
guest's source, modules, includes and macro definition tokens.

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

## Execution after the November 19 handoff

The compiler at `314833ada` emits original `core + alloc` LowIR, a bring-up
stamp and a native-link manifest with an empty skip list. That run still has 54
unproven obligations and 20 waivers. At that checkpoint the original Linux
`std` replay failed in `quota_v2`. The handoff records the saved commands,
artifacts and traces. [November 19 handoff](../handoff.md)

Subsequent fixes cleared `quota_v2`, the exclusive hashbrown reborrow and the
closure from an associated initializer. Vtable preparation now retains locally
bound callback lifetimes. Two real-core `Debug` programs passed native execution
with empty skip lists, and the regular and native object-dispatch selections
passed. Original `std` moves past the callback failure to a generated `Debug`
implementation in libc that borrows an unaligned packed field. The packed derive
repair copies fields before borrowing them and adds the required `Copy` bounds.
It passes the native formatting observer and preserves the rejection of a
written packed-field borrow. The next original `std` replay reports a C aggregate
ABI error at `core/src/future/mod.rs`. The rejected signature takes a thin raw
pointer and returns a wide raw pointer in the C convention. The corresponding
source declaration and definition are `__rust_panic_cleanup` in `std` and
`panic_abort`. Pinned rustc passes the data and metadata as two x86-64 SysV integer
words. The classifier now uses that layout for direct wide raw pointers. Native
round trips through an independent C provider verify the vtable pointer, slice
address and length on both compilers. The latest original graph clears that
boundary and stops in `core::io::error::Repr::fmt` at `core/src/io/error.rs:495`.
It still records 87 unproven obligations and 10 waivers with an empty skip list.
The actual `core::io::Error` formatting observer passes through native
execution after the callable and projection lifetime repairs. The original
Linux `std` graph with those repairs and the ABI fix clears `Repr::fmt`. It
now reports incompatible LowIR symbol declarations while lowering
`panic_abort::__rust_start_panic`, with 58 unproven obligations, 14 waivers and
an empty skip list. A symbol trace identifies `__rust_abort` as the collision.
`std` defines a Rust `fn() -> ()` that terminates the process, and `panic_abort`
imports the same version-mangled symbol as `fn() -> !`. The checked lowering
resolves this exact import to its compatible `std` export for the native symbol
and preserves the caller's typed divergence. A focused control checks the
single definition, unreachable continuation, standalone import and rejection
of an incompatible export. The original shared graph clears the collision and
reaches `Option::and_then` at `core/src/option.rs:1542`. Lowering reports
`generic aggregate requires instantiated fields` at the callback invocation.
The trace locates the missing `Option<&bool>` fields in the selected closure's
native signature. Three ordinary closure and generic-enum reductions pass on
both compilers. The original inputs and empty skip list produce 49 unproven
obligations, 43 waivers and no completed LowIR on that replay. The selected
original closure's public result uses its own eight-region binder, with a
different canonical type from the opened body result. Native preparation now
reuses the verified callable signature's field producer. Focused closure
lowering and lifetime checks pass. The original shared graph clears
`Option::and_then` and next reports `tracked function pointers require a genuine
caller-location reification shim` during lowering attributed to `get_context`
in `core/src/future/mod.rs`. The emission trace identifies a different function
item. The frozen expanded graph resolves `DefId(28340)` to `begin_panic`.
The original `std/src/panicking.rs:44` initializes the used `EMPTY_PANIC`
static with `begin_panic::<&'static str> as fn(&'static str) -> !`.
On pinned Rust 1.98.0, a direct call to a tracked function observes the call
site. Both a static function pointer and an equivalent local cast observe
the function declaration site, at optimization levels 0 and 2. The graph
still has an empty skip list, 47 unproven obligations and 50 waivers.
The trace and independent observer are under `obj/parallel/next-library-probes/`
in the performance checkout.

The separate strict core export fails before metadata publication in
`FlattenCompat::next` in `core/src/iter/adapters/flatten.rs:527`. Small independent
programs reproduce the lifetime failure when an inferred trait method item goes
to a higher-ranked `FnOnce` parameter. A closure, a free function item and the
fully qualified trait method pass. Proof extraction retained an input-refinement
equation while dropping its witness's scoped allocation. The checked fix retains
that allocation in the callable binder. The exact native projection suite passes.
The original strict replay now passes that site and stops at `flatten.rs:569`,
where `FlattenCompat::try_fold` passes a returned opaque `FnMut` to a
higher-ranked input. The small adapter reducer matches this failure on RustGM
and compiles on pinned rustc. The second fix retains the witness allocated when
expanding the opaque callable's declared inputs. Seven positive reducers pass
and the static-only callback still fails on both compilers. The next strict
core replay reaches `SyncView::async_call_mut` in `core/src/sync/sync_view.rs:271`
and fails to prove the method's returned associated type outlives its target.
A generic forwarding method reproduces the failure, while the equivalent UFCS
call passes. The captured arguments identify the same associated type and
distinct lifetime slots, one owned by receiver inference and the other by the
caller's signature. The body-coercion fix uses the existing projection input
matcher to solve that invariant relation in the caller's lifetime graph. The
forwarding and unrelated-lifetime controls agree with pinned rustc. The concrete
forwarding observer also mutates through the returned reference and passes native
execution on both compilers. Strict `core` now reaches `Formatter::wrap_buf` at
`core/src/fmt/mod.rs:1768`, where the wrapper passes `self.buf` through a callable
that shortens the lifetime of its mutable trait-object reference. The reduction
shows the borrow proof requiring both directions between the object bounds.
The checked relation now permits shortening the immediate object bound on a
selected reborrow. The original reference ancestry, nested mutable reference
invariance and invariant trait inputs still participate in the proof. Positive
and negative controls agree with pinned rustc. The native control mutates the
original value from 7 to 9 on both compilers. The successor strict replay
clears `Formatter::wrap_buf` and reaches `ControlFlow::break_value` at
`core/src/ops/control_flow.rs:194`. The diagnostic is `active cleanup requires
const Destruct in its actual defining environment`. The typed trace requests
whole-field destruction for the discarded `Continue` payload. The defining
environment carries a conditional `Destruct` premise on `Self`. The local
enum reduction with the same premise reproduces the failure. The cleanup
selector now keeps the exact enclosing type of an initialized field. It uses
an explicit aggregate `Destruct` premise on that place for the field and
preserves the obligation for a separate argument of the same field type.
Three focused source outcomes agree with pinned rustc. Strict `core` clears
`break_value` and advances to the or-pattern in `ControlFlow::into_value`
at `core/src/ops/control_flow.rs:417`. It requests `Destruct` even though
both alternatives move the payload. The reduced enum compiles with pinned
rustc and fails with the same RustGM error. The original ownership trace
marks two active physical whole-payload cleanups on the enum. The companion
controls that leave a non-const payload or a separate argument alive fail
on both compilers. The candidate repair recovers the selected owned fields
from distinct variant tests during unguarded capture. The reduced enum moves
past the cleanup check to source transport validation, and both live-value
opponents still fail the cleanup check. Three focused tests pass in the
performance checkout. The original replay of this candidate is pending.
The last strict replay has zero bring-up records and no metadata. All four
frozen Unicode consumers match their
declared output on pinned rustc with prebuilt std at opt-level 2. Their RustGM
runs remain due.

### Self-compile milestones

| Order | Work | Exit evidence |
| --- | --- | --- |
| 1. Unblock `std` | Trace the projected field's bound lifetime back from `Batch::publish` through `storage_roots` and field instantiation. Identify the source expression and distinguish a provisional signature root from a lost caller substitution. Put the fix at the producer. | A pinned-rustc positive that exercises the same producer, a genuine lifetime-escape rejection and a focused RustGM gate. Use the original `quota_v2` replay as the exact regression until a reduction reproduces its failure. Both original `core + alloc` and Linux `std` replay with their saved inputs, environment, empty skip list and limits. The invalid binder remains a hard error. |
| 2. Close M1–M3 | Complete the zero-skip retest of the 18 core consumers, carrying forward verified results where the inputs and affected paths permit. Repair the `Box::call_once` concrete callable or template issue, the B-tree `descend` lifetime mapping and Rc/Arc call syntax through `Deref` to `dyn Fn`. Continue original `std` from each concrete blocker through LowIR, static library artifacts for the abort profile, and native linking. | Complete core and alloc consumer receipts, including Vec and String after affected changes. The `std` println, fs, env and HashMap consumer matches its declared native outcome, followed by the remaining declared std cases. Record emitted artifacts and any remaining bring-up obligations or waivers. |
| 3. Close M4 and exercise crate transport | Vendor the locked `unicode-ident`, `tinyvec_macros`, `tinyvec` and `unicode-normalization` sources. Take editions, features and dependency edges from the frozen unit plan. Bring up the dependency graph and execute the Unicode consumers. In parallel, advance real per-crate export/import from `core` through `alloc`, the selected `std` graph and the compiler dependencies. | The four Unicode consumers match the declared outcomes. A fresh dependent reads authenticated metadata and links RustGM-generated objects. Cover the demanded generic and const bodies, resolved identities, proof inputs and native links with cross-crate positive and rejection controls. Guests can expose later source failures while this transport is being built. |
| 4. Build stage 1 (M5) | Freeze the stage procedure, including the locked graph, unmodified `dev/src/lib.rs` and `dev/bin/rustgm.rs` and their reachable sources. Use the pinned rustc-built stage 0 to compile target Rust, then use the pinned CPPGM object path to link the compiler. Start with a CLI invocation that exercises a real input and drive remaining failures from the compiler's demand list. | The resulting stage 1 executable, selected through `RUSTGM_TEST_APP`, passes every committed PA1–PA24 text oracle. Record the executable hash, expected and executed case counts, and exact receipts. |
| 5. Rebuild and qualify (M6–M7) | Have stage 1 rebuild the same frozen graph. Work through strict-mode blockers by cause, starting as soon as each crate can be replayed. Account for the existing core/alloc and std obligations and waivers individually. Rerun both stages strictly from clean directories for qualification. | Compare every crate's LowIR and metadata and the linked compiler byte for byte. Require zero bring-up records and zero trapping stubs across both strict builds. Exercise the perturbed-input comparison failure and the one-record strict-gate failure required by PA45. |

For the first checkpoint, preserve body-region identity through storage
normalization and exercise the two existing body-region cases from the trace.
Include `field_outlives::preparation_preserves_the_occurrence_binder_and_shares_large_graphs`
and `field_outlives::recursive_requirements_remap_complete_projected_inputs` in
the focused selection alongside the new exact control. Track implicit opaque
lifetime shortening from `'b: 'a` as its own regression. After the original
`std` replay, record the next failing phase or the emitted artifacts before
selecting the next batch. [Field storage](../dev/src/sema/bodies/field_storage.rs) · [Field dependency traversal](../dev/src/sema/stored_fields/dependencies.rs) · [Region proof](../dev/src/borrow/regions/check.rs)

Use the saved guest command and the finite source-proportional work allowance
with the 1 GiB attempted and 64 MiB retained caps for frontier replays. For
empty-skip native evidence, use one frozen compile per consumer or give the
runner an explicit mode that treats any added skip as a failure. The current
`bringup_linked.py` can extend the copied skip list and still pass on exit zero
and matching stdout. It also carries the historical fixed work cap. Drive
consumer execution from the declared args, environment, stdin, status, stdout
and stderr, including expected aborts. Recheck the atomics widths, hashbrown
SIMD, allocator/start/TLS, platform symbols and the actual LowIR volume at the
native boundary as the workload reaches them. [Bring-up runner](../scripts/bringup_linked.py) · [Consumer outcomes and runtime probes](self-hosting-runtime.md)

Start the strict executor from the existing target-unit and link tooling. Freeze
an ordered per-unit inventory with source hashes, crate identities, extern aliases,
features, target cfg, generated inputs and environment. Pin stage 0, CPPGM, the
native linker, native inputs and link order. The current target executor emits
metadata for library units only, and the CLI rejects binary `--metadata-output`.
Close that gap in the existing typed output path for the PA45 per-crate
comparison. Run `scripts/course.py paN --app "$RUSTGM_TEST_APP"` for each of
PA1–PA24 under the shared guards with the variable set to the pinned stage 1
executable. Compare raw bytes from independently built directories to catch
path and ordering leaks. Diagnose differences at the first differing unit
before investigating the linked image. Capture elapsed time and peak memory
per stage and publish the frozen
procedure, nonempty manifest and stage hashes after independent review and the
PA45 release checks. [Target executor](../scripts/target_units.py) · [CLI artifact options](../dev/bin/crates.rs) · [Backend link tooling](../scripts/backend.py) · [PA45 contract](../pa45/README.md)

### Performance work from the agent audits

The audits were source reviews at `707b5b9d`. The macro-cap replay isolates the
earlier limit symptom. Establish current work counters before using any historical
counts as a baseline. The table gives the initial order within each workstream.

| Work | Implementation to test | Work counters and equivalence controls |
| --- | --- | --- |
| 1. Macro matcher scratch | Grow the three generation-tagged tables only when an arm exceeds the initialized length. Preserve the existing generation rollover reset. [Matcher](../dev/src/expand/macros/matcher.rs#L194) | Vary a first-token-failing wide arm and short successful invocation count independently. Count initialized entries and matcher steps. Initialization follows maximum width, with a separate rollover control and byte-equal expansions and ambiguity diagnostics. |
| 2. Derive literal grants | Verify the immutable origin interval once per receipt within the allowance session. Retain per-use token, member, role and feature checks. [Literal grants](../dev/src/resolve/expansion/derive_lists/literal_grants.rs#L132) · [Origin validation](../dev/src/resolve/expansion/derive_lists/vocabulary.rs#L31) | Vary interval width and qualifying requests independently. Count visits per receipt and per-use checks. Exercise invalid receipts and wrong-token/role/feature requests after a warm hit. |
| 3. Expansion frame charge | Validate the immediate parent by exact indexed record and charge for the actual read. [Permission frames](../dev/src/resolve/expansion/permission_frames.rs#L128) | Vary frame ordinal and nesting separately. Count reads and ledger work, and keep wrong-parent, inactive-source and missing-frame diagnostics. |
| 4. Formatting provenance | Reuse the completed provenance and header evidence when semantic readiness resumes at the same format site. Keep the checked constructor proof tied to the actual request. [Resolve](../dev/src/resolve/expansion/format_values.rs#L100) · [Semantic body checking](../dev/src/sema/bodies/formatting_values.rs#L31) | Count origin/header lookups by distinct site and readiness resumes. Compare formatting output and constructor selection, and retain the malformed-origin and wrong-constructor controls. |
| 5. Trait coherence | Filter incompatible rigid nested constructors before queueing open impl pairs. Keep candidates that can still unify, including wildcard, projection and blanket cases, in the existing diagnostic order. [Pair registration](../dev/src/sema/traits/implementations.rs#L484) · [Nested-head control](../dev/tests/invariants/rejected_overlap_pairs.rs#L154) | Count candidates inspected, retained pairs and instantiations as header count and nesting width grow. Replay original core and compare true overlaps and specialization diagnostics. The historical 20,986 rigid rejections out of 22,895 pairs give a starting hypothesis. |
| 6. Textual macro lookup | Index names and hygiene contexts across immutable lexical segments. Keep pending output barriers and their publication/wake behavior. [Textual lookup](../dev/src/expand/textual.rs#L152) | Vary scope depth and distinct names independently and together. Count link visits and retained lookup rows. Exercise warm and cold queries, nearest shadowing, hygienic lookup, completed output and sibling scope isolation. |
| 7. Body outlives proofs | Reuse bounded, lazy reachability per source within one immutable body proof. Preserve query order and the first failure. Schedule after the `quota_v2` producer fix so the two changes have independent evidence. [Region graph](../dev/src/types/region_graph.rs#L338) · [Declaration scaling control](../dev/tests/invariants/input_contracts/answers.rs#L67) | Scale one source against many targets in a chain, then vary sources and include a single-query control. Count traversed nodes/edges and retained bytes. Include cycles, unreachable targets, universe boundaries and the lifetime-escape rejection. |
| 8. Borrow dataflow | Prototype sparse or touched-word state for value liveness, cleanup and loans with measured dense behavior. Scope each change to its own pass and preserve CFG joins and ordered cleanup. [Value liveness](../dev/src/borrow/liveness.rs#L347) · [Cleanup](../dev/src/borrow/liveness/cleanup.rs#L245) · [Loans](../dev/src/borrow/loans.rs#L504) | Vary tracked values or loans and CFG blocks separately. Count touched words, component resets and peak state bytes for sequential short-lived borrows, then check dense joins, loops, loan conflicts and native drop order. |
| 9. Configuration walks | Reuse precise subtree starts for attributed owners, with demand-local or parser-owned range facts. [Configured syntax](../dev/src/expand/crate_syntax.rs#L75) | Vary nested owner depth and innermost list width. Count visited nodes and retained ranges. Preserve enabled/disabled subtree behavior and deferred literal diagnostic order. |
| 10. File-module ancestry | Reuse canonical file-identity membership along each inherited module path, including deferred resumes. [Module loader](../dev/src/resolve/load.rs#L470) | Count membership work and retained state as same-directory nesting grows. Exercise alias cycles, deferred file modules and legal file reuse by siblings. |

The first fixed-CPU profile of the original core, `unicode-ident` and native
consumer graph put 45.07 of 77.79 measured CPU seconds in body inference and
12.66 in declarations. Parsing and expansion took 5.00 seconds. The profiled
opt3 compiler at `c7e4d7378` emitted the same LowIR, native-link manifest and
native executable as the timed run. On three smaller, identical-source native
controls, current RustGM took 4–7% longer than the rebuilt October 17 compiler
at `8d2e03b5f` and remained 1.81–2.01× faster than the pinned rustc. The
historical timing programs are unavailable for an exact replay, and the
standalone controls time a different scope from compiling the real core graph.
[Historical timing record](pa18-pa21-buildout.md#L8877)

Hardware call-stack samples attribute 57.8% of user cycles to the body inference
phase. Trait-selection frames appear in 46.0% of that phase's samples, inference
frames in 28.9%, and substitution or interning frames in 15.6%. These shares
overlap. Macro matching appears in about 0.23% of full-graph samples. Preparing
demanded instances accounts for another 10.5% of cycles. Dense initialization of
destruction state accounts for 15.7% of the cycles within that phase. Trait
coherence appears in 0.77% of declaration samples. Its separate scaling control
still queues 15, 276 and 4,560 pairs for 6, 24 and 96 open impls whose nested
heads are disjoint.

The open impl lookup control uses one receiver type and distinct rigid trait
arguments. Raising both the impl and query count from 8 to 16, 32 and 64
raised observed body trials from 64 to 256, 1,024 and 4,096. The instance
stage repeats that work. Each query has one matching header. RustGM and rustc
executed the same declared native outcomes.

The first index keys open impls by the nominal constructor of their first
non-lifetime trait argument for fully resolved goals. The same four inputs now
take 8, 16, 32 and 64 trials in each stage. Holding eight goals fixed also
holds selection to eight trials as the registry grows from 8 to 32 impls.
All nine replays produced byte-identical LowIR and native executables.
These are deterministic work counts from the development build. Optimized
elapsed-time measurements and indexing deeper arguments remain open.
[Open argument buckets](../dev/src/sema/traits/implementations/open_arguments.rs)

Prioritize work counters in body inference and declaration semantics on the
real core graph. Start from existing `BodyWork` counters for trait selections,
canonical inference, projection selection and suspensions. Track candidate
visits, substitutions and readiness retries independently so the largest source
of repeated semantic work is visible. Measure coherence pairs on original core.

### Trait query representation and reuse

The type store interns types and their child lists. Its structural `ShapeKey`
folder already shares canonical transformations between independent queries.
Candidate trials reuse these type identities, while each trial imports the live
inference terms it needs.
Completed proof reuse already covers concrete, debt-free answers within one
fixed caller. Crate-wide body reuse covers verified concrete implementation
answers in neutral callers. Closed dispatch also has an independently certified
cache for eligible selections. Measure how often each reuse path applies, hits
or declines on the frozen real-core graph. Count requests and distinct identities
by phase, including repeated subgoals and goals that appear in both body and
closed instance selection. Trial setup appeared in 12.99% of body-phase cycle
samples. The profile does not quantify how much of that work can be shared.
[Type store](../dev/src/types/mod.rs#L260) · [Caller reuse](../dev/src/sema/traits/selection/search/completed_proofs.rs#L6) · [Crate-wide reuse](../dev/src/sema/traits/selection/memo.rs#L1) · [Closed dispatch reuse](../dev/src/sema/dispatch.rs#L535)

On the frozen core and Unicode identifier graph, the typed body facts retain
169,870 implementation-selection rows. The 162,229 rows with concrete
arguments name 3,337 distinct `(impl owner, arguments)` identities, of which
2,935 appear at multiple source sites and 730 appear in multiple files. These
counts quantify repeated targets. To estimate reusable solver work, measure
the distinct requesting goals and effective premises that can produce each
target, along with the existing memo hit rates.

An instrumented run of the original strict core sources and the original
core/alloc/std bring-up graph counted the work through their current failure
frontiers. It counted distinct lookup fingerprints, so the cardinalities below
are estimates. Instrumented builds supply work counts only.

| Quantity | Strict core | Shared core/alloc/std |
| --- | ---: | ---: |
| Validated candidate lookups | 768,399 | 1,167,551 |
| Distinct validated candidate lookup keys | 13,878 | 28,734 |
| All ready candidate lists | 932,339 | 1,366,368 |
| Ready lists with zero or one owner | 782,065 | 1,189,176 |
| Ready lists with 256 or more owners | 7,560 | 8,498 |
| Implementation trials | 742,205 | 957,492 |
| Trials with no input graph edges | 718,871 | 931,059 |
| Charged work importing trial inputs | 2,484,625 | 3,097,114 |
| Additional charged work instantiating headers and conditions | 8,444,434 | 12,026,977 |
| Existing crate-wide memo hits | 70,855 | 79,779 |
| Queries whose memo key was in the rejected state | 123,067 | 142,850 |
| Memo lookup requests from effect-qualified goals | 27,102 | 27,779 |

The validated candidate key uses registry extent, selection availability and
purpose, trait identity, the receiver head, structural shape, and the
discriminating argument. Prototype sharing ordered lists for repeated lookups
with multiple owners. Lists of at least 256 owners account for under 1% of
the lists and at least 49% of emitted owner entries in each run. Include the
empty and single-owner common cases as overhead controls. The next
instantiation probe should compare distinct
`(impl owner, effect)` recipes and count repeated lowering of declared
conditions, parameter shapes and capture-independent substitutions. The current
data prioritizes those immutable header facts over a general inference overlay.
The body memo keys on the trait, requested const effect and input type IDs.
A direct probe populated the memo from a completed ordinary `Clone` impl.
With the earlier key, conditional and const requests could retrieve the warm
ordinary answer. The corrected key isolates those requests and retains the
ordinary hit. The dedicated private regression exceeded the development
harness budget. A cause breakdown of rejected entries remains open.

The candidate-list prototype keeps four owner IDs inline and shares wider
ordered lists in validated ordinary selection. It checks pending dependencies
before cache lookup and clears the cache when a header is published or
validated. The cache keeps at most 256 entries and admits lists with 16 to 512
owner slots. Projection and negative-coherence checks pass. Six frozen inputs
with 8, 64 and 128 open impls emit byte-identical LowIR. The 384-header method
control exhausts the baseline's method-selection budget. Eight independent
trait-obligation inputs with the same registry sizes and a 384-impl group also
emit byte-identical LowIR. For 64 uses of a single lookup key, the body
inference counter falls by 3,969, 8,001 and 24,129 steps at widths 64, 128 and
384 respectively. Each of the 63 warm lookups saves the list width minus one
step. All distinct-key and eight-impl controls match the old step count.

Immutable nonincremental opt-level 1 binaries emitted byte-identical LowIR in
each observation across the four inputs below. Each wall measurement used
eight A/A samples and eight alternating ABBA and BAAB blocks on CPU 12. A
separate run used six paired blocks with fully active user instruction and
cycle counters. The repeated wide control saves selection steps, with a small
effect on total retired instructions.

| Input | Paired wall change | A/A wall range / median | Paired user instruction change |
| --- | ---: | ---: | ---: |
| Ordinary generic call | −1.70% | 46.7% | +0.20% |
| 8 headers, repeated key | −1.94% | 27.9% | +0.13% |
| 384 headers, repeated key | +2.37% | 63.3% | −0.04% |
| 384 headers, distinct keys | −0.44% | 37.4% | +0.29% |

Raw wall, CPU, RSS, counter and output receipts are in the frontier checkout
under `obj/parallel/candidate-owner-sharing/measurement/{paired,paired-pmu}`.
The same immutable pair ran on the frozen original core and `unicode-ident`
graph, through LowIR. Two A/A runs took 32.01 and 30.80 seconds. The ABBA and
BAAB blocks retired 0.522% and 0.524% fewer user instructions, respectively,
with byte-identical LowIR, native-link manifests and diagnostics. Paired wall
time moved by +4.17% and −2.42%. Peak RSS stayed near 915,700 KiB. Each run
had zero skips, 46 unproven obligations and eight waivers. The raw receipt is
under `obj/parallel/candidate-owner-sharing/measurement/whole-graph/`.

The dedicated private test checks ordering,
storage identity and warm work. Its first build found two missing packed-derive
fixture initializers, which are corrected. The private harness build exceeded
the five-minute development budget on the two subsequent attempts.

Resolved goals now share an ordered list when they select the same open
buckets and their closed-shape lookups are both empty. A matching closed
bucket retains its shape identity. Unresolved goals retain their existing
conservative lookup. In the 64-query distinct controls, body inference work
falls from 197,803 to 193,834 at 64 headers, from 390,635 to 382,634 at 128
headers, and from 1,161,963 to 1,137,834 at 384 headers. The repeated and
eight-header controls retain their prior charges. All nine frozen inputs
emit byte-identical LowIR, including a mixed open and closed dispatch case
that executes successfully on both compilers. Focused projection, negative
coherence, formatting, architecture and strict Clippy checks pass. The same
immutable opt-level 1 measurements reproduce byte equality on four controls.
The extension changes retired user instructions by +0.01% on the ordinary
control, +0.18% at eight repeated headers, +0.31% at 384 repeated headers
and +0.21% at 384 distinct headers. On original core and `unicode-ident`,
the ABBA and BAAB blocks retire 0.035% and 0.037% more user instructions than
the first candidate cache. Paired wall time moves by +0.005% and +15.43%,
with A/A runs of 53.23 and 50.80 seconds. Every observation emits identical
LowIR, native-link manifests and diagnostics. Raw observations are under
`obj/parallel/candidate-owner-sharing/measurement/whole-graph-distinct/`.
[Candidate lookup](../dev/src/sema/traits/implementations.rs#L194)

Track reuse within each trial as well as across goals. An interned trait
application already records the deepest escaping binder in its arguments and
whether any argument contains a type or const parameter. When a substitution
cannot change either, it can reuse the application ID in constant work even
for distinct requests with large argument lists. Arguments that do change
still use the shared fold and keep the trial's actual region and parameter
mapping. The check uses the existing arena metadata. Fresh folders each take
two charged steps including capture preparation at widths 8, 128 and 2,048.
A bound lifetime in the same wide input still maps to its supplied capture.
Five focused trait and projection checks, formatting, architecture and strict
Clippy pass. Immutable nonincremental opt-level 1 binaries differ only in this
change. Each control uses eight A/A observations and six paired ABBA or BAAB
blocks. Every pair emits byte-identical LowIR.

| Input | Paired user instruction change | Paired wall change |
| --- | ---: | ---: |
| Ordinary generic call | −0.48% | −0.03% |
| 8 trait arguments, 32 distinct requesting goals | −1.88% | +0.02% |
| 64 trait arguments, 32 distinct requesting goals | −4.19% | −0.21% |
| 256 trait arguments, 32 distinct requesting goals | −6.61% | −1.93% |

The A/A wall ranges are 1.96% to 4.78% of the respective medians. On the frozen
original core and `unicode-ident` graph, the ABBA and BAAB blocks retire 2.254%
and 2.252% fewer user instructions. Wall time moves by −16.19% and +10.50%, with
A/A observations of 51.55 and 52.50 seconds. All ten runs emit identical LowIR,
native-link manifests and diagnostics. They record zero skips, 46 unproven
obligations and eight waivers. Peak RSS stays near 915,000 KiB. Raw observations
are under `obj/parallel/trait-capture-fold/{paired-pmu,whole-graph}/` in the
frontier checkout.
[Application facts](../dev/src/types/traits.rs) · [Argument folding](../dev/src/types/substitute/predicates.rs)

1. **Index impl headers once.** Extract rigid constructors and their positions
   once per published header. Prototype shared paths or ordered posting lists
   for lookup and coherence. Preserve wildcard, pending, projection and opaque
   candidates and deterministic order. The current lookup gathers and sorts a
   fresh owner list. In a fixed registry view, reuse immutable ordered slices
   or lazily merge a few slices when query filters differ. Candidate identities
   can be shared across distinct goals and caller environments. Cache the
   discriminating shape by canonical input ID where available so key building
   can reuse the same structure. Aim for work proportional to the queried path
   and compatible headers, independent of total registry size when wildcard
   fanout stays bounded.
   [Candidate lookup](../dev/src/sema/traits/implementations.rs#L194)
2. **Share unchanged inputs between trials.** Keep canonical subterms as IDs
   and test whether a frame can prepare its remaining immutable goal structure
   once. Let each trial own the changed type and const bindings, lifetime
   equations and rollback state. Vary the number of alternatives, the size of
   the shared input and the number of bindings each candidate changes. Count
   shared edges, trial-local nodes, imported bytes and peak retained bytes.
   The search frame resolves a canonical goal for completed-proof lookup.
   Candidate lookup resolves the inference predicate again. Count how often
   the second resolution can consume the first result and how much it revisits.
   Only share that result while the frame retains the same inputs. A projection
   rewrite or new inference equations can change the candidate set.
   Measure the same header across different requesting goals. The declared
   effect-qualified conditions, the predicate's parameter uses and the
   decisions about vacuous binders can be derived once per validated owner and
   effect. The fresh captures, local type variables and proof evidence still
   belong to the trial. Measure header preparation separately from replay into
   the trial's inference context.
   [Trial construction](../dev/src/sema/traits/selection/search/frames.rs#L166) · [Header instantiation](../dev/src/sema/traits/selection/instantiation.rs)
3. **Reuse proof structure where the inputs permit it.** Measure repeated
   goals using a canonical key with alpha-renamed slots and the identity of the
   effective premises, binder scope, mode and registry view. Start with
   completed answers and test whether recipes can retain shared subproofs and
   replay fresh bindings for caller-dependent obligations. Count recipe misses,
   hits, bytes and replayed obligations. Account for competing candidates whose
   eligibility can change with an added premise, including premises absent from
   the selected proof. Test different premises for the same
   goal, fresh caller lifetimes, pending publication, negative evidence,
   specialization, cycles and ambiguity.

For each prototype, hold source size and request count fixed while varying the
fraction of shared keys, subterms or premises. Separately grow the amount of
irrelevant registry data at fixed compatible fanout. Track cold index and key
construction, warm lookup, retained memory and the work at a changed registry
or caller boundary. A repeated batch still has one result per request. The
target is search and instantiation work that follows distinct reusable facts
and real candidates, without charging each request for all shared structure.

The fresh instance cleanup path is a separate opportunity to make work
independent of unrelated state. Of 437 sampled destruction table resizes,
427 occur during materialization and 223 pass through `with_contract`. The
existing checked-caller path shares concrete answers across a demanded
traversal and appears in one resize sample. Count total initialized slots, slots
visited per fresh materialization and distinct visited `TypeId`s across
materializations. Keep the reachable type graph fixed while adding unused
types, then hold the store fixed and vary how much two adjacent instances
share. Compare a demand-allocated table with sharing verified structural
answers over one unchanged destructor registry, preserving per-instance
work limits and accounting for arena identities, incomplete heads and failed
queries. Fresh `with_contract` queries now use sparse ordinary state in place
of allocating across the type store. They preserve the existing ordinary
Copy and opaque rules. In the storage probe, adding 65,536 unrelated types to
a reachable graph of ten raised the old initialized row count from 30 to
65,566. The sparse query keeps ten rows and 756 bytes of accounted storage
throughout. Answers and query steps match. At 8,194 reachable types, the dense
and sparse controls account for 328,312 and 589,992 bytes respectively. The
existing long-lived queries retain dense storage. Three focused cleanup tests,
format, strict Clippy and architecture completed successfully. Their overall
runner hit its deadline after recording the test results. Optimized timings
and execution of the private storage invariant remain pending.
[Fresh materialization](../dev/src/body/instantiate.rs#L179) · [Destruction query](../dev/src/types/destruction.rs#L123)

The matcher scratch change passed its focused regression and an opt-level 1
paired run with seven frozen inputs and byte-equal expansions. At 65,536 arm
positions and 16,384 invocations the median was 1,117.90 ms before and 67.79 ms
after. The ordinary control moved from 28.88 ms to 32.48 ms, with 21.1% A/A
spread and all three paired blocks slower. A fixed-CPU rerun of the ordinary
control used 12 alternating paired blocks. It measured 41.60 ms before and
40.33 ms after, with a median paired ratio of 0.977 and 16.7% A/A spread. The
ordinary case has no stable speed estimate at that noise level. Attribute ledger
charges by caller and phase, including the first sticky transition, to order the
frame and formatting changes. Land each algorithmic change independently so a
frontier advance or regression can be attributed to its cause.

The expansion-frame check now reads the recorded immediate parent by index. At
flat widths 8, 32 and 128, the public allowance observer paid the same charge
for the first and last invocation in each run. Previously the last invocation
paid 3, 5 and 7 extra units respectively. A forged grandparent failed at all
three nested depths, and the two focused source-panic permission tests passed.
[Macro ledger attribution](parallel/macro-budget.md#attribution-analysis-only)

The derive-origin memo reduces the ledger work for 64 requests for the same
formatter receipt from 38,464 to 11,818. The new implementation takes 604 units
for the first request and 178 for each subsequent request in the same allowance
session. Repeating the requests in fresh sessions takes 38,656 units. The
focused integration passed, including the independent Rust outcomes. A public
API probe confirmed that a warm memo still rejects an invalid token span and
withholds an unrelated feature. The private harness build reached the
five-minute gate before it could run.

For each change, define the work bound and cache lifetime before implementation.
Run the exact affected `make fastcheck` selections, independent source outcomes
where applicable, a scaling control and an ordinary control. At meaningful
algorithm checkpoints, use immutable measurement binaries of the same build
profile, frozen inputs, an exclusive resource lease, A/A calibration and paired
ABBA runs. Retain raw samples, peak memory, work and storage counts, and byte
equality of the affected outputs. On the failing original std graph, require the
same diagnostic and frontier for equivalent performance runs. Repeat on the
successful graph when available. Run the broad matrix at qualification or when
investigating a regression. [Performance contract](../spec.md#ownership-allocation-and-complexity) · [Validation rules](../TESTING_AND_REFERENCES.md#development-checkpoints)

Keep the frontier's source replays and repairs in `rustgm-pa26-work`. The runtime
lane owns consumer and link evidence, and the publication lane prepares the
stage receipt and release checks. Assign performance batches to a free existing
worktree. Use the shared resource and storage wrappers, the 150 GiB disk reserve
and the frontier's 24 GiB output cap. Keep `make observe` for frontier diagnosis,
and record raw evidence under `obj/`. [Resource controller](parallel-work.md#resource-controller)
