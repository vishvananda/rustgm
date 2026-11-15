# Trait-member calls inside const closures

Clean original core40 stops at `slice/cmp.rs:184`, in the const closure calling
`PartialOrd::partial_cmp` inside `SlicePartialOrd::partial_compare`. Its exact
source Call and closure owner were already retained, but conditional defining
permission recognized only function owners.

The shared authenticated closure contract now serves generic invocation and
trait-member defining checks. It validates actual semantic source, const wrapper,
completed declaration, public/execution signatures and selected closure identity.
A conditional member request keeps its own source-use owner and completed typed
Call. Existing full member signatures, arguments, selected method/UFCS facts,
const-trait identity, borrows, concrete dispatch and executable Code remain
required. The enclosing function supplies no replacement source owner. Const
trait-default parents remain fenced: their exact local Self premise requires a
separate scoped continuation. No authority table, dense field, quota, wire,
backend, ABI or storage change is introduced.

All six previously retained unannotated const closure inputs now pass prepared
lowering, with their accepting predictions and source bytes unchanged. The stale
owner-gap classifications were removed. This closes their prepared-lowering
boundary; separate runtime/CTFE and generic closure execution requirements retain
their own status.

[PA25 minima](../pa25/tests/spec/closure-members/manifest.json) and
[PA31 source roles](../pa31/tests/spec/closure-members/manifest.json) each predict
12 outcomes:9 accept/3 reject. Rust1.98 independently agrees with all12. Ten source
outcomes qualify:7 accept/3 reject, covering UFCS, method syntax, concrete and
generic inputs, runtime execution, scaling, missing const bounds, ordinary
closures and wrong arguments. Source CTFE opaque-result typing and actual
const-default Self premises remain required accepting boundaries. All12 ordinary
attempts remain unqualified at Destruct import preparation. The custom Compare
trait reduces the original PartialOrd call shape; no exact original item copy or
complete upstream comparison implementation is claimed.

Evidence under `obj/parallel/closure-members/`:

- `declaration.json`, `setup-correction.json` and `baseline2/` preserve predictions
  and immutable40 diagnostics. The initial Rust-only parse used `impl const`;
  it was corrected to Rust1.98's `const impl` before any candidate execution,
  with every expected outcome unchanged.
- fast2=1794712084032770967 passes six exact selections in53.15s, including the
  six completed older closure cases, new semantics/scaling/required gaps, captured
  calls and unsafe members. Full format, architecture and selected production/CLI
  strict Clippy pass. fast1's sole failure was the stale six-case gap classifier;
  all six actually lowered successfully, as recorded in its output.
- native1794712252305956183 passes three groups/four actual programs in41.09s:
  the new selected Compare implementation and captured-call control print41/59;
  two ordinary unannotated closure programs also pass through the frozen backend.
  Independent ordinary Rust runtime and CTFE consumers both print41/59. RustGM
  CTFE remains open.
- `replay/` authenticates3967 compiler sources, all packet hashes and24 complete
  CLI attempts:10 qualify,14 retain explicit limits. Prepared lowering and scaling
  replay pass. At8/32/128 member closures, inference/canonical work is6952/1425,
  27736/4325,110872/15901. Immutable CLI:
  5bbc391f93e44f44d66ab288e85d29d63b8f3e05e40c1dd99192843fa9a08994.
  No speed improvement is claimed.
- 315 harness tests,497 roots/13 integration binaries,44 stages/one component,
  128 pinned upstream fixtures and126 Reference files pass.

The existing private closure corruption guard now covers both generic Invoke and
actual trait-member Call bodies. This extension awaits full CI; no cold local
private harness was run. Full43669171589 on the previous73eec244c checkpoint has
passed strict lint/architecture/harness and release/course tests and is running
native qualification. It must finish before a successor is launched.

Clean optimized41 and an unchanged original-core retry are next. Full original
metadata/LowIR/native consumers, all other named accepting requirements, whole
PA25–44 and the separate113-case PA25 component retain their previous status.
