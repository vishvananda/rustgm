# Conditional initialization execution boundary

The next finite slice consumes existing Plan.flags, Condition::Initialized and
Flags events through the already completed concrete Execution/Initial/Seal owner.
Original stays symbolic; no physical initialization/read/loan/CFG permission is
obtained from an evaluator bit. The closed Struct/written const Drop profile is
otherwise unchanged.

The neutral producer maps each actual flag to its index in that same sorted plan
vector. An optional nonzero index represents a call guard. The separate active
call inventory remains complete even when its dynamic guard is false. Full event
ranges and guard indices are authenticated before immutable Code publication.

A non-Clone cold Cursor replaces the two existing cursor/parent fields through
one optional Box. It retains exact frame instance, exact footer Rc (whose Weak is
once bound to Code), event and parent continuation, and owned bit words. Actual
Frame 248 is guarded at compilation; Cursor 48, Call 28 and unchanged View 240 are
measured in static3 type-size output. Ordinary frames keep None and no
new allocation. Active old zero-flag profiles now pay the cold cursor too.

Storage is allocated/zeroed once per frame. Actual entry flag writes are replayed
from the real parameter StorageLive transitions. Block/edge/loop traversal resets
only the event cursor, not flags. A false guard skips the method only after the
same actual reached event/target/header/State/OwnDrop checks. A true guard uses the
existing real source child and exact Unit return continuation. No synthetic flag
clear or inferred initialization is introduced.

Prepayment uses the existing interpreter slot budget in Constant-sized units for
the Box and words before growth, while the parent's existing storage stays live.
Zeroing, range writes and checking pay existing Work; failed work stays consumed.
The owning frame's return subtracts cold units exactly once. Code/contract/view
projection pays call growth, range/domain validation and original/new live overlap.

References are under obj/parallel/source-transport/ctfe-flags/reference-v1 and
reference-v2. All 23 source outcomes and eight real Rust native observers match
frozen predictions before candidate execution. Final scoped qualification is
recorded below; no broader profile or component completion is claimed.


Final meta2 passes all-target/all-feature strict metadata without private linking.
Final1 passes 52 exact public tests, 16 actual native programs, harness and upstream.
The eight new native observations match the eight independent Rust observers;
the prior eight native programs remain unchanged. The source map binds 3582 dev
Rust files. New hostile frame/footer/domain/range/pre-growth controls compile;
execution remains CI-pending. The separate evidence JSON binds every reference,
failure, final receipt, input and native LowIR/object/executable hash.

Entry storage is initialized once, then only actual events set/clear flags. Loop
edges reset only the event cursor. Body preparation rejects a false guard, and
both true and false execution decisions preserve real target/header/State checks.
Wrong-frame, foreign-Code/footer, out-of-range/high-bit and unpaid storage states
are not initialization evidence. The original active-call inventory is unchanged.
No guard value grants Copy, Destruct, const, loan or runtime-body permission.
