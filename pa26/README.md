# RustGM Programming Assignment 26: Language and library hooks

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA25. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Operators, indexing, iteration, question-mark, built-in derives and formatting.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

desugaring; trait selection; early return; derive bounds; formatting.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.

The builtin boundary includes `option_env!`: its expansion needs the library's
nominal `Option<&'static str>` identity and Some/None constructors. PA8 supplies
the eager argument worklist and explicit environment provider, but must not
fabricate core/std paths while those library identities are unavailable.
Formatting, assertion and panic-related macro hooks similarly need explicit
library/runtime contracts and tests at their owning stage.

## Maintainer operator checkpoint

The first implementation checkpoint admits the `Neg`, `Not`, arithmetic,
bitwise, shift and shared `Index` library contracts. Demand-loaded metadata
registers distinct trait, parameter, method and `Output` declaration identities
for `core::ops` and `std::ops`. Ordinary namespace lookup selects aliases and
shadowed names; a user trait with the same spelling grants no operator behavior.

Operator operands retain their actual types. Selection proves the complete
trait application, normalizes its associated `Output`, and checks implementation
methods through the existing trait conformance machinery. Lowering consumes the
selected ordinary method callable. Owned operators consume their operands in
source order. Shared indexing borrows its receiver before evaluating the index,
then dereferences the method's returned reference as a place; the ordinary
ownership and region checks enforce its loan and mutability.

This checkpoint is not an assignment release. Mutable indexing is extended by
the later checkpoint below. Assignment and comparison hooks, iteration,
question-mark, additional derives, formatting and
the remaining library method boundary remain implementation work. The
existing scalar operations, built-in array/slice indexing and earlier derives
retain their earlier owning contracts.

## Maintainer nominal checkpoint

Demand-loaded `Option`, `Result` and `ManuallyDrop` metadata supplies canonical
nominal and constructor identities through ordinary type and value namespaces.
Aliases retain those identities; local lookalikes retain ordinary declarations.
`Option` and `Result` have ordinary enum fields, constructors and pattern checks.
This supplies the nominal foundation for `option_env!` and question-mark.
`option_env!` now retains an explicit library-value tag and optional decoded
string child through expansion; semantics fixes `Option<&'static str>` and
selects its real variant for ordinary aggregate lowering. Environment snapshots
remain explicit provider inputs. Question-mark control flow remains separate
implementation work.

`ManuallyDrop<T>` preserves T's transparent layout and formation requirements.
Its private field cannot be constructed or destructured by callers. Its explicit
aggregate policy suppresses automatic field destruction and the corresponding
drop lifetime duties; enclosing ordinary aggregates still destroy their ordinary
fields. Copy and Clone remain conditional on T's capabilities. This checkpoint
does not expose manual extraction, dereference or explicit destruction methods.

Body lifetime and ownership contracts pair by declaration or anonymous body
identity, independently of shared diagnostic provenance. Metadata variants and
constructors never receive invented source nodes. See
`docs/pa26-nominal-checkpoint.md` for the typed boundary and integration notes.

The option_env checkpoint and explicit remaining PA26 responsibilities are
recorded in `docs/pa26-environment-checkpoint.md`. PA26 remains planned while
later independently accepted work proceeds; partial checkpoints do not complete
this assignment's full hook families.

The maintainer constructor correction retains genuine variant construction
separately from const-function factories such as `ManuallyDrop::new`. Constant
evaluation remains eligible; runtime calls cannot acquire static promotion
merely from aggregate storage. Draft controls are in
`tests/constructor-promotion-outcomes.json`, with qualification recorded in
`docs/pa26-constructor-promotion.md`. This does not activate PA26.

## Maintainer mutable indexing checkpoint

`IndexMut<Idx>` inherits the single canonical `Index<Idx>::Output` and supplies
an ordinary mutable receiver/result contract. Stores, primitive compound
assignment and mutable borrows select it through typed place demand; shared
reads retain Index. Nested projections, unsized outputs, generic bounds and
mutable-reference dereference chains preserve normal loans and lifetimes.
Raw-pointer dereferences can retain shared indexing without weakening pointer
permissions. Runtime evaluation order is independently compared with rustc.

Mutable method receivers refine indexed places through the same typed facts.
Square-index receiver borrows are immediate, while ordinary method calls retain
two-phase receiver reservation.

Forty-four draft outcomes and twenty native programs are recorded in
`tests/index-mut-outcomes.json` and `docs/pa26-index-mut-checkpoint.md`.
Capturing closures use PA24's separate execution contract. Direct library-trait
dot-method lookup is extended by the later checkpoint below. Custom compound assignment through
IndexMut is extended by the assignment checkpoint below.
This checkpoint does not activate PA26.

## Maintainer assignment-trait checkpoint

All ten arithmetic, bitwise and shift assignment traits use canonical mutable
receiver contracts with unit output. Selected ordinary calls preserve trait
assignment's LHS-first evaluation and two-phase receiver reservation, including
generic custom implementations. Primitive compound syntax retains RHS-first
evaluation and scalar inference. Thirty-five independently qualified outcomes
and 25 native programs are recorded in `tests/assignment-outcomes.json` and
`docs/pa26-assignment-checkpoint.md` after the primitive-method checkpoint below.
This checkpoint does not activate PA26.

Canonical library operator/assignment impl headers now enforce the orphan rule
using exact local and foreign identities before publication. Nineteen qualified
declaration controls and eight native programs are recorded in
`tests/orphan-outcomes.json` and `docs/pa26-orphan-checkpoint.md`.

## Maintainer primitive-assignment checkpoint

Canonical trait selection can supply typed primitive method evidence for all ten
assignment families, including shared-reference RHS forwarding and generic or
reified calls. A private validated selection token demands an ordinary checked
body through the existing region, ownership, loan and arithmetic pipeline.
`tests/primitive-assignment-outcomes.json` declares the independent source
controls; `docs/pa26-primitive-assignment.md` records the implementation and
remaining boundaries. Direct library-trait method lookup is extended below.
Implicit hook availability remains unfinished. PA26 remains planned.

## Maintainer canonical dot-method checkpoint

Imported canonical operator and assignment traits participate in ordinary dot
lookup through their exact registered identities. Aliases, anonymous imports,
glob imports and parameter bounds retain their normal scope and priority rules;
unimported canonical traits do not supply lexical candidates. Receiver matching,
ambiguity, call inference and selected implementation proof use the existing
method pipeline, including primitive assignment method factories.

`tests/dot-method-outcomes.json` declares 26 independently qualified controls
with 17 accepted native programs. They include shared/mutable indexing,
evaluation order, two-phase reservation, moved operands and opposing user-trait
candidates. The visibility index is built once and cached by scope and name;
widths 8, 32 and 128 retain linear work. See `docs/pa26-dot-methods.md` for the
typed contract and qualifications. This does not activate PA26 or supply the
remaining comparison/Deref, iteration, question-mark, derive or formatting hooks.

## Maintainer primitive Add checkpoint

Owned primitive `Add` methods now use the existing canonical selection and checked
factory for generic, qualified, dot and reified calls. Typed descriptors distinguish
value-returning binary methods from mutable assignment methods; canonical Output
normalization retains ordinary selection and formation evidence. Overflow checks
remain the existing arithmetic contract. Twenty-four declared controls and thirteen
native programs are recorded in `tests/primitive-add-outcomes.json` and
`docs/pa26-primitive-add.md`. Shared-reference RHS and other primitive value methods
remain explicit boundaries. This checkpoint does not activate PA26.

The later owned binary-family checkpoint extends that same descriptor and checked
factory to subtraction, multiplication, division, remainder, bitwise operations
and shifts. Forty-four independently qualified controls and 31 native programs
cover mixed shift RHS types, Boolean/float domains, same-named user traits and
checked traps. See `tests/primitive-binary-outcomes.json` and
`docs/pa26-primitive-binary.md`. Primitive reference/unary forms and the other
required hook families remain separate work; PA26 stays planned.

## Maintainer equality checkpoint

Canonical `PartialEq<Rhs: ?Sized = Self>` and `Eq` declarations now participate in
ordinary trait selection, operators and qualified/dot/reified calls. Equality
borrows both operands independently in source order. Default `ne` enters the
ordinary original-template proof pipeline; overrides retain their source bodies.
Primitive method evidence uses the existing validated factory and checked CFG
route, with Boolean result and distinct borrow slots. Float PartialEq preserves
NaN behavior; floats do not acquire Eq.

`tests/comparison-outcomes.json` declares 52 independently qualified outcomes,
including 31 native controls. Reference forwarding and derive execution remain
explicit accepted boundaries; the bounded ordering checkpoint below extends
PartialOrd/Ord. See `docs/pa26-comparisons.md`. This does not activate PA26.

The equality performance preparation also retains an independently accepted
unsuffixed assignment-method gap: `let mut n=3u32; n.add_assign(4)` is rejected
while `4u32` succeeds. Numeric-domain candidate inference for assignment methods
remains required work; see `docs/pa26-comparison-performance.md`.

## Maintainer ordering checkpoint

Canonical `PartialOrd<Rhs: ?Sized = Self>` and `Ord` retain their equality
prerequisites, exact member identities and independent shared borrow slots.
Relational operators select their actual override. Default relations enter the
ordinary original-template proof pipeline; primitive methods build checked
comparisons and canonical Option/Ordering values, including unordered NaN.
Comparison Ordering remains distinct from atomic Ordering.

`tests/ordering-outcomes.json` declares 42 independently qualified outcomes with
28 implemented native programs and an explicit accepted Ord-helper boundary.
See `docs/pa26-ordering.md`. Ord helpers, structural/reference implementations,
derive execution and the remaining hook families are still required; PA26 stays
planned.

## Maintainer owned Ord-helper checkpoint

Ord `min`, `max` and `clamp` now use checked provided originals with owned inputs,
method-level Self:Sized and ordinary return/drop/loan contracts. Min retains the
first tie, max the second; clamp preserves selected comparison overrides and
uses the existing abort runtime for invalid bounds. Helpers gain no const-call
permission or primitive factory exemption.

`tests/ord-helper-outcomes.json` declares 28 qualified controls with 21 native
programs, including three checked failures. The earlier unchanged Ord-helper
boundary is now accepted/native. Ordering's own comparison traits and the other
remaining families are still required; see `docs/pa26-ord-helpers.md`.
This checkpoint does not activate PA26.
