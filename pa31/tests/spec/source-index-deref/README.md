# Index operands through source Deref calls (source core)

Eighteen no_core source-core programs index a value whose type reaches its
indexed place through `Deref` implementations, as `shared.borrow()[0]` does
with core's `Ref`, checked against rustc 1.98. Each is the `symbolic-index`
support prefix (which declares `Index` and `IndexMut`), the `source-field-deref`
`Deref` and `DerefMut` declarations, one `BitOrAssign` declaration and a suffix.

The index operand is autodereferenced. At each step below the built-in
references the operand is indexed where it is an array or slice or may
implement `Index`; otherwise a type that may implement `Deref` contributes one
actual call and its target is tried next. Every step of a mutable index place
(a store, a compound assignment, `&mut`, a `&mut self` receiver) calls
`deref_mut`, and the index itself then uses `IndexMut`. The calls run once, in
source order, before the index operand is evaluated.

- Accepted:
  - `shared`, `nested` and `references` read through one call, two calls, and
    `&Wrap` and `&&Wrap`;
  - `slice-target` and `custom-target` reach a slice and a type with its own
    `Index<Key>` implementation;
  - `own-index` has both `Index` and `Deref`; its own `Index` is called and
    `deref` is not;
  - `mutable` stores, compound-assigns and borrows mutably, through one and two
    calls and through `&mut Wrap`; `late-mutable` calls a `&mut self` method on
    the element, which selects `DerefMut` after the index was typed;
  - `generic` indexes `t[0]` under `T: Deref<Target = [u8; 3]>`;
  - `native` runs every form above against `observer.c`, which prints each
    `deref`, `deref_mut` and `index` call in order.
- Rejected:
  - `missing` has neither `Index` nor `Deref` (E0608);
  - `shared-only-borrow` and `shared-only-store` need `DerefMut` (E0596,
    E0594);
  - `wrong-index` and `hidden-target` index with a type the reached array, or
    the type's own `Index`, does not take (E0308);
  - `immutable-binding` stores through an immutable binding (E0596).
- Recorded gaps, which rustc accepts. Both fail closed:
  - `reference-impl`: rustc tries `Index` on each reference before it
    dereferences it. An `Index` implementation for a reference type is never
    selected here, so an operand that may have one stops at typing;
  - `ref-mut-binding`: `let ref mut y = w[1]` does not make its index place
    mutable (neither does it for a direct `IndexMut`), so the shared calls are
    rejected by the borrow checker.

Further limits. The outer head decides each step, as for a field or method
receiver: a type whose own `Index` takes another index type is not
dereferenced (`hidden-target`, as rustc). A built-in array or slice index
lowers to a bounds guard, and demanded native LowIR stops at the source panic
routing boundary while the library installs `panic_bounds_check` (see
`symbolic-index`). `native` therefore indexes only through source
`Index<u8>` implementations; the built-in index after `Deref` calls is typed,
built and borrow-checked here and executed only in the bring-up profile. In a
constant context such an index is not yet accepted: its const check fails
closed (`const borrowed place changed its actual operand borrow`).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run, except the two recorded gaps; both, the two post-declaration byte
changes and the oracle argv are recorded in `outcomes.json` (18/18 agree). The
cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.
`dev/tests/access_sources/dereferenced_indexes.rs` checks every case through
typing, body construction and the borrow checker, and its ignored native test
runs `native`.
