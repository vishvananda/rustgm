# RustGM Programming Assignment 42 — Async lowering

This is a draft handout. The assignment's grading suite will accompany
its release.

Use [PA31](../pa31/README.md)'s actual source callable and coroutine
declarations to lower async functions, async blocks, and generated
coroutines. Use the ordinary typed body, ownership, and trait machinery
from earlier assignments.

## Required implementation

Build a state machine with a stable identity for each generated body.
Determine which values must survive a suspension. Preserve required
locals and their initialization state across suspension and resumption.
Use the selected future and poll contracts for readiness, pending state,
and completion.

Compiler-generated coroutines preserve their resume input, yielded item,
and returned value, with the actual `Coroutine` associated items and
`CoroutineState` variants. Preserve the authored body of a `gen` block.
Its `Iterator` and `FusedIterator` behavior must follow the selected
implementation and exhaustion state. Ordinary manual implementations
retain their own typed bodies and call semantics.

Keep generated state, source declarations, and manual implementations
distinct. Transfer checked generated bodies through validated metadata
when a consumer is in another crate. [PA43](../pa43/README.md) adds loans
across suspension, pinning, cancellation, and destruction.

## Tool surface

Use the existing LowIR and crate-build interfaces. Retain the source body,
state transitions, and checked operations through lowering. Run the
generated state machine with the pinned backend.

## Check your implementation

Exercise pending and ready polls, stored locals, repeated suspension,
and resumption after a yield. Test manual resume effects separately
from generated transitions. Observe exact yielded and returned values,
and check exhaustion on later iterator calls.
