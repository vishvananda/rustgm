# Closure creator requirements (source core)

Eleven no_core source-core libraries reduce the prepared-lowering failure of
pinned core's `is_sorted_by` 8380 (`slice/mod.rs:4782`), which calls
`self.array_windows().all(|[a, b]| compare(a, b))`. They are checked against
rustc 1.98. Each case's first line names its prefix,
`default-const-closures/count.rs` before its `ZeroablePrimitive`, which
declares the `Fn` traits.

The closure's parameter region comes from the callee's lifetime, an inference
region of the creator. Its body passes the parameter to a captured
`FnMut(&'a T, &'a T)`. It therefore needs the callee region to outlive `'a`,
and `T: 'a`. rustc's borrow checker makes them closure requirements, which the
creator proves from its own signature.

- Positives:
  - `captured-call` is the form without a pattern;
  - `tuple-pattern` and `array-pattern` destructure the parameter;
  - `nested-closures` makes the call through two closures;
  - `declared-outlives` (`T: 'a`) and `concrete-u8` need only the region
    requirement;
  - `declared-static` is a `T: 'static` control;
  - `captured-call-fn` calls a function item instead.
- Negatives:
  - `static-region` needs the callee region to be `'static` (E0521);
  - `static-type` needs `T: 'static` (E0310);
  - `where-unimplied` rejects its where clause before any closure (E0309).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 11
declarations agree. The batches are in
`obj/parallel/array-pattern/core/declared.json`. The cases keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/creator_requirements.rs` checks every case's typing through the
CLI. It runs prepared lowering for every case that types, and checks bounded
work. Its ignored native test runs four ordinary-mode programs against rustc.
See `docs/pa31-closure-creator-requirements.md`.
