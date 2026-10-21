# AtomicU32 fetch_sub

This isolated checkpoint adds exact canonical `AtomicU32::fetch_sub` as a safe
runtime-only intrinsic. Its identity appends index 8 without renumbering earlier
operations. The existing intrinsic payload, canonical inherent receiver,
reification, source-sensitive ordering policy and const eligibility paths remain
in use. Every Rust Ordering is legal for this RMW operation.

Lowering uses the pinned backend's existing `atomic_add_fetch` operation with a
wrapping u32 negation of the operand. Adding the original operand to the captured
updated value, also at u32 width, recovers the old value. Each selected ordering
branch performs one atomic operation and no other access to the atomic object.
The existing typed ordering switch translates Rust tags to backend codes
0/3/2/4/5 for Relaxed/Release/Acquire/AcqRel/SeqCst; no backend vocabulary changes.

The predeclared corpus contains 21 independently qualified full-object outcomes
and all three frontend views. Opposing controls cover direct, variable and
aliased orders, UFCS and reified calls, safe-to-unsafe callable conversion, shared
static storage, type/arity errors, and packed implicit receiver borrowing versus
raw address formation. Const calls reject specifically as RuntimeOnly, including
aliases, unsafe blocks and retained skipped ordinary/generic branches; reified
pointer calls retain their separate non-const-call rejection. Packed receiver
alignment remains an executable-checking obligation after successful type view.

Four ignored-by-default native groups compare old/new values, all five orders,
zero/MAX wraparound, reification and ignored effects in shared/nested storage.
At 8/32/128 callers, body checking uses 179/683/2699 steps and seven canonical
atomic methods. Only one subtraction shim is demanded. Its five branches each
contain one add-and-fetch, one u32 negation and one u32 reconstruction. It loads
only the u8 Ordering argument outside atomic operations. These are bounded work
and structural checks, not wall-time or concurrency measurements.

Strict all-target/all-feature Clippy, the 1,359-file audit and 15 affected source,
scaling and regression tests pass. The original test-only dead-code failure is
retained under `obj/parallel/pa28/subtract-first`; the corrected batch is under
`subtract-second`. The shared qualifier now supports explicitly distinct type
and execution outcomes while preserving existing uniform callers. Final source,
object, native and upstream evidence is in `pa28/tests/atomic-subtract-outcomes.json`.
No active references, assignment activation or full-release claim is included.

Further atomic widths/types, bitwise/min/max RMWs, fences and accessors remain
separate frontend work. Frozen CPPGM implements bitwise atomics using CAS loops,
so they require their own typed loop proposal and qualification. The remaining
memory/numeric intrinsic families and aggregate/zero-sized/wide volatile boundary
also remain. PA27 still has explicit raw-reference reconstruction, general byte
reinterpretation/provenance offsets, shared-derived interior writes,
pointer-bearing union Images and broader packed/pattern release obligations.
