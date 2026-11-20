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

- **Package what exists:** publish the remaining implemented components and
  reconcile the PA1–PA20 release records. The [unit graph](../assignment-plan.json)
  records current implementation and publication status.
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
the function declaration site, at optimization levels 0 and 2. The native
lowering now prepares the actual declaration location when an address is
demanded and emits one forwarding shim per selected instance. The ordinary
pointer ABI includes the full arguments and result. The shim supplies the
extra location argument and preserves a diverging target. A virtual call to
a tracked trait declaration forwards the caller location. An impl-only
attribute reaches the definition location through a shim. Six native controls
match pinned rustc, including aggregate results, a retained static with a
diverging pointer and both virtual cases. The existing transport-fence tests
also pass. The first integrated original replay found a stored noncapturing
closure while preparing caller-location dependencies in
`std/src/panicking.rs::increase`. It reports 44 unproven obligations and eight
waivers, with an empty skip list and no LowIR. The caller-location preparation
keeps that closure on its existing checked closure entry. The replay with that
repair crosses caller-location preparation and reaches
`std/src/sys/thread_local/destructors/linux_like.rs:register` in C ABI
classification. The callback uses a local `repr(transparent)` wrapper around
`c_int` as its result. The classifier's transparent scalar route asks for a
nullable carrier, and the carrier walk does not recognize a plain integer
leaf. The replay took 48.30 seconds, with 41 unproven obligations, 50 waivers,
zero skips and no LowIR. Receipts are under
`obj/parallel/tracked-reify-check/` in the frontier checkout.

Transparent payload layout facts now carry an ordinary scalar identity separately
from an available nullable encoding. The C plan uses the integer or floating
class of that payload. Incoming and outgoing C boundaries marshal the scalar
through the original Rust object's storage, including through a function
pointer and in a mixed INTEGER/SSE record. The C observer prints
`-91 -5 2.50 -10` for both pinned rustc and RustGM. Five committed PA12 and PA20
oracles for Rust transparent records reproduce. The old aggregate-profile
test still expects rejection of a fieldless C enum that both RustGM
builds accept with identical LowIR. The native and oracle receipts are under
`obj/parallel/transparent-scalar-boundary/` in the frontier checkout.

The original Linux graph now emits 5,343,094 bytes of LowIR and a 55-byte
bring-up stamp. The first replay has 41 unproven obligations and 53 waivers.
The combined ABI and scoped destructor checkpoint emits identical LowIR, stamp
and native-link manifest with 41 unproven obligations and 52 waivers. Both
runs use zero skip rows. The same compiler emits 667,928 bytes of LowIR for
the original core and alloc graph with 27 unproven obligations, 19 waivers and
zero skips. Pinned CPPGM compiles that merged LowIR to a 206,232-byte object and
`ar` packages it into a native archive. The native core and alloc receipts are
under `transparent-scalar-boundary/alloc-native/` in the frontier checkout.
CPPGM rejected that std LowIR at line 160968. The call supplied one
argument to a tracked `Result::unwrap` whose native signature has two. The
target forwards its second argument to the panic routine. Pinned rustc reports the
ordinary source call site for a direct tracked item call, the actual core
`FnOnce::call_once`, `FnMut::call_mut` or `Fn::call` member site for generic
invocation, and the function definition for a reified pointer. The original
graph now supplies the checked caller-location operand at that call. It emits
5,344,907 bytes of LowIR with 41 unproven obligations, 52 waivers and zero
skips. Pinned CPPGM produces a 1,599,216-byte object, and `ar` packages it
into a 2,028,120-byte archive. Separate original-core guests match pinned
rustc for seven direct, generic and reified call forms and two virtual
calls. An explicit `FnOnce::call_once` guest still reaches symbolic concrete
dispatch. Formatting, architecture and strict Clippy passed, and the
focused development batch timed out during the test build. The original
std replay is under `tracked-reify-check/integrated-std-tracked-invoke/` and
the backend and guest receipts are under `tracked-item-invoke/` in the
frontier checkout.

The unchanged M3 `println` consumer still reaches
`selected Source start requires checked startup instance lowering` with the
caller-location repair. Its receipt is under
`tracked-item-invoke/std-consumer-println/` in the frontier checkout.
The startup successor prepares the actual generic callable template, proves
the main result's `Termination` and declared startup bounds, and queues that
specialization with the ordinary executable demand roots. Its C entry passes
the reified Rust main, sign-extended argument count, original argument vector,
and the pinned default SIGPIPE byte. It truncates the returned status to
`i32`. Both exact startup tests pass. Through CPPGM's object bridge the
authored Source control receives three arguments and exits with status 23,
matching pinned rustc. Both compilers reject the control with its
`Termination` implementation removed. Startup artifact transport retains its
existing fence. ABI observations and the native receipts are under
`obj/parallel/checked-source-start/` in the frontier checkout.

The frozen `println` replay with the loan and startup repairs now reaches
`alloc/src/collections/btree/node.rs:1102`, the inline const that checks
`BorrowType::TRAVERSAL_PERMIT` in `NodeRef::descend`. It reports
`selected type retains an unmapped lifetime`, with 15 unproven obligations,
14 waivers, zero skips and no LowIR. A clean, nonincremental opt-level 1 host
compiler supplied this observation after an incremental archive failed to
link. The exact receipt is under
`obj/parallel/checked-source-start/std-consumer-println/` in the frontier
checkout.

A standalone trait default and a lifetime-bearing argument reproduce the
B-tree diagnostic. The associated constant also reproduces it outside an
inline const. The frozen compiler's trace reaches associated constant
selection after opening the caller lifetime in the trait goal. The selected
provided initializer still carries the original bound slot. Restoration
tries to apply equalities from the opened selection to that original slot.
The nine pinned-rustc controls and the failing restoration trace are under
`obj/parallel/btree-inline-lifetime/` in the frontier checkout.
Pinned rustc also builds and runs the borrowed reducer and rejects its
static-only impl twin. The receipt is under `obj/parallel/btree-default-oracle/`
in the performance checkout. The fix opens the provided initializer's
original arguments in the dispatcher's existing mapping before applying the
selection's equations. The optimized compiler builds all nine reducers, and
their native runs match the declared outcomes. The three already supported
controls keep byte-identical LowIR. Both compilers reject the static-only
impl. Formatting and architecture checks pass. Private validation timed out
building the test harness.

The original `println` guest now emits 8,332,813 bytes of LowIR with zero skips,
14 unproven obligations and 57 waivers, and CPPGM links the executable. It
aborts before printing in the `catch_unwind` shim. A diagnostic copy of its
LowIR runs the try callback in four such shims and advances to the
`assert_zero_valid` shim. The original emitted program, native reducer results
and backtraces are under `obj/parallel/btree-default-successor/`,
`obj/parallel/btree-reducer-native/` and `obj/parallel/println-native-abort/`
in the frontier checkout.

The abort-profile candidate validates the existing intrinsic prototype and
reached direct call, emits the try callback with its actual data pointer and
returns false after normal completion. Pinned rustc executes the focused
observer and returns 23 from the try callback. The RustGM code and focused gate
are in the frontier checkout. The zero-validity probe is consistent with the
152-byte Linux `sigaction` in std startup. Its actual libc fields include
`Option<extern "C" fn()>`. The unqualified checker currently declines enums.
Verify the exact instance and use its retained niche layout to determine which
variant zero bytes select.

The startup successor takes enum fields from the retained niche or tag and
checks the selected fields for zero validity. A null function-pointer niche
selects the empty variant. A bare function pointer still fails the check.
The longer tagged scan charges the layout work budget, while the common
first-variant zero tag is a direct read. The CTFE route retains its
existing conservative behavior. The optimized callback and enum controls,
and the original `println` replay, use the same candidate build under
`obj/parallel/std-startup-successor/` in the frontier checkout.
The four native enum controls agree with pinned rustc. Both zero-valid
inputs return successfully, and the nonzero tag and bare function pointer
terminate with SIGABRT. The plain callback and generic data-struct callback
both pass the native observer with result 23. The frozen original `println`
replay fails before emission at the specialized `catch_unwind` in
`std/src/panicking.rs:544`. The debugger confirms that the selected
function-item signature and the checked call-site signature have distinct
canonical IDs. The checked try and catch callbacks both take the
specialization's data pointer. The actual data argument in the checked call
has a distinct pointee `TypeId`. The raw records are in
`std-startup-successor/callback-parameters.txt` in the frontier checkout.
The next trace reaches the same closure identity in both Data applications.
Its enclosing-region arguments and capture environment use different
canonical coordinates. The startup successor compares type structure at
this ABI boundary and keeps lifetime checking with the reached call's
proof. The development compiler's replay hits the frontend time limit.
The optimized observation compiler emits 8,331,309 bytes of LowIR for the
original `println` consumer, with zero skips, 14 unproven obligations and
55 waivers. CPPGM links it and all three executions match the declared
exit status, stdout and stderr byte for byte. The exact receipt is under
`obj/parallel/std-startup-optimized/std-consumer-println/` in the frontier
checkout. The HashMap, environment and filesystem replays use the same
compiler and source pins. HashMap and HashSet match all three declared
native runs with zero skips, 18 unproven obligations and 60 waivers.
The environment consumer reaches the trait obligation at
`a.parse().ok()` in `env-args.rs:16`. The filesystem consumer reaches
the nested caller-output transport fence in `std/src/sys/io/kernel_copy/linux.rs`.
The three receipts are in `obj/parallel/std-startup-consumers/` in the
frontier checkout. The startup code passes formatting, architecture and
strict Clippy. The ten remaining std consumers use the same compiler and
the shared oracle's exit, abort and stderr rules. `process-exit`, `exit-code`,
`sync-local`, `path-ffi-time`, `panic-default` and `panic-hook` each match
their declared outcomes in three native runs. `main-result` and `float-math`
emit LowIR and link, then abort where the oracle expects exit 1 and 0,
respectively. Native debugger replays of the same executable hashes locate
the first traps at `size_of_val` during drop in `main-result` and `floorf64`
before the first output in `float-math`. `io-stdin` stops in the frontend at
an unambiguous trait obligation in `main`. `threads` stops in
`alloc/src/boxed.rs` because the
reached `call_once` has neither a concrete body nor a checked template.
The batch receipt and per-case diagnostics are in
`obj/parallel/std-startup-remaining/` in the frontier checkout. Eight of
the fourteen original std consumers have matched under the bring-up profile.

The dynamic extent change reads the existing size and alignment words from
the actual trait object's vtable. A native C observer checks both for two
different trait objects, a slice and a sized array. The same frozen input
aborts on the predecessor and passes with zero skips on the candidate.
Both intrinsic names still report their bring-up qualification waivers.
Receipts are in `obj/parallel/dynamic-extent-predecessor/` and
`obj/parallel/dynamic-extent-independent-pointers/` in the frontier checkout.
The optimized compiler also passes the complete std-graph extent probe and
the original `main-result` consumer in three native runs each. The original
consumer emits 7,499,385 bytes of LowIR with zero skips, 14 unproven
obligations and 51 waivers. All three runs match the declared output and
exit status 1. Nine of fourteen original std consumers now match in bring-up.
The exact receipts are in `obj/parallel/dynamic-object-extent/`.

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
on both compilers. The repair recovers the selected owned fields
from distinct variant tests during unguarded capture. The reduced enum moves
past the cleanup check to source transport validation, and both live-value
opponents still fail the cleanup check. Three focused tests pass. The original
strict core replay clears `ControlFlow::into_value` and reaches the body of
the array `try_map` implementation at `core/src/array/mod.rs:640`. It reports
`active cleanup requires const Drop in its actual defining environment`.
The trace identifies `Drain<'_, '_, T, F>` and the actual conditional `Drop`
implementation. The environment carries `T: [const] Destruct`, while the
defining cleanup requested unconditional `Drop`. The repair uses the same
conditional effect as ordinary const-function body checking. The exact closed
destructor selection and execution checks remain separate.
Two reduced const cases pass cleanup selection and reach later transport
checks. The missing-bound case is rejected, and the ordinary control reaches
the same transport check. Four focused cleanup tests pass. The strict core
replay now reaches `try_from_fn_erased` at `core/src/array/mod.rs:909` with
`projected lifetime components require a scoped proof`. The trace reaches
`TypeComponents::drop_inputs` from destruction-region liveness during the
staged loan proof. Its scalar-only return rejects the depth-zero projected
argument. The strict replay has zero bring-up records and emits no metadata.
The scoped custom-destructor query now retains the actual input components of
an associated argument, with explicit declaration `'static` evidence available
where it applies. Ordinary argument checks retain their existing array and
binder rules. The higher-ranked controls agree with pinned rustc. An active
projected guard rejects a second mutable borrow, a dropped guard permits the
borrow and an independently static result carries no loan through the guard.
Three focused tests pass. The replay of the same pinned graph clears
`try_from_fn_erased` and reaches the defining body of
`core::array::write<T, N>` at `core/src/array/mod.rs:1087`. It reports
`active cleanup requires const Destruct in its actual defining environment`.
The source moves the initialized guard into `mem::forget` after the loop.
A reduced pinned-rustc control accepts this move. Its abort-profile MIR calls
`forget` and has no drop terminator in the defining body. Removing the move
produces E0493 for the active guard destructor. RustGM's rejection comes from
the staged plan's eligibility selection. An unused mutable reference was
enough to retain a qualification subject after the guard moved into
`forget`. The selected subject now takes its access flag from the actual
cleanup schedule. Conservative qualification still consults the retained
history. In the precise control, RustGM clears cleanup selection and reaches
the source-role transport boundary. Removing `forget` or the precise
live-drop feature produces the expected cleanup error in RustGM and pinned
rustc. Three focused ownership and qualification tests pass. The initial
strict replay at this frontier emitted zero bring-up records and no metadata.
The exact source controls
are under `obj/parallel/projected-drop-scope/source-controls/` and the replay
is under `obj/parallel/strict-core/projected-drop-scope/` in the performance
checkout. The mutable-borrow controls are under
`obj/parallel/precise-borrow-history/` there.

The optimized strict replay crosses the array cleanup check and reaches
`core/src/slice/sort/stable/quicksort.rs:24` with
`access conflicts with a live reference loan`. It emits zero bring-up records
and no artifact. A loop that replaces its mutable slice with a borrowed
subslice contains the same ownership pattern as a scalar control that assigns
`v = identity(v)` on successive iterations. RustGM rejects both the scalar
control and the loop with tupled reborrow results. Pinned rustc accepts both.
Both compilers accept the one-shot tuple control and reject a write through
the parent before the outstanding child reference's use. The replay and
controls are under `obj/parallel/strict-core/precise-borrow-history-opt/` and
`obj/parallel/strict-quicksort-reducer/` in the performance checkout.

The scalar body's typed CFG borrows through the local pointer, evaluates the
returning call and stores the returned reference into that pointer. The next
iteration previously saw the loan through the pointer's prior value. The loan
transfer now indexes those loans by definite containing storage slots and
retires their masks after checking the write. The original referent's other
loans still have their own identities. Both the regular and ordered-cleanup
transfers use the same statement path. Four exact ownership and loop tests
pass, including projected reference slots, a conditional replacement, a loan
on the pointer storage, and a live loan on the original referent. RustGM and
pinned rustc accept both loop reductions and reject the premature parent
write. The successor controls are under
`obj/parallel/strict-quicksort-candidate/` in the performance checkout.

The optimized replay crosses quicksort and reaches
`core/src/iter/traits/iterator.rs:232` in `Iterator::count`. The const
closure consumes an owned `Self::Item`, with a conditional `Destruct` premise
on the enclosing method. The strict diagnostic is
`const cleanup selection requires its actual source function`. The original
closure already carries its defining owner and inherited parameter context in
its region contract. The cleanup selector at that checkpoint required a
function body key.
The owned generic reduction reproduces the failure. Pinned rustc accepts it
with the declaring premise and reports E0493 when that premise is removed.
The borrowed and concrete controls clear cleanup selection in both compilers
and RustGM reaches the subsequent source-role transport check. The original
replay records zero bring-up rows and emits no artifacts. Receipts are under
`obj/parallel/strict-core/loan-slot-replacement/` and
`obj/parallel/strict-const-closure-cleanup/` in the performance checkout.

The closure cleanup successor authenticates the child body key, semantic
source, public and execution signatures, and the child's retained defining
environment. It checks the owned generic and projected item under the
declaring `Destruct` premise. Both paid controls clear cleanup checking, and
the unpaid controls retain the active-cleanup rejection and match the pinned
rustc outcome. A demanded generic closure instance, nine existing capture
cases, and the ordinary and constant-initializer controls pass the focused
checks. The captured CTFE control still reaches its callable-type error.
The receipts are under `obj/parallel/strict-const-closure-candidate/`,
`obj/parallel/strict-const-closure-projections/` and
`obj/parallel/const-closure-captured-controls/` in the performance checkout.
The strict original core replay advances through the closure cleanup frontier.
On the same frozen source hashes and environment, it reaches the export guard
`general fundamental policy requires authenticated trait/nominal metadata transport`
in 37.78 seconds. It records zero bring-up rows and emits no artifacts. The
existing export path recognizes fundamental transport for validated sizing-role
traits. The strict library now needs transport of the authenticated general
declaration policy and a fresh consumer that uses it in coherence. The replay
is under `obj/parallel/strict-core/const-closure-environment/` in the performance
checkout. [Policy transport guard](../dev/src/sema/header_attributes/definition_policy.rs#L389) ·
[Local policy and cross-crate controls](pa31-fundamental-policy-checkpoint.md)

The v14 declaration-policy transport carries the two trait policy bits and a
reserved bit in the nominal header for fundamental structs. The consumer binds
those facts once to the validated declaration in its existing imported header
ledger. The older formats retain their source publication guard. Six pinned
rustc provider/client controls agree with their declared outcomes for
fundamental locality, closed trait disjointness and forced dyn incompatibility.
Formatting and architecture checks pass. The private harness builds timed out.
The optimized provider emits v14 metadata. Fresh consumers accept closed trait
disjointness, reject overlap for the ordinary trait, reject the ordinary
foreign wrapper, and reject the forced trait object. The marked-wrapper client
reaches traversal but lacks a completed local source producer for scratch
accounting in the first candidate. The integrated version charges the
existing local input-graph budget. The provider's metadata matches the first
candidate byte for byte, and all six fresh consumers match their declared
outcomes. Both trait-object cases use private declarations to isolate
formation from scalar signature export.
Receipts are under `obj/parallel/v14-definition-policy-rustc/`,
`obj/parallel/v14-definition-policy-successor/`, and
`obj/parallel/v14-definition-policy-repaired/` in the performance checkout.

The unchanged strict `core` replay with this encoding advances to
`incoherent target authority and inherent index require their wider metadata profile`.
It takes 36.86 seconds, records zero bring-up obligations and emits no artifacts.
The pinned core sources mark `core::error::Error`, `ffi::CStr`, `wtf8::Wtf8`
and the I/O error struct. Downstream alloc and std declare inherent methods on
these owners or their trait objects. Each consumer must authenticate the
defining target and its own marked methods, then join their validated headers
to the existing nominal or principal-trait method bucket. The current
inherent metadata profile covers provider-owned nominals. The strict receipt is
under `obj/parallel/strict-core/definition-policy/`.
[Local target contract](pa31-incoherent-target-checkpoint.md)

The v14 Has transport records the exact defining struct, enum or trait and
checks each foreign impl's own feature and marked-member permissions. In a
fresh client, all six covered target cases agree with pinned rustc. The
foreign nominal method also runs through CPPGM and the C observer. The v13
publication guard still rejects the marked provider. This validates target
authority and local method lookup in the implementing crate. A separate
client still needs transport of the validated foreign method index and
complete dyn Self applications. Formatting, architecture and strict Clippy
pass for the integrated change. The optimized strict original core replay
clears the inherent-target export gate in 33.77 seconds. The next failure is
`method dispatch policy requires authenticated trait metadata transport`.
It reports zero bring-up records and emits no artifacts. Original
`core/src/iter/traits/collect.rs:280` marks `IntoIterator` with the array
and boxed-slice flags. Reuse the checked trait policy in metadata and the
method-name token's edition at adjusted-receiver lookup.
Receipts are under `obj/parallel/v14-incoherent-authority/` and
`obj/parallel/strict-core/incoherent-target-optimized/` in the performance checkout.

The v14 exporter now carries both authenticated method-dispatch flags in the
defining trait row. Six fresh array clients match their pinned Rust outcomes
through native execution. The marked trait selects the borrowed impl in 2015
and 2018 and the owned impl in 2021 and 2024. The unmarked and qualified 2018
calls select the owned impl. The v13 exporter rejects the marked trait, and
the separate fixture using synthesized iteration authority keeps its legacy
fence. An initial provider with the borrowed impl in its own artifact reached
the separate early/captured callable frame transport fence. The passing
clients declare the impls locally with a local trait parameter. Receipts are
under `obj/parallel/v14-dispatch-policy/` and
`obj/parallel/v14-dispatch-client-impl/` in the frontier checkout.
The unchanged PA34 by-value `Box<[u8]>` fixture links with the original std
graph and matches stdout `9` in 2018 and `7` in 2024. Its bring-up skip lists
are empty. The receipt is under `obj/parallel/dispatch-boxed-original/` in
the frontier checkout.

The optimized replay of the same 309 strict core inputs clears the dispatch
gate in 32.90 seconds and reaches `optimization intent requires authenticated
metadata transport`. It records zero bring-up entries and emits no artifacts.
The original source uses `optimize(size)` on abort-profile panic functions.
The strict export must retain their checked function policy for a fresh client.
The receipt is in `obj/parallel/strict-core/dispatch-policy/` in the frontier
checkout.

The v14 function header now carries named-function optimization intent. A
fresh SourceClient reads `none`, `size` and `speed` from their actual imported
functions, and reads checked absence on the unmarked function. Metadata
round-trips byte for byte, and the four calls match the native C observer.
Pinned rustc accepts the marked provider. The v13 source fence stays active,
and an unmarked v13 provider emits identical metadata and LowIR on both
compiler revisions. The focused compiler passes strict Clippy and the
architecture audit. Receipts are in `obj/parallel/v14-optimization-intent-client/`
in the frontier checkout. Closure intent still needs its body metadata.

The next optimized strict replay of the unchanged 309 core inputs clears
optimization intent and reaches `SIMD lane limits require authenticated
representation metadata transport` in 27.51 seconds. It records zero
bring-up entries and emits no artifacts. The included `portable-simd` crate
source marks the actual generic `Simd` declaration with a limit of 64 in
`crates/core_simd/src/vector.rs:107`. The receipt is in
`obj/parallel/strict-core/optimization-intent/` in the frontier checkout.

The v14 packed-SIMD producer and a fresh source client pass the four layout
controls with a limit of eight. Six lanes have size 24 and alignment 8;
eight lanes have size and alignment 32. The nine- and sixteen-lane imports
fail at the actual struct's limit, with no published layout. The v13 producer
rejects the authored limit, while a separate consumer can emit its own
ordinary metadata with the v14 dependency. The frozen original PA41 provider
and import sources match all nine pinned-rustc compile outcomes. A separate
RustGM provider with bodiless `size_of` and `align_of` declarations reaches
the intrinsic publication boundary. The passing reduction imports the
generic `V` and demands concrete layout directly through the typed pipeline.
Receipts are in `obj/parallel/v14-simd-limits-final/` in the frontier checkout.

The unchanged 309-input strict core replay clears the SIMD gate in 28.04
seconds and reaches `deprecation declaration transport is not implemented
for this metadata profile`. It reports zero bring-up records and emits no
artifacts. The receipt is in `obj/parallel/strict-core/simd-limits/` in the
frontier checkout.
The v14 declaration checkpoint carries each checked notice once and maps
affected declarations to its actual lexical origin. The importer applies the
declaring crate's release rule and uses the consuming crate's lint policy.
Fresh clients match pinned Rust for active and inherited diagnostics, `TBD`,
a future staged version, `allow` and `expect`. The focused suite checks the
metadata round-trip, existing compatibility warnings and the legacy import fence.
The same unmarked provider emits identical LowIR and metadata bytes. Version
thirteen still rejects a provider with deprecation annotations. The frozen
inputs and focused comparisons are under `obj/parallel/v14-deprecation-design/`
and `obj/parallel/v14-deprecation-candidate/` in the frontier checkout.

The exact 309-input strict replay on the optimized observation compiler clears
the deprecation fence in 34.43 seconds. It reaches `reservation implementation
transport requires a wider metadata profile` with zero bring-up records and
no artifacts. The original declaration is `const impl<T> From<!> for T` at
`core/src/convert/mod.rs:804`. Pinned Rust confirms that a matching reservation
alone cannot prove a use-site obligation, a real implementation can provide
an executable answer, and the reservation still participates in compatible-world
overlap. References and the native result `7` are under
`obj/parallel/v14-reservation-reference/`. The original strict observation is
under `obj/parallel/strict-core/deprecation-optimized/` in the frontier checkout.

Version fourteen carries a completed reservation's checked header and member
declarations. A fresh importer reconstructs its original const effect and uses
the reservation for coherence. The ordinary and const reductions both import
successfully. Their use-site obligations are rejected, and the independent
positive implementation still returns `7` natively. The old profile rejects
the reserve annotation, and the unmarked control emits identical LowIR and
metadata on both compiler revisions. The imported overlap diagnostic retains
the generic message. The older v13 automatic-implementation provider hits the
same unwind-marker publication fence on both revisions. Candidate observations
are under `obj/parallel/v14-reservation-candidate/` in the frontier checkout.
[Fresh client regression](../dev/tests/crate_linking/versioned_clients.rs)

The optimized strict replay at `cbc823943` clears the reservation gate on all
309 unchanged `core` inputs. After 28.72 seconds it reaches `source
macro/semantic-name provenance metadata is not implemented`, with zero
bring-up records and no artifacts. The gate checks contextual item bindings,
opaque item marks and marked local bindings together. The next reduction
needs to identify which names a published row would have to resolve and which
references already use checked declaration identities. The receipt is under
`obj/parallel/strict-core/reservation/` in the frontier checkout.
[Existing provenance boundary](parallel/macro-contexts.md)

Version fourteen now publishes checked macro-generated body locals and value
parameters by declaration and body slot. Labels use the resolved block
destinations of the retained CFG. Five reduced providers publish and import
in separate processes, with a native observer returning `42` for each.
Pinned Rust accepts all eight reduction providers. Contextual item names
and macro-generated exported definitions still fail their separate gates.
The v13 observations agree with the previous compiler and the unmarked
v13 and v14 providers emit byte-identical LowIR and metadata. The four
focused checks pass. Receipts are in `obj/parallel/v14-macro-provenance/`
in the frontier checkout. [Fresh body regression](../dev/tests/crate_linking/hygienic_exports.rs)

The unchanged `core` replay still hits the name-provenance gate on the
optimized `e4e134081` observation after 37.85 seconds. It records zero bring-up
rows and emits no artifacts. At this profile the check is specifically for
contextual item bindings. The resolved binding table has 901 non-root entries.
It records 733 associated functions, 12 associated types, 40 generic formals
and 116 private functions inside other bodies. The body helpers consist of
44 `runtime`, 44 `compiletime` and 28 `do_panic` definitions from the
`const_eval_select` and `const_panic` macros. The replay is under
`obj/parallel/strict-core/macro-body-names/`, and the exact binding survey is
under `obj/parallel/v14-macro-provenance/exact-binding-observer/`.

The v14 contextual item check now admits actual generic formals and members
of checked trait impls through their typed slots. Three reduced providers
import in fresh processes and independently execute the generic call, the
trait impl's method, and the associated type check. Each returns `42` natively.
A projected-return call fails with the same applied-signature diagnostic on
its unmarked counterpart. Private helper functions still need authenticated
original body identities before they can pass this check. Five focused
linking tests pass on the joined revision. Evidence is under
`obj/parallel/v14-contextual-items/` in the frontier checkout.

The next v14 slice publishes private functions owned by another function
using their body and declaration coordinates. A fresh client calls two
same-spelled helpers from distinct macro expansions, and a separate client
evaluates an imported const helper. Both reductions return `42` natively.
The focused regression rejects a direct call to the private helper through
the module namespace. It also checks byte-exact metadata round-trip and the
older provenance gates. Four focused cases and strict Clippy pass.

The optimized `036ba5ddd` replay reaches `source type reflection contract
transport is not implemented` after 28.14 seconds on the same 309 `core`
inputs. This revision checks reflection before contextual names, so the
replay leaves the full contextual binding check pending. It records zero
bring-up rows and emits no artifacts. The actual declarations are `TypeId`
in `core/src/any.rs:723` and `Type` in `core/src/mem/type_info.rs:13`. The
receipt is under `obj/parallel/strict-core/private-helpers/` in the frontier
checkout.

Version fourteen carries the actual reflection struct declarations and
revalidates their imported nominal kinds. A fresh native client reads the
nonexhaustive description and observes the identity's two pointer words.
A separate client exercises generic and lifetime-parameterized declarations.
Both native observers return `42`. Three malformed source roles and three
accepted providers match the pinned Rust outcomes. The v13 producer gate is
preserved. In each profile, the unmarked provider emits identical metadata
and LowIR on both compiler revisions. Reflection intrinsics still require
their own operation and provenance proofs. The receipts are under
`obj/parallel/v14-reflection-declarations/`.

The optimized `dd7cd52df` replay checks contextual names before reflection on
all 309 unchanged `core` inputs. It passes both declaration checks and
reaches `source Div contract transport is not implemented` in 31.62 seconds
with zero bring-up records and no artifacts. The receipt is under
`obj/parallel/strict-core/reflection-roles/`. The reduced `Div` declaration,
ordinary impl and const impl pass pinned Rust and reach the same export fence
in RustGM. Both compilers reject the ordinary impl used from a const function.
The reduced receipts are under `obj/parallel/v14-division-transport/baseline/`.

Version fourteen now publishes the required `Div` declaration and ordinary
implementation. The importer uses the authenticated trait's implicit `Self`
clause to check const use. The fresh runtime client prints `42` under both
compilers. Both reject that same implementation from a const initializer
because it does not satisfy the const trait obligation. Version thirteen
retains its division gate. A source-marked non-const method reaches its own
policy fence. A provider wrapper that calls `Div` also reaches a separate
applied-signature normalization error during replay.

`68d0e2474` adds positive const implementation transport for the authenticated
`Div` role. A fresh importer re-proves the actual implementation's effect,
method conformance and associated result. The const client folds the call
to `return u8 42`. Both fresh native binaries print `42`, and the paired
ordinary implementation still fails the const obligation. Three focused
checks pass with the exact effect regression. Evidence is under
`obj/parallel/v14-division-transport/const-positive-final/`.

The optimized `cd2522f2a` replay of the same 309 `core` inputs passes the
division declaration gate and stops at
`source range helper contract transport is not implemented`. The run takes
28.80 seconds, records zero bring-up entries and emits no artifacts. The
range roles come from `RangePattern` in `core/src/pat.rs`. It has two required
associated constants, a caller-tracked `sub_one` method and original const
implementations. The replay receipt is under
`obj/parallel/strict-core/division-roles/`.

`be9de53f0` transports the pinned `RangePattern` declaration in version
fourteen. The two required constants retain their `Self` types and absent
values. The imported `sub_one` header preserves its conditional effect,
caller tracking and absent body. A fresh client accepts the trait bound
and rejects a call that requires an absent `u8` implementation, matching
pinned Rust. Version thirteen preserves its range fence. The focused tests
and deterministic metadata round trip pass. The concrete integer and char
implementations still need their associated values and checked bodies.
Evidence is under `obj/parallel/v14-range-transport/declaration-final/`.

The optimized `be9de53f0` replay of the unchanged 309 inputs clears the range
declaration gate. It stops during metadata export after 31.65 seconds with
`source declaration kind is outside the initial source profile`, zero bring-up
records and no artifacts. Adding the original `extern crate self as core`
declaration to a reduced provider reproduces the same fence. Pinned Rust
accepts both reduced providers, and RustGM emits strict artifacts for the one
without the alias. The replay and controls are under
`obj/parallel/strict-core/range-declarations/` in the frontier checkout.

`8718740ad` publishes the private self alias as a scoped binding to the
provider's actual root. A fresh client calls the public function that uses
the alias internally, and a lookup through the private alias receives the
privacy diagnostic. Independent metadata compilation with pinned Rust agrees.
Two RustGM publications produce identical LowIR and metadata, and the
namespace round trip preserves the original root and privacy. Version
thirteen reaches the original declaration-kind fence. The alias, range and
deprecation checks pass in one focused batch. Evidence is under
`obj/parallel/v14-root-self-alias/independent-metadata/` and
`obj/qualification/fast/1795130911238788050/` in the frontier checkout.

The optimized `8718740ad` replay of the same 309 inputs clears the alias
gate and stops at `source associated constants require their wider profile`
after 43.84 seconds. It records zero bring-up entries and produces no
artifacts. The checked declaration inventory names `f128::RADIX` in the
pinned `core/src/num/f128.rs` as the first blocked member. `RADIX`, `BITS`,
`MANTISSA_DIGITS` and `DIGITS` carry `u32` values. Their common inherent owner
already needs the `f128` type tag, and `EPSILON` adds a value of that type.
The existing constant payload has 128 bits. The strict receipt and typed
inventory are under `obj/parallel/strict-core/root-self-alias/` in the
frontier checkout.

`9f0cde6a0` adds exact `f16` and `f128` type recipes in version fourteen.
Two publications produce identical metadata and LowIR. Fresh clients match
pinned Rust on both the correct field types and the `f128` to `f64` type
rejection. Version thirteen retains the scalar type gate. A fresh executable
using wide-float literals reaches the runtime-lowering gate. The focused
float, range and reflection tests pass. Evidence is under
`obj/parallel/v14-float-types/` and `obj/qualification/fast/1795135765398208848/`
in the frontier checkout.

`3b2a4396e` carries evaluated scalar constants on authenticated Core primitive
inherent owners in v14. The producer evaluates unused constants, preserves
each declaration and member link, and exports the const function's original
read. The fresh consumer checks the same owner and type before seeding the
completed value. The focused client reads the `u32` and `u64` owners and
evaluates the imported const function to `42`. Private access, mixed result
types and v13 publication retain their existing diagnostic paths.

Pinned Rust and RustGM agree on a separate staged `f128` owner with `u32`
constants. The stable read yields `2`. The unstable read requires its feature
and yields `128` once enabled. Two publications give identical metadata. An
independent C-ABI observer prints `42` for both compilers after reading the
imported const function, with CPPGM providing RustGM's native object. Receipts
are under `obj/parallel/v14-primitive-constants/staged-confirmed/`,
`obj/parallel/v14-primitive-constants/independent-native/` and
`obj/qualification/fast/1795138100667015272/` in the frontier checkout.

The optimized strict replay of `3b2a4396e` reaches the associated-constant
diagnostic in 26.99 seconds on the same 309 pinned inputs. The declaration
trace admits `RADIX`, `BITS`, `MANTISSA_DIGITS` and `DIGITS` on the actual
`f128` owner and stops at `f128::EPSILON` in `core/src/num/f128.rs:194`.
Transport of the value needs the existing 128-bit payload and the exact
wide float recipe. The replay records zero bring-up entries and emits no
artifacts. The strict receipt and exact trace are under
`obj/parallel/strict-core/primitive-constants/` in the frontier checkout.

`2851f1e33` preserves wide named values using the current v14 type recipes
and constant payload. The producer encodes its evaluated value, the packet
checks the exact width, and the fresh consumer rebuilds that same typed value.
The focused provider publishes the pinned `f16::EPSILON`, `f128::EPSILON` and
`PI` declarations. A fresh client evaluates an imported generic const
function and two direct reads. The completed values match all three expected
bit patterns. Pinned Rust accepts the exact provider and client and rejects
the half-to-quad assignment with E0308. Evidence is under
`obj/parallel/v14-primitive-constants/wide-exact-independent/` and
`obj/qualification/fast/1795141065976588995/` in the frontier checkout.

The optimized strict replay with `2851f1e33` and `672467557` advances to
`named, associated or generic alias transport requires its wider profile`
after 25.12 seconds on the unchanged 309 inputs. The actual refusing
declaration is `pub type Digit32 = u32` in the pinned
`core/src/num/imp/bignum.rs`. It has a closed canonical target and a public
name. The replay records zero bring-up entries and emits no artifacts.
The receipt and exact declaration trace are under
`obj/parallel/strict-core/wide-constants/` in the frontier checkout.

`089896edf` publishes a v14 public closed module alias with a named
declaration and its actual canonical target recipe. An imported alias
installs the same target type in its declaration cell. The sparse
column checks the exact public module owner and carries renamed
reexports through the ordinary namespace and availability columns.
Alias parameters, associated aliases and deferred proof requirements
stay on their existing paths. The v13 export still rejects the public
named alias.

The focused fresh-client case publishes `Digit32`, a tuple target, a
nominal target and aliases reexported from public and hidden modules.
It validates decode and encoded byte equality, imports the aliases in
a new RustGM process and rejects a private name, an incompatible
value and an applied type argument. The exact strict Clippy, format,
architecture and functional gate passed in 97.33 seconds. Evidence is
under `obj/qualification/fast/1795144369638357771/` and
`obj/parallel/v14-type-aliases/dev-fast-foundation/` in the frontier checkout.

On the same 309 inputs the optimized strict replay passes `Digit32`
and reaches the declaration-kind gate at `DecimalSeq::MAX_DIGITS` after
26.97 seconds. The exact compiler trace identifies DefId 574 and
the pinned `decimal_seq.rs` owner. The replay records zero bring-up
entries and emits no strict artifacts. A reduced provider preserves
`pub const MAX_DIGITS: usize = 768` and uses the value in an array
field's length. Pinned Rust accepts the provider and consumer and
rejects the private read and wrong result type. RustGM reproduces
the provider export failure. The receipts are under
`obj/parallel/strict-core/closed-type-aliases/` and
`obj/parallel/v14-nominal-constants/baseline/` in the frontier
checkout.

`b556df814` publishes an evaluated scalar constant under its exact
closed inherent implementation of a provider-owned nominal. It reuses
the v14 typed value row, authenticates membership and preserves the
separate Core primitive authority. The fresh-client case decodes the
packet and evaluates both `DecimalSeq::MAX_DIGITS` and the imported
`const fn` to `768`. The same case rejects a private read, a wrong type
and a v13 publication. The existing primitive-inherent owner control
also passes. Strict Clippy, formatting, the architecture audit and
the two selected cases finish in 100.67 seconds. The receipt is under
`obj/qualification/fast/1795145920873708384/` in the frontier checkout.

The next optimized strict replay passes `DecimalSeq::MAX_DIGITS`
and reaches the declaration gate for `POWER_OF_FIVE_128` in 34.97
seconds on the same 309 source inputs. The agreeing resolver trace
identifies DefId 618, a module-restricted static array of `(u64, u64)`
in `core/src/num/imp/dec2flt/table.rs`. The replay records zero
bring-up entries and emits no strict artifacts. The receipt and
trace are under `obj/parallel/strict-core/closed-nominal-constants/`
in the frontier checkout.

`b25c216b5` transports checked immutable process-wide statics with closed
scalar, tuple and fixed-array values. An imported generic body names the
provider's allocation by its canonical declaration. The reduced
`POWER_OF_FIVE_128` case preserves its private module restriction and exact
array width. Two independently linked native programs, built with RustGM
and pinned rustc, each verify one address across two generic specializations
and read the same four scalar values. The focused regression passes the
metadata round trip, privacy and type controls and the version 13 fence.
The qualification receipt is `obj/qualification/fast/1795152957489392666/`.
The two native receipts are under `obj/parallel/v14-static-arrays/` in the
frontier checkout. This qualification covers the reduced source and uses
the development compiler.

The same development compiler reached its 230-second limit on the
unchanged 309 original Core inputs with no diagnostic or artifact.
The timeout receipt is under `obj/parallel/strict-core/static-arrays-dev-fast/`.
The optimized replay on `b25c216b5` passes the static declaration gate
and reaches associated-constant publication after 28.47 seconds. The
exact declaration trace identifies DefId 621,
`Lemire::MAX_EXPONENT_FAST_PATH` in `core/src/num/imp/dec2flt/mod.rs`.
The `i64` trait default reads `Self::SIG_TOTAL_BITS` through the
supertrait chain, so its value depends on the selected implementation.
The replay produces zero bring-up records and no strict artifacts.
The replay and declaration trace are under
`obj/parallel/strict-core/static-arrays-observe/` in the frontier checkout.
A reduced provider keeps the original default expression and its two
supertrait edges. RustGM type-checks it and reaches the same publication
diagnostic. Pinned rustc links a distinct consumer and returns
`10 22 27 91` for two provider implementations, a consumer implementation
using the default, and a consumer override. The control is under
`obj/parallel/strict-core/static-arrays-observe/lemire-default-control-v3/`.

`76751780c` carries primitive division in v14 original bodies. A fresh
client evaluates an imported floating quotient to `22` at compile time.
The native controls agree with pinned Rust on floating and signed
division results and on aborts for zero and signed overflow. The
existing v13 wire rejection controls pass. The focused qualification
is under `obj/qualification/fast/1795155629594143261/`, and the native
evidence is under `obj/parallel/v14-division/` in the frontier checkout.

The reduced Lemire initializer builds as its own typed constant
template with one block, one local and nine values. The body uses a
free constant, subtraction, a selected supertrait constant request,
two casts and division. The trait transport still needs to carry the
initializer's real declaration and its open associated-value request
so each consumer can select its own implementation. The template view
is under `obj/parallel/strict-core/static-arrays-observe/lemire-ir/inspect-v2/`.
`0e58b426c` makes original reservation take the actual declaration body kind
and its context. The ordinary and compiler-operation ownership, forged-wire
and division controls pass in `obj/qualification/fast/1795156227224835594/`.

`7a5329550` publishes and imports an ordinary required trait constant in
v14. The producer retains the actual value-namespace member and its
inherited declaration scope. A fresh client uses the same imported
`Table<T>::ROW` with `u8` and `u16` implementations, yielding `19` and
`513` in LowIR. It rejects a missing constant and a mismatched type.
The required-constant packet encodes identically after decoding. The
v13 rejection, existing named-value, Range and division controls retain
their expected outcomes. The four exact integration tests, formatting,
architecture and selected strict Clippy pass. Pinned Rust independently
produces `19 513` and rejects the two negative controls. The private
codec checks and strict original Core replay remain pending. Evidence is
under `obj/qualification/fast/1795160908867247551/` and
`obj/parallel/v14-associated-constants/required-rustc-v1/` in the frontier
checkout. The provided initializer remains the next transport requirement.

`d7a9ddcc8` in the frontier worktree publishes a trait constant's
original initializer and each explicit override under their checked
declaration owners. The producer retains lexical locals, inherited
slots, original-body numbering and actual default/override member
links. The metadata decoder retains the v14 initializer context.
The provider emits 8,439 bytes of metadata, which re-encode exactly.
The focused control checks coverage of the two constant bodies and
three function originals. All five selected integration tests,
formatting, the architecture audit and strict Clippy pass. Pinned
Rust independently runs the separate-crate control and prints
`4 8 9 12`.

A fresh RustGM client decodes the same artifact and initially stops at
`v4 method conformance requires its actual override`. `7839da8d0` keeps
an inherited initializer under its trait owner while installing the
implementation's own members. Four focused checks pass after the owner
repair. `dc9017c11` classifies imported method, type and constant links
by the installed declaration kind. It re-proves each constant override's
canonical type and lifetime contract and verifies that inherited links
refer to an actual trait default. The two focused constant checks pass,
including the required generic constant regression. A fresh client of the
same 8,439-byte packet now passes imported conformance and fails at
`source original preflight omitted its actual function header`.
Fresh client body reconstruction and execution, and the open request to
the actual supertrait, remain pending in the frontier worktree. Evidence is
under `obj/qualification/fast/1795164896161285346/` and
`obj/parallel/v14-associated-constants/provided-stable-v2/`.

Macro deprecation still requires its own published definition and
expansion-time diagnostics. The independent staged reference diagnoses a
deprecated macro and accepts a call through an annotated `use` alias of a
current function. [Deprecation CLI regression](../dev/tests/crate_linking/versioned_clients.rs)

All four frozen Unicode consumers match their declared output on pinned rustc
with prebuilt std at opt-level 2. RustGM's XID scalar sweep matches all three
native executions against the original `std` bring-up graph and locked
`unicode-ident` 1.0.24. The frontend emits 7,318,968 LowIR bytes in 46.78 seconds
with zero skips, 14 unproven obligations and 50 waivers. Source hashes, the unit
plan, link manifest and runtime receipts are under `obj/parallel/m4-xid-frozen/`
in the frontier checkout. The small-vector replay adds the three locked
`tinyvec` cfg values to the shared session for this input. A scan of 2,132
other source files finds those values only in two stability attributes. The
replay reaches `ArrayVec::append` in the locked Rust 2018 `tinyvec` and stops
at its custom-message `assert!`. The original replay is under
`obj/parallel/m4-small-vectors-frozen/`.

Source panic dispatch now selects the original `panic_2015` handler for
direct legacy panic calls. A custom-message legacy assertion resolves the
caller's `panic!` and preserves the actual message tokens. The focused
selection passes all 118 macro-metadata cases, the caller-scope cases and
the existing guard check. The 2015 and 2018 native clients each print
`45 46 41`, matching the independent compiler. The modern wrapper retains
its provider edition. The small-vector graph passes expansion and stops at
coherence on `impl<A: Array> Into<Vec<A::Item>> for TinyVec<A>` at
`tinyvec.rs:1427`. It emits no artifact. Evidence is under
`obj/parallel/legacy-panic-native/` and `obj/parallel/m4-small-vectors-legacy/`.
[Pinned Rust assertion dispatch](https://github.com/rust-lang/rust/blob/88d9e12ae/compiler/rustc_builtin_macros/src/assert.rs) ·
[Pinned Rust panic dispatch](https://github.com/rust-lang/rust/blob/88d9e12ae/compiler/rustc_builtin_macros/src/edition_panic.rs)

Coherence can now retry a deferred query with fresh existential variables for
its projected inputs. It uses absence from that broader query only when the
actual crate perspective still closes the inputs. All seven reduced conversion
cases match pinned Rust, including an exposed projection and two genuinely
competing impls. The existing fundamental and cross-crate regressions pass.
The unchanged small-vector build clears conversion coherence and reaches
`tinyvec.rs:154`. The generated `DerefMut::deref_mut` body reports
`unknown enum variant`. The run emits no artifacts and leaves the skip list
empty. Evidence is under `obj/parallel/m4-into-overlap/` and
`obj/parallel/m4-small-vectors-projection/`.

The enum method reduction reproduces that error with a direct
`Self::Target` signature. The annotation walk now honors the projection
already retained for that path. The enum variant and qualified function
controls pass, and eight reduced type-check outcomes agree with pinned Rust.
Two generic bodies reach the ownership and destruction gate in the loan
inspection view. The focused execution test instantiates the generic method
and lowers a concrete mutation through its returned reference. Evidence is
under `obj/parallel/m4-mirrored-variant/`.

The optimized `104856e7e` compiler clears the enum signature error on the
unchanged small-vector graph. It reports 12 body blockers in the three
`tinyvec` modules and the consumer. They concern projection normalization,
finding an unambiguous associated type implementation and comparison method
selection. The consumer itself reaches a projection-normalization error.
The run leaves the skip list empty and emits no artifact. The strict
309-input `core` replay with the same compiler still reaches the division
transport gate with zero bring-up records. Receipts are under
`obj/parallel/m4-small-vectors-associated-path/` and
`obj/parallel/strict-core/latest/`.

The reduced small-vector call fails with the real `IntoIterator<Item =
A::Item>` bound even when the inherent method body is empty. An equivalent
local trait, a direct projection through the same generic array impl and
ordinary associated type method inputs all pass. Both compilers reject a
wrong scalar argument. A trait impl using `IntoIterator` also reaches an
associated equality ambiguity before its body is needed. The reductions
are under `obj/parallel/m4-arrayvec-extend/`.

Three additional controls use the unchanged original `core`, `alloc` and
`std` sources with the reduced application as the guest. The direct array
bound and both projected bounds produce LowIR. The projected calls pass
with explicit and inferred element types. The first explicit run reaches
its 75-second limit, and the retry finishes in 45.61 seconds. The inferred
run finishes in 45.67 seconds. An empty implementation of the original
`core::iter::Extend<A::Item>` reproduces the normalization error at the array
argument. Pinned Rust accepts that source. RustGM reaches the error after
35.48 seconds with an empty skip list. The implementation body can be reduced
to `let _ = iter`. The same original library graph accepts an `Extend<u32>`
implementation with inferred array elements. It also accepts the projected
`Extend<A::Item>` call when the iterator type is explicitly `[u32; 3]`. Both
controls produce LowIR. Evidence is under
`obj/parallel/m4-arrayvec-extend/original-iterator-controls/`.

Normalization and identifier
consumers require independent guest feature scopes because the Cargo graph
enables `std` for `unicode-normalization` and leaves it off for `tinyvec`.
[Declared Unicode outcomes](../pa35/tests/unicode-consumer-outcomes.json)

### Self-compile milestones

| Order | Work | Exit evidence |
| --- | --- | --- |
| 1. Unblock `std` | Trace the projected field's bound lifetime back from `Batch::publish` through `storage_roots` and field instantiation. Identify the source expression and distinguish a provisional signature root from a lost caller substitution. Put the fix at the producer. | A pinned-rustc positive that exercises the same producer, a genuine lifetime-escape rejection and a focused RustGM gate. Use the original `quota_v2` replay as the exact regression until a reduction reproduces its failure. Both original `core + alloc` and Linux `std` replay with their saved inputs, environment, empty skip list and limits. The invalid binder remains a hard error. |
| 2. Close M1–M3 | Complete the zero-skip retest of the 18 core consumers, carrying forward verified results where the inputs and affected paths permit. Repair the `Box::call_once` concrete callable or template issue, the B-tree `descend` lifetime mapping and Rc/Arc call syntax through `Deref` to `dyn Fn`. Continue original `std` from each concrete blocker through LowIR, static library artifacts for the abort profile, and native linking. | Complete core and alloc consumer receipts, including Vec and String after affected changes. The `std` println, fs, env and HashMap consumer matches its declared native outcome, followed by the remaining declared std cases. Record emitted artifacts and any remaining bring-up obligations or waivers. |
| 3. Close M4 and exercise crate transport | Vendor the locked `unicode-ident`, `tinyvec_macros`, `tinyvec` and `unicode-normalization` sources. Take editions, features and dependency edges from the frozen unit plan. Bring up the dependency graph and execute the Unicode consumers. In parallel, advance real per-crate export/import from `core` through `alloc`, the selected `std` graph and the compiler dependencies. | The four Unicode consumers match the declared outcomes. A fresh dependent reads authenticated metadata and links RustGM-generated objects. Cover the demanded generic and const bodies, resolved identities, proof inputs and native links with cross-crate positive and rejection controls. Guests can expose later source failures while this transport is being built. |
| 4. Build stage 1 (M5) | Freeze the stage procedure, including the locked graph, unmodified `dev/src/lib.rs` and `dev/bin/rustgm.rs` and their reachable sources. Use the pinned rustc-built stage 0 to compile target Rust, then use the pinned CPPGM object path to link the compiler. Start with a CLI invocation that exercises a real input and drive remaining failures from the compiler's demand list. | The resulting stage 1 executable, selected through `RUSTGM_TEST_APP`, passes every committed PA1–PA24 text oracle. Record the executable hash, expected and executed case counts, and exact receipts. |
| 5. Rebuild and qualify (M6–M7) | Have stage 1 rebuild the same frozen graph. Work through strict-mode blockers by cause, starting as soon as each crate can be replayed. Account for the existing core/alloc and std obligations and waivers individually. Rerun both stages strictly from clean directories for qualification. | Compare every crate's LowIR and metadata and the linked compiler byte for byte. Require zero bring-up records and zero trapping stubs across both strict builds. Exercise the perturbed-input comparison failure and the one-record strict-gate failure required by PA45. |

The target-unit executor rejects explicit bring-up options before launch and
checks successful compilation stderr for bring-up records before publishing
LowIR or metadata. A successful command with a bring-up record or a missing
artifact quarantines the unit's complete set of emitted peers. Its dependent
crates stay blocked. All 23 target-unit and six comparison tests pass. The
repeat fixture builds emit the same LowIR, library metadata and native-link
manifests in independent directories. Receipts are under
`obj/parallel/compiler-execution-freeze/strict-native-manifests/`.
[Executor](../scripts/target_units.py) · [Controls](../scripts/tests/test_target_units.py)

The binary-root planner freezes the original `dev/bin/rustgm.rs` as the sole
target and its dependency on `dev/src/lib.rs`. The pinned Cargo 1.98.0 run for
`19b7bb08c` records six units for Linux x86-64 with abort, opt-level 1 and overflow
checks. The four registry archives match `Cargo.lock`. The graph retains each
unit's edition, cfg features and extern aliases, including the `alloc` feature
on `tinyvec` and `std` on `unicode-normalization`. The two compiler targets and
`unicode-ident` have no package features. All 35 planner and snapshot controls
pass. The exact tool, source and graph pins are under
`obj/parallel/compiler-dependency-freeze/` in the frontier checkout. Compiler
artifacts and the strict sysroot graph still require execution. [Planner](../scripts/cargo_plan.py)

The execution profile freezes those same six units from `dd7cd52df` at the
target executor's current opt-level 0 with abort and overflow checks. All
six units pass the executor's profile validation. Opt-level 1 was the sole
profile gap in the earlier inventory. Dependencies, features, editions,
root and unit order match. The receipt is under
`obj/parallel/compiler-execution-freeze/`. The selected sysroot units and
per-crate artifacts still need to join this plan.

The six-unit Cargo inventory contains no sysroot dependency edges. The strict
executor selects source-client mode from the dependency closure, so the
combined inventory must bind each crate to its actual `core`, `compiler_builtins`,
`alloc` and `std` artifacts. The current prelude adapter admits only `no_std`
source clients. The frozen `rustgm` binary uses `std` and returns
`std::process::ExitCode`. A reduced client should resolve the imported edition
prelude and execute the imported startup specialization for its actual
`Termination` implementation before the full binary replay.
[Executor](../scripts/target_units.py) · [Prelude](../dev/src/resolve/standard_imports.rs) ·
[Startup](../dev/src/sema/entry/startup.rs)

`4ba012e31` selects the v14 source transport for the Core unit and
includes that profile in its artifact identity. `9286212a7` keeps
Cargo's sysroot-unit marker and accepts the two pinned target-spec
queries needed for combined planning. All 43 planner and target tests
pass. The pinned Cargo invocation joins the original compiler and
stdlib lockfile closures in a 28-unit graph with the original
`rustgm` binary as its sole root. It has 22 target compile steps and
three host compile and run pairs. All six Cargo compiler invocations
are queries. The target plan exposes 42 `noprelude` edges across the
six application and dependency targets. Three sysroot steps also
need their host script outputs. The graph and dry-run receipts are
under `obj/parallel/strict-compiler-unit-graph/combined-v3/` in the
frontier checkout.

`3d6e8fbd5` preserves all 42 `noprelude` edges as explicit-only externs.
The 23 target planner tests and three focused linking tests pass. Pinned
Rust agrees on explicit imports and hidden implicit paths, and the three
source-client controls agree on the core alias selected during injection.
The unchanged combined graph ran the original `compiler_builtins`, `libc`
and `std` host scripts with 141, 27 and 8 directives. All 22 target units
now have expressible commands and authenticated host inputs. Target
compilation still awaits the original artifacts. The evidence is under
`obj/parallel/cargo-noprelude/` and
`obj/parallel/strict-compiler-unit-graph/combined-host-v1/`.

The first strict executor replay reached `core` and rejected its root
`implicit_provenance_casts` lint. A pinned Cargo command capture refused all
target compilations and confirmed that Cargo supplies `--cap-lints allow`
for the build-std `core` unit. `410b5bbd3` applies that cap to build-std
targets and includes the direct extern bindings in their consumer identities.
The corrected unoptimized replay reached the 120-second `core` timeout with
no diagnostic and no artifacts. The 25 planner, six comparator and five
linker checks pass. Evidence is under
`obj/parallel/strict-compiler-unit-graph/{cargo-core-command-v1,combined-target-v2}/`.

The fresh optimized observer build exhausted its 285-second limit.
The earlier optimized `b25c216b5` pin then replayed the corrected exact
unit plan and reached the associated-constant publication gate in 36.80
seconds. Unit `core` emitted zero bring-up records and no accepted artifacts.
The executor quarantined its empty LowIR file and blocked dependent units.
The command, compiler provenance and unit receipt are under
`obj/parallel/strict-compiler-unit-graph/combined-target-v3/`.

`3ef6b79e9` requires LowIR, metadata and a native-link manifest for every
unit. The terminal record carries the binary's identity, defining edition
and typed dependency inventory. Its publisher waits for LowIR serialization
and flush. Both dependency inputs reject the terminal format. The target
graph repeats with byte-identical artifacts, and the encoded provided-body
consumer emits the new terminal and link records with the same LowIR.
Three focused CLI checks, 29 script tests and all six target-graph outcomes
pass. Evidence is under
`obj/parallel/compiler-execution-freeze/terminal-artifact-validation/`.

The strict native linker takes the selected binary's LowIR and the library
closure in recorded target order. It checks the compiled dependencies against
their actual metadata arguments, verifies input and tool hashes, and lowers
each unit to a checked CPPGM object. It applies the binary's ordered native
library directives and records the published executable hash. Two independent
builds of the strict library-and-binary control link to byte-identical
executables and run successfully. The controls reject a reordered closure, an
omitted transitive library and a changed linker pin. The actual compiler link
still awaits its strict sysroot and compiler artifacts. Receipts are under
`obj/parallel/compiler-execution-freeze/native-closure-probe/production-qualified/`.
[Stage linker](../scripts/link_self_hosting.py) · [Native manifest](../dev/bin/link_manifest.rs)

The PA45 stage comparator checks the same frozen graph and executor, the
stage 1 compiler pin used by stage 2, and the actual strict compiler
diagnostics. It compares every unit's LowIR and metadata and then the linked
compiler byte for byte. Its controls cover an altered artifact, a bring-up
record, missing binary crate metadata, the wrong stage 2 input and linked
output drift. The real stage comparison awaits the per-crate artifacts and
both linked compiler builds. [Comparator](../scripts/compare_self_hosting.py) ·
[Controls](../scripts/tests/test_compare_self_hosting.py)

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
native linker, native inputs and link order. Retain LowIR, metadata and the native
link manifest for every unit, including the compiler binary. Link the binary with
its recorded dependency closure and use the per-crate artifacts for comparison.
Run `scripts/course.py paN --app "$RUSTGM_TEST_APP"` for each of
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
   once per published header. For a crowded receiver bucket, choose a nested
   path that separates rigid headers. Store owner IDs by constructor at that
   path, with one ordered fallback list for headers that give no rigid answer.
   For a rigid query at that path, merge the matching and fallback lists in
   their existing order. Track how often an unresolved query needs the whole
   bucket. Evaluate this index in both lookup and coherence while preserving
   wildcard, pending, projection and opaque candidates. The current lookup gathers and sorts a
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
   The search frame now passes its resolved concrete, opaque-free predicate
   from the completed-proof lookup into candidate indexing on the same startup
   path. The handoff takes constant work in the width of the trait input.
   Pending headers are still checked on the live goal. Normalized and resumed
   paths resolve their actual inputs. Six focused tests pass, including caller
   premises, projected headers, callable aliases and negative implementations.
   Frozen controls run on immutable optimized binaries with eight A/A
   observations and six ABBA or BAAB blocks each. With 32 distinct requesting
   goals, 8, 64 and 256 trait arguments retire 0.50%, 1.42% and 2.48% fewer user
   instructions, respectively. The ordinary control retires 0.09% fewer. The
   LowIR bytes match in all observations. On the frozen original core and
   `unicode-ident` graph the ABBA and BAAB blocks retire 0.765% and 0.759%
   fewer instructions. Paired wall time moves by −2.10% and −9.79%, with A/A
   observations of 51.51 and 66.49 seconds. All ten runs agree byte for byte on
   LowIR, native-link manifests and diagnostics. Each reports 46 unproven
   obligations, eight waivers and zero skips in the shared bring-up profile.
   Raw observations are under
   `obj/parallel/trait-resolved-inputs/{paired-pmu,whole-graph}/` in the
   frontier checkout.

   Measure the same header across different requesting goals. The declared
   effect-qualified conditions, the predicate's parameter uses and the
   decisions about vacuous binders can be derived once per validated owner and
   effect. Keep the resulting ordered clauses as canonical IDs and record which
   roots need trial-local type, const or lifetime substitutions. A canonical
   clause lowered out of an empty binder can also be shared across owners at
   the same effect. Test whether a trial can copy only the bindings it changes
   while retaining unaffected roots by ID.
   The fresh captures, local type variables and proof evidence still belong to
   the trial. Count distinct prepared owners and effects, unique clause and
   application IDs, dependent roots and replayed edges. Hold trial count and
   header width fixed while varying the fraction of shared roots. This exposes
   repeated work even when whole-input timings grow linearly.
   [Trial construction](../dev/src/sema/traits/selection/search/frames.rs#L166) · [Header instantiation](../dev/src/sema/traits/selection/instantiation.rs)

   The declared-condition prototype holds the candidate and trial count at
   128 in each shared-owner and distinct-owner case. At widths 8, 32 and 64,
   it retires 0.46%, 0.54% and 0.57% fewer user instructions in the shared
   cases. The distinct cases retire 0.08–0.16% more, and the two ordinary
   controls retire 0.05–0.10% more. All 272 samples emit identical LowIR.
   Paired wall differences fall within A/A variation. The review identified
   optional cache reservations on the mandatory source ledger. A separate
   bounded-pool candidate saves 431 and 3,903 charged instantiation steps at
   widths 8 and 64. The mixed ordinary control adds 67 lookups with zero hits.
   The original-source observers reach their deadline before reporting
   eligibility counts. The current changes remain experimental. Raw data is under
   `obj/parallel/trait-header-reuse/paired-pmu/` in the performance checkout.

   Impl instantiation now streams canonical formals and reads the existing
   type ID for implicit `Sized`. The fixed 32-trial cases hold owner count,
   inputs and output constant between revisions. All ten optimized controls
   retire 0.15–0.31% fewer instructions, including the distinct-owner and
   lifetime-only controls. Wall time and cycles remain within A/A variation.
   All 340 timed outputs match, and 16 optimized positive and rejection
   controls agree on LowIR and diagnostics. Four focused public tests pass.
   Evidence is under `obj/parallel/trait-parameter-inputs/` in the performance
   checkout.

   Canonical trait arguments already have a slice ID. The inference predicate
   keeps an owned argument vector inside a boxed application. Instantiation
   copies the canonical argument slice to a temporary vector and builds the
   inference vector. Isolating a trial maps every argument into a new vector
   even when the trial imports zero graph edges. Test a canonical application
   ID with a sparse overlay for inference-local slots. Reuse fixed arguments
   and retain any const, region, normalization or slot facts that require local
   processing. Measure the cold application preparation and the warm work as
   the fixed width grows with the number of affected slots held constant.
   [Inferred predicate](../dev/src/types/infer/predicates.rs#L17) · [Trial input mapping](../dev/src/types/infer/refinement.rs#L91)

   The inference arena registers a new local term even when its value is
   already a canonical type. A shared vector of local term IDs would still
   bind the representation to a single trial. Reuse the canonical application
   and materialize local terms only where a relation needs them. For equality
   between two applications, compare unchanged canonical slices by ID before
   visiting their changed positions. Keep the full comparison when the
   canonical slices differ and the solver has no indexed evidence for a
   rejection. Count local term creation and argument visits separately from
   copied graph edges. Zero imported edges currently allows both costs to grow
   with argument width.
   The existing slice ID captures whole-list equality. For wide lists with
   different IDs and mostly equal positions, compare a shared fixed portion
   plus indexed changed slots with interned blocks that can be compared by ID.
   Use both common-prefix and scattered changes. Include short applications
   so extra metadata and indirection are charged on ordinary inputs.
   Carry the canonical application through construction and trial import
   so warm lookup can read its existing identity directly.
   After one pass to prepare a width-`W` base, target constant work for an
   unchanged application and work proportional to the number `K` of changed
   slots for related applications with the same base. For independently
   formed bases, an interned tree of fixed-size blocks can skip equal
   subtrees, with up to `O(K log W)` visited nodes for sparse differences
   when the two trees have the same shape. Keep a flat representation for
   short lists and compare cold interning, warm relations and retained bytes.
   [Local term allocation](../dev/src/types/infer.rs#L258) ·
   [Argument relations](../dev/src/types/infer/predicates/equations.rs#L79)

   Deferred canonical body requirements now check the existing completed
   proof map before constructing an inference predicate. A hit charges one
   lookup step, publishes the existing evidence, and creates zero inference
   terms. Immutable development binaries agree byte for byte on semantic
   output and diagnostics for seven sizing and formation positives, an
   unpaid bound rejection and an ordinary scalar control. Formatting, the
   architecture audit and strict Clippy pass. The initial focused integration
   build hit its development deadline. Optimized timings remain pending.
   The controls and receipt are under `obj/parallel/canonical-hit-placement/`
   in the performance checkout.
   [Deferred canonical requirements](../dev/src/sema/bodies/trait_goals/canonical.rs)
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

   A fixed-size control has 32 distinct outer goals with one compatible impl
   apiece. Each impl requires a child trait proof. Both variants declare the
   same inventory and differ only in whether their conditions all name the
   same child or use different children. A second pair adds an identical
   unrelated caller premise. All four variants emit identical LowIR and pass
   the same native sum check with the instrumented compiler and pinned rustc.

   | Caller and child pattern | Distinct canonical fingerprints | Trials | Charged trial setup steps |
   | --- | ---: | ---: | ---: |
   | Neutral, distinct children | 160 | 160 | 2,464 |
   | Neutral, shared child | 36 | 160 | 2,464 |
   | Unrelated premise, distinct children | 162 | 194 | 2,600 |
   | Unrelated premise, shared child | 38 | 194 | 2,600 |

   The probe counts fingerprints of mode, source policy and predicate at
   canonical search frames. Full reuse keys also need the caller and registry
   dependencies. Every trial imported zero
   inference edges. Both neutral variants had 32 distinct top-level body memo
   keys and 32 cold lookups. The memo records the same 30 hits in both premise
   variants. The fixed-size pairs expose repeated child search behind different
   parents even when a whole-goal cache sees distinct requests.
   Each of the 160 neutral trials receives a one-owner candidate list and
   allocates one local input term, despite importing zero inference edges.
   The shared variant has 124 repeated fingerprint visits. This is an upper
   bound on exact-fingerprint reuse in this control. The safe share depends
   on complete eligibility and evidence at each visit.
   Prototype reuse at the child frame. Measure registry and effective caller
   dependencies from the whole search, including alternatives and refutations.
   In the repeated family the target is one child search per compatible context
   and a lookup per requesting parent. The sources, compiler hash, native
   checks and raw counters are in the frontier checkout under
   `obj/parallel/trait-shared-child/`.

   The local query's completed map can reuse a child within one search.
   Cross-query child reuse is opt-in. The body allocates its reusable proof map
   on the canonical requirement path for qualifying sizing goals, while the
   inferred body goal path probes the neutral-caller memo only at the root.
   Start the child experiment with debt-free closed goals under a fixed,
   validated registry and neutral caller. Tag selection mode and source
   policy in the reuse identity. Count completed eligible children, searches
   actually avoided, and retained entries across distinct roots before testing
   selective dependence on non-neutral caller premises. This first prototype
   can expose the gain from placement of the memo without first implementing
   a caller premise dependency index.
   Preserve selected proof structure where a consumer reads it. For
   example, the Deref projection path collects implementation evidence from
   child nodes. Reusing only its root source would lose those descendants.
   Use a shared proof reference or decline reuse for that consumer. For
   later reuse across different premises, track which premise buckets the
   search consulted, including empty buckets that affected candidate
   eligibility. Validate their identities with the existing premise index.
   Keep scope- and witness-dependent proofs on the original path until
   replay can restore their local bindings.
   [Child completion](../dev/src/sema/traits/selection/search/continuation.rs#L337) ·
   [Canonical body proof map](../dev/src/sema/bodies/trait_goals/canonical.rs#L171) ·
   [Inferred body memo](../dev/src/sema/bodies/trait_goals/inference.rs#L515) ·
   [Deref proof consumer](../dev/src/sema/traits/selection/search/chain.rs#L17)

   The bounded candidate for neutral body queries replays 31 selected child
   DAGs and reduces validated trials from 96 to 34 on the shared input.
   The 64 earlier trials are unchanged. It retires 8.24% fewer user
   instructions there and 0.74% more on the neutral distinct-child control.
   Both premise variants and the ordinary scalar control stay within 0.11%.
   The fallback control reaches the optional limit twice, continues through
   ordinary search and also records four successful replays. Across the
   positive controls all 134 ordered proof graphs, semantic output, LowIR,
   diagnostics and native outcomes match. The frozen rejection and all 170
   measurement observations match in output. The 2.47% paired wall reduction
   remains within the 2.91% shared-case A/A spread. The production gates pass.
   The integrated early source guard reduces 44 child probes to zero across
   six source-backed body queries, with identical outputs and diagnostics.
   Five standalone controls preserve the measured revision's exact replay,
   frame and trial counts. The bounded private harness attempts leave private
   execution pending. Evidence is under `obj/parallel/child-proof-reuse/` in
   the performance checkout.

   A shadow observer on the original `core` and `unicode-ident` graph
   counts 132,251 source-backed body roots and 24,372 ordinary child descents.
   It prepares 715 ordered DAG captures and finds 658 warm completions that
   could reuse the current recipe shape. They account for 2.70% of ordinary
   child descents and 15,816 recorded query steps. The observer leaves normal
   search in place and preserves output and diagnostics. No replay runs on
   this path. The source admission guard stays in place while work moves to
   trial frame copies and predicate graph imports. Evidence is under
   `obj/parallel/source-child-headroom/` in the performance checkout.

   The shadow counts come from neutral memoizing roots on that core and
   Unicode graph. A further 2,225 descents reach the explicit-argument gate.
   Admitting those to a shadow still requires exact argument identities and
   the remaining query, allocation, alternative and proof checks. Root reuse
   across caller premises also requires re-proving their influence on the
   selected candidate. The candidate-header relation itself builds a mixed
   argument pair vector and a type-only pair vector. `begin_trial` calls
   `relate_predicates` directly in 588 of 26,865 sampled chains. The next
   experiment counts relation widths and allocations, then tests a stack
   singleton with the existing ordered checks and debts.

Source automatic-trait classification now compares canonical provider IDs in
the compact language-role cells. A selected role still checks its declaration
and ordinary application. The imported catalog retains its ordered origin
checks. The exact optimized `5ac9117de` and `a134aa651` comparison retires
**1.60–1.61% fewer instructions** on the original `core` and `unicode-ident`
bring-up graph, from 96.12 billion to 94.58 billion. All ten observations
match LowIR, native-link manifests and diagnostics, and both native scalar
sweeps agree. The graph still reports zero skips, one unproven obligation and
seven waivers. Paired wall changes fall within the 25.4% A/A spread. Focused
source controls retire 0.35–1.56% fewer instructions, with the ordinary scalar
control flat. The source child-proof guard still records six memoizing queries
and zero probes. Evidence is under `obj/parallel/compact-provider-marker/`
in the performance checkout. [Provider role lookup](../dev/src/sema/compiler_items/binding_views/compact.rs)

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

The initializer and anonymous-body candidate extends sparse ordinary cleanup
to both fresh paths. The source observer counts 32 initializer and 96
anonymous materializations at every width, with the same reached rows and
steps as the type store grows from 61 to 8,253 entries. Its four focused
public tests pass. Immutable optimized `19b7bb08c` and `eca64c7b1` runs match
in all 136 observations. The 8,192-alias input retires 0.77% fewer instructions,
and the ordinary control retires 0.22% more. The integrated bounded variant
chooses dense storage through 32 types after type preparation. The observer
confirms dense storage at 31 and 32 types and sparse storage at 33, with the
same three reached rows and six steps per materialization in that control.
The ordinary instruction change is +0.01% within the 0.034% calibration
spread. It retires 0.82% fewer instructions in the widest control, with costs
of 0.22% at 33 and 0.11% at 61 types. All 204 optimized observations preserve
output and diagnostics. The twelve preflight outcomes match pinned Rust and
the four focused checks pass. Wall time stays within calibration spread, and
the full source graph remains unmeasured for this change. Evidence is under
`obj/parallel/initializer-sparse/` and `obj/parallel/bounded-fresh-destruction/`
in the performance checkout.

The inferred predicate now stores the trait identity and effect inline when
the application has no explicit arguments. The record stays at 24 bytes on
x86-64. A debugger sample counted 294 such goals among the first 512 canonical
trait constructions in body semantics. The immutable optimized pair at
`558ea1a22` and `7e15ac62d` retires 0.252% fewer instructions in both paired
blocks on the original `core` and `unicode-ident` bring-up graph, from 94.501
billion to 94.263 billion. All ten observations preserve LowIR, diagnostic
and native output. The focused comparisons preserve 98 ordered selected
proof graphs, with no extra source child-proof probes and a flat scalar
control. Wall time and cycles vary across blocks. Three focused public
tests and fourteen CLI outcomes pass. Four private tests remain pending
after the harness builds reached their deadlines. Evidence is under
`obj/parallel/inline-empty-trait/` in the performance checkout.

Within comptime body checking, the immutable report owns up to 32 successful
lint-owner receipts for one checker. Each caller still authenticates its own
header and pays the original lint debit. An exhausted warm attempt runs the
original ordered scan to preserve the first failed witness and charge. The
shared controls replay 31 of 33 requests and retire 1.18–2.93% fewer
instructions. All 238 timed focused outcomes match. The original `core` and
`unicode-ident` bring-up graph retires 0.865–0.867% fewer instructions in both
paired blocks, from 94.263 to 93.447 billion. All ten outputs, diagnostics and
native results match. The graph still charges 31,992,265 lint read steps
during body checking on both compilers. Wall time varies by block. Strict
production and CLI Clippy pass, and the ordered budget test passes through
the disposable observer CLI. The joined development compiler reproduces all
ten frozen output and diagnostic checks. The private harness reached its
deadline before executing the selected inventory. Evidence is under
`obj/parallel/lint-owner-witness/` in the performance checkout.

Completing trait search frames in their stack slots removes six compiled
completion copy sites. The original `core` and `unicode-ident` bring-up graph
retires 0.803% fewer instructions, 0.900% fewer load operations and 1.233%
fewer store operations in the paired run. All six observations and both
native executables match. Wall time and cycles stay inside A/A variation.
Nineteen optimized outcomes and 98 ordered selected proof graphs match,
including the original trial, frame and replay counts. All nineteen outputs
also match after integration with version fourteen transport. Both source controls
retire 1.37–1.39% fewer instructions. The private harness reaches its build
deadline before running cases. Evidence is under
`obj/parallel/trait-frame-import/` in the performance checkout.

Starting inference type import with its first visit in a local variable
removes the eager one-element traversal allocation. The original
`core` and `unicode-ident` graph counted 2,966,218 eager allocations.
2,820,202 imports (95.08%) finish without scheduling a traversal. Predicate
preparation accounts for 2,623,198 initial visits, and 97.76% of those finish
directly. The matched reads, edge charges and first failure stay exact at
nine work-limit boundaries.

An immutable A/A and ABBA pair retires 0.335% fewer instructions, 0.396%
fewer load operations and 0.280% fewer store operations. All six graph runs,
70 focused observations, 98 ordered proof graphs and both native programs
matched. Observed wall time varied from 29.05 to 40.69 seconds in the
candidate pair. A/A also varied, leaving the wall effect unresolved.
Evidence is under `obj/parallel/trait-predicate-import/` in the performance
checkout.

Tracked inference imports now keep the first term mapping in the temporary
tracker. On the frozen original graph, 1,670,351 of 2,029,613 tracked maps
had exactly one type key. DHAT measures 8,238 to 30 tracked map allocations
on the source-backed shared control and 320 to zero on the distinct control.
The ordinary scalar control has 256 total allocations on both builds.

The immutable `f688afa46` comparison retires 0.726% fewer instructions,
0.558% fewer load operations and 0.613% fewer stores on the original graph.
Wall medians are 30.583 and 31.208 seconds within the A/A spread of
29.200 to 40.338 seconds. All six graph observations and both native results
match, as do 70 focused observations and 98 ordered proof graphs. Six direct
mapping controls preserve the results and charged work. The optimized code
is integrated at `942bb15f6`. Raw counters and allocation profiles are
under `obj/parallel/trait-first-map/` in the performance checkout.

Equality publication sees 18,903 of 36,130 original-graph requests with
`'static` as the sole publishable lifetime name. The isolated `33de8a3db`
candidate skips SCC reconstruction on those requests when a conservative
bound fits the original work limit. It retires 0.080% fewer instructions,
0.111% fewer load operations and 0.087% fewer stores. Wall times stay within
A/A variation. Six whole-graph samples, two native results and 42 focused
observations match. A typed driver confirms the work-limit boundary and
named-to-static equality after the private harness reaches its build
deadline. The candidate remains in the performance checkout, with receipts
under `obj/parallel/proven-static-region/`.

Comparison calls make 70,642 owner reads on the original graph. The observer
finds 64,988 reusable reads, covering 9,765,448 witness-validation steps and
214,408,088 bytes of witness rows. Sharing the existing table displaces a net
2,343 comptime hits. The paired `128d56267` experiment preserves the work
ledger, diagnostics, LowIR and native results. Retired instructions fall
2.85% in both orders. The paired wall medians rise from 29.650 to 37.162
seconds in ABBA and from 29.575 to 31.610 seconds in BAAB. User cycles increase
in both orders. Separate calibration windows run the same baseline in 29.4
to 49.2 seconds. The candidate remains isolated. Evidence is under
`obj/parallel/comparison-owner-receipts/` in the performance checkout.

The isolated `a99d67a12` candidate reuses just one adjacent marker witness
within the comparison request. It saves 1.207 billion retired instructions
on the frozen `core` and `unicode-ident` graph, a 1.316% reduction. The
focused control replays, original LowIR and native scalar sweeps match.
The exact typed driver preserves the first budget failure and charged work
after the standard private harness reaches its build deadline. In ABBA the
baseline runs take 49.988 and 31.865 seconds while the candidate takes 43.005
and 31.311 seconds. The elapsed-time effect awaits a stable reversed pair.
The report and immutable pins are under
`obj/parallel/comparison-marker-receipt/` in the performance checkout.

`12a2c0cac` uses stack slices for the singleton trait equations. The original
core and Unicode graph sees 389,950 single-pair candidate headers out of
780,760. Across the measured entry paths, the observed widths and allocation
sizes account for an estimated 806,162 temporary allocations and 19.53 MB.
The source-backed allocation control confirms a drop of 1,946 blocks, and
the ordinary and width-32 controls keep the same allocation counts. Six
focused tests preserve relation order, budget failures and bound lifetimes.

The immutable pair retires 0.192% fewer instructions, 0.196% fewer loads and
0.190% fewer stores on the frozen graph. All graph and native outputs and
all 70 smaller samples match. The widest controls use up to 0.013% more
instructions, and the ordinary control stays within A/A spread. Wall time
and user-cycle effects remain within the baseline variation. The report and
raw profiles are under `obj/parallel/scoped-singleton-equation/` in the
performance checkout.

The isolated `d7a306b6c` experiment reuses substitution buffers within one
trait search poll. It preserves all 34 checked ordered proof DAGs and matches
15 optimized controls. The source-backed control allocates 571 fewer blocks
and 52,438 more cumulative bytes. The cold control allocates 64 more blocks.
The original graph retires 0.090% more instructions, 0.125% more loads and
0.153% more stores in the immutable pair. Wall medians are 29.905 and
33.035 seconds in that block. The candidate median falls within the 31.757
to 33.613 second A/A range. All LowIR, link manifests and native outputs match. Disassembly and the observed call counts model about 100.8 MB
of additional explicit copying on the measured search route. The experiment
remains isolated. The report and raw profiles are in
`obj/parallel/trait-instantiation-scratch/` in the performance checkout.

`672467557` reads the existing canonical trait argument span in order during
instantiation. The paired graph preserves exact output bytes and proof order.
The observed graph reaches 390,810 nonempty argument spans. The source-backed
allocation control drops 1,943 blocks and 62,176 bytes at the removed collection
site. Four integration cases on the combined compiler cover mixed argument
order, nested binders and the wide constant client. They pass in 172.38 seconds.

ABBA and BAAB retire 0.327% and 0.331% fewer instructions, with about 0.45% fewer
loads and 0.52% fewer stores. The ordinary control retains 257 allocations and
adds about 1,300 to 1,450 instructions. Both orders show adverse elapsed
medians, with different magnitudes. In the latter order the same baseline
binary ranges from 34.585 to 54.351 seconds as sibling CPU occupancy moves
from 31% to 97% at effectively identical instruction counts. The paired work
reduction qualifies. The wall and cycle effects need comparable core occupancy.
The report is in `obj/parallel/canonical-argument-span/` in the performance
checkout and the integration receipt is in
`obj/qualification/fast/1795141587723920143/` in the frontier checkout.

`9c643b901` stores the common single explicit inferred trait argument in
the existing 24-byte predicate. The workers' adjacent immutable comparison
preserves 96 ordered implementation proof DAGs, eleven CLI input outcomes
and all six measured original-graph outputs. The source-backed control
allocates 7,825 fewer blocks, including 4,992 at matched trial and header
sites. The full graph uses 0.365% fewer retired instructions, 0.615% fewer
load uops and 0.560% fewer store uops. The focused width-17 control uses
about 0.41% more retired instructions. The ordinary scalar control uses
about 0.08% fewer instructions and about 7% more sampled cycles in both
orders. Whole-graph elapsed and cycle results remain inconclusive. One
candidate run received 35.807 seconds of task clock across 96.913 seconds
elapsed, and the same baseline changed from 30.599 to 40.957 seconds with
sibling occupancy. No updated RustGM-to-rustc elapsed ratio is available.
The report is in `obj/parallel/inline-single-inferred-argument/` in the
performance checkout and the isolated fast gate is
`obj/qualification/fast/1795141696010580141/`.

In the 26,865-sample original-core profile, 6,222 body samples lack a
visible trait-related stack frame. The two lint-provenance leaf symbols
have 151 body samples with a caller-site tracking ancestor and 195 with
a checked-tracking ancestor. Those ancestor counts overlap. The observed
settled-type resolution chain outside visible trait ancestry accounts for
0.42% of the whole profile across declarations and bodies.

A count-only observer on exact `9c643b901` reproduces the existing output
bytes, statuses and diagnostics on 14 focused pairs and the frozen
746-input core and Unicode graph. It observes 70,271 sealed caller-tracking
witness scans. Sharing the existing 32 receipt slots predicts 62,951
new tracking hits and displaces 770 existing comptime hits while adding
240. That saves 62,421 net physical scans, covering 3,957,568 witness
steps. The compiler must still pay its original work ledger. Every observed
replay debit preview succeeds. The separate tracking table and one-slot
request receipt predict fewer hits.

`f5e3fbdcf` applies that receipt reuse to direct, selected and unresolved
override tracking from the same source body checker. The exact adjacent
optimized pair preserves the original 31,992,265 lint read steps and
1,878,898 bytes charged during body checking. Both native caller-location
controls and all 66 measured outputs match. The paired original graph
retires 1.258% fewer instructions, 1.566% fewer load operations and 1.688%
fewer store operations. Same-binary instruction spread is 0.0085%.
The sampled wall and cycle changes are within baseline variation. The
ordinary scalar control uses about 0.05–0.07% fewer instructions.
Seven private test bodies pass through a disposable typed export. Linking
the standard private harness exceeded the development guard. The
qualification, graph and focused receipts are in
`obj/parallel/caller-tracking-witness/` in the performance checkout.

A review of the v5literal C++ compiler translation at `a59264bd` identifies
two designs to test on RustGM. Its class partial-specialization lookup
chooses an argument position and an exact type or outer template identity,
with an ordered fallback for unindexed candidates. RustGM currently
uses its open nominal-argument index only after the predicate resolves.
A shadow should count additional exclusions at deeper rigid type paths
and require every exclusion to agree with the existing header mismatch
before the first condition or budget failure. The original graph places
trial construction in 2,005 of 26,865 sample chains and header
instantiation in 1,580. The counts overlap.

On exact `f5e3fbdcf`, the shadow classified 204,141 of 810,102 trials
that passed the current index. All 204,141 reached an actual header
mismatch under legacy execution. The original 746-input graph and 15
focused pairs preserved status, diagnostics and output bytes. Nested
`Self` accounted for 174,278 of the mismatches. At least 87,676 of those
had zero allocated regions and zero parent debits, of which at least
35,612 also had canonical frames. A pinned debugger control straddled
the first query-debit cutoff by one step. The legacy and shadow paths
both changed from a confirmed mismatch to the original budget failure
at that boundary. A replay would need to preserve identity, original work
charges and reservation order.
The raw receipts are `graph-v2/receipt.json`,
`focused-v2/receipt.json` and `first-charge-v3/receipt.json` under
`obj/parallel/v5-rigid-discriminant/` in the performance checkout.

The repeat census on the same graph found 84,509 nested primitive
mismatches at root queries. Only 27 passed the broader admission check
that requires a canonical frame and unchanged proof state. All 27 keys
were distinct. The stricter fixed-root check admitted none. A focused
fixture recorded 13 same-debit repeats in distinct searches, including
12 repeats across different caller environment classes. The graph had
zero hits, so the scoped replay stays out of production. The graph and
seven focused pairs match archived outputs, diagnostics and original
counts. The receipts are `graph-v1/receipt.json`,
`focused-v1/receipt.json` and `graph-recurrence.json` under
`obj/parallel/v5-nested-self-repeat/` in the performance checkout.

The reference also records typed dependency events during analysis and
retains their first-use order. A shadow on exact `f5e3fbdcf` preserves
the archived Core and Unicode graph output and all 19 focused pairs.
Four dispatch walks inspect 622,431 slots. An ordinal index would reduce
those to 62,494 candidate visits and spend another 295,299 visits building
the lists. All 38,711 phase checks preserve the lists' lengths and order.
The invocation precheck alone inspects 197,562 slots without seeing an
`Invoke`. The earlier dispatch scan can record its presence without a
separate allocation.

Dependency discovery reaches 39 bodies in the original scalar graph.
In the instantiated control, growing from zero to 512 unrelated
assignments at a fixed 16 calls raises the visits from 665 to 14,489
while the 23 callable events remain constant. All 16 independent
focused native controls agree with pinned Rust. The census counts slot
visits. Evidence is under
`obj/parallel/v5-typed-operation-index/` in the performance checkout.
Shared normalization preparation remains a separate candidate, with
context-specific proofs and budgets attached to each caller.

The isolated invocation precheck candidate `9d6a6f10f` passed four focused
checks. The optimized pair matches all eight typed ledgers and the archived
original graph products. A/A followed by ABBA on that graph reports 0.019%
fewer retired instructions and 0.027% fewer retired loads. Graph wall and
cycle differences fall within the 22% A/A range. The long inert control
retires 0.006% more instructions and 0.041% more stores. The candidate stays
isolated while the performance lane investigates larger sources of work.
The receipts are under `obj/parallel/v5-typed-operation-index/invoke-presence/`
in the performance checkout.

The cleanup lifetime census on the original graph matched the archived
artifacts and diagnostics. Across 23,546 cleanup scopes, 1,390,136 of
1,869,108 seed pairs repeat the same active set at an adjacent point
in the same recording interval. The original solver already covers
1,547,871 of all 1,891,776 mapped seeds with full extents. A run model
uses 3.85 MB for headers and identities, compared with 14.95 MB of
original raw pair payload. These are aggregate payload counts.
Independent destructor controls keep five concrete cleanup scopes,
increase the region pairs from 110 to 1,888, and match the Rust and
RustGM native result. Candidate `72264239b` retains the ordered runs
through mapping and replays the original solver inputs. Its static
checks passed. Private execution remains pending after the harness
reached its build deadline.

The optimized pair matches the original graph, 17 focused outcomes and six
typed loan ledgers, including the first work failure. Exclusive A/A and
ABBA measure 89.615 billion to 89.842 billion retired instructions
(+0.253%) on the original graph. Both pairs agree. Loads rise 0.267% and
stores 0.318%. The two orders disagree on wall time and cycles, with
about 18.5% A/A variation. The ordinary scalar control uses 0.127% more
instructions and the long plain-loan control uses 0.115% fewer. Destructor
widths 32 and 128 use 0.978% and 1.532% fewer instructions. The candidate
stays isolated because of the original-graph increase. The next analysis
examines whether full extents can avoid materializing already-covered
seeds while preserving charged work and failure order. The counter and
preflight receipts are under `obj/parallel/v5-nontrait-schedule/` in the
performance checkout.

`4757cac6d` leaves ordinary seed order intact and charges an ordered
suffix for opaque parameter regions that the same identity table already
contributed as full extents. The census identifies 855,947 eligible seed
records, equivalent to 6.85 MB of cumulative mapped staging. Malformed
points retain the original path. The optimized candidate matches all 18
CLI comparisons and nine typed loan ledgers, including the first error.
An independent debit control matches 276,717 budget states.

All 36 counter observations preserve output hashes. The original graph
retires 0.102% fewer instructions, 0.112% fewer loads and 0.072% fewer
stores in the exclusive comparison. The two elapsed pairs are 9.12%
and 13.29% slower, and the same baseline executable ranges from
29.18 to 41.65 seconds across the paired runs. The change remains isolated
pending investigation of the elapsed regression. The optimized pair
and measurements are under
`obj/parallel/v5-nontrait-schedule/full-suffix/` in the performance checkout.

### Local annotation traversal

The observer on `f5e3fbdcf` records 970,361 active configured AST visits
across 375 original source units. It classifies 631,869 visits (65.12%) as
inert for this pass and 338,492 as possible annotation or context events.
The longest inert gap spans 7,809 visits. A control grows from 9 to 519
visits with three events and four extra work units throughout. The frozen
746-input graph and 12 focused comparisons preserve output bytes. The
focused cases also agree with the pinned Rust 1.98 outcomes.

The resolver's reference inventory misses 10,162 semantic references
responsible for 10,136 actual annotation events. The next observer uses
final configured source admission together with semantic-reference writes,
including references resolved during the pass. It compares their order
with the existing traversal and checks the first failing node across
work limits. It also counts the extra storage and writes needed to produce
the event stream. The old `d552a9635` graph profile attributes 203 of
26,865 sampled call chains (0.756%) to the pass. The census measures
eligible visits. Speed and storage improvements need a qualified candidate.
The census report is retained as `obj/parallel/local-annotation-census/result.md`
in the performance worktree. Its original-graph receipt has SHA-256
`dab127f3b42eff036244df0ea148df6bb99d7ffb9395c43c16d6874f99b953b2`.

The producer shadow matches all 970,361 configured traversal positions
in order. Reference publication during the pass creates 7,547 events at
later positions, which a sealed event list would miss. The instrumented
publication hooks leave 64 required events unmarked and keep 98 extra.
The dense prototype uses 14,985,904 bytes. It stays as an observer while
the worker identifies the remaining reference writers and evaluates a
more compact representation. The frozen graph and 22 focused outcomes
preserve output bytes. Thirty-four work-limit comparisons preserve
diagnostics, and the shadow predicts the exact failing node in all 31
traversal failures. The report is retained at
`obj/parallel/local-annotation-producer/result.md`. Its graph receipt
has SHA-256
`14d7d138c402c410d32fcefe42bb0b314ef889e0878afbeaaff4b8226ba747e0`.

### Macro expansion and ledger work

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
