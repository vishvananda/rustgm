# Actual source Deref method contracts

The42 complete inputs declare35 independent Rust accepts and7 rejections.
Forty checked-body/LowIR outcomes now pass their finite candidate selection,
including paid/unpaid bounds and lifetimes, captures, generic method arguments,
method priority, real constant values23/41 and genuine move/borrow/const errors.
The two128-layer generic-chain positives remain required: noise0 exhausts the
existing inference budget and noise128 is not qualified. Neither is a negative
language test. Canonical work is linear at qualified depths8/32; total inference
is quadratic, with both counters unchanged by128 unrelated declarations.

Independent Rust/C and candidate RustGM/CPPGM observers print
`receiver 23 41\n` and `receiver-order 23 125 41 134625\n`, status zero.
The trace verifies one operand evaluation, shared/mutable calls and argument
order. Actual source calls preserve full source/const/lifetime proofs.

The support prefix comes from the existing dereference-source foundation;
original bytes remain untouched. Const variants explicitly declare const Deref
and const implementations. Direct/explicit and wrapper-priority controls retain
the earlier comparison, while wrapped array cases retain core's actual shape.
Every input and auxiliary appears in the manifest. Historical CLI type-only
outcomes do not establish complete move/loan checks; use full checked bodies.
The generated independent receipt binds observed byte-identical copies and
separates the preimplementation baseline from current qualification. It creates
no course oracle or assignment release.

Ordinary library forms remain in [PA26](../../../../pa26/tests/spec/deref-methods/README.md).
See the [implementation checkpoint](../../../../docs/pa26-implicit-deref-methods.md)
and [initial reduction review](../../../../docs/pa26-deref-method-reductions.md).
