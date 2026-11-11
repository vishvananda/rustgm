# Field preparation for actual borrowed places

These five ordinary PA18 programs retain the exact bytes declared before the
field-preparation change. Rust 1.98.0 object checks independently accept shared,
mutable and concrete receivers and the destructor program; moving an unbounded
generic value out of a shared reference rejects. The target is Linux x86-64,
edition 2024, panic abort, overflow checks enabled and debug assertions disabled.
The owning assignment manifest declares every source and its full LowIR outcome.

An implicit reborrow constructs a referent place. Immutable destruction queries
need that place's instantiated fields, including in an unused generic body.
The compiler now supplies the actual selected borrow to its existing field
preparation paths, preserving source order, declaration waits and normalization
proofs. Reference types elsewhere in the inference arena supply no extra roots.

`drop-native.rs` calls the generic borrowed method and observes the original
non-Copy value's destructor exactly once when its owning scope ends. Both pinned
Rust and CPPGM executions pass. The other three accepted programs are unused-body
compilation contracts. Existing `shared-call-reborrow` inputs remain unchanged
and check generic reference reuse, Copy and the genuine mutable-reference move.

Frozen predictions and independent observations are retained under
`obj/parallel/projected-properties/borrow-{plan.json,probe/}`. The associated-output
method conditions belong to [PA20](../../../../pa20/tests/spec/projected-method-inputs/README.md).
