# Rigid candidate headers

Pinned core's `Peekable::next_if_eq` 5023 (`peekable.rs:318`) failed on
`self.next_if(|next| next == expected)`. The closure compares `&I::Item` with
`&T` under `T: ?Sized, I::Item: PartialEq<T>`. Two impls are candidates for
`&I::Item: PartialEq<&T>`. Core's `&A: PartialEq<&B>` (`cmp.rs:2117`) is proved
by the premise. The blanket `impl<F: FnPtr> PartialEq for F` (`ptr/mod.rs:2592`)
has a header that needs `F = &I::Item` and `F = &T`, so it would need
`T == <I as Iterator>::Item`.

Trait selection related the impl header to the goal with the strict scoped
relation. A projection leaf in that relation returns `Error::Normalization`, so
the blanket stayed a deferred alternative and the goal was ambiguous. rustc
normalizes the goal's inputs first. `<I as Iterator>::Item` has no further
normal form under the caller's environment, so it is a rigid alias. rustc
relates a rigid alias structurally: it equals an unknown or the same alias,
never a parameter or another constructor. The blanket therefore does not apply.

The plain-parameter forms (`A: PartialEq<T>`) already passed, because `T` and
`A` mismatch directly. Simple preludes did not reproduce the failure. The
blanket impl, core's `with_negative_coherence` and the validated `PartialEq`
member family all had to be present (see Reductions).

## Change

- **Candidate header relation.** Trait selection's impl trial now calls the
  new `Context::equate_candidate_header` instead of `equate_predicates`. It
  runs the same relation first. Only when that returns `Error::Normalization`
  and the trial goal carries rigid normal-form marks are all inputs related
  again in blocked mode, in the same disposable trial context. A blocked pair
  that joins a recorded rigid normal form to a parameter or any non-alias
  constructor refutes the header (`Error::Mismatch`). Any other blocked pair,
  including a projection or opaque result from the impl header, still defers.
- **Marks.** The marks already existed. Input normalization's complete rewrite
  records each remaining projection root as a rigid normal form
  (`retain_rigid_results`), and imports carry them into trial contexts.
  Previously only the certified premise relation read them. The doc comments
  now name both readers.

No premise, impl condition, lifetime or hidden representation is read. No new
engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- Two rigid aliases of different items, or of one item with different inputs,
  still defer. rustc would compare them structurally. No current reduction
  needs that.
- Opaque and future heads still defer.
- Readiness (`readiness.rs`) and result templates (`result_templates.rs`)
  keep the strict relation. They do not decide this candidate.

## Reductions

`obj/parallel/peekable-eq/reduce/declared.json` records eight declared batches.
Outcomes were declared before each batch ran against rustc 1.98.0 and the clean
05289eb6 pin.

| Batch | Prelude | Result |
| --- | --- | --- |
| 1 | non-const `PartialEq` | RustGM rejected the prelude itself ("source trait header is outside the pinned provider profile"); no claim |
| 2–3 | const `PartialEq`, direct, closure and method shapes, `&mut` impls | all agree with rustc |
| 4 | added the `FnPtr` blanket | rustc E0119; RustGM agreed |
| 5 | added `Clone`/`Copy for &T` | rustc E0119; RustGM agreed |
| 6 | added `#![rustc_coherence_is_core]` | rustc E0119; RustGM agreed |
| 7 | prefix from `pa31/tests/spec/fn-pointer-coherence/core-references.rs` (core's `with_negative_coherence`) | RustGM rejected the prelude ("source comparison changed its complete member family"); no claim |
| 8 | same, adding core's `ne` and `Not for bool` | reproduces |

In batch 8 the four projected forms fail on 05289eb6 and pass in rustc. The two
plain-parameter controls pass on both. The missing-premise control rejects on
both.

## Contracts

Six ordinary PA20 programs, `pa20/tests/spec/rigid-candidate-headers/`, keep
the shape. A generic `impl<A: Pick<B>, B> Pick<W<B>> for W<A>` sits beside a
header that would need the rigid alias to equal another head. A local `Marker`
blanket stands in for `FnPtr`.

| Case | Form | Rust | 05289eb6 |
| --- | --- | --- | --- |
| `param` | `impl<X: Marker> Pick<X> for X` needs `T == I::Item` | accept, execute | reject |
| `concrete` | closed `impl Pick<W<u8>> for W<u16>` needs `I::Item == u16` | accept, execute | reject |
| `closure` | the obligation inside a closure over the projected item | accept, execute | reject |
| `method` | an impl-parameter projection with a method-level premise | accept, execute | reject |
| `missing-premise` | no `I::Item: Pick<T>` | E0277 | reject |
| `concrete-only` | only the closed header fits by shape | E0277 | reject |

The rejections keep RustGM's shared "trait obligation requires an unambiguous
matching assumption or implementation" text. Ambiguity and absence use the same
message, so the rejection cause is the outcome, not a distinct diagnostic.

Two first drafts were wrong, and rustc caught them before any claim
(`obj/parallel/peekable-eq/packet-attempt1/`). An open `impl<B> Pick<W<B>> for
W<u16>` overlaps, because downstream crates may implement `Pick<_>` for `u16`
(E0119). Both pins agreed with rustc. The closed `concrete` and `concrete-only`
cases replaced them before their execution. An ordinary `==` probe (`std-eq`)
passes on both pins and stays outside the course.

The source-core forms are seven declared PA31 reductions,
`pa31/tests/spec/rigid-candidate-headers/`. Four projected forms accept: direct
`T: ?Sized`, sized, closure and impl-parameter method. Two plain-parameter
controls accept, and a missing premise rejects. `outcomes.json` records 7/7
rustc agreement.

The closure form uses `pa24.closures`, so it lives in PA24 as
`pa24/tests/spec/rigid-candidate-headers/closure.rs`. A first qualification run
was stopped before generation when that placement was found.

## Qualification

Focused receipt 1794479969351084364 passes in 40.9s. It covers full
formatting, the architecture audit, strict Clippy for the suite and the CLI,
and six exact tests:

- the new outcome, source-core CLI and scaling tests;
- the existing associated-result equality, trait-header projection and
  projected inherent header regressions.

Ignored native receipt 1794480010371269767 executes all four positives through
the maintainer backend harness. Before generation, the locked CPPGM backend
validated, compiled and ran every candidate LowIR positive, plus the `std-eq`
probe (`obj/parallel/peekable-eq/packet/native.json`).

Bounded work (`obj/parallel/peekable-eq/scaling-work.txt`), with calls
8/32/128:

| Calls | Inference | Canonical | Trait selections |
| --- | --- | --- | --- |
| 8 | 5100 | 16 | 9 |
| 32 | 20004 | 16 | 33 |
| 128 | 79620 | 16 | 129 |

That is about 621 inference steps and one trait selection per call. Adding 128
unrelated `Marker` impls changes no counter. Adding 128 unrelated `Pick` owners
with their own bodies adds 512 inference and 2024 canonical steps for those
bodies, and no trait selection.

Text qualification (`obj/parallel/peekable-eq/text-review/passed.json`) uses
exploratory candidate 487e0881, built from this checkpoint's exact tree:

- Every positive crosses the independent Rust/CPPGM object bridge, and both
  course rejections are independently rejected by Rust.
- All 1604 prior PA20 and 699 prior PA24 cases reproduce, as do PA1–PA19,
  PA21–PA23 and the released PA25 const-machine unit. All 28,361 prior
  reference hashes are unchanged.
- Normal scoped generation then adds six reviewed triples, in batches
  `pa20/reference-batches/1794480179657437113-29b843b5a7a9.json` and
  `pa24/reference-batches/1794480180299680140-136a186feab9.json`.

PA20 now has 1609 cases (1150 accept), and PA24 has 700 (502 accept). Upstream
verification, the 468-root maintainer suite registration, layout, metadata and
harness unit checks pass. No planned assignment is activated.

## Clean pin and core progress

Clean published source ec763f142 freezes CLI
`487e08812453bc5bf88735c38d5225cf344f95cc9fd05783b5ac28b559c4aead`, with 3910
Rust hashes and an empty patch (`obj/parallel/peekable-eq/candidate-dev-fast`).
It is byte-identical to the text-qualification candidate.

**Original core.** The unchanged 120s observer ends at its deadline after
121.137s, with 28 partial records and 13,270 attempts. The last item is 6977,
fmt/builders.rs:341. Against clean 05289eb6 (49 records, 14,192 attempts, last
item 8659) it clears `next_if_eq` 5023 and `is_sorted` 5811. No new failure
appears among previously passing reached bodies. The first remaining failure is
`first_size` 5171 at step_by.rs:232, as in the exploratory observation below.
How far a partial run reaches depends on machine load. A first clean attempt
under a load average near 80 stalled and is kept only as
`original-clean-loaded-1/NOT-EVIDENCE.md`.

The postcheck verifies all 722 source hashes, 375 opens, six configuration
bindings and unchanged immutable bindings. The working tree already carried the
next checkpoint's edits, so it checks the 3910 compiler hashes against the
blobs of the producer's recorded head (`post-original-clean-head.py`). The
progress SHA is
`289509ffe82c564df05e30f8575addbc211653cd6f6181c207cf64b065110cb2`.

**Measurement.** Fifty exclusive A/A plus ABBA accepting observations against
05289eb6 preserve complete outputs. The controls are an ordinary import, the
method-priority graph, source-core reference equality over plain parameters
beside the `FnPtr` blanket, a projected receiver lookup and the inferred-path
projected header. Paired wall ratios are 1.022/0.959/0.983/1.098/1.012,
against A/A 0.989/1.046/0.883/1.204/0.819, at 18–55 ms. No optimized,
neutrality or original-core speed is claimed; the bounded-work test above
qualifies the work.

**Exploratory observation.** Pin `obj/parallel/peekable-eq/wip1-dev-fast`
(34ce5ba3) carries the relation before the test and fixture work. The unchanged
120s observer ends at its deadline after 121.012s, with 51 partial records and
14,682 attempts. The last item is 9521, escape.rs:137. Against clean 05289eb6
(49 records, 14,192 attempts, last item 8659) it clears `next_if_eq` 5023 and
`is_sorted` 5811. No new failure appears among previously passing reached
bodies.

The first remaining failure is `first_size` 5171 at step_by.rs:232: "quantified
body requirement needs a matching assumption or implementation". `other_size`
follows, then `fold` 157:7745 (unresolved call inputs).

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
