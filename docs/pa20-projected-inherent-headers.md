# Projected inherent headers

Six pinned core adapter bodies fail at `.into_try()`: `MapWhile`, `Scan`,
`Skip`, `Take`, `TakeWhile` and the adapters-module `try_fold`/`try_rfold`.
The first is try_fold 4961 at map_while.rs:62. Its receiver is the
`ControlFlow<R, Acc>` returned by an inner `try_fold`, and the method lives in
`impl<R: Try> ControlFlow<R, R::Output>`. That implementation header contains
a projection.

Method applicability instantiated the header as
`ControlFlow<?R, <?R as Try>::Output>` and compared it structurally with the
receiver. The projection leaf made that a hard inference error, reported as
"invalid method candidate inference graph". The minimal
`fn code<R: Try>(c: ControlFlow<R, R::Output>) -> R { c.into_try() }` failed the
same way. With fresh path variables, `ControlFlow::into_try(c)` happened to work
because the variables bind the projection without a comparison. The explicit
`ControlFlow::<R, R::Output>::into_try(c)` failed in the body checker.

## Change

- **Applicability.** When the selected owner's declared self type or receiver
  type contains a projection, `applicability::prepare` relates it with the new
  `Context::equate_shape_goals`. That shares `equate_deferred`'s blocked-pair
  unification but registers nothing for resumption. Each postponed
  associated-result leaf becomes a projection-equality condition, proved by the
  existing method trait selection under the caller's assumptions. This mirrors
  rustc's probe, which normalizes an impl header into fresh variables plus
  projection obligations. Projection-free candidates keep `equate_shape`
  unchanged.
- **Owner binding.** `bind_method_owner` registers a projected header's
  occurrences with the body normalizer before relating it to the prefix. The
  existing deferred equation then relates it after normalization, as
  `shape_equation` already does for binder folds. It is never compared
  structurally.

No hidden representation is read and no new engine, model field, backend,
storage policy or compiler limit is added.

## Contracts

Ten Rust 1.98 outcomes were declared before any compiler ran
(`obj/parallel/candidate-graph/packet/declared.json`): seven accept and three
reject, all ordinary PA20 programs.

The seven positives check returned values and execute natively:

| Case | Form |
| --- | --- |
| `assoc` | `ControlFlow<R, R::Output>` receiver |
| `bound` | `R: Try<Output = A>` receiver |
| `concrete` | concrete receiver |
| `explicit-path` | explicit-argument path |
| `inferred-path` | inferred path; a baseline control |
| `call-result` | receiver is a call result |
| `by-ref` | `&self` autoref |

The baseline 0665938a rejects all six non-control positives.

Two of the rejections are course cases. `concrete-mismatch`
(`Flow<Num, u16>`) and `missing-try` reject in Rust (E0599), and the candidate
rejects them for the intended cause, "no applicable inherent method for this
receiver".

The third, `unbound` (`fn code<A, R: Try>(c: Flow<R, A>)`), stays a required
contract in `pending.json`. Rust rejects it with E0599, and the candidate also
rejects it. But method selection keeps `<R as Try>::Output == A` Deferred under
a bare `R: Try`, so the generic body normalizer is what rejects, not candidate
refutation. Its outcome agrees, but its cause is not qualified as the intended
rejection.

Reductions with source-protocol shapes (`closure`, `helper`, `split`) live in
`obj/parallel/candidate-graph/reduce/`. Two reducer prelude errors were caught by
rustc before any claim: a missing `future_trait`, and a missing
`ControlFlow: Try`. Both RustGM pins agreed with rustc on those inputs.

## Qualification

Focused receipt 1794476432590185735 passes in 42.5s: full formatting, the
architecture audit, strict Clippy for both suites and the CLI, and eight exact
tests. The tests cover the new outcome and scaling tests, plus the existing
projected-receiver, projected-candidate, bound-check/trait-fallback,
lookup-requirement and property-condition regressions. Ignored native receipt
1794476537101008170 executes all seven positives through the maintainer backend
harness. Before generation, the locked CPPGM backend validated, compiled and ran
every candidate LowIR program (`packet/native.json`).

Bounded work, with calls 8/32/128:

| Calls | Inference | Selection steps | Selection probes |
| --- | --- | --- | --- |
| 8 | 1739 | 900 | 9 |
| 32 | 6467 | 3564 | 33 |
| 128 | 25379 | 14220 | 129 |

That is about 111 selection steps and one probe per call. Adding 128 unrelated
types changes no counter. Adding 128 unrelated inherent impls with a same-named
`into_try` method changes no selection counter. Only their own bodies add
inference and canonical work.

Text qualification (`obj/parallel/candidate-graph/text-review/passed.json`) uses
exploratory candidate 05289eb6, built from this checkpoint's exact tree:

- Every positive crosses the independent Rust/CPPGM object bridge, and both
  course rejections are independently rejected by Rust.
- All 1595 prior PA20 cases reproduce, as do PA1–PA19, PA21–PA24 and the
  released PA25 const-machine unit. All 28,333 prior reference hashes are
  unchanged.
- Normal scoped generation then adds nine reviewed triples, in batch
  `pa20/reference-batches/1794476670166608559-4178f67efca9.json`.

PA20 now has 1604 cases (1147 accept). Upstream verification, the maintainer
suite registration, layout, metadata and harness unit checks pass. No planned
assignment is activated.

## Clean pin and core progress

Clean published source b1063aab2 freezes CLI
`05289eb6710bc876240e420a44a21cb036912f3494d838ed91c2f640feb759da`, with 3908
Rust hashes and an empty patch (`obj/parallel/candidate-graph/candidate-dev-fast`).
It is byte-identical to the text-qualification candidate.

**Original core.** The unchanged 120s observer ends at its deadline after
121.396s, with 49 partial records and 14,192 attempts. The last item is 8659,
str/traits.rs:125. Against clean 0665938a it clears all six candidate-graph
bodies: try_fold 4961/5066/5196/5247/5350 and try_rfold 5092. No new failure
appears among previously passing reached bodies.

The postcheck verifies all 722 source hashes, 375 opens, six configuration
bindings, 3908 current and immutable compiler hashes, and unchanged profile and
limits. The progress SHA is
`2d909e79ca43832a4ecb222c321d8076356a3cd7737fd2f3b56b46a3fe1ed29c`.

The first remaining failure is next_if_eq 5023 at peekable.rs:318: an
unresolved trait obligation behind the "typed source shape" diagnostic
formatting fence. Then come first_size/other_size (quantified body
requirements) and fold 157:7745 (unresolved call inputs). The exploratory
789ed468 observation agrees: 50 records, last item 9248.

**Measurement.** Forty exclusive A/A plus ABBA accepting observations against
0665938a preserve complete outputs. The controls are an ordinary import, the
method-priority graph, a projected receiver lookup and the inferred-path
projected header. Paired wall ratios are 1.080/1.137/1.073/1.071, against A/A
0.733/0.755/1.013/1.055, at 20–45 ms. Within one binary, runs vary by about
±20%, including on the unaffected ordinary control. No optimized, neutrality or
original-core speed is claimed; the bounded-work tests above qualify the work.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
