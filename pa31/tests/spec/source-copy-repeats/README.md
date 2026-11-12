# Source Copy repeats (source core)

Nine no_core source-core libraries reduce pinned core's `DisplayBuffer::new`
(`net/display_buffer.rs:14`), checked against rustc 1.98. They share the
`source-copy-captures` prefix: the `source-div` foundation, core's const `Fn`
traits, and `UsizeInner` and `NonZero<T>` with manual `impl Copy`.

`[x; N]` with N > 1, or a const generic N, needs `x: Copy` unless `x` is a
constant. A source `impl Copy` satisfies that.

- `manual`, `nonzero`, `conditional` and `parameter-premise` repeat a manual,
  `NonZero`, conditional (`G<usize>`) or premise (`G<T>` under `T: Copy`) Copy
  operand;
- `const-length` repeats into `[UsizeInner; N]` in a const fn, and
  `call-length` calls it from a root `main`, so `--emit-lowir` lowers it;
- `length-one` repeats a non-Copy operand once, which needs no Copy;
- `clone-only` and `clone-premise` repeat a Clone-only operand (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (9/9 agree). It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`. `dev/tests/source_closures/implemented_copies.rs`
checks every case. See `docs/pa24-source-copy-captures.md`.
