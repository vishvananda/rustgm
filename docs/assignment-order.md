# Assignment order and completion review

Adopted 2026-10-27. This replaces numeric implementation/release ordering for
unreleased PA25–PA44. PA numbers remain stable ownership identifiers: fixtures,
feature tags, historical receipts and student links keep their paths. The
machine-checked [work plan](../assignment-plan.json) supplies the actual order.
[The requirement ledger](assignment-requirements.json) retains all46 audited
PA25–PA30 contract families, including every known missing/unsupported positive.
The owning handouts remain the full assignment contracts. No requirement is
deleted, converted to an expected Rust rejection or waived by this change.

## Findings and adopted phase boundaries

The old cumulative chain introduced false dependencies. PA25's symbolic pointer
CTFE requires PA27/28 memory semantics, while runtime unsafe/atomic operations
do not require CTFE. Source operators/intrinsics need PA31 identities, but PA31
previously depended on the same operations through all prior assignments.
Core at PA32 needs PA34 editions/build support and PA41 SIMD/target/assembly.
Moving entire PAs earlier would preserve these cycles.

Split the responsibilities into explicit phase units instead:

- Local source identity, signatures and feature/header authority precede executing
  source intrinsic, callable, formatting and panic bodies. Whole PA31 stays open.
- Native unsafe places and runtime memory kernels precede symbolic constant
  storage. The typed scalar/closed-value const machine is independently useful.
- Declaration/signature metadata precedes generic bodies. Completed constant
  graphs depend on symbolic CTFE; local C calls do not depend on any metadata.
- Core's exact target/build/runtime prerequisites precede its artifact gate.
  Broader target/edition/unwind closure remains required, with its own exits.
- Original core-only crates start immediately after core; they need neither alloc
  nor std. Alloc and its consumers follow; std can proceed independently of
  completing every serde_json consumer.

Actual local Try includes the source-declaration prerequisite. Genuine
Formatter/panic consumers depend on source-hook dispatch, rather than canonical
bootstrap operators alone. Generic transport qualifies separate producer/fresh
consumer artifacts; it does not require the downstream whole-core artifact
as its own exit. These boundaries avoid recreating a cycle in prose while
keeping the JSON graph acyclic.

An edge names a required producer with the unit's documented profile. It does
not require unrelated full-language closure. The actual consumer still fails
on every active unsupported feature: a selected upstream source path cannot be
waived because its broader PA number is later. All required unused original
bodies/initializers still pay semantic checks; demand selects code generation
and fresh instance proof, not which invalid source is ignored.

## Assignment status decisions

PA21–PA24 are now **implemented**, with explicit release-evidence links in
assignments.json. This recognizes their already completed course releases:

| PA | Existing release evidence |
| --- | --- |
| 21 | [Generic relationship qualification](pa21-validation.md) |
| 22 | [Unsized release](pa22-release.md) |
| 23 | [Trait-object release](pa23-release.md) |
| 24 | [Closure release and publication](pa24-release-review.md), [performance](pa24-final-performance.md) |

These statuses describe the documented pinned course profiles, not arbitrary
Rust compatibility or a new full qualification of the current compiler. PA1–PA20
remain implemented-draft pending their individual release-record reconciliation.
The ordinary PA6/PA19 module-versus-primitive lookup regression has an actual
independently observed reducer and a separate repair lane; it is not excused as
SourceCore-only behavior. Internal/SourceCore shared numeric solver evidence
still needs an ordinary reproducer before changing its published PA21 profile
conclusion. Preserve PA13 and PA19's documented Reference/rustc differences.

**None of PA25–PA30 is complete as written.** They contain implemented finite
components and real remaining production gaps, as well as missing publication.
Const closures; full symbolic validity/storage; remaining source hooks/formatting;
atomic/intrinsic families; body/policy transport; and broader FFI representations
remain explicit requirements. Splitting or generating references cannot close
those gaps. No new active text suite or oracle is claimed by this review.

`implementation=implemented` on a work unit means its named finite profile has
implementation evidence. `publication=unpublished` means its release work is
still pending. Whole-PA implemented status requires its complete contract, all
owned unit releases, a nonempty active oracle manifest and release evidence.
The harness rejects missing/future/cyclic dependencies, orphan requirements,
partial requirements mapped only to completed units, empty component publication
and whole-PA completion over unpublished work.

## Adopted work order

The table is topological. Independent units can proceed in parallel; the
resource policy stays three guarded jobs, unchanged150GiB reserve, one heavy
job per worktree, bounded output and five-minute development checks. A component
release does not activate its entire parent assignment. Each unit's exact finite
scope, predecessor IDs and executable exit are in assignment-plan.json.

| Order | Owner | Work unit | Implementation | Publication |
| --- | --- | --- | --- | --- |
| 1 | PA29 | Crate identities and declaration metadata (`declarations`) | implemented | unpublished |
| 2 | PA27 | Unsafe permission and native places (`unsafe-places`) | implemented | unpublished |
| 3 | PA30 | Local C ABI and linkage (`c-abi`) | implemented | unpublished |
| 4 | PA31 | Local compiler declarations and attribute authority (`source-declarations`) | partial | unpublished |
| 5 | PA25 | Typed constant machine and inspection (`const-machine`) | implemented | unpublished |
| 6 | PA28 | Runtime memory and scalar volatile kernels (`runtime-memory`) | implemented | unpublished |
| 7 | PA28 | Core AtomicU32 protocol (`atomic-u32`) | implemented | unpublished |
| 8 | PA26 | Canonical operator and indexing kernels (`operator-kernel`) | implemented | unpublished |
| 9 | PA26 | Core iteration and residual protocols (`iteration-kernel`) | implemented | unpublished |
| 10 | PA27 | Interior mutation and wide-value validity (`raw-validity`) | partial | unpublished |
| 11 | PA25 | Symbolic constant storage and provenance (`const-memory`) | partial | unpublished |
| 12 | PA25 | Advanced const queries and callables (`const-callables`) | partial | unpublished |
| 13 | PA26 | Source operators, iteration and library hook closure (`hook-closure`) | partial | unpublished |
| 14 | PA26 | Built-in derives, formatting and panic consumers (`format-derive`) | partial | unpublished |
| 15 | PA28 | Source intrinsic and atomic family closure (`intrinsic-closure`) | partial | unpublished |
| 16 | PA41 | Nominal SIMD copy layout and C128 ABI (`simd-abi`) | implemented | unpublished |
| 17 | PA41 | Typed target features and call permissions (`target-features`) | partial | unpublished |
| 18 | PA41 | Required core assembly path (`assembly`) | partial | unpublished |
| 19 | PA30 | Foreign representation and callable closure (`ffi-closure`) | partial | unpublished |
| 20 | PA29 | Completed constant and foreign value metadata (`value-metadata`) | partial | unpublished |
| 21 | PA29 | Provided-body public artifact profile (`provided-profile`) | implemented | unpublished |
| 22 | PA29 | Generic and source body transport closure (`generic-transport`) | partial | unpublished |
| 23 | PA34 | Edition and crate build pipeline (`build-editions`) | partial | unpublished |
| 24 | PA31 | Compiler runtime and source callable integration (`source-runtime`) | partial | unpublished |
| 25 | PA32 | Original core artifact and independent consumers (`core-artifact`) | missing | unpublished |
| 26 | PA35 | Original core-only crate graph (`core-crates`) | missing | unpublished |
| 27 | PA33 | Original alloc and allocator/runtime consumers (`alloc-artifact`) | missing | unpublished |
| 28 | PA35 | Original alloc-only serde_json graph (`alloc-crates`) | missing | unpublished |
| 29 | PA36 | Original std and platform consumers (`std-artifact`) | missing | unpublished |
| 30 | PA37 | Procedural macro host (`proc-host`) | missing | unpublished |
| 31 | PA38 | Original derives and attribute macros (`proc-derive`) | missing | unpublished |
| 32 | PA39 | Unwinding and C-unwind (`unwind`) | missing | unpublished |
| 33 | PA40 | Remaining edition and attribute closure (`edition-closure`) | missing | unpublished |
| 34 | PA41 | Remaining target and assembly closure (`target-closure`) | partial | unpublished |
| 35 | PA42 | Async lowering (`async`) | missing | unpublished |
| 36 | PA43 | Async ownership and pinning (`async-ownership`) | missing | unpublished |
| 37 | PA44 | Complete frontend acceptance (`frontend-acceptance`) | missing | unpublished |

A fixture can declare `work_units` selecting its actual owning unit(s). Feature
owners are then checked against those units' transitive prerequisite graph and
the active PA1–PA24 foundation. Thus a source-hook fixture can explicitly depend
on PA31 declaration binding without an unrelated PA42 async exemption. Undeclared
legacy cases retain the existing numeric placement rule. This changes scheduling
and fixture validation, never semantic feature or execution permission.

## Next concrete work and publication queue

1. Integrator: close source-declarations' LIB-046 storage growth, then rerun
   unchanged original core once on the qualified compiler. Actual immutable
   debugger attribution shows cumulative storage67109064 bytes and combined
   work2940826 below the unchanged10M work cap. Published witness storage has
   169844 records and257732 capacity; per-witness small vectors dominate.
   Compact the same owner into exact witnesses plus one flat tree buffer; retain
   every attribute/input witness, prepay allocations and reject shifted ranges.
   Do not raise quotas or drop semantic/diagnostic policy.
2. B: finish ordinary PA6/19 primitive module/member fallback, integrate the
   already qualified bare-type and genuine str/slice chain, then resume actual
   Formatter/Arguments→panic/Abort. Freeze each ordinary/source outcome before
   observing it. This is hook/format closure, not another synthetic API factory.
3. C's typed target-feature handoff is qualified in isolation and awaits root
   integration. Join it with the shared SIMD query in the real assembly consumer,
   then qualify backend capabilities and original `_mm_stream_ps` semantics.
   No AVX/runtime/export permission follows from the attribute alone.
4. Publish **unsafe-places**, then **c-abi** as the first finite components:
   reconcile existing source/native/scaling evidence, inventory every fixture
   and unsupported residual, declare a nonempty component manifest, generate
   reviewed text oracles only after independent outcomes, and run missing release
   checks in CI. Next publish declaration metadata and the const machine.
   Parent PA27/30/29/25 statuses remain planned until their full remaining units
   close. Do not repeatedly rebuild a broad local release suite to rename a label.
5. After each coherent integration, check for a reusable original library artifact
   and fresh consumers. Advance complete source/body/policy transport against
   that exit, retaining all written bounds, const/stability/unsafe/caller policy
   and original/fresh-instance proof. Successful reduced submodules or parsing
   371 opened files are not a core build. All722 pinned inputs remain unchanged;
   no core/alloc/std artifact exists at adoption.

Current development checks are focused unoptimized builds with strict Clippy
all/pedantic, complete formatting and architecture/placement audits. Reuse
properly bound unchanged evidence. Scheduled/manual CI runs full release, native,
debug and independent suites. Component and whole-assignment releases additionally
run their actual handout release checks and appropriate performance qualification.
No source delegation or generated library replacement is permitted.

## Reorganization validation

The final metadata-only checkpoint passes234 harness tests, all active oracle
layout/provenance/discovery checks, maintainer registrations and original upstream
checksums in13.55s. The37-unit graph and46-family mapping also pass their
validator. Evidence is under `obj/parallel/assignment-reanalysis-v3/`; receipt
SHA-256 is `c4af28ffabd162d72faaa28c6ed78effc3b503bb2f535c649578c2f414f41d4a`.
The first graph check correctly caught requirement26.1 mapped only to a completed
kernel; its ordinary lookup residual now also maps to hook-closure. No compiler
behavior or reference bytes changed to obtain these status/plan results. This
is not a new full compiler release/performance run.

The fixture-dependency follow-up passes236 harness tests plus the same complete
layout/registration/upstream checks in13.75s (actual command time is retained
in `obj/parallel/assignment-reanalysis-v4/receipt.json`). It verifies an explicitly
owned source-binding prerequisite is admitted and unrelated future features
still reject. Legacy fixture and oracle bytes remain unchanged.
