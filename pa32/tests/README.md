# PA32 fixture ownership

Active consumer grading suite (`suite_kind: "consumers"`, `core-artifact`
released). [suite.json](suite.json) is run by
`scripts/grade_consumers.py pa32` (or `make -C pa32 test`): it compiles
unmodified pinned Rust 1.98.0 `core` standalone twice in strict zero-bringup
mode to verify byte-identical `core.rgcrate` / `core.lowir` /
`core.native-links.json`, compiles and executes all 18 `no_std` consumers in
`spec/core-consumers/` against the pinned `rustc 1.98.0` `.stdout` oracles
declared in `core-consumer-outcomes.json`, and checks the 2 panic-abort FFI
unwind lint controls in `spec/ffi-unwind-lint/cases.json`. See
[pa32-core-bootstrap-release.md](../../docs/pa32-core-bootstrap-release.md).

Regenerate or compare the independent `rustc` stdout oracles with
`scripts/consumer_oracle.py m1` (see the
[runtime tracker](../../docs/self-hosting-runtime.md)).
