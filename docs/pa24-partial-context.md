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

## Immutable development measurement

The frozen compiler for `6b684da5` has SHA-256
`509020188d72abde37fc091603a0cb9f1f97ffa5cf17b1837f168dd80947ee1f`.
Its preceding `e7437974` baseline has SHA-256
`55d69e47ad10e27d4a502e24f759b87aa3f98869ab377cc7304ed1ccbb29eb0e`.
Both use the same release profile. All nine 8/32/128-function inputs were
independently object-qualified before the exclusive CPU-2 measurement. The six
shared ordinary/fully-known closure inputs retained identical LowIR, stderr and
status. Four A/A observations precede five ABBA blocks, with three executions
per observation; 234 observations retain all 702 executions, plus eight RSS
samples per input and Callgrind counts. The three partial-context inputs reject
on the baseline and are candidate-only measurements.

Shared instruction changes range from -0.149% to +0.087%. Median paired wall
changes range from -0.27% to +2.44%; the known-context-128 input has +2.44% across
five positive blocks, despite essentially unchanged instruction count. A/A
spread across shared inputs ranges from 1.15% to 7.90%, so this short study does
not establish a tight global latency bound. Shared candidate RSS medians are
24–354 KiB lower. No claim of a speedup is made.

New partial-context inputs take 4.17/7.66/21.84 ms, 6.67/24.81/97.34 million
instructions and 8,818/9,634/11,176 KiB median RSS at widths 8/32/128. Work is
consistent with linear scaling over this range. Exact plans, source/compiler
hashes, every observation and instruction profiles remain under
`obj/parallel/partial-context-performance/measurement-v1/`; resource receipt
`cdaee657c6dd4dfe85d189a4eef9d2c7` returned zero. This is a development checkpoint,
not the still-pending PA24 assignment release.
