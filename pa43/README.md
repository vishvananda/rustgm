# RustGM Programming Assignment 43 — Async ownership

The graded suite for this assignment is pending.

Extend [PA42](../pa42/README.md)'s suspension state machines with borrowed
storage, pinning, cancellation, and async destruction. Reuse the ownership
and loan checks from PA14–PA17 and the applicable pointer validity rules
from PA27.

## Ownership across suspension

Prove the lifetime and place of each retained borrow. Preserve captured
storage at the required address when the future is pinned. Check reborrows,
receiver selection, `Unpin` requirements, and projection legality. Keep
storage and cleanup obligations valid at each suspension, resumption, and
exit. Include self-referential state, async closures, and async trait
methods.

## Cancellation and destruction

Use the actual source `AsyncDrop` declarations and checked Pin receiver.
Implement the compiler-generated operation for `async_drop_in_place`.
Retain its suspension and cleanup behavior, including recursive cleanup
of owned fields. Source placeholders and declaration metadata alone
supply no generated state machine.

Destroy only the initialized values owned by the suspended or completed
state. Preserve source cleanup order and the exclusion between Copy and
destruction. Use [PA29](../pa29/README.md)'s typed artifact boundary for
imported operations and the source and body transport needed to execute
them.

## Tool surface

Use the existing LowIR and crate-build interfaces. Lower the checked
state machines, poll calls, and cleanup paths for the pinned backend.

## Check your implementation

Cancel at each suspension state and observe the actual cleanup order.
Test completion, self-reference validity, moved captures, and recursive
async destruction. Include a separate consumer of an imported async
operation. Run native controls to check the selected constructor,
polling, and cleanup behavior.
