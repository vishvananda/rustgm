# PA26 canonical language trait admission

Canonical library operator/assignment trait implementations now satisfy the
ordinary orphan rule before their headers enter the implementation registry.
The checker visits Self followed by the trait's type arguments. At least one
must be local, and no uncovered type parameter may precede the first local
type. A primitive Self remains permitted with a local RHS. References preserve
their referent's locality and do not cover parameters; tuple, array and ordinary
nominal wrappers cover their parameters without making nested local types local.

The scoped ownership query uses exact canonical library identities and resolved
nominal declaration IDs. Local aliases retain their target identity; canonical
Option and its aliases remain foreign. Principal local trait objects use the
actual local trait inventory. Normalized associated projections retain their
resolved nominal identity. No spelling, source span or diagnostic provenance
grants locality. The current PA29 scalar profile imports no nominal or trait
declarations; future metadata for those kinds must extend this ownership seam
with explicit foreign identity before exposing imported declarations.

This corrects an independently confirmed existing gap: five rustc-invalid orphan
or uncovered-parameter declarations were accepted before this patch, including
primitive-pair impls and `Option<Local>` Self. The correction is independent of
the proposed primitive method evidence and factory, so builtin selection cannot
mask an invalid user implementation or silently override it.

Nineteen declared outcomes in `pa26/tests/orphan-outcomes.json` agree with Rust
1.98.0 object emission: ten accepted declarations and nine language rejections.
Eight accepted programs execute through the pinned external LowIR backend.
Controls include primitive Self/local RHS, local references and aliases, local
trait lookalikes, covered generic parameters inside tuples/arrays/Option,
empty local enums, local/foreign principal trait objects and a concrete
associated projection resolving to a local type. Existing overlap and trait
conformance checks remain separate required checks.

All nineteen focused operator/call/static/syntax groups pass, including the
existing 24 assignment and twenty IndexMut native programs. Strict all-target,
all-feature Clippy passes; the architecture audit reports 1,269 files and zero
findings; layout passes 38 stages, and pinned upstream verification passes
24 fixtures plus 126 Reference files. Evidence is under
`obj/parallel/pa26/primitive-assignment/`: the original matrix qualification,
`orphan-extra-qualification.json`, `orphan-final.json` and
`orphan-extra-final.json`. The pre-fix frontend probe explicitly recorded all
five invalid declarations as accepted. Initial probe formatting and typed API
wiring failures are retained in their diagnostic logs and resource receipts.

This is a scoped admission checkpoint, not a PA26 release or a claim that every
external trait family is implemented. Generic primitive method implementations,
comparison/Deref, iteration, question-mark, derives, formatting/runtime hooks
and remaining library method lookup still require their own implementation and
qualification. No references or assignment activation changed.
