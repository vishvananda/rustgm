# AsyncDrop investigation, 2026-11-05

This is evidence and an interface proposal, not implementation or assignment
completion. The isolated branch investigate/async-drop-20261105 starts at
071dfba494461cb3c68a1bc11d33427e81c82184. Production, global specifications,
trackers and backend pins are unchanged. Only ignored backend.local.json was
copied from the integrator's current locked configuration; the oracle verified
that configuration. The previous signature branch and warm cache are preserved.

## Frozen observations

One guarded batch ran37 authored Rust1.98.0 probes in3.45s:26 metadata and11
runtime programs, edition2024, x86_64 GNU, panic abort, overflow checks enabled,
two compiler/codegen threads. Each input and prediction was frozen before its
observation.34 initial predictions matched. The other three are important
corrections: the pinned compiler accepts AsyncDrop::drop(pin), pin.drop(), and
reifying <Token as AsyncDrop>::drop, all with empty stderr. Core's documentation
says explicit calls are forbidden, but the implementation does not enforce that
rule for AsyncDrop. Do not copy synchronous Drop's explicit-call/function-value
denial into this producer. The original predictions remain intact.

Other metadata observations establish:

- Real Pin<&mut Self> async unit method plus synchronous Drop accepts. Missing
  synchronous Drop rejects even when the AsyncDrop impl is unused.
- Wrong receiver mutability/type, non-unit Output, unsafe/generic method,
  ordinary unit-returning method, specialized nominal target and blanket target
  reject. An ordinary equivalent RPIT Future implementation accepts.
- Copy plus both destructor impls rejects. Copy plus AsyncDrop alone rejects for
  the missing synchronous Drop; it did not produce a separate Copy diagnostic.
- Actual helper formation retains unsafe-call and borrowed-storage lifetime
  obligations and Future::Output=(). Escaping local storage rejects.
- Unused authentic source AsyncDrop roles with empty, ordinary or renamed member
  sets accept. The real source Pin async method declaration also accepts.

Runtime traces distinguish actual generated glue from the helper's empty written
body. A and S label source async/synchronous methods; P/R label a one-suspension
future. Every listed runtime program compiled and completed successfully.

| Probe | Exact stdout, lines separated by / |
| --- | --- |
| Synchronous ordinary scope | S1 |
| Async scope | body / A1 |
| Actual helper, custom async method | created / A7 / done |
| Actual helper, synchronous-only target | created / sync / done |
| Actual helper, trivial target | created / done |
| Helper, parent plus two fields | A0 / A1 / A2 |
| Synchronous parent plus two fields | S0 / S1 / S2 |
| Helper suspension/resumption | A / P / pending / R / B / ready |
| Synchronous cancellation after helper pending | A / P / pending / canceled |
| Synchronous cancellation of unpolled capturing future | canceled |
| Helper completed then polled again | A7 / ready / ready |

The two cancellation observations match the pinned compiler's explicit FIXME
that synchronous drop of a coroutine with async drop is currently ignored. They
are version-specific observations, not a general cancellation guarantee. Helper
Ready repoll is observed for this generated helper; it is not a blanket Future
repoll guarantee. Explicit method calls were qualified for metadata acceptance,
not runtime field-cleanup behavior.

## Pinned source basis

Read-only upstream root is
/home/vishvananda/rustgm-sources/rust-1.98.0-88d9e12ae178fab0fb5cc050a94da85685d449ea.
The implementation supplies facts beyond the library documentation:

- rustc_hir_analysis/src/check/mod.rs124–139 requires a synchronous companion and
  reuses check_drop_impl coverage for async destructors. Actual async destructor
  impls in staged core/std are rejected by a compiler assertion.
- rustc_middle/src/ty/util.rs423 onward records the actual async impl identity
  for a local ADT. The same applicability/nominal coverage requirements matter.
- rustc_ty_utils/src/instance.rs69–94 selects AsyncDropGlueCtor and AsyncDropGlue,
  including for trivial targets. The helper's written body is not executable
  ordinary implementation authority.
- rustc_mir_transform/src/elaborate_drop.rs226–247 selects the first actual trait
  associated item for method-only destruction, separately from full helper glue.
  It constructs real Pin input and a Future polling loop.
- shim/async_destructor_ctor.rs47–158 distinguishes async recursion, synchronous
  fallback and trivial completion. Fields retain declaration order.
- rustc_trait_selection/src/traits/misc.rs81 checks actual synchronous destructor
  ownership for Copy. Valid AsyncDrop already implies that ownership.

Exact inspected-source hashes are in source-pins.json beside the raw evidence.
No upstream source or reference output was edited or substituted.

## Proposed finite interfaces and order

These require integrator agreement before any production edit.

1. **Ordinary source authority.** Add a narrow compiler_items async-cleanup owner
   that borrows authenticated existing trait/function headers and the current
   role ledger. Preserve Rust-valid unused alternative declarations. An implicit
   consumer separately demands the actual selected first method, safe Rust ABI,
   real Pin<&mut Self> receiver proof and Future Output=(). Reuse the existing
   Future/opaque and receiver interfaces; names alone grant no operation.
2. **Nominal applicability and synchronous join.** Keep one Destructors owner.
   A sparse async family can reuse Implementation/Coverage for each genuine async
   impl and join its ItemId to the existing synchronous item record. Reuse the
   inverse nominal argument mapping, lifetime coverage and trait conformance;
   reject missing synchronous companion and specialized/foreign/blanket targets.
   Keep async methods out of the existing synchronous methods denial index and
   special &mut Self receiver scheduler. Valid Copy exclusion then follows the
   existing synchronous record. Before queries can conclude absence, extend the
   existing narrow registration seal to distinguish completed async registration
   from unpaid source claims; do not add an authority registry or dense nodes.
3. **Typed source helper selection.** A source_async_drop_glue query analogous to
   source_drop_glue should authenticate the actual helper instance, element type,
   borrowed region and opaque Future result. It returns compiler-owned operation
   intent, never permission to execute the written placeholder. Metadata profiles
   lacking that contract retain an explicit publication/import fence.
4. **Actual execution prerequisite.** Implement the real future representation,
   constructor/poll state, ownership/loans and destruction recipe before emitting
   helper or async cleanup code. Keep needs-async-drop distinct from needs-drop:
   synchronous-only parents/fields, recursive async fields, zero-length arrays,
   wrappers and coroutine states require context-sensitive selection. Reuse the
   structural walker, instance scheduler and immutable registry snapshots instead
   of a second source scan. Completion, pending state and cancellation are separate
   consumers; ordinary explicit calls are not implicit full destruction glue.

A first finite producer/consumer checkpoint can validate actual local nominal
companion/conformance/Copy rules and generic source trait consumers. It cannot
claim native AsyncDrop execution or original core/alloc/std completion. The
compiler-owned helper must remain fenced until step4 exists, even if its authentic
signature is accepted. No production change is made by this investigation.

## Receipts

All raw files remain under this checkout's
obj/parallel/async-drop/investigation-v1/. The wrapper command was
`timeout 230s python3 scripts/resources.py run --wait-seconds 0 --max-output-gib 16 -- python3 scripts/storage.py run --prune-under obj/parallel/async-drop -- python3 obj/parallel/async-drop/investigation-v1/run.py`.
The batch exits1 because three frozen predictions were disproved, not because of
an infrastructure failure. Every compiler status was0 or1 and every runtime
status0, with no panic or timeout; runtime stderr was empty. Compiler diagnostics
are preserved separately. Executables/metadata were
reused and removed after each case; sources, observations and diagnostics remain.

- predictions.json SHA-256:0fe01d772697bdaa900101b39ef2315abcee6acc0218595dd126b71e9b4c6aa6.
- receipt.json:6eb55a6caa32f12938f917c1a80f0b21a4ad371446152ec6b0c7b24440621d21.
- reviewed-observations.json:60d22c89030e753e13f2ef87f034b70cd80043edecb264b9c5699e052f8acd75.
- source-pins.json:f6c4e13dc62ff1a57d52dbe1f6e4a2a430fcbded1e83188eebc5d0de4b0c6b3c.
- Sorted input path/NUL/content-SHA/newline aggregate:d42e2f9d287ad1ae67a669b6d18940e842d0377163b3aa740a637a2cd5c80073.
- Current locked backend SHA:f60c0760f39f1f554486f5a1eb990d2c0fef663fbd1bff944cd1eef8a014a581.

These are independent Rust observations. No RustGM compilation, implementation,
new native-backend qualification, full assignment suite or release claim is made.
