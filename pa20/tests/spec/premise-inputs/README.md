# Premise-selected inputs (PA20)

A path call such as `X::Item::e(a, b)` leaves the trait's `Rhs` to inference.
The caller's premise `X::Item: E<Y::Item>` selects it, binding `Rhs` to the
premise's own projection `Y::Item`. That projection must reach its ordinary
normalizer before the argument's expectation or coercion reads its head, as
pinned core's `Pin` comparison forwarding needs for `Ptr::Target: PartialEq<Q::Target>`.

- `direct` passes an exact reference, `double-reference` a `&&Y::Item` that
  coerces, and `method` calls the member on a receiver;
- `impl-forwarding` is core's forwarding impl over `It` instead of `Deref`;
- `qualified` (`<X::Item as E<Y::Item>>::e`) and `annotated` (the coerced
  operand annotated first) are controls;
- `missing` names a different `Rhs` in its premise; rustc rejects it (E0308).

Every positive executes and checks its result. Outcomes were declared before
rustc 1.98.0 and before any RustGM run of these bytes. See
[the checkpoint](../../../../docs/pa20-premise-inputs.md).
