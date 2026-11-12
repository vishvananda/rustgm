# Abstract callable inputs (PA24)

A function item whose generic inputs are inferred only through a callable
bound must meet an abstract projection input such as `I::Item`, as pinned
core's `Iterator::max` passes `Ord::cmp` to `max_by`'s
`FnMut(&Self::Item, &Self::Item) -> Ordering` bound. The item's reference
inputs take fresh signature lifetimes; the identical projections beneath them
still match.

- `generic-item`, `unbounded-item` and `two-reference-item` pass generic free
  fn items with one or two reference inputs;
- `trait-member-item` passes the member path `Num::num`, `default-method-item`
  does so in a default method over `Self::Item`, and `ord-cmp` passes std
  `Ord::cmp`;
- `parameter-item` passes the member path for a type parameter, a control;
- `mismatch` passes a concrete `u16` item and `distinct` one generic item for
  distinct projections; rustc rejects both (E0631).

Every positive executes and checks its returned value. Outcomes were declared
before rustc 1.98.0 and before any RustGM run of these bytes. See
[the checkpoint](../../../../docs/pa24-abstract-callable-inputs.md).
