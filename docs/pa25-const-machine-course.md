# PA25 typed constant machine course corpus

The const-machine component remains **unpublished**, and PA25 remains planned.
Its complete declared candidate corpus is in
[tests/units/const-machine.json](../pa25/tests/units/const-machine.json).
This publishes a reviewable test contract; it does not release the component or
replace the remaining symbolic-memory and advanced-callable requirements.

The component retains its dependency-plan scope: ordinary const eligibility,
bounded typed CFG execution, closed initializer/query readiness and deterministic
inspection. Source bodies, actual generic arguments, unsafe requirements and
cleanup obligations retain their ordinary checks. A const declaration does not
grant execution, and a checked value does not waive its source proof.

| Cluster | Cases | Required behavior |
| --- | ---: | --- |
| Ordinary eligibility | 31 | Direct, recursive, indirect and method calls; mutation; source-context and pointer restrictions |
| Destruction eligibility | 15 | Returned values and transfers versus actual local, argument and partial destruction |
| Ordinary initializers | 9 | Required unused constants/statics, reached faults and genuinely lazy branches/associated values |
| Selected initializers | 17 | Actual complete arguments, forward dependencies, recursion/cycles, faults and body restrictions |
| Native values | 9 | Constants and runtime calls, recursion, loops, statics, methods, generic transfer and typed queries |
| Inspection | 5 | Entry-free roots, deterministic sharing/counts, compressed repeats, unused errors and closure identities |
| Repeat counts | 8 | Forward/associated/parameter counts; wrong types, local capture, nonconst calls and reached/unreachable faults |

Expected outcomes are declared before candidate execution. The eligibility and
destruction sources come from the existing const-functions controls, with their
existing explicit empty-main adapter. Ordinary initializer sources retain their
declared bytes. Selected initializer sources use the existing explicit
main-calls-run adapter. Inspection adopts existing inputs without changing them.
Native value and repeat-count cases declare their asserted values in source;
failure reaches the ordinary division-by-zero runtime path. The independent
compiler must generate original Rust objects for all accepted cases.

Repeat counts use the existing annotation traversal, owning generic/region
context and typed usize query scheduler. The body consumes the resulting
source-keyed length; bare const parameters retain their symbolic array type.
No source is reparsed, and no new global traversal or common type/body field is
introduced. Reached calls still require their real checked bodies and eligibility.
The scaling control uses widths 8, 32 and 128, one shared callee CFG, and linear
execution/storage bounds for the distinct query roots.

Before release, qualify every declared input against pinned Rust 1.98 objects,
compare repeated text and count output, generate references through the course
runner, and run the complete eligible native corpus with the locked backend.
Reconcile the interpreter recursion/work/storage and inspection limit controls,
all retained component requirements, existing release/performance evidence and
the owning handout's release gates. A successful small corpus cannot close an
uncovered requirement or the parent PA.

The actual original-core blocker is tracked separately in
[the library tracker](library-build-blockers.md).

## Current qualification

All 94 inputs independently agree with pinned Rust object-compilation outcomes.
Repeated text is identical, and the five inspection cases preserve their count
contracts. Generated references pass a fresh course comparison and complete
unit provenance/discovery checks. All 89 LowIR cases pass independent checking,
including 52 executable comparisons through the locked CPPGM backend. The full
corpus batch takes 22.68 seconds after resource admission. Its receipt is
39cafff8738fe27562a9b315c3a391791bc3746a6e96e3477a70ceae83acdf54.

The underlying compiler checkpoint passes 24 exact affected controls, full
formatting and architecture checks, and strict production/selected-test Clippy
in 41.61 seconds including admission. The receipt is
0441305b539f441c936363df5a5bdd0fe871cb769d8022afe4938955186679b4.
Both use compiler af969042ae18f644542d5ed3f5433f1f19bd1385dc9c0746564888542b264e13.
The unit reference receipt is
ce00e0b896aaac70261eca86c7eaf4f69a779f5269771ebcddb420f554649389.

Invalid enclosing-local capture retains the existing unavailable-value
diagnostic; its rejection agrees with Rust. This is not a claim of identical
diagnostic wording. The full scheduled/manual qualification now explicitly runs
this candidate contract while its component release review remains pending.
