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
Closure capture and structural constant-address consumers retain explicit
required boundaries; this packet does not redefine their completion or supply
a generated coroutine implementation.
