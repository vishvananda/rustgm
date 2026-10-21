# PA26 primitive value-returning Add

This checkpoint extends the existing primitive assignment factory with owned
primitive `Add` methods. Generic bounds, explicit qualified calls, lexical dot
calls and reified function pointers use ordinary canonical trait selection.
The descriptor explicitly distinguishes assignment and value-returning binary
forms. Independent validation checks the canonical method, complete instance,
substituted signature, receiver, RHS, result, binder count and scalar operation.

The existing selected projection path resolves only the exact canonical
`Add::Output` member with an empty associated argument family. Its parent
selection and formation evidence remain in the normal proof graph. Equal scalar
outputs do not identify unrelated inputs or erase reference lifetime obligations.
The factory returns an ordinary binary Rvalue through the existing region,
ownership, loan and arithmetic checks; checked overflow retains its runtime trap.
Its private permit remains tied to the exact validated selected instance.
Assignment bodies still store through their mutable receiver and return unit,
including their existing independent shared-reference RHS lifetime abstraction.

`pa26/tests/primitive-add-outcomes.json` declares 24 controls: 15 Rust acceptances
and nine rejections. Thirteen accepted programs execute through the external
CPPGM backend. Shared-reference RHS and `Sub` value methods are accepted by Rust
but remain explicit frontend boundaries in this slice. Rejections include wrong
RHS/result, noninjective output inference, projected-reference static escape,
const method calls and unsafe function-pointer calls without an unsafe context.

Evidence is retained under `obj/parallel/pa26/addition/`. The initial 23 inputs
were independently qualified before implementation in `qualification.json`; the
additional projected static escape was declared before its independent oracle.
`third.json` records all 24 source/Rust outcomes, 13 native positives, three
private factory/cache invariants and all 19 operator regression groups passing.
Strict release all-target/all-feature Clippy, architecture audit1380/0, layout38
and pinned upstream24+126 checks also pass. Earlier failed admission-filter and
test-only lint observations remain recorded. A mistaken private-test filter
selected zero tests in `second.json`; only the corrected three executed tests
in `third.json` count as qualification.

At widths 8/32/128, distinct generic callers retain one selected factory and one
dispatch selection. LowIR definitions are 12/36/132 and instructions49/145/529,
with constant increments per additional caller. This is structural scaling
evidence, not a wall-time performance claim.

PA26 remains planned. Additional primitive value operators/reference forwarding,
comparison/Deref, iteration, question-mark, additional derives and formatting
remain implementation work. PA30 aggregate ABI, variadic and complete metadata
and release wiring remain separate work. No activation, oracle regeneration or
public push is included.
