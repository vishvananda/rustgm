# PA26 canonical ordering checkpoint

PA26 remains planned. This checkpoint adds canonical `PartialOrd`/`Ord`
contracts, selected relational operators and ordinary method materialization.
It publishes no text references and does not activate the assignment.

## Typed contracts

`PartialOrd<Rhs: ?Sized = Self>` retains `Self: PartialEq<Rhs>` and two independent
borrow slots: `for<'a, 'b> fn(&'a Self, &'b Rhs) -> Option<Ordering>`.
`Ord` retains both `Self: Eq` and `Self: PartialOrd<Self>`; its required `cmp`
method returns `Ordering` with the same independent borrows. Exact trait/member
IDs distinguish `partial_cmp`, `lt`, `le`, `gt`, `ge` and `cmp`. Operators select
their actual relation method, including overrides, through the existing ordinary
call, operand-borrow and coercion facts. Shared operand evaluation remains LHS
before RHS. Canonical Output projections for arithmetic methods are unchanged.

Comparison `core::cmp::Ordering`/`std::cmp::Ordering` has its own canonical
nominal identity and module. Its signed i8 discriminants are Less=-1, Equal=0
and Greater=1. Atomic Ordering retains its prior identity and representation;
matching spelling grants neither conversion nor selection permission. One cached
canonical cmp module serves both trait and nominal declarations. Lookup still
checks imports, aliases, shadowing, prelude availability and source ambiguity.
Bare qualified and reified methods demand the same registered metadata.

The four provided relation bodies are typed original CFGs under their actual
canonical method IDs. They call canonical `partial_cmp`, branch over its exact
Option/Ordering variants and return explicit Boolean values. Their registered
signatures, parameter environments and callable contracts enter ordinary original
region, ownership, loan and destruction checking before specialization. Incremental
and batch contract collection cover them. User overrides retain their source
bodies; no special default-body permission bypasses those checks.

Primitive selection extends the existing private comparison descriptor and
validated complete-instance token. The checked factory reads both shared
operands and builds ordinary comparisons, branches and canonical aggregates.
`partial_cmp` returns None for unordered floating inputs; signed, unsigned,
Boolean and character ordering retain their scalar semantics. `Ord` evidence
excludes floats. Result-shape preparation demands the actual selected nominal
fields before ordinary proof, including when no source constructor mentions them.

## Qualification

`pa26/tests/ordering-outcomes.json` declares 42 independently qualified outcomes:
28 implemented positives, 13 language rejections and one Rust-valid Ord-helper
boundary. All 28 implemented programs execute successfully through the pinned
CPPGM object backend and host linker, alongside independent Rust 1.98.0/native
qualification. The reusable ignored native test covers 27; the crate-level
no-prelude control is additionally executed by the retained standalone matrix.

Controls cover generic, qualified, bare and reified calls, unsized/heterogeneous
RHS, independent input lifetimes, NaN and infinities, all relation overrides,
prerequisite failures, float Ord rejection, wrong result/receiver contracts,
atomic/comparison Ordering identity, aliases/shadowing, no-prelude lookup,
competing user traits, capture sharing and operand loans. The borrow-conflict
control rejects at completed LowIR proof; its types-only inspection may succeed.

Three provided-body invariants and seven primitive-factory invariants pass.
Six nonzero ordering selectors exercise exact member/result/borrow-slot
validation and the ordinary proof route. The complete `language_operators`
source/native suite passes all 25 groups, including the explicitly ignored CPPGM
tests. Strict all-target/all-feature release Clippy, architecture audit (1,461
files; zero findings), layout and pinned upstream checks pass.

Evidence is retained under `obj/parallel/pa26/ordering/`: the independently
declared hypotheses and qualifications, fifth-batch full LowIR/native matrix,
private invariants and seventh-batch final regression/static gates. Earlier
failed observations remain intact. An early experiment changing generic
IsVariant lowering was reverted; provided bodies instead use ordinary branch
tests and Boolean return blocks. Two earlier enum/Option match controls retain
exact LowIR equality against the frozen equality compiler. Generic aggregate
lowering is unchanged. Cumulative text comparison remains an integration gate;
no references were regenerated here.

## Remaining scope

Ord `max`/`min`/`clamp`, Ordering's own trait implementations and convenience
methods, and structural/reference/tuple/array/string/function-pointer comparison
implementations remain outstanding. Equality derive execution and its accepted
reference-forwarding boundary are also unchanged.

Other PA26 requirements still include primitive reference/unary and Deref hooks,
iteration, question-mark, required derives and formatting/runtime hooks.
Unsuffixed primitive assignment-method RHS inference remains an explicit gap
(`n.add_assign(4)` versus the supported `4u32`). PA30's broader ABI work remains
queued separately. This bounded checkpoint does not declare either assignment
complete.
