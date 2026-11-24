# RustGM Programming Assignment 39 — Panic and unwinding

The graded suite for this assignment is pending.

Extend [PA36](../pa36/README.md)'s panic-abort profile with unwinding.
Reuse [PA17](../pa17/README.md)'s cleanup plans and
[PA30](../pa30/README.md)'s foreign-call rules.

## Required implementation

Retain the selected panic strategy and callable unwind policy during
checking and lowering. In unwind mode, execute required cleanup on
exceptional paths. Run each required destructor exactly once and preserve
the original cleanup ordering. Handle a second panic during unwinding
according to the selected runtime. Keep the existing abort behavior
available.

Use the declared runtime, personality, and cleanup representation.
Implement `C-unwind` interoperability and enforce the selected caller
and callee's unwind boundary. Extend [PA32's `ffi_unwind_calls` check](../pa32/README.md#diagnostics-used-by-core)
to foreign calls that can propagate an unwind through their enclosing
function. Check the required lowering and runtime capabilities with the
pinned CPPGM backend.

## Tool surface

Select the panic profile as part of the crate build. Preserve its identity
in the callable and link information. Link and run the completed
unwinding programs through the configured runtime.

## Check your implementation

Test a panic with ownership cleanup, a destructor that panics during
unwinding, propagation across a permitted C-unwind boundary, and
abort-mode execution. Observe the destructor sequence in the running
program as well as its termination.
