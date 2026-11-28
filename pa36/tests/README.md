# PA36 fixture ownership

Active consumer grading suite (`suite_kind: "consumers"`, `std-artifact`
released). [suite.json](suite.json) is run by
`scripts/grade_consumers.py pa36` (or `make -C pa36 test`): it compiles and
executes all 14 `std` consumers in `spec/std-consumers/` against unmodified
pinned Rust 1.98.0 `core`/`alloc`/`std` and its platform dependencies in strict
zero-bringup mode, comparing each binary (3 repeats) against pinned `rustc
1.98.0` `.stdout` oracles declared in `std-consumer-outcomes.json`, and checks
byte-identical recompilation of `println`. See
[pa36-std-bootstrap-release.md](../../docs/pa36-std-bootstrap-release.md).

Regenerate or compare the independent `rustc` stdout oracles with
`scripts/consumer_oracle.py m3` (see the
[runtime tracker](../../docs/self-hosting-runtime.md)).
