# Unresolved projection conditions in lookup

Census 24 stopped at `Iterator::intersperse`, `iter/traits/iterator.rs:698`:
`Intersperse::new(self, separator)` reported "candidate properties require
associated type normalization". Its inferred prefix has an unresolved iterator
argument. The lookup's Sized builtin reaches that projection; Clone is a
separately polled trait condition. The earlier description calling Clone the
failing builtin was corrected by the diagnostic trace before implementation.

## Rule and scope

The Projection arm of `types/infer/properties.rs` asks the existing charged
`Context::resolved` walk about its reachable inputs. Unresolved inputs remain
possible, as an unresolved variable already does. A resolved projection retains
the existing normalization fence; every other inference failure remains an
error. This filter grants no Copy, Clone or Sized proof. The selected callable
independently proves all of its obligations after argument inference.

No normalization service, binder rule, metadata row or dense field changes.
Normalizing resolved projections under the actual caller's premises remains a
follow-up after nested-proof reuse.

## Controls and bounded work

The thirteen files in `pa31/tests/spec/projection-lookup-conditions/` preserve
the exact bytes of the predeclared h01–h13 probes. Their pinned rustc 1.98.0
outcomes are reused by source hash. The candidate matched every corrected probe
prediction before fixture execution. The packet's outcomes.json records every
input; PA31 remains planned, without a graded manifest or generated text oracle.

- Six fixes accept: `free-inferred`, `core-default-method`,
  `inferred-concrete`, `free-copy`, `free-sized`, `method-unresolved`.
- `inferred-concrete-fail` still rejects, now at the selected callable's body
  proof, matching rustc's E0277. `explicit-concrete-fail` retains its rejection.
- `unique-inherent` and `explicit-concrete-fallback` remain accepted. The
  latter already selected Make::new on the base; there is no witnessed
  error-propagation bug to record for that control.
- `explicit-rigid`, `explicit-concrete`, `method-rigid` keep byte-identical
  normalization diagnostics, as named known rejects-valid.

Three new private invariants check these outcomes, exact reachable-prefix work
and exhaustion, and lookup scaling. A property request pays one request step
plus its resolution visits; repeating it without equations pays one cache
lookup. Exhaustion stays an error. With two callers and K = 1, 4, 8 projection
conditions, added selection work is exactly 104K for path lookup and 116K for
unresolved-receiver lookup. Selected owners and all five other Methods counters
equal the twins with no projection condition. Existing supertrait and empty
binder observation lines remain byte-identical to their saved baselines.

The paired survey covers 2,032 inputs. Its projection-bound spelling excludes
an ordinary double-colon continuation; qualified projection forms are also
included. Four bounded runs at two processes compare status and complete output
hashes. Only the six fixes and the one diagnostic move above differ. Signals,
timeouts and other infrastructure statuses fail the survey.

## Original core and evidence

Census 25 uses immutable CLI `e9b628a3`, with all 722 original inputs, 375
opened sources, target/profile and limits unchanged. It checks 42,310
declarations with none failed or blocked, then stops in `local_annotations` at
"constant storage layout requires completed fields". Body checking remains
unreached. This is analysis evidence, not a core artifact or passing build.

A separately declared analysis-only two-emitter trace locates the new stop at
`sync/atomic.rs:1590`, in `AtomicPtr::from_mut`'s assertion involving
`align_of::<AtomicPtr<()>>()`. The aggregate declaration exists, but its storage
shape is unprepared. The stop is file 199, establishing that file 175 completed;
the former Intersperse diagnostic is gone. Instrumentation was saved and fully
reverted, and neither diagnostic emitter is in the implementation. This new
storage lead precedes the provisional `Split::new` lead. The f01 leak-check
repair still gates widening bound-region lookup.

Evidence is under `obj/parallel/census-batch22/`: corrected declarations,
immutable pins and build diffs, probe results, observations, four survey parts,
census25 and attribution. Exact source-binding supplements reconstruct each
pin's source bytes from its saved HEAD/diff, including pre-formatting bytes and
the temporary diagnostic emitters; raw observer bindings remain retained.
Fastcheck `1794260103818124314` passes 11 exact tests, production/private strict
Clippy, formatting and architecture checks. Two earlier runs exhausted the
development budget during compilation, before tests; their failed receipts
remain infrastructure evidence. Full successor qualification belongs to CI.
