# Source Copy captures and repeats

Pinned core's `StepBy` helpers `first_size` 5171 and `other_size` 5172
(`step_by.rs:232/237`) failed with "quantified body requirement needs a
matching assumption or implementation". Each returns
`impl Fn(usize) -> usize` from a `move` closure that divides by a captured
`step: NonZero<usize>`. `NonZero<T>` is Copy only through core's
`impl<T> Copy for NonZero<T> where T: ZeroablePrimitive {}`.

Closure capture classification (`sema/bodies/closures/uses.rs`) decides
whether a by-value use of a captured place copies or moves. It asked only the
derive-based Copy query (`types/copy.rs`), which never sees a source
`impl Copy`. The use was classified as a move, the closure became `FnOnce`, and
the opaque `impl Fn` bound failed. rustc classifies the capture with
`type_is_copy_modulo_regions`, which selects `Copy` through the trait system.
The body's borrow checker already decided copies through the source `Copy`
trait (`body/copy_requirements.rs`), so only the capture classifier disagreed.

## Change

- **Capture classification.** When the derive query says a captured type is
  not Copy and the crate is a source library (`source_bindings` present),
  `capture_copy` now selects the library's `Copy` language item
  (`compiler_items::copy_declaration`) for the capture's term. It runs under
  the body's trait assumptions and query environment, after the existing
  `may_have_subject` filter. A proved selection classifies the use as a copy.
  Absent, deferred or refuted selection keeps it a move, and a refutation still
  reports its own rejection.
- **Probe only.** As with rustc's copy-modulo-regions query, the selection only
  classifies the capture. Its evidence is dropped, and any region equalities it
  records are truncated. The body's source Copy requirements prove every actual
  copy again, with all their lifetime and formation debts.

Ordinary programs have no source bindings and return before the probe. No new
engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- An extra selection runs for each source-library capture that the derives
  leave non-Copy. Copy-by-derive, primitive, shared-reference and numeric-hole
  captures do not reach it.
- Pending proof stays a move rather than waiting. A capture whose Copy proof
  depends on a still-unresolved input is therefore still `FnOnce`. No current
  reduction needs that.
- Ordinary mode still rejects a manual `impl Copy` ("implementation requires a
  trait declaration"). That is a separate rejects-valid PA close-out item.

## Reductions

`obj/parallel/step-size/core/declared.json` records every attempt and batch.
Outcomes were declared before each ran against rustc 1.98.0 and clean pin
487e0881.

| Step | Prelude or cases | Result |
| --- | --- | --- |
| attempt 1 | `pub const` `Fn` traits under `staged_api` without const stability | rustc and RustGM both rejected; no claim |
| attempt 2 | `const impl Clone for NonZero` without matching const stability | rustc and RustGM both rejected; no claim |
| attempt 3 | `source-div` foundation plus `const_ops`-unstable `Fn` traits and `NonZero` | reproduces `other-size` and the returned copy |
| batch 4 | inner, conditional and `FnMut` captures; borrow, `FnOnce` and once-called local controls | captures fail; controls pass |
| batch 5 | Clone-only, failed-condition, parameter-premise and Clone-premise controls | 4/4 exit outcomes as declared |
| batch 6 | root `main` calls the closures twice | accept, and `--emit-lowir` lowers |

Batch 5 noted Rust's expected error as E0525, but Rust reports E0507 (move out
of a captured variable in an `Fn` closure). The declared exit outcomes were
right, and the note records the correction.

## Contracts

Twelve declared reductions, `pa31/tests/spec/source-copy-captures/`, pass
rustc 1.98.0 12/12 (`outcomes.json`).

| Case | Form | Rust | 487e0881 |
| --- | --- | --- | --- |
| `other-size` | core's `move \|n\| n / step` | accept | reject |
| `returned-copy` | returns the captured `NonZero<usize>` | accept | reject |
| `inner` | manual `impl Copy` struct | accept | reject |
| `conditional` | `impl<T: Copy> Copy for G<T>` at `G<usize>` | accept | reject |
| `fn-mut` | `impl FnMut` result | accept | reject |
| `fn-bound` | closure passed to `F: Fn` | accept | reject |
| `parameter-premise` | `G<T>` under `T: Copy` | accept | reject |
| `call-size` | `other-size` called twice from `main`, LowIR | accept | reject |
| `call-bound` | `fn-bound` called twice from `main`, LowIR | accept | reject |
| `clone-only` | Clone without Copy | E0507 | reject |
| `failed-condition` | `G<Owned>` | E0507 | reject |
| `clone-premise` | `G<T>` under only `T: Clone` | E0507 | reject |

The rejections keep the quantified body requirement text, because the closure
is `FnOnce` and cannot meet its `Fn` bound. Ordinary course fixtures cannot
express a manual `impl Copy` until ordinary mode accepts one, so PA24 gains no
text outcome here.

## Qualification

Focused receipt 1794482201388217456 passes in 91.1s. It covers full
formatting, the architecture audit, strict Clippy for the closures and frontend
suites and the CLI, and five exact tests:

- the new CLI outcome test (both modes for the two `call-*` cases) and the
  scaling test;
- the existing source capture-mode and capture-plan tests, and source-core
  callback formation through the CLI (PA19/24/25/31).

Receipt 1794482346658352744 runs all 40 `source_closures` tests, and PA24's
complete text oracle reproduces with the dev-fast CLI.

Bounded work (`obj/parallel/step-size/scaling-work.txt`), for 8/32/128 captured
manual-Copy closures:

| Captures | Inference | Canonical | Trait selections |
| --- | --- | --- | --- |
| 8 | 31734 | 6196 | 375 |
| 32 | 39102 | 6700 | 375 |
| 128 | 68574 | 8716 | 375 |

That is about 307 inference and 21 canonical steps per captured closure, body
included. Adding 128 unrelated source Copy types, each with its own `Clone`
body, adds a constant 384 inference and 3124 canonical steps at every width,
and no trait selection.

## Clean pin and core progress

Clean published source d0e926179 freezes CLI
`12358002bd7972e54b7b2bf7009ccf7639e5c72605201191eb825bd006e311e6`, with 3911
Rust hashes and an empty patch (`obj/parallel/step-size/candidate-dev-fast`).
It is byte-identical to the exploratory pin below.

**Original core.** The unchanged 120s observer ends at its deadline after
120.908s, with 46 partial records and 14,613 attempts. The last item is 9415,
task/wake.rs:681. Against clean 487e0881 (28 records, 13,270 attempts, last
item 6977) it clears `first_size` 5171 and `other_size` 5172. No new failure
appears among previously passing reached bodies. The first remaining failure is
`fold` 5290 at zip.rs:275.

The postcheck verifies all 722 source hashes, 375 opens, six configuration
bindings, unchanged immutable bindings, and the 3911 compiler hashes against the
blobs of the producer's recorded head. The progress SHA is
`37271de7a631b5976d5196f8b6bbc2ac552d6065b275ac0873eeb1e2be3b0e8e`.

**Measurement.** Fifty exclusive A/A plus ABBA accepting observations against
487e0881 preserve complete outputs. The controls are an ordinary import, an
ordinary closure over a projected item, source-core reference equality, and two
source-core opaque `Fn` closures that copy a `usize` capture or borrow a
`NonZero<usize>` capture. Paired wall ratios are
0.878/0.965/0.967/1.025/1.001, against A/A 0.983/1.024/0.807/1.022/0.984, at
19–422 ms. External load rose to a load average near 30 as the run started. No
optimized, neutrality or original-core speed is claimed; the bounded-work test
above qualifies the work.

An accepted source-core program that reaches the probe, such as a once-called
closure or an `impl FnOnce` that copies a `NonZero` capture, prints different
`--emit-types` type numbering on the two pins, because the probe's selection
interns its goal types. Its acceptance is unchanged, and no ordinary output
changes (PA24's text oracle reproduces), but such programs cannot be
output-equality timing controls.

**Exploratory observation.** Pin `obj/parallel/step-size/wip1-dev-fast`
(12358002) carries the change before the test and fixture work. The unchanged
120s observer ends at its deadline after 121.049s, with 45 partial records and
14,240 attempts. The last item is 8723, str/traits.rs:700. Against clean
487e0881 (28 records, 13,270 attempts, last item 6977) it clears `first_size`
5171 and `other_size` 5172. No new failure appears among previously passing
reached bodies.

The first remaining failure is `fold` 5290 at zip.rs:275 (unresolved call
inputs). Later reached failures include `count`/`max`/`min` in
iter/traits/iterator.rs (diagnostic formatting fence) and `new` 5862 in
net/display_buffer.rs, "repeated array element must implement Copy". That body
repeats `MaybeUninit<u8>`, which is also Copy only through a manual impl, so it
is likely the same derive-only query in array repetition.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.

## Repeat operands

The same derive-only query rejected `[x; N]` with N > 1 in sema
(`aggregates.rs` `validate_repeat`) when `x` is Copy only through a source
`impl Copy`. Core's `DisplayBuffer::new` 5862 (net/display_buffer.rs:14)
repeats `MaybeUninit::uninit()` for a const generic `SIZE` and failed with
"repeated array element must implement Copy". The body already reads each
repeat operand's source Copy requirement (`copy_requirements.rs` `Repeat`), so
only sema disagreed.

- **Shared probe.** The selection above is now `Checker::source_copy_probe` in
  `sema/bodies/bounds.rs`, beside the body's source Copy context. Capture
  classification and `validate_repeat` both call it. A repeat whose element
  the derives leave non-Copy now asks the probe for the element's `known` term.
  A proved selection accepts. Anything else keeps the existing rejection.
- rustc registers `Copy` for a repeated non-constant operand as an obligation.
  Here the probe only decides early acceptance. The body's source Copy
  requirement proves the obligation again with its debts, and it rejects with
  the same message if that proof fails.

Nine declared reductions, `pa31/tests/spec/source-copy-repeats/`, share the
capture prefix (`obj/parallel/repeat-copy/core/declared.json`) and pass rustc
1.98.0 9/9 (`outcomes.json`).

| Case | Form | Rust | 12358002 |
| --- | --- | --- | --- |
| `manual` | `[UsizeInner(1); 4]` | accept | reject |
| `nonzero` | `[s; 3]` for `s: NonZero<usize>` | accept | reject |
| `conditional` | `[G(1usize); 3]` with `impl<T: Copy> Copy for G<T>` | accept | reject |
| `parameter-premise` | `[x; 3]` for `x: G<T>` under `T: Copy` | accept | reject |
| `const-length` | `[UsizeInner(0); N]` in a const fn, as `DisplayBuffer::new` | accept | reject |
| `call-length` | `const-length` called from `main`, LowIR | accept | reject |
| `length-one` | `[Owned(1); 1]`, no Copy needed | accept | accept |
| `clone-only` | `[Owned(1); 3]` | E0277 | reject |
| `clone-premise` | `[x; 3]` for `x: G<T>` under only `T: Clone` | E0277 | reject |

Focused receipt 1794483519530720203 passes in 113.4s. It covers formatting, the
architecture audit, strict Clippy for the closures, frontend and ownership suites
and the CLI, PA18's complete text oracle with the dev-fast CLI, and six exact
tests: both packet CLI tests, the two-form scaling test, the source capture-mode
test, ordinary generic cleanup (including the ordinary non-Copy repeat
rejection), and source-core callback formation.

Bounded work for 8/32/128 repeat functions
(`obj/parallel/repeat-copy/scaling-work.txt`) is 29702/30974/36062 inference,
6054/6126/6414 canonical and 375 trait selections throughout, about 53
inference and 3 canonical steps per repeat. The 128 unrelated Copy types again
add a constant 384 inference and 3124 canonical steps.

**Exploratory observation.** Pin `obj/parallel/repeat-copy/wip1-dev-fast`
(e18b4674) carries the change before the test and fixture work. The unchanged
120s observer ends at its deadline after 121.012s, with 39 partial records and
14,578 attempts. The last item is 9351, task/poll.rs:148. Against clean
12358002 it clears seven repeat bodies: `DisplayBuffer::new` 5862,
`float_to_decimal_common_exact`/`_shortest` and
`float_to_exponential_common_exact`/`_shortest` 7025–7029, and two `fmt`
bodies, 7054 and 7056. All of them repeat `MaybeUninit`. No new failure appears
among previously passing reached bodies. The first remaining failure is still
`fold` 5290 at zip.rs:275.

**Clean pin.** Clean published source a5fdccff7 freezes CLI
`ed7d5ed7d238a2b3f704a40d9a12701baa5e21ba7950ba560df16ca5799f4753`, with 3911
Rust hashes and an empty patch (`obj/parallel/repeat-copy/candidate-dev-fast`).
Formatting moved source lines after the exploratory build, so it is not
byte-identical to e18b4674. A first unchanged observation starved when external
load rose to a load average near 41. It reached item 5021 with no records and
is kept only as `original-clean-loaded-1/NOT-EVIDENCE.md`. The rerun ends at
its deadline after 120.870s, with 41 partial records and 15,245 attempts. The
last item is 10458, x86/avx.rs:1065. Against clean 12358002 it clears the same
seven repeat bodies, with no new failure among previously passing reached
bodies. The postcheck verifies all 722 source hashes, 375 opens, six
configuration bindings, unchanged immutable bindings and the 3911 head blobs.
The progress SHA is
`1b73fe9ca4d56df3836062f3281612df51a61e5665a8a79cffd5fd8043ffa35f`.

**Measurement.** Sixty exclusive A/A plus ABBA accepting observations against
12358002 preserve complete outputs. The controls are the five capture controls
above plus an ordinary shared-reference repeat (PA18
`functions/accept-repeat-shared`). Paired wall ratios are
0.894/0.987/0.985/0.894/0.894/1.082, against A/A
0.951/1.032/0.989/0.979/0.967/1.060, at 19–593 ms. External load rose to a
load average near 30 as the run ended. No optimized, neutrality or
original-core speed is claimed.
