# Captured generic const calls

Original core candidate39 stops at `ops/try_trait.rs:415`, in
`NeverShortCircuit::wrap_mut_1`: a returned const move closure invokes the captured
`F: [const] FnMut(A) -> T`. Defining invocation validation previously accepted
only function owners.

Closure invocations now authenticate their retained semantic source, actual const
wrapper and declaration, public/execution signatures, and selected closure
identity. The existing closure argument API supplies complete enclosing formals
and inherited lifetimes, including declaration order before an opaque parent's
callable template exists. Source effect, exact callable receiver and borrow,
opened signature, argument/result identity, place checking and actual executable
const Code remain separate requirements. There is no parent-function owner
substitution, new persistent authority table, dense record, quota, wire, backend
or storage change.

[PA25 ordinary minima](../pa25/tests/spec/captured-const-calls/manifest.json) and
[PA31 source roles](../pa31/tests/spec/captured-const-calls/manifest.json) each
predict25 outcomes:20 accept/5 reject. Rust1.98 independently agrees with all25.
Fourteen source outcomes qualify:10 accept/4 reject, covering shared/mutable,
borrowed, returned opaque, inherited parent formals, two arguments, real runtime
execution and scaling. Missing const effect, immutable captures and wrong
arguments/arity are rejected for their actual causes. Eleven source attempts
remain unqualified: local closure cleanup (including one negative masked by that
fence), consuming closure cleanup, and opaque-result CTFE typing. All25 ordinary
attempts remain unqualified at Destruct import preparation. Every independent
positive remains an accepting requirement. The original18 inputs and predictions
were preserved when seven returned-adapter controls were added before execution.
The packet preserves the original adapter shape; no byte-exact original item or
complete upstream implementation is claimed.

Full43665225882 found a separate malformed-IR regression: a non-function Call
signature returned no permission before reaching the strict source-signature
check. `source_call` now rejects that invalid kind explicitly. The existing
private corruption assertion is unchanged. That full passed3086 private tests
and all public release targets, with one private failure; subsequent course,
native and debug stages did not run. The new closure corruption guard and this
repair await successor full CI (expected3088 private); no cold local private
harness was run.

Evidence under `obj/parallel/captured-const-calls/`:

- `declaration.json`, `baseline/`, `returned-declaration.json` and
  `returned-reference/` preserve predictions, immutable39 baseline and all25
  independent Rust results. Both ordinary Rust runtime and CTFE observers print
  `41 59`; the RustGM CTFE consumer remains open.
- fast4=1794710365842385778 passes four exact semantic/scaling/required-gap/unsafe
  controls in40.72s, with full format, architecture and selected production/CLI
  strict Clippy. fast3 additionally passes borrowed calls and the six older
  unannotated const closure accepting-gap controls; its returned-once test exposed
  the separately retained consuming-cleanup requirement. fast1 rejected invalid
  command syntax before validation; fast2 exposed the local-cleanup boundary.
- native1794710495049524427 passes four groups in38.00s, including the new actual
  captured callable observer `41 59`, borrowed/explicit call controls and both
  unsafe member runtime/CTFE consumers, through the frozen backend.
- `replay/` authenticates3967 compiler sources and all packet hashes. All50 CLI
  attempts have exact recorded outcomes:14 qualified,36 explicitly unfinished.
  Exact prepared lowering and scaling replay pass. At8/32/128 returned adapters,
  inference/canonical work is3382/2160,13486/7436,53902/28516. Its immutable CLI is
  ad8b948907471b23e0f5616996c1b7f4aa9bf5932db15ae306a2914a5061faeb.
  No timing or speed improvement is claimed.
- 315 harness tests,497 roots/13 integration binaries,44 stages/one component,
  128 pinned upstream fixtures and126 Reference files pass.

A clean optimized40 compiler and unchanged genuine original-core retry are next.
No original-core metadata/LowIR/native artifact or whole PA25–44 release is
claimed. The separate113-case PA25 component and every older accepting requirement
retain their prior status.
