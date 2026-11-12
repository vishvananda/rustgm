# Helper return callable outputs

Pinned core `FlattenCompat::iter_try_fold` (function 4720, flatten.rs:431) fails
on `self.iter.try_fold(acc, flatten(&mut self.frontiter, &mut fold))?` with
"associated type projection requires normalization". Reductions isolate the
cause. When the helper's own generic `R` occurs only in its opaque callable
output (`impl FnMut(B, T) -> R`), inference fails even when no input is projected.
A concrete output or explicit helper arguments pass in the baseline.

Two gaps combine:

- `callables/output.rs::expose` obtains `<X as FnOnce<A>>::Output` only from
  `match_signature`. That needs a closure or fn signature, so a declared opaque
  gets Mismatch.
- The input refinement from the helper-input checkpoint binds the helper's `T`,
  but the trait goal stays deferred on the output-only capture.

## Change

- **Selection.** Suppose the unique declared callable clause refines the opaque's
  capture inputs while its subject is still unresolved. Then the frame publishes
  those equations as partial progress (`Action::Yield` with `Pending::Declared`)
  and restarts its selection when resumed. The restart happens only in the input
  scope, with inference enabled and outside normalization. The restarted frame
  still proves the complete trait, formation and lifetime requirements.
- **Output.** When the signature match reports Mismatch for a declared opaque,
  `expose` calls `results::infer_output`. It instantiates the unique published
  Output equality clause over the opaque's inferred captures and relates its
  callable inputs through the existing Probe. It then returns the declared result
  term. This is forward normalization through the declaration, never inversion
  of the output. The existing `preserves_inputs` gate still guards the export.
- **Bodies.** A statement or assignment whose type is a pending associated
  result, such as a `?` output, is not yet definitely Never. The never-check
  therefore uses `contextual_head`, as divergence checking already does.

`infer_inputs` and `infer_output` share one clause instantiation and one
probe-first relation, so an incompatible declaration leaves the caller's graph
unchanged. The change reads no hidden representation and caches no unresolved
caller term. It adds no new engine, dense-model field, backend, storage policy
or compiler limit.

## Contracts

Twenty-seven independently declared Rust 1.98 outcomes are 20 accept/7 reject:

- PA24 ordinary: 15 course cases, 11 accept/4 reject, plus one executable
  contract that stays required.
- PA31 actual source: 11 cases, 8 accept/3 reject. These include the original
  `acc = self.iter.try_fold(acc, flatten(..))?` consumer through the source Try
  protocol, as a statement, as split and turbofish forms, and with explicit
  arguments.

Twenty-six are qualified as complete API outcomes by
`helper_return_outputs_follow_declared_results_after_inputs_settle`. Baseline
a08ab1a4 rejects every inferred form and accepts the explicit twins.
Wrong-output, wrong-input, unconstrained-output and unsatisfied-output-bound
reject in both compilers.

The required executable (`pa24/tests/spec/flatten-helper-outputs/pending.json`)
calls `try_fold` through a borrowed receiver. It stops at the pre-existing
"opaque storage requires a checked defining type" gate, and so do its explicit
and concrete-output twins and the baseline. The gap is therefore not caused by
this change, but executing core's `&mut self` try_fold will need it closed.
It stays a serialized positive and is not reclassified as a rejection.

## Qualification

Focused receipt 1794473199674806426 passes in 91.5s:

- full formatting, the architecture audit, and strict Clippy for four suites and
  the CLI;
- suite_closures (8 tests, including the new outcome test, the helper-input test
  and the call-linear work test), suite_inference (9), suite_declarations (1) and
  suite_frontend (1);
- all 699 PA24 text cases.

Natively, `native-value` and its explicit twin are compiled by rustc and by
RustGM→LowIR→CPPGM. Each object is linked with the same C observer. All four
programs print `flatten-output 7 9 1234`.

Text qualification precedes normal scoped generation:

- Every new positive crosses the independent Rust/CPPGM object bridge.
- The nine declared empty objects define no code or data in either compiler.
- The four rejections are independently rejected by Rust.
- All 684 prior PA24 references and the 23 other active stages reproduce byte
  for byte. All 28,287 prior reference hashes are unchanged.

Generation adds 15 reviewed triples and the batch
`pa24/reference-batches/1794473168865300305-9ff75ed0ff9d.json`. PA24 now has 699
cases (501 accept/198 reject) and 100 object controls. No planned assignment is
activated. Upstream verification, the maintainer harness and layout checks pass.

## Clean pin and core progress

Clean published source 0af99f4a6 freezes CLI
`0665938a6c2cab3a0cead8b7f1f128ab161c42fa32df8a349374b790ddacec23`, with 3907
Rust hashes and an empty patch (`obj/parallel/try-helper-outputs/candidate-dev-fast`).
It is byte-identical to the earlier exploratory wip2 binary built from the same
compiler source.

**Measurement.** Forty exclusive A/A plus ABBA accepting observations against
a08ab1a4 preserve complete outputs. The controls are an ordinary import, the
method-priority LowIR graph, and the explicit entry-free and exported helper-output
libraries. Paired wall ratios are 1.061/1.080/1.033/0.996, against A/A
1.117/0.902/1.002/1.168, at 18–73 ms per run. That is within process noise, so
no optimized, neutrality or original-core speed is claimed. The actual-source
packet is API-qualified and outside the CLI source profile, so it is not a
measurement control.

**Original core.** The unchanged 120s observer ends at its deadline after
121.207s, with 55 partial records and 14,206 attempts. The last item is 8679,
str/traits.rs:374. Against a08ab1a4 it clears iter_try_fold 4720, iter_try_rfold
4725, try_fold 4731/4790 and try_rfold 4743/4797. No new failure appears among
previously passing reached bodies.

The postcheck verifies all 722 source hashes, 375 opens, six configuration
bindings, 3907 current and immutable compiler hashes, and unchanged profile and
limits. The progress SHA is
`ea6dd657c7f5bc51cd24594d75e094a3cd69402a0d8e328f14e910ef594a2bc6`.

The first remaining failure is try_fold 4961 (file 147:1768, map_while.rs:62):
"invalid method candidate inference graph". Of the 55 records:

| Count | Cause |
| --- | --- |
| 10 | scoped const continuations |
| 7 | repeat-element Copy |
| 7 | projection normalization |
| 6 | method candidate graphs |
| 7 | unimplemented diagnostic formatting |

The 14,206 attempts are fewer than a08's 14,659, under different host load,
while the previously failing bodies now check completely. Partial counts make no
throughput claim, and this is neither a complete census nor a core artifact.

Full CI 43188445172 on 0af99f4a6 passed fast checks and every release
integration suite. One private test failed, and the later course and native
steps did not run. `provisional_results_preserve_known_never_cfg_and_explicit_feature_frontier`
pinned the old normalization fence for `fn delayed()->u8{Delayed<<1;}`. There,
`1` selects between `Shl<i32>` (Output `!`) and `Shl<usize>`, so the statement's
type is a pending associated result.

With this checkpoint's contextual never-check, the block reports "incompatible
types" at `{`. Rust 1.98 rejects it with E0308 ("expected `u8`, found `()`"),
exactly like the sibling `binding` case that the same test already expected, and
the packet's independent observations record that. Only that one expectation is
updated, with the Rust agreement in a comment. Its `branch` case is unchanged.
The CI log's actual RustGM output, "incompatible types" at byte 4742, is exactly
`rfind("{Delayed<<1")`. Two focused private runs, 1794477130209495369 and
1794477429020339780, passed formatting and architecture but exhausted the fixed
five-minute budget in the cold private-lint build before running the test. The
full successor runs it; no budget was raised.
The full successor is recorded with the next checkpoint.
