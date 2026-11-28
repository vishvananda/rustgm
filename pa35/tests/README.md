# PA35 fixture ownership

Active consumer grading suite (`suite_kind: "consumers"`, finite contract with
`unicode-crates` released; `core-crates` and `alloc-crates` deferred to PA45).
[suite.json](suite.json) is run by `scripts/grade_consumers.py pa35` (or
`make -C pa35 test`): it compiles and executes all 4 consumers in
`spec/unicode-consumers/` over unmodified pinned Rust 1.98.0 stdlib and
RustGM's locked upstream crates (`unicode-ident 1.0.24`, `tinyvec_macros 0.1.1`,
`tinyvec 1.10.0`, `unicode-normalization 0.1.25`) in strict zero-bringup mode,
comparing each binary (3 repeats) against pinned `rustc 1.98.0` `.stdout`
oracles declared in `unicode-consumer-outcomes.json`, and checks byte-identical
recompilation of `xid`. See
[pa35-unicode-crates-release.md](../../docs/pa35-unicode-crates-release.md).

Regenerate or compare the independent `rustc` stdout oracles with
`scripts/consumer_oracle.py m4` (see the
[runtime tracker](../../docs/self-hosting-runtime.md)).
