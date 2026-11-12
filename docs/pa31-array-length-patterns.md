# Array length patterns

Pinned core's `is_sorted_by` 8380 (`slice/mod.rs:4782`) failed with "array
pattern requires a known fixed-array type":

```rust
self.array_windows().all(|[a, b]| compare(a, b))
```

`array_windows` is generic over `const N: usize`. Only the closure's pattern
fixes `N = 2`.

## Diagnosis

rustc's `check_array_pat_len` handles an array type whose length is not yet
known:

- A pattern without a rest equates the array with `[T; n]`, where `n` is the
  pattern's element count.
- A pattern with a rest is E0730.
- A const parameter length is not an inference variable. Equating it with
  `[T; n]` fails (E0308).

RustGM's `array_pattern` accepted a known length, a slice, and a wholly unknown
type without a rest. An array whose length was an inference variable failed
closed.

`obj/parallel/array-pattern/core/declared.json` records four declared batches.
Batch 1 ran rustc and the clean closure-infer candidate (dbeadba6) before any
implementation. That candidate rejects every case except the `rest-known`
control. All declarations agree with rustc.

## Change

An array pattern without a rest equates an array with an inferred length to
`[T; n]`, keeping its element type. A rest with an unknown length and a generic
length still fail closed with the existing message.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- `generic-len` rejects with "array pattern requires a known fixed-array type".
  rustc reports a mismatched type (E0308). Both reject.
- `core-form` and its known-length control `core-form-known` first typed and
  then stopped in prepared lowering with "required lifetime does not outlive
  its target". The closure's parameter region comes from the callee's
  instantiated lifetime. Its body passes the parameter to a captured
  `FnMut(&'a T, &'a T)`.
  - This predated the change. The base and the earlier 22902d20 failed the same
    way without any array pattern (`d-ident`).
  - With a function item in place of the captured `FnMut` (`d-ident-fn`), the
    case lowered.
  - The [closure creator requirement change](pa31-closure-creator-requirements.md)
    fixes it, and both cases now lower.
- `let [a, b] = p` inside a closure body stops at "compound capture patterns
  are not implemented" (`d-let-in-closure`). That gap is separate and predates
  this change.

## Contract

**PA31** (`pa31/tests/spec/array-length-patterns/`, thirteen source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`).

| Case | Form | Rust | RustGM |
| --- | --- | --- | --- |
| `infer-len` | `call(\|[a, b]\| *b)` with `FnMut(&[u8; N])` | accept | accept |
| `core-form` | core's `array_windows(v).all(\|[a, b]\| compare(a, b))` | accept | accept |
| `core-form-known` | the same with `array_windows::<T, 2>` | accept | accept |
| `by-value` | `call2(\|[a, b, c]\| c)` with `FnMut([u8; N])` | accept | accept |
| `let-infer` | `let [a, b] = make()` | accept | accept |
| `ref-pattern` | `call(\|&[a, b]\| b)` | accept | accept |
| `nested-infer` | `call3(\|[[a, b], [c, d]]\| *d)`, two lengths | accept | accept |
| `rest-known` | `call::<3, _>(\|[a, ..]\| *a)` | accept | accept |
| `let-ergonomic` | `let [a, b] = w` with `w: &'a [T; 2]` | accept | accept |
| `rest-unknown` | `call(\|[a, ..]\| *a)` | E0730 | typing |
| `known-mismatch` | `call::<3, _>(\|[a, b]\| *b)` | E0527 | typing |
| `match-two-lengths` | arms `[a, b]`, then `[a, b, c]` | E0527 | typing |
| `generic-len` | `let [a, b] = x` with `x: [u8; N]` | E0308 | typing |

Diagnostics in batches 3 and 4 are not graded:

- `closure-ergonomic`, a known-length closure parameter;
- `d-tuple`, a tuple pattern;
- `d-ident` and `d-ident-fn`, without a pattern;
- `d-let-in-closure`.

Their copies in `obj/parallel/array-pattern/diag` append `fn main() {}`, so the
CLI's LowIR path runs prepared lowering.

## Qualification

Focused receipt 1794518100658477797 passes in 49.9s. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the inference
and patterns suites and the CLI, plus four exact tests:

- the two new tests in `dev/tests/array_length_patterns.rs`. The first runs the
  packet's typing through the CLI and prepared lowering for every case that
  types. It requires the exact typing reason for each rejection and the exact
  lowering reason for the two gaps. The second checks bounded work. Since the
  [closure creator requirement change](pa31-closure-creator-requirements.md),
  the first requires every accepted case to lower; that checkpoint's receipts
  qualify it.
- `patterns::array_patterns_keep_huge_rest_storage_sparse` and
  `product_errors_and_move_conflicts_match_rustc`, for existing array patterns.

Native receipt 1794517931410841561 passes in 40.3s with the CPPGM backend. It
runs the new ignored test's four ordinary-mode programs against rustc:

- a closure pattern fixes a callee's `N`;
- a `let` pattern fixes a generic constructor's `N`;
- a by-value closure pattern;
- a windows loop in core's form.

Receipt 1794517872108986075 passed the same selection before the packet
README and tracker entries. Earlier attempt 1794517548025290366 found
`core-form`'s lowering gap.

No text reference contains the replaced error, so no text oracle changes.
`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/array-pattern/scaling-raw.txt`), for 8/32/128
functions, each passing `|[a, b]| *b` to `call`:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 37268 | 5844 | 431 |
| 32 | 62012 | 5940 | 599 |
| 128 | 160988 | 6324 | 1271 |

That is 1,031 inference steps, 4 canonical steps and 7 trait selections per
function, linear across widths.

## Core progress

Exploratory pin wip1 (6043c8da, `obj/parallel/array-pattern/wip1-dev-fast`)
carries this change. No core observation of it has run yet. The unchanged 120s
observer has recently stopped before item 7631 under shared load, well before
8380.
