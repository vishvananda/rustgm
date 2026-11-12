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

## Core progress (exploratory)

Pin 0665938a was built from this checkpoint's exact compiler source, but from a
dirty tree, so it is labelled exploratory. The unchanged 120s observer ends at
its deadline after 121.214s, with 19 partial records and 12,827 attempts. The
last item is 6109, net/socket_addr.rs:673.

It clears iter_try_fold 4720, iter_try_rfold 4725, try_fold 4731/4790 and
try_rfold 4743/4797. No new failure appears among previously passing reached
bodies.

The first remaining failure is try_fold 4961 (file 147:1713, map_while.rs:62):
"invalid method candidate inference graph". Of the 19 records, six are candidate
graphs, seven are unimplemented diagnostic formatting for trait paths or source
shapes, and two are quantified body requirements.

Host load varied, so partial counts make no throughput claim. This is neither a
complete census nor a core artifact.

A clean pin, A/A plus ABBA measurement against a08ab1a4, an unchanged
clean-pin observation with its postcheck, and a full CI successor are pending.
