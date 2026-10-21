# Quantified callable publication checkpoint

PA24 and later assignments remain planned. This publishes compiler snapshot
`1857ac39` with the test-only owner expectation repair `113dc7c1`; it does not
include the later projected-signature, reference-Self or equality integrations.
The code adds complete public-quantifier selection for captured callable objects,
generic/nested reference receiver adapters and proof-environment repairs. It also
includes independently qualified initializer ownership, shared-reference Copy,
non-Copy raw transfer and U32 atomic extrema changes.

## Functional qualification

The frozen compiler passed 2,254 release maintainer tests, 211 explicitly enabled
native maintainer test functions, formatting, strict all-target/all-feature Clippy,
architecture/file/harness/layout/upstream/backend checks and all PA1–PA23 text
suites. Independent PA9–PA23 source qualification passed 6,595 cases. Active
native suites passed 4,284 source checks, including 2,366 executable comparisons
and 314 explicitly classified code-generation-only controls.

The initial full Cargo run retained one stale assertion about an inline
initializer's source owner. The implementation intentionally gives that body its
own declaration owner. Only that test expectation changed; its complete four-test
target then passed. The 2,250 already-passing tests were not rerun for a test-only
repair. The original failure and repair are both retained, rather than reporting
all raw gate rows as successful. Native and independent suites ran afterward.

Evidence is in `rustgm-validation-work/obj/parallel/quantified-integration-v1/`.
The default release binary SHA-256 is
`8091103388ad9be595cbe2fe7a40e2f0cb64cd66009bbe7f722abb789319148e`;
the all-feature qualification binary is
`06ea2989fc2563aab97558412cbffa73de3dd86ea9ce31f9b85e4caba12b4e2a`.
This document is a later documentation-only addition to the qualified snapshot.

## Immutable performance comparison

The default candidate was compared with public `be2e9cbc`, binary SHA-256
`a1ddeb165396a057719cfce797408b18a7f10ff944a05207e55a634da05bc739`.
All 27 frozen inputs were independently object/native-qualified before timing.
Four shared families and five newly supported families use widths 8, 32 and 128.
Shared inputs retain byte-identical output. New inputs are candidate-only
observations because the baseline rejects them; no speedup is claimed for them.

An exclusive repository resource lease covered A/A calibration, five paired
ABBA blocks with three executions per observation, RSS observations and
Callgrind. All observations and hashes are retained in
`obj/parallel/quantified-performance/measurement-v1/`. Shared instruction changes
range from +0.0095% to +0.2343%; median paired wall changes range from -5.55% to
+2.87%, with A/A spread of 1.70–9.25%. These noisy wall observations do not
establish a general speedup. Shared median RSS deltas range from -322 to +526 KiB.
The lease excludes registered RustGM jobs, not unrelated host workloads.

The candidate-only quantified-repeated, quantified-distinct, projected-repeated
and atomic-extrema instruction counts grow approximately with input width over
this range. Deep nested reference receivers are a known exception: widths
8/32/128 cost 5.71/24.98/330.58 million instructions and median wall times
8.52/13.84/57.72 ms. This exposes superlinear work outside the separately measured
linear adapter-plan counters. Retained profiles are under investigation; no
linear full-pipeline claim or assignment-completion claim is made for this case.
