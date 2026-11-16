# Captured copies (source core)

Seven no_core source-core programs copy a `T: Copy` value inside a capturing
closure of a generic function, as pinned core's `copy_fold` and
`copy_try_fold` (`iter/adapters/copied.rs`) do with `move |acc, &elt| f(acc,
elt)`. The bring-up skip list stubbed both for this cause, and `slices.rs`
then stopped at `Take::try_fold`'s `check`, whose instance needs `copy_fold`'s
checked opaque. They are checked against rustc 1.98. The prefix is the
`source-call-members` prefix through its `&mut F` call impls.

Like rustc, a closure's body has its creator's where-clauses whether or not it
captures, so `T: Copy` holds when a capturing closure reads a `T` through a
reference.

- `captured-read` copies `&e: &T` in a closure that also captures a `u8`;
- `captured-reference` is `move || *x` for `x: &T`;
- `captured-callee` passes the copied value to a captured `F: FnMut(T)`;
- `once` copies in a closure that moves a captured value, so it is `FnOnce`
  and borrows no receiver;
- `copy-fold` is core's `copy_fold`, returned as an opaque closure;
- `take-copied` runs core's `Take::try_fold` and its returned `check` closure
  over `copy_fold` of items `&u8`, the shape `slices.rs` reaches through
  `Copied::max`;
- `not-copy` is `copy_fold` without the `T: Copy` bound (E0507).

Each accepted program exports `entry` and `probe`; `observe.c` prints
`41 59`. Outcomes were declared before the frozen rustc 1.98.0 oracle. Both are
recorded in `outcomes.json` (7/7 agree). The packet keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/explicit_call_members.rs` checks every case and runs the accepted
ones natively.
