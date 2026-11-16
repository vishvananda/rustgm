# Promoted calls (source core)

Three no_core source-core programs borrow the result of a const function call
and return the borrow as `'static`. In runtime code rustc 1.98.0 promotes such
a borrow to a constant only when the callee is `#[rustc_promotable]` and every
argument is itself promotable, as alloc's `task.rs` returns
`&RawWakerVTable::new(clone_waker::<W>, ...)` from a generic function. Their
prefix is the `source-promotable` packet's NoCore foundation up to its `word`
declaration; `Table::new` is marked and `unmarked` is the same const function
without the marker. `observe.c` prints the three values `observed` passes it.

- `marked` returns `&word()` and, from a generic function, `&Table::new(keep::<T>,
  seven::<T>)`, whose reified function items are constant arguments; the
  generic caller's promoted value is evaluated for its `Marker` instance;
- `unmarked` returns `&unmarked(keep::<T>, seven::<T>)` and is a rejected
  control (E0515);
- `runtime-argument` passes a parameter to `Table::new` and is a rejected
  control (E0515).

Outcomes were declared before either compiler ran and checked against rustc
1.98.0, whose status, errors and native output are recorded in `outcomes.json`:
all 3 statuses, both error codes and the output agree.
`dev/tests/promoted_calls.rs` checks every case and runs the accepted one
natively.

The generic-callee, generic-unmarked and generic-runtime-argument inputs
repeat these outcomes with a type parameter on `word` itself. Promotion
authenticates its source marker and complete checked instance through a
distinct interpreter entry. Ordinary generic function roots and conditional,
intrinsic and drop-glue entry fences remain unchanged.
