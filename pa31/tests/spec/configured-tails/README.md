# Configured block tails (source core)

Sixteen no_core source-core libraries check blocks whose written tail is
configured out, and statements written without a semicolon, against rustc
1.98. Pinned core's `fmt/num.rs` `impl_Display` bodies end in
`#[cfg(not(feature = "optimize_for_size"))] { ... }` followed by a removed
`#[cfg(feature = "optimize_for_size")] { ... }`. `method-form` and `macro-form`
reduce that shape, the second from `macro_rules` as in core. Each case's
prefix is the first 356 lines of `const-parameter-derive/actual-source.rs`.
The other cases are authored.

rustc's block has no separate tail: after configuration, its last expression
statement written without a semicolon is the tail. Every other such statement
must have type `()`. An interpolated `$s:stmt` stays without a semicolon, so a
`;` written after it is an empty statement.

- Positives:
  - `tail-cfg-out`, `three-blocks`, `let-value`, `method-form`,
    `macro-form`, `attributed-promoted` and `unsafe-block-promoted` promote
    the last enabled block after removed ones;
  - `all-out-unit` removes every block of a unit function;
  - `tail-cfg-in`, `match-statement-unit` and `diverging-statement` are
    controls: an enabled tail, a unit `match` statement and a `return` block.
- Negatives (E0308):
  - `semicolon-kept` writes `;` after the enabled block, so the block is
    `()`;
  - `both-kept` keeps two non-unit blocks;
  - `type-mismatch-promoted` promotes a `bool` block in a `u8` function;
  - `if-statement-nonunit` writes a non-unit `if` statement;
  - `statement-fragment-semicolon` writes `$s;` with `$s` matching `{ 1 }`.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. Fifteen of
the sixteen declarations agree. `statement-fragment-semicolon` was declared
accepted, and rustc rejects it; rustc's outcome is used.
`obj/parallel/cfg-tail/core/declared.json` records four batches, two of them
ungraded diagnostics, one of which was malformed. The cases keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/configured_tails.rs` checks every case's typing through the CLI.
It runs prepared lowering for every accepted case, and checks bounded work.
See `docs/pa31-configured-tails.md`.
