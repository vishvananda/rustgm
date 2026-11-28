# PA33 fixture ownership

Active consumer grading suite (`suite_kind: "consumers"`, `alloc-artifact`
released). [suite.json](suite.json) is run by
`scripts/grade_consumers.py pa33` (or `make -C pa33 test`): it compiles and
executes all 12 `no_std` + `alloc` consumers in `spec/alloc-consumers/` against
unmodified pinned Rust 1.98.0 `core` and `alloc` in strict zero-bringup mode,
comparing each binary (3 repeats) against the pinned `rustc 1.98.0` `.stdout`
oracles declared in `alloc-consumer-outcomes.json`, and checks byte-identical
recompilation of `vec`. See
[pa33-alloc-bootstrap-release.md](../../docs/pa33-alloc-bootstrap-release.md).

Regenerate or compare the independent `rustc` stdout oracles with
`scripts/consumer_oracle.py m2` (see the
[runtime tracker](../../docs/self-hosting-runtime.md)).

`spec/vec-collect-inference/` adds four complete source-profile regressions owned
by `vec-collect-compile-outcomes.json`: inferred and explicitly typed mapped Vec
collection, a wrong collection item, and a floating non-Step range. The original
Vec consumer harness prefix is preserved and attributed. This separate compile
outcome manifest leaves PA33 planned; it does not create a graded suite or a
maintainer integration root.
