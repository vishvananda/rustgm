# Actual source Field dereference calls

The19 frozen controls use actual source Deref/DerefMut/Target declarations,
ordinary implementations and real bodies. They cover shared/mutable and late
mutable selection, nested/temporary evaluation, direct-field shadowing, full
lifetime applications, moves/loans/escaping borrows, packed fields, raw pointers
and genuine const mutation. Independent native observers freeze exact effects
before RustGM execution. Explicit raw references remain distinct from an
implicitly generated raw-pointer borrow.

The initial raw-explicit positive prediction was disproved by pinned Rust's
deny-default dangerous_implicit_autorefs diagnostic. Its unchanged source and
corrected negative are retained, with original prediction/diagnostic receipts.
Advanced lexical allow/warn/expect/forbid/cap controls live in owning PA40's
source-field-autorefs packet. No deny result is counted through an unsupported
producer/profile fence.

The semantic-only inspection test is separate from the full pipeline/native
consumers. Source/owner/binder/range/link corruption controls remain private.
The separately frozen [capture packet](../source-field-captures/README.md)
qualifies genuine custom-Deref receiver-prefix captures and call-time effects.
Structural constant-address consumers, imported capture transport and generated
coroutine implementations remain required.

The union-borrow pair preserves the actual source DerefMut receiver read: safe
union-field borrowing rejects with the unsafe-operation cause; its lexical
unsafe-block counterpart passes. These two predictions were independently
checked before candidate observation. Existing ordinary reference/tuple union
read opponents and direct-store positives remain in the PA27 union packet.
