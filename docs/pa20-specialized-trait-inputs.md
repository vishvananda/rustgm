# Specialized trait inputs

Pinned core's `Zip` `fold` 5290 (`iter/adapters/zip.rs:275`) failed with "trait
obligation requires an unambiguous matching assumption or implementation;
diagnostic contract: call inputs remain unresolved". The body is in the
`TrustedRandomAccessNoCoerce` impl of the private specialization trait
`ZipImpl<A, B>`, and calls `ZipImpl::size_hint(&self)`. Core has three
`impl<A, B> ZipImpl<A, B> for Zip<A, B>` headers: a default one over
`Iterator`, one over `TrustedRandomAccessNoCoerce` and one over
`TrustedRandomAccess`.

The path leaves the trait inputs as inference variables:
`Zip<A, B>: ZipImpl<?X, ?Y>`. Under that body's assumptions, the first two
impls both apply, and each resolves the inputs to `ZipImpl<A, B>`. Trait
selection let specialization order two proved candidates only for a fully
resolved goal (`frames.rs` `preferred_candidate`), so the pair stayed
unordered and the goal ambiguous. rustc's winnowing drops a specialized
candidate whenever its specializing peer must apply, even with inference
variables in the goal.

## Change

- **Agreeing inputs.** `preferred_candidate` now also asks specialization to
  order two proved implementation candidates of a goal with inference
  variables when both recorded the same resolved goal. The choice then cannot
  change inference. Everything else is unchanged: the pair is ordered by the
  existing `specialization::order`, only a use query may complete a pair
  comparison, and a crate without a specialization graph is always unordered.
- `specialization::order`'s contract comment now names both cases.

No new engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- This is a subset of rustc's rule. When candidates resolve the inputs
  differently, such as `impl<T> P<T> for Q` and a specializing `impl P<u8> for
  Q` at `P::p(&q)`, rustc drops the specialized impl and infers `u8`. RustGM
  keeps that goal ambiguous: the generic candidate leaves `?T` unresolved and
  defers. The declared `specialized-distinct` reduction records it (rustc
  accepts, RustGM rejects) outside the course packet, as a rejects-valid
  limitation. No current core blocker needs it.
- Region differences between the resolved goals keep the pair unordered.

## Reductions

`obj/parallel/zip-specialization/core/declared.json` records two declared
batches on the capture prefix with `min_specialization`. Rust outcomes were
declared before rustc 1.98.0 and any RustGM run; the RustGM outcomes are
observations against clean pin ed7d5ed7.

## Contracts

Nine declared reductions, `pa31/tests/spec/specialized-trait-inputs/`, pass
rustc 1.98.0 9/9 (`outcomes.json`).

| Case | Form | Rust | ed7d5ed7 |
| --- | --- | --- | --- |
| `trait-path` | `ZImpl::size(&self)` in the specializing impl's by-value method, as in core | accept | reject |
| `trait-path-ref` | the same through `&self` | accept | reject |
| `qualified` | `<Self as ZImpl<A, B>>::size(&self)`, inputs known | accept | accept |
| `method` | `self.size()` | accept | reject |
| `default-body` | the path in the default impl, where only it applies | accept | accept |
| `three-level` | a third impl over a `Faster: Fast` marker, as with `TrustedRandomAccess` | accept | reject |
| `caller-concrete` | `ZImpl::size(&z)` for `Z<S, S>` with `S: Fast` | accept | reject |
| `call-concrete` | the same called from `main`, LowIR | accept | reject |
| `distinct-inputs` | `impl P<u8>` and `impl P<u16>` without specialization | E0283 | reject |

In `call-concrete`'s LowIR, both calls reach the specializing bodies, which
return 1. No default body, which returns 0, is lowered; only the entry point
returns 0. `distinct-inputs` keeps RustGM's unresolved-call-input rejection.

## Qualification

Focused receipt 1794485005340361996 passes in 74.8s. It covers formatting, the
architecture audit, strict Clippy for the projections suite and the CLI,
PA20's complete text oracle with the dev-fast CLI, and three exact tests:

- the new CLI outcome test, with the `call-concrete` lowering check;
- the new scaling test;
- the rigid candidate header outcomes, the neighbouring candidate relation.

A first run that also selected the private winnowing test
(`sema::specialization::selection_tests`) exhausted the five-minute budget in
its cold private Clippy build (receipt 1794484706929253742) before any test
ran. Instead, all nine existing `specialization-selection` controls that test
reads give its expected outcomes through exploratory pin aec1ea8c's CLI
(`obj/parallel/zip-specialization/selection-controls.json`). Full CI runs the
private test.

Bounded work (`obj/parallel/zip-specialization/scaling-work.txt`), for 8/32/128
`ZImpl::size(z)` calls on `Z<S, S>`:

| Calls | Inference | Canonical | Trait selections |
| --- | --- | --- | --- |
| 8 | 38793 | 7488 | 519 |
| 32 | 64977 | 9888 | 903 |
| 128 | 169713 | 19488 | 2439 |

That is about 1091 inference, 100 canonical and 16 selection steps per call:
two impl trials and their marker conditions. Adding 64 unrelated specialized
pairs, each with its own trait and self type, adds a constant 640 inference and
24,640 canonical steps for their own declarations at every width, and no trait
selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/zip-specialization/wip1-dev-fast`
(aec1ea8c) carries the change before the test and fixture work. The unchanged
120s observer ends at its deadline after 120.860s, with 44 partial records and
15,680 attempts. The last item is 11021, x86/avx512f.rs:223. Against clean
ed7d5ed7 (41 records, 15,245 attempts, last item 10458) it clears `fold` 5290.
No new failure appears among previously passing reached bodies.

The first remaining failure is `count` 5712 at iter/traits/iterator.rs:229,
behind the diagnostic formatting fence. `max`, `min`, `max_by_key` and
`min_by_key` follow with the same text, and `last` 5713 needs a scoped
continuation for source const callable conditional obligations.

**Clean pin.** Pin 3a1a6772 is built from clean source bcbafa25a, with 3912
Rust hashes and an empty patch
(`obj/parallel/zip-specialization/candidate-dev-fast`).
Formatting and the test files changed the sources after the exploratory build,
so it is not byte-identical to aec1ea8c. Its unchanged observation ends at its
deadline after 120.882s, with 43 partial records and 15,502 attempts. The last
item is 10751, x86/avx2.rs:2314. Against clean ed7d5ed7 it clears `fold` 5290,
with no new failure among previously passing reached bodies, and the first
remaining failure is again `count` 5712. The postcheck verifies all 722 source
hashes, 375 opens, six configuration bindings, unchanged immutable bindings and
the 3912 head blobs. The progress SHA is
`925a7e486f7abc20c747ac9c36af9348fe4cd6ae1358e65a204efe1c3dc86598`.

**Measurement.** Sixty exclusive A/A plus ABBA accepting observations against
ed7d5ed7 preserve complete outputs. The controls are an ordinary import, an
ordinary `min_specialization` default method (PA19
`default-methods/trait-blanket`), source-core reference equality, a source Copy
capture (`source-copy-captures/inner`), and the `qualified` and `default-body`
cases, which both pins accept. Paired wall ratios are
0.973/0.960/0.980/1.021/1.011/0.962, against A/A
0.724/0.957/0.976/0.952/0.972/1.063, at 10–442 ms. The 10 ms import control's
A/A shows the timer noise at that size. External load stayed near a load
average of 9. No optimized, neutrality or original-core speed is claimed; the
bounded-work test above qualifies the work.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
