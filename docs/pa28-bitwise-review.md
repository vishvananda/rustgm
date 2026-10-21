# AtomicU32 bitwise read-modify-write

This checkpoint adds exact canonical `fetch_and`, `fetch_or`, `fetch_xor` and
`fetch_nand` identities at appended intrinsic indices 9–12. All four are safe
runtime-only inherent methods with the existing U32 receiver, operand, Ordering
and result signature. Existing aliases, UFCS, method reification and whole-CFG
const eligibility retain the same typed intrinsic payload. Every Rust Ordering
is legal for these methods.

The implementation uses the frozen CPPGM backend's existing atomic load and
compare-exchange vocabulary. Each selected ordering branch performs one initial
Relaxed U32 atomic load into a private expected slot, then retries CAS. The loop
loads expected from local scratch, computes the selected U32 bitwise operation,
and supplies its result to CAS. Nand complements And with XOR against u32::MAX.
On failure the backend updates expected; only that scratch is reloaded. Success
returns the captured pre-CAS value. Receiver, operand and ordering expressions
are evaluated once. Even an unchanged desired value still executes CAS.

| Rust Ordering | CAS success code | CAS failure code |
| --- | ---: | ---: |
| Relaxed | 0 | 0 |
| Release | 3 | 0 |
| Acquire | 2 | 0 |
| AcqRel | 4 | 0 |
| SeqCst | 5 | 0 |

Relaxed failure is legal for all five success orders; the successful RMW retains
the requested order. Frozen CPPGM's analogous C++ helper uses the same
seed/scratch/retry scheme with stronger success-derived failure ordering for
Acquire/AcqRel/SeqCst. This frontend deliberately selects Relaxed failures rather
than copying that optional strengthening. No direct bitwise atomic opcode or
backend modification is introduced.

At runtime an operation performs one initial atomic load and N CAS attempts,
where N is at least one. There is no other atomic-object access. Each generated
shim has one U32 scratch slot, one Ordering-argument load and five fixed retry
loops. Each loop has a seed load, a CAS instruction and a failure backedge. Typed
invariants verify desired-value dataflow, scratch-address identity, failure
reload, and captured old-value return. Compiler construction has fixed bounded
work per shim; runtime retry count depends on contention.

The predeclared source corpus has 27 pinned full-object outcomes and all three
frontend views: 13 accepted, 13 rejected throughout, and one packed receiver
rejected at executable checking after a successful type view. Const calls,
retained skipped branches, aliases and unsafe blocks reject with the specific
RuntimeOnly rule; reified const calls retain their separate non-const-call error.
Opposing controls preserve canonical identity, shadowing, type/arity and packed
raw-address rules. Three fixture names were clarified after the retained baseline
observations; their bytes are unchanged and the contract records the old paths.

Six ignored-by-default native groups compare every ordering, U32 masks and
complement, zero/MAX values, reification/UFCS, ignored effects in shared/nested
storage, unchanged-value updates, and once-only source argument evaluation.
Native runs are explicitly uncontended. Failed-CAS expected updates rely on the
separately qualified pinned backend capability; typed loop invariants verify
that generated failure edges consume those updates. No concurrent-execution
stress or throughput claim is made.

At 8/32/128 callers, checking uses 443/1739/6923 body steps with eleven
canonical atomic methods and four demanded bitwise shims. The existing ordinary
no-Atomic control retains byte-identical output in all three views. Strict
all-target/all-feature Clippy, the 1,362-file audit, thirteen affected regression
tests and pinned upstream verification pass.

Exact source/native/scaling/gate evidence is in
`pa28/tests/atomic-bitwise-outcomes.json`. The initial local variable-shadowing
compile error, test-only missing equality trait assertion, and filename ownership
audit failure are retained separately. An additional equality hypothesis was
corrected after inspection: the local-shadow fixture imports canonical
AtomicU32/Ordering, so its four added deferred method records shift inspection
identities and LowIR symbol IDs. That exact disagreement is retained. The
separate ordinary fixture has no canonical metadata and matches the prior
subtraction checkpoint byte for byte. No active references, assignment
activation, full-release claim or CPPGM changes occur here. Other atomic widths,
min/max/update methods, fences, accessors and remaining memory/numeric families
remain separate work; scalar-only union and broader PA27 boundaries remain.
