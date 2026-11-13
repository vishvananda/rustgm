# Configured block tails

Release candidate3's core census recorded two `fmt/num.rs` bodies, 145 and
166, that failed with "missing body type constraint". Two later bodies of the
same expansion were blocked. Both are `fmt` methods of `impl_Display`'s
generated `Display` impls:

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    #[cfg(not(feature = "optimize_for_size"))]
    {
        const MAX_DEC_N: usize = $Unsigned::MAX.ilog10() as usize + 1;
        let mut buf = [MaybeUninit::<u8>::uninit(); MAX_DEC_N];
        unsafe { f.pad_integral(true, "", self._fmt(&mut buf)) }
    }
    #[cfg(feature = "optimize_for_size")]
    {
        ${concat($fmt_fn, _small)}(*self as $T, true, f)
    }
}
```

The error was at the removed second block.

## Diagnosis

`obj/parallel/cfg-tail/core/declared.json` records four declared batches.
Batch 1 separated the causes on candidate3:

- A removed tail after an enabled block failed. So did a removed tail after
  removed blocks in a unit function. A removed first block before an enabled
  tail passed.
- `{ 1 }; #[cfg(false)] { 2 }` failed the same way, instead of the type
  mismatch rustc reports.
- Two enabled non-unit blocks were accepted, and rustc rejects them (E0308).

Configuration marks a disabled owner's subtree inactive and does not rewrite
the tree. A block kept its removed tail, and typing asked for the tail's type.
The parser also stored `{ 1 }` and `{ 1 };` alike as one statement kind, so
the tail rustc chooses could not be recovered. Nor was a block-like statement
without a semicolon required to have type `()`.

rustc's block has no separate tail. After configuration, its last expression
statement written without a semicolon is the tail, and every other such
statement is checked against `()`. Batches 2 and 4 showed that an interpolated
`$s:stmt` expression statement also stays without a semicolon. A `;` written
after `$s` is an empty statement, so `discard!({ 1 })` and `discard!(g())` are
E0308. Batch 4 was a diagnostic declared after batch 2 found this, and it
replaced batch 3's malformed non-block probe, which had no `Add`. Batch 3's
two other diagnostics confirmed that a semicolon written in source attaches.

## Change

- **Statements.** `Kind::Statement` records whether its semicolon was
  written. A block-like expression without one, and every interpolated
  statement fragment, has none.
- **Tails.** The parser records each block and statement expansion whose tail
  carries attributes. Configuration checks only those. If a block is enabled
  and its tail is removed, its last enabled statement becomes the tail when
  that statement has no semicolon and is not a braced macro call. Otherwise
  the block has no tail.
  - The promoted statement node becomes a transparent group around its value,
    so the statement's attribute owner is kept.
  - The block's statements are copied once into a new edge list without the
    promoted statement, and the removed tail is appended inactive. Every node
    keeps its parent, and children still precede their owners.
- **Typing.** A statement without a semicolon coerces its value to `()`. A
  diverging value coerces. A braced macro statement is checked through its
  expanded statements instead.
- **Assertion text.** The assertion pretty printer writes `;` only after a
  written semicolon.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- A braced macro statement before a removed tail is not promoted, so that
  block has no tail. rustc would take the tail from the macro's expansion.
  No core site needs it.
- The `()` rule now rejects programs the base accepted, as rustc does:
  non-unit block-like statements and `$s;` with a non-unit expression
  fragment.
- rustc's empty statement after `$s` gets no node of its own. The parser
  consumes the `;`, as before.
- RustGM reports "incompatible types" where rustc reports E0308.

## Contract

**PA31** (`pa31/tests/spec/configured-tails/`, 16 source-core reductions).
Fifteen declarations agree with rustc 1.98.0 (`outcomes.json`).
`statement-fragment-semicolon` was declared accepted, and rustc rejects it.
Eight cases come from batch 1 and eight from batch 2. Batch 2 was declared
after the change was written and before it was built. The base is release
candidate3 (fc41ecb7c) for batch 1 and release candidate4 (28c5f9780) for
batch 2.

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `tail-cfg-out` | a removed tail after an enabled block | accept | missing | accept |
| `three-blocks` | two removed blocks after an enabled one | accept | missing | accept |
| `let-value` | the shape in a `let` initializer | accept | missing | accept |
| `method-form` | core's shape in a method | accept | missing | accept |
| `macro-form` | core's shape from `macro_rules` | accept | missing | accept |
| `attributed-promoted` | a second attribute on the enabled block | accept | missing | accept |
| `unsafe-block-promoted` | an enabled `unsafe` block | accept | missing | accept |
| `all-out-unit` | every block removed in a unit function | accept | missing | accept |
| `tail-cfg-in` | a removed first block, an enabled tail | accept | accept | accept |
| `match-statement-unit` | a unit `match` statement | accept | accept | accept |
| `diverging-statement` | a `return` block statement | accept | accept | accept |
| `semicolon-kept` | `;` after the enabled block | E0308 | missing | mismatch |
| `both-kept` | two enabled non-unit blocks | E0308 | accept | mismatch |
| `type-mismatch-promoted` | a promoted `bool` block | E0308 | missing | mismatch |
| `if-statement-nonunit` | a non-unit `if` statement | E0308 | accept | mismatch |
| `statement-fragment-semicolon` | `$s;` with `$s` matching `{ 1 }` | E0308 | accept | mismatch |

"Missing" is "missing body type constraint" and "mismatch" is "incompatible
types", both at typing.

## Qualification

Three focused receipts pass with exact selections. Each covers assignment
metadata, formatting and the architecture audit.

- 1794535102692142154 (150.2s), the first build: strict Clippy for the
  frontend suite and the CLI, strict private Clippy, and the 17 existing
  `crate_syntax` tests, among them the child-before-owner arena invariant.
- 1794535458102483411 (52.9s), on the final source:
  - the two new tests in `dev/tests/configured_tails.rs`. The first runs the
    packet's typing through the CLI and prepared lowering for every accepted
    case. It requires "incompatible types" for each rejection. The second
    checks bounded work and lowers every width;
  - the new `configured_out_tails_promote_the_last_unterminated_statement`.
    For five configurations it checks the promoted group or the missing tail,
    the inactive removed tail after the statements and child-before-owner
    order. The receipt also reruns the child-before-owner invariant;
  - PA10's body IR text oracle, 81 of 81.
- 1794535529588719380 (50.2s): the PA4 (200), PA8 (126), PA11 (191), PA13
  (805), PA14 (124), PA16 (329) and PA17 (205) text oracles, all passing.

1794535701316248473 (41.1s) reran the two packet tests on the final tree,
with the documentation.

Earlier attempts:

- 1794534998243210361 and 1794535416277763548 failed formatting.
- 1794535039635173226 failed Clippy: `sema/scopes/walk.rs` matched the old
  statement kind through an alias.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/cfg-tail/scaling-raw.txt`), for 8/32/128
functions, each with a removed tail after an enabled block:

| Functions | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 3635 | 2444 | 67 |
| 32 | 3731 | 2468 | 67 |
| 128 | 4115 | 2564 | 67 |

That is 4 inference steps and 1 canonical step per function, and no trait
selection, linear across widths.

## Core progress

No core observation of this change has run yet.
