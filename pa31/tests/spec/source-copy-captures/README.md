# Source Copy captures (source core)

Twelve no_core source-core libraries reduce pinned core's `StepBy` helpers
`first_size` and `other_size` (`iter/adapters/step_by.rs:232/237`), checked
against rustc 1.98. The prefix is `source-div/foundation.rs`, plus
`unboxed_closures`, core's const `Fn`/`FnMut`/`FnOnce` declarations and a
`NonZero<T>` that, like core's, is Copy only through
`impl<T> Copy for NonZero<T> where T: ZeroablePrimitive {}`.

A `move` closure that copies a captured Copy value by value is still `Fn`.

- `other-size` is core's `move |n| n / step`, and `returned-copy` returns the
  captured `NonZero<usize>`;
- `inner` captures a manual `impl Copy` struct, and `conditional` captures a
  conditional `impl<T: Copy> Copy for G<T>` at `G<usize>`;
- `fn-mut` returns `impl FnMut`, and `fn-bound` passes the closure to an
  `F: Fn` bound;
- `parameter-premise` captures `G<T>` under `T: Copy`;
- `call-size` and `call-bound` call the closures twice from a root `main`, so
  `--emit-lowir` lowers the copying bodies too;
- `clone-only` (Clone without Copy), `failed-condition` (`G<Owned>`) and
  `clone-premise` (`G<T>` under only `T: Clone`) move out of a captured
  variable in an `Fn` closure (E0507).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (12/12 agree). It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`. `dev/tests/source_closures/implemented_copies.rs`
checks every case. See `docs/pa24-source-copy-captures.md`.
