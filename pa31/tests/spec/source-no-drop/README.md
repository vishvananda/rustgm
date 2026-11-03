# Source NoDrop and union field requirements

This draft packet owns source compiler-binding behavior for the actual
`bikeshed_guaranteed_no_drop` language item. It consumes PA12's nominal types,
PA26's real Copy/Clone premises and PA27's union and ownership rules. PA31 remains
planned. This packet does not complete MaybeUninit, metadata or library builds.

The pinned registry requires an actual trait with zero written generic
parameters, including lifetime and const parameters. Its standard declaration
denies explicit implementations and dynamic compatibility. The role does not
itself require safe/ordinary/empty/auto flags: the independent header controls
preserve other declarations that Rust accepts, without granting executable
methods or const implementations.

For the actual ordinary positive trait goal, shared/mutable references and the
actual ManuallyDrop identity satisfy the builtin rule. Tuples and arrays require
the same marker of each component, including an array's element when length is
zero. Every other type requires its actual source Copy proof. A normal struct
with no destructor, or one named ManuallyDrop, receives no special treatment.

Union field admission first uses Rust's narrow trivial-Copy shortcut: primitive
scalars, function items, shared references, raw pointers, recursive arrays and
patterns, and recursive tuples of at most three fields. Otherwise it requires
the actual NoDrop trait when declared, falling back to the actual Copy trait
when absent in no_core. The shortcut is local to field admission. It does not
prove a source trait bound or authorize copying a value. Conditional tuple Copy
retains the element proofs even when the shortcut's arity limit is exceeded.

Contract-v3 freezes35 source outcomes and the genuine native relay result `8`.
Reference-v4 independently matches every source outcome and executes that result
with pinned Rust and the C observer. Earlier source predictions are retained:
v2 incorrectly rejected a four-u8 tuple, whose actual elements prove Copy.
The unchanged positive remains required; a separate four-u16 tuple lacks its
element Copy proofs and must reject. A subsequent observer attempt used C++
linkage for a C observer and failed to link; reference-v4 uses the C compiler.

Local candidate behavior is qualified: all35 full original-body outcomes, native
relay `8`, private19, production5, PA12 125/125 and66 constant-view CLI controls.
The [checkpoint](../../../../docs/pa31-no-drop-checkpoint.md) records scope,
receipts and the global compact type fact that avoids rescanning shared deep
types across declarations. Borrow/move opponents require full CFG proof.
General nominal/role transport, fresh imported consumers and whole original
core/alloc/std artifacts remain open. NoDrop is not a destructor-elision or
borrow/const/unsafe permission.
