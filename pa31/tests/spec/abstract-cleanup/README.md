# Abstract cleanup (source core)

Six no_core source-core programs move an abstract value conditionally in a
generic body, as pinned core's `TakeWhile::next` (`iter/adapters/take_while.rs`)
does with `let x = self.iter.next()?`. The bring-up skip list stubs it,
`Peekable::nth` and `Error::provide` for this cause. They are checked against
rustc 1.98. The prefix is the `source-call-members` prefix through its `&mut F`
call impls.

A rigid projection such as `I::Item`, or an opaque type, has no structure the
generic body can move from, so its possible cleanup is that of the whole value,
as it is for a type parameter.

- `projected` moves `x: I::Item` out of a matched `Option` only if a predicate
  holds, as `Iterator::find` does;
- `take-while` is core's `TakeWhile::next`, with a `u8` flag and a `match` for
  `?`;
- `opaque-kept` moves an `impl Iterator` value out of an `Option` conditionally;
- `dropped` runs `take-while` over owned items whose `Drop` records the value
  it drops, and 255 on a second drop, so a rejected item is dropped exactly
  once;
- `parameter` is a control with a type parameter instead of a projection;
- `moved-read` borrows `x` after the conditional move (E0382).

Each accepted program exports `entry` and `probe`; `observe.c` prints
`41 59`. Outcomes were declared before the frozen rustc 1.98.0 oracle. Both are
recorded in `outcomes.json` (6/6 agree). The packet keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/moved_cleanup.rs` checks every case and runs the accepted ones
natively.
