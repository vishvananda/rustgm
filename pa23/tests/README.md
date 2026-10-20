# PA23 fixtures

The manifest owns 399 course-authored LowIR fixtures: 262 accept and 137 reject.
All outcomes were declared and checked with pinned rustc before generating text
oracles. Accepted inputs include 85 native execution controls and 177 explicit
compile-only controls. The latter check acceptance and unused bodies without
claiming to execute those bodies.

| Family | Cases | Responsibility |
| --- | ---: | --- |
| annotations | 59 | Dyn compatibility, associated bindings, quantified principals and default binding time |
| conversions | 33 | Concrete implementations, equalities and object coercion proof |
| members | 34 | Intrinsic object evidence, method selection and scoped requirements |
| markers | 147 | Structural and explicit builtin markers, object marker sets and proof cycles |
| hidden-results | 25 | Defining-body readiness, hidden lifetimes, independent captures and retained proofs |
| dispatch | 44 | Virtual/default/inherited calls, ABI, conditional method slots and reification |
| lifetimes | 20 | Container defaults, object extents and erased referent obligations |
| upcasts | 19 | Supertrait conversions, raw metadata and inherited associated equalities |
| storage | 18 | Nested dynamic tails, alignment, constants and promotion |

`maintainer_origin` identifies the independently qualified source table. The
prior PA19 future-object boundary is preserved byte-for-byte in
`dispatch/prerequisite-boundary/`; it now declares acceptance. Inputs with
runtime assertions use the existing `check`/`fail` helper. No fixture was
imported from a new third-party source.

Source tests cannot directly inspect rustc's unspecified vtable layout.
Maintainer typed-IR tests separately inspect the course destructor header and
invoke it on valid storage, comparing the observable destructor result with
ordinary Rust destruction. Keep that evidence distinct from source acceptance
and the text oracle.

Every source belongs to `manifest.json`; `.ref`, `.ref.stderr` and
`.ref.exit_status` are generated together by the explicit reference command.
Signals, exit 86, timeouts and panics in the compiler are never valid rejections.
Unregistered files and personal tests do not contribute to the default totals.
See the handout and `docs/pa23-release.md` for the contract and release checks.
