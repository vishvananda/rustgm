# Array length patterns (source core)

Thirteen no_core source-core libraries reduce pinned core's `is_sorted_by`
8380 (`slice/mod.rs:4782`), which calls
`self.array_windows().all(|[a, b]| compare(a, b))`. They are checked against
rustc 1.98. Only the closure's pattern fixes `array_windows`'s `N`. Each case's
first line names its prefix,
`default-const-closures/count.rs` before its `ZeroablePrimitive`, which
declares the `Fn` traits.

rustc's `check_array_pat_len` equates an array of unknown length with
`[T; n]` when the pattern has `n` elements and no rest. A rest with an unknown
length is E0730, and a const parameter length does not equate with `n` (E0308).

- Positives:
  - `infer-len`, `by-value` and `ref-pattern` are closure parameters;
  - `core-form` is core's form;
  - `let-infer` is a `let`;
  - `nested-infer` infers two lengths.
- Controls: `core-form-known`, `rest-known` and `let-ergonomic` have known
  lengths.
- Typing rejects:
  - `rest-unknown` (E0730);
  - `known-mismatch` and `match-two-lengths`, whose first arm fixes the length
    (E0527);
  - `generic-len` (E0308).

`core-form` and `core-form-known` type. Prepared lowering then stops with an
existing closure lifetime gap. It occurs with or without length inference, and
the base reproduces it without an array pattern.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 13
declarations agree. `obj/parallel/array-pattern/core/declared.json` records
four batches, including ungraded diagnostics. The cases keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/array_length_patterns.rs` checks every case's typing through the
CLI. It runs prepared lowering for every case that types, and checks bounded
work. Its ignored native test runs four ordinary-mode programs against rustc.
See `docs/pa31-array-length-patterns.md`.
