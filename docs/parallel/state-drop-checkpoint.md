# Real unpolled construction and empty-capture future Drop

Two frozen programs now execute genuine construction and state-dependent scope
exit through the locked native backend. Neither enters its deferred infinite
loop. One source prelude deliberately provides no u8 Clone/Copy implementation.
This is a zero-capture, unpolled milestone. It does not qualify poll, completion,
capture/pattern effects, general cleanup or the full future runtime assignment.

The constructor remains a generated body for the actual public function, separate
from its one checked AsyncBody template. Future Drop has its own existing
FutureEntry key and real for<'a>fn(&'a mut Future)->() signature, with instanceNone.
It has no invented Drop DefId or ordinary Original. Its CFG projects the real
frame/state addresses, reads the state, completes an unstarted future and returns,
returns for Complete, and traps on Poisoned/unknown state. Native evidence covers
the unstarted path only. Completed/poisoned paths still need actual poll consumers.
Nonempty input/capture families and remaining formation debts explicitly fence.

Private non-Copy permits authenticate the full application, actual source family,
current arena, operation and signature worlds. The existing checked family and
ordinary region/initialization/loan/cleanup proof remain mandatory. A permit lends
the exact compiler state read through the generated preparation and Copy view;
ordinary and captured reads retain their actual source Copy obligations. The view
also checks the actual immutable body pointer. No move-through-reference exception
or global primitive Copy implementation was added. Existing ordinary/const proof
wrappers and the const ownership token remain intact.

The existing structural recipe key retains full future/capture/lifetime identity.
Method::Future stores one full TypeId and owns capture cleanup without duplicate
field recursion. Future-only cleanup requests use a paid optional cold component;
ordinary4B Instance rows remain unchanged. The existing active-family map caches
completed Drop families by full key, with source/store revalidation before warm
reuse. Native Drop lookup reuses the existing generated-adapter index. Poll does
not enter that index. The same moved forwarding map remains the sole native
generated-family authority; missing/forged ordinary bodies cannot supply it.

Actual compiled measurements: Permit40, outer factory Permit16,
generated index key12/KV24/handle32, Work40, Method8 and
Recipe32. Owner20, Body456, Kind24 and Builder1776 stay unchanged. Actual cold
vectors, boxes, index growth and source validation work are prepaid. There is no
new per-body registry or default heap allocation on the ordinary empty path.
CopyView adds one paid borrowed handle8; its32B size assertion awaits the private
test compilation below rather than being counted as a compiled measurement here.

Gate1793914454535655591 passes24 exact public tests across4 targets in115.28s,
including real constructor/Drop, missing-u8-Copy, ordinary borrowed-move opposition,
all stateframe/cold-warm-store/schema/protocol checks, six compiler recipes/legacy
bytes, scalar dispatch and three atomic controls. Formatting, architecture and
strict selected Clippy pass. Native receipt dropstate/native.json passes both
programs in2.38s using locked b7c6b8e72 tools. The executable SHA is
efb76edf6e332c78b3e886a71df22cd46188b6ca63e7a3e3a5d5d7350a004c74.

Independent exact no_core controls compile owned Token movement0 and reject moving
Token through &mut with1/E0507 and the intended cause. Both async programs preserve
the already frozen standard-core companion's authored async/main bytes and its
0/native0 outcome. Exact no_core async's missing ResumeTy infrastructure is not
semantic parity. The original wrong-Output opponent remains1/E0308 independently
and fails actual unused-body checking in the candidate. No source expectation was
changed to evade a compiler fence.

Retained failed receipts include the enum/import collision, two strict lints, and
the actual missing-Project dominance failure. That check was repaired by adding
the two required producer operations, without weakening the checker. The earlier
pre-cleanup native0 remains explicitly unqualified infrastructure evidence. Two
new private invariants cover11 graph/owner/state forgeries, warm foreign-store
rejection and actual nonempty capture fencing; cfg(test) compilation/execution
remain pending integration/CI. No cold private link, broad matrix or new backend
build was run. Adjacent evidence binds exact source, fixture and receipt hashes.

Next is real Future::poll using the actual required member and independently
opened public binders, consumed checked application/formation/region/loan/cleanup
proof, retained coordinate mapping/environment, and the one owned deferred body.
Inputs must move once; patterns/body wait for first poll. Poisoned-before-transfer,
Ready/Output moves, cleanup-before-Complete, repeated/completed polling, unpolled/
completed capture destruction and genuine native traces remain required. Full
await/suspension, self references, panic/unwind, cancellation, recursive async
cleanup/helper execution and authored-body metadata remain required. Symbolic
compiler-operation recipes still grant no runtime authority; core need not request
async_drop. Original core/alloc/std and later assignments are not complete.
