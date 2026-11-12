# Negative receiver candidates

Pinned core's `Result<&mut T, E>::cloned` 6721 (`result.rs:1795`) failed with
"invalid method trait selection". Its body is `self.map(|t| t.clone())` with
`t: &mut T` and `T: Clone`. Core also declares
`impl<T: PointeeSized> !Clone for &mut T {}`.

## Diagnosis

`obj/parallel/result-cloned/core/declared.json` records three declared
source-core batches. Each uses the prefix of
`pa31/tests/spec/default-const-closures/count.rs` through its Fn traits, then
a reduced `Result`-like enum.

1. Batch 1 carried core's `Clone for &T` impl (`*self`) without `Copy for &T`.
   rustc rejects it (E0507), so those forms are not reductions. RustGM accepts
   them, which is the existing `&T` Copy gap recorded in the
   [premise input checkpoint](pa20-premise-inputs.md).
2. Batch 2 shows that the negative impl alone produces the failure. Every
   method-call form fails, whether closure, annotated closure or direct
   parameter, with or without core's `&T` impls. The path call `T::clone(t)`
   and the forms without the negative impl pass.
3. Batch 3 checks that an ordinary bound obligation `need::<&mut u8>()` keeps
   its existing rejection.

A temporary instrumented build (`wipdbg1`) printed the error that the census
fold hides: "conditional negative implementation requires scoped debt
payment".

The receiver search first tries the autoref step `&&mut T`. That step requires
`&mut T: Clone`. Selection completes with the negative impl as the only
candidate, but the negative's own lifetime equation with the step's region is
unpaid. Only automatic traits pay such private equations
(`negative_equalities.rs`), so the refutation stays conditional. Ordinary trait
use rejects every conditional refutation at the selection boundary, and the
probe stopped there instead of trying the next step, `T: Clone`.

## Change

- **Typed boundary.** The root guard now returns a typed
  `Failure::Conditional` that keeps the refutation. Every existing entry point
  maps it to the same message as before.
- **Receiver steps.** A new `select_applicable` entry returns that refutation to
  the method receiver step, which consumes it with `Refutation::exclude` as an
  inapplicable candidate. Lookup then continues to autoderef, as in rustc.

This is sound for the probe because a refuted root has no positive candidate. A
positive candidate completing beside the negative makes the pair unordered,
which defers. A deferred or partial candidate also defers. Whether the
negative's debts hold decides only whether the negative itself applies. The
step publishes no negative fact and caches no absence.

No new engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- Ordinary obligations, path conditions (`paths/conditions.rs`) and coherence
  keep the conditional boundary. `bound-obligation` and PA29's `clone-mut`
  still reject with that text, where rustc reports E0277. Core's reached bodies
  only need the receiver step.
- Bound-region receiver goals (`begin_quantified_borrowed`) keep the boundary.
  No reached core body needs them.
- `missing` now rejects with "no applicable method for this receiver"; it
  previously failed with "invalid method trait selection". rustc reports
  E0599.

## Contract

**PA31** (`pa31/tests/spec/negative-receivers/`, ten source-core reductions).
They agree with rustc 1.98.0 10/10 (`outcomes.json`).

| Case | Form | Rust | b41f1792 |
| --- | --- | --- | --- |
| `core-closure` | core's `self.map(\|t\| t.clone())` | accept | reject |
| `annotated` | the closure parameter typed `&mut T` | accept | reject |
| `direct` | the call on a `&mut T` parameter | accept | reject |
| `reference-closure` | core's form with core's `&T` impls | accept | reject |
| `reference-direct` | the direct call with core's `&T` impls | accept | reject |
| `path` | `T::clone(t)`, a control | accept | accept |
| `positive-closure` | no negative impl, a control | accept | accept |
| `shared` | the `Result<&T, E>` form, a control | accept | accept |
| `missing` | no `T: Clone` premise | E0599 | reject |
| `bound-obligation` | `need::<&mut u8>()` under `T: Clone` | E0277 | reject |

Ordinary mode cannot express the negative impl. Two declared ordinary probes
of the same call, `ordinary/mut-clone` and `ordinary/mut-clone-closure`
(`obj/parallel/result-cloned/ordinary/`), are accepted by rustc. RustGM
rejects both before and after this change, with "concrete dispatch cannot use a
symbolic assumption" and "closure signature lifetime interface differs from
its body". These are separate ordinary-mode gaps, recorded and not fixed here.

## Qualification

Focused receipt 1794500346852324029 passes in 233.4s under external load. It
covers formatting, the architecture audit, strict Clippy for the inference,
linking and traits suites, the CLI and the private library, plus seven exact
tests and PA20's and PA24's complete text oracles with the dev-fast CLI:

- the two new tests (`dev/tests/method_selection/refuted_receivers.rs`): the
  packet through the CLI, and bounded work;
- projected receivers with bound checks and trait fallback;
- the PA29 modifier and automatic impl controls, whose `clone-mut`,
  `deref-shared` and `arguments-send` rejections keep the conditional text;
- private source-coherence refutation outcomes and owned negative equations.

Earlier attempts stopped at formatting and at the architecture audit. The
test's first name repeated the `negative` filename stem, so it was renamed. A
later attempt named an ignored native test, and another found that four private
invariants read `Search::result` as a message, which now stays a test-only
boundary. No graded reference contains "invalid method trait selection", so no
graded outcome can change. `make test-upstream` verifies the pinned upstream
fixtures. No graded case or reference is added, and no planned assignment is
activated.

Bounded work (`obj/parallel/result-cloned/scaling-raw.txt`), for 8/32/128
functions calling `t.clone()` on `&mut T` beside the negative impl:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 29960 | 5846 | 399 |
| 32 | 32864 | 5966 | 471 |
| 128 | 44480 | 6446 | 759 |

That is 121 inference steps, five canonical steps and three trait selections per
call over the prefix's fixed cost. Adding 64 unrelated `Clone` impls adds a
constant 960 inference and 1,716 canonical steps at every width, and no trait
selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/result-cloned/wip1-dev-fast`
(9c1bd81d) carries the change before the test and packet work. The unchanged
120s observer ends at its deadline after 121.022s, with 6 partial records and
13,613 attempts, under external load of 13–24. The last item is 7607,
slice/ascii.rs:339. Against clean b41f1792 (14 records, 14,292 attempts, last
item 8825) it clears `cloned` 6721. No new failure appears among previously
passing reached bodies.

The first remaining failure is `AtomicPtr::new` 6835 at sync/atomic.rs:1478
("constant storage layout requires a closed normalized type"), then
`into_inner` 6842.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
