# PA26 assignment-trait checkpoint

The ten arithmetic, bitwise and shift assignment traits have canonical
demand-loaded identities appended after the existing language hooks. Each
contract takes `&'a mut Self` and an owned `Rhs` (defaulting to Self), returns
unit, and retains one callable region binder. Neutral `Output::Unit` metadata
has no associated declaration or owner; optional identity accessors keep it
distinct from declared and inherited Output projections. Ordinary conformance
rejects an invented Output member, shared receiver or non-unit result.

Compound syntax selects the trait path when an operand is nominal, a generic
parameter or a projection. Already primitive operands and unresolved numeric
domains retain scalar inference. In particular, a primitive Self with a local
nominal RHS still selects its user implementation, including when Self starts
as an unsuffixed integer. An undemanded hook does not trigger operand traversal
or metadata allocation. Ordinary caller goals retain generic dispatch before
specialization; generic operations cannot silently switch evaluation order
when instantiated.

Selected method facts carry the mutable receiver conversion and normal call
signature. Body lowering evaluates the LHS place, reserves its two-phase
receiver borrow, evaluates the RHS, then emits an ordinary call. Existing
primitive compound assignment retains RHS-before-LHS evaluation. Native
side-effect controls independently distinguish these orders and verify the
generic custom-type path. IndexMut supplies a mutable place through its
existing immediate index borrow; ordinary loan, mutability, alignment and
ownership checks continue to apply. No new Body IR operation or source
reconstruction is used.

The 35 declared cases in `pa26/tests/assignment-outcomes.json` were independently
qualified with Rust 1.98.0, edition 2024, object emission: 24 accepted cases,
ten language rejections and one explicitly valid Rust boundary. All 24 accepted
programs execute with matching status through the pinned external LowIR backend.
They cover every assignment family, default/custom RHS, aliases, generic calls,
mixed primitive/local operands, fields, mutable indexing, reference/raw-pointer
places, receiver reservation, unit results and source evaluation order. Negative
controls cover missing or lookalike traits, incompatible contracts, immutable
places, move reuse and overlapping mutable loans. Ten metadata checks verify
unit output and the one mutable receiver binder.

The later primitive-method checkpoint implements the generic primitive assignment
case through canonical selection evidence and a checked ordinary body factory;
see `docs/pa26-primitive-assignment.md`. Its unchanged former boundary fixture
now accepts and executes, bringing this suite to 25 accepted programs and ten
language rejections. The former custom compound-assignment IndexMut boundary
also accepts its unchanged fixture bytes. No references were regenerated.

Strict all-target/all-feature Clippy and all seventeen focused test groups pass
across language operators, call operators, static permissions and syntax
transfer. This includes the existing twenty IndexMut native programs and the
24 new assignment native programs. Architecture audit: 1,268 files, zero
findings; layout: 38 stages; upstream: 24 pinned fixtures and 126 Reference files.
Evidence is under `obj/parallel/pa26/assignment/`: declared-source hashes and
object outcomes in `qualification.json`, focused/native results in
`focused.json`, final results in `final.json`, and resource receipt
`4e870e4e4f554c0d879a5d521102efce`. The initial test-module path wiring failure
is retained as `initial-check-failure.json` and its diagnostic log. No language
or native failure was hidden by the correction.

This remains a partial PA26 checkpoint. Other primitive operator implementations,
comparison/Deref hooks, iteration, question-mark residual/control flow,
additional derives, formatting/runtime contracts and remaining library methods
still require implementation. Direct library-trait dot-call lookup remains an
explicit gap. Cumulative scaling and release qualification are required before
activation; no references or assignment activation changed.
