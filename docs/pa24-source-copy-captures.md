# Source Copy captures

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

## Core progress

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

A clean pin, its unchanged observation with the postcheck, and measurement
follow publication.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
