# GAT region premises (source core)

Fifteen no_core source-core libraries reduce pinned core's `SplitInclusive`
`clone` 8624 (`str/iter.rs:1438`). Its `self.0.clone()` selects
`SplitInternal`'s `Clone`, whose `P: Pattern<Searcher<'a>: Clone>` premise
needs `<P as Pattern>::Searcher<'a>: Clone` for the impl's fresh `'a`. They
are checked against rustc 1.98. Each case's first line names its prefix,
`default-const-closures/count.rs` before its `ZeroablePrimitive`. Core's
`Pattern`, `Searcher` and `SplitInternal` follow it.

rustc relates the rigid projection's lifetime arguments invariantly and leaves
the region equality to its borrow checker.

- Positives:
  - `core-form` is core's form;
  - `path-call`, `where-projection`, `local-value`, `copy-premise` and
    `wrapper-chain` reach the same premise through a path call, a projection
    where clause, a free function, a `Copy` bound and a second wrapper;
  - `mutual-outlives` uses a premise for `'b` with `'a: 'b, 'b: 'a`;
  - `static-use`, `searcher-method` and `direct-field` are controls.
- Negatives:
  - `missing-bound` and `wrapper-missing-bound` lack the bound (E0599);
  - `other-lifetime`, `static-premise` and `outlives-only` give a premise for
    another lifetime. rustc's borrow checker rejects them, and RustGM's
    prepared lowering lifetime proof does.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 15
declarations agree. `obj/parallel/pattern-clone/core/declared.json` records
four batches, including a malformed batch and three ungraded diagnostics. The
cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/gat_region_premises.rs` checks every case's typing through the CLI.
It runs prepared lowering for every case that types, and checks bounded work.
See `docs/pa31-gat-region-premises.md`.
