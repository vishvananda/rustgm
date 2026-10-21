# Partially inferred closure expectations

PA24 remains planned. Function-pointer expectations now preserve lifetime
binders when their input or output types contain inference holes. The expected
signature and closure inputs cross the same isolated refinement mapping, so
shared holes retain identity. One binder fold opens all expected children
together, preserving nested callable binders and enclosing lifetime aliases.

An unresolved function pointer still has a binder even when it introduces zero
late lifetimes. Calls now remove that empty frame before using references to an
enclosing lifetime. Previously the shortcut left a depth-one lifetime on an
ordinary call operand and produced an invalid lifetime-identity diagnostic.
Inferred call substitution now folds one bundle of inputs and output, sharing
work across common subgraphs instead of walking each child independently.

Eleven new controls were independently qualified with pinned rustc 1.98.0 object
compilation before the final implementation: seven valid input/output holes,
enclosing lifetimes, generic arguments, nested pointers and mutable references;
four invalid crossed outputs, local escapes, static escapes and inconsistent
input types. The crossed-output control constrains both input types explicitly,
so its rustc rejection establishes a lifetime failure rather than ambiguous
type inference. Original predictions and the corrected control remain in
`obj/parallel/partial-closure-context/oracle-v1/` and `oracle-v2/`.

The complete contextual matrix contains 27 source outcomes, including 18 native
positive cases. Together with existing uncontextual lifetime controls, closure
pointers, safety, source bodies and callable conventions, all 20 focused test
groups pass. Strict all-target/all-feature release Clippy and the syntax-aware
audit (1,207 files, zero findings) pass. Evidence is in
`obj/parallel/partial-closure-context/gates-v3/`; earlier failed gates and the
typed-contract diagnostic probe remain available beside it.

The foreign-convention native harness now places its outputs under the
integrator's registered `obj/parallel/` budget and scoped cleanup roots.
