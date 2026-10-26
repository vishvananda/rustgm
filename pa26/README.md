# RustGM Programming Assignment 26: Language and library hooks

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA25. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.

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
boundary is now accepted/native. Ordering's own comparison traits are extended
by the later checkpoint below; the other hook families remain required. See
`docs/pa26-ord-helpers.md`.
This checkpoint does not activate PA26.


## Maintainer Ordering implementation checkpoint

The exact canonical comparison Ordering type now supplies PartialEq, Eq,
PartialOrd and Ord evidence through the ordinary selected method factory.
Validated nominal reads become signed discriminant comparisons; primitive
operand contracts remain unchanged. Aliases preserve identity, atomic Ordering
and local lookalikes receive no such implementation. Generic Ord helpers reuse
the checked provided originals.

Its real derived PartialEq/Eq structural property is published only with the
validated completed nominal schema. Named constant patterns use the ordinary
constant/pattern worklist. `tests/ordering-impl-outcomes.json` declares 30
source controls and 20 selected native programs, including all nine variant
pairs. Reference/array comparisons remain required Rust-valid implementation
boundaries. See `docs/pa26-ordering-implementations.md`. This checkpoint does not
activate PA26 or complete its remaining hook families.

## Maintainer reference comparison checkpoint

Canonical reference PartialEq supports all four shared/mutable Self/Rhs pairs;
PartialOrd supports matching outer mutability. Eq and Ord retain exact pointee
requirements. Every reference layer preserves ordinary source-impl competition
and the selected child proof. Checked method factories project through references
and form shared pointee reborrows, preserving overrides without moving an
intermediate mutable reference. Borrowed Ord helpers use their checked provided
originals and ordinary ownership/drop proof.

The original generic reference-operator gap and earlier equality/Ordering
reference-boundary fixtures are positive source/native controls. Sixty-one draft
outcomes and 24 selected native programs are declared in
`tests/reference-comparison-outcomes.json`; implementation and qualification are
recorded in `docs/pa26-reference-comparisons.md`. Concrete str comparisons, elided
reference-impl lifetime arguments and dot-call RHS autodereference inference
remain explicit Rust-positive requirements. Supported opposing forms do not
replace those original failing programs. Structural tuple/array/slice comparisons
and the remaining hook families are still required.
This checkpoint does not activate PA26.

The slice-length checkpoint is qualified separately in
`docs/pa26-slice-length.md`. It uses a canonical slice impl owner and an ordinary
checked const method body; arrays retain receiver adjustment rather than gaining
an array inherent method. Its 49 source outcomes and 30 native programs pass;
the three earlier source gaps remain required work. Explicit array-reference
to slice `as` conversion remains a separately recorded PA22 requirement.

The later impl-owned omitted-lifetime seal and argument-local callable coercion
checkpoint close the retained reference-impl lifetime and FunctionItem
trait-input controls through ordinary declaration and caller proof. See
`docs/pa19-impl-lifetime-sealing.md` and `docs/pa19-call-argument-context.md`.
Dot-call RHS inference, concrete str and structural comparisons remain required;
this does not complete their original Rust-positive sources. The explicit
array-reference→slice cast follow-up is recorded separately in
`docs/pa22-reference-unsizing-casts.md`.


## Maintainer sequence equality checkpoint

Canonical array/slice PartialEq and Eq now use exact container/length identities
and selected element conditions. Heterogeneous elements and the pinned slice
reference combinations retain their actual contracts. Checked equality loops
call element ne; sequence ne uses its checked provided original. Mutable
references are reborrowed without moving intermediate reference values.

The draft `tests/sequence-equality-outcomes.json` records 43 implemented source
outcomes and two unchanged required later-contract positives. The 32 accepted
implemented programs execute independently and through CPPGM. Sequence ordering,
tuple/str comparisons and other required hooks remain unfinished. See
`docs/pa26-sequence-equality.md` for lazy contract publication, size/scaling
controls and exact text review. This does not activate PA26.

## Maintainer integer MAX checkpoint

The twelve integer primitives now expose canonical typed `MAX` constants through
ordinary qualified paths, aliases, constant queries and scalar patterns. A lazy
value-use owner keeps them separate from callable factories and source
initializers; `Self::MAX` preserves inherent-versus-trait precedence.
`tests/primitive-constant-outcomes.json` declares source and native controls;
`docs/pa26-integer-max.md` records the implementation, layout and evidence.
Float/char MAX, other primitive constants and legacy numeric modules remain
required work. PA26 remains planned.

## Maintainer user-defined iteration checkpoint

Canonical `Iterator` and `IntoIterator` now retain real associated-type and
method obligations. `for` uses separately selected `into_iter` and `next` calls,
canonical `Option`, ordinary receiver loans and existing scope cleanup.
The canonical identity and `&mut I` blankets use checked generic originals,
including unsized mutable-reference forwarding. Draft controls are in
`tests/iteration-outcomes.json`; unsafe destructor observers belong to PA27.
See `docs/pa26-iteration-checkpoint.md` for the boundary and qualification.
Canonical iteration dot methods use the existing lexical/prelude lookup and
ordinary receiver proof; `docs/pa26-iteration-method-visibility.md` records
explicit imports, no-prelude controls and source/canonical ambiguity.
Safe shared slice/array iteration and canonical `core::slice::Iter` now use
checked generated originals, exact Item/source lifetimes and ordinary cleanup.
`tests/shared-iteration-outcomes.json` declares the shared source/native controls;
see `docs/pa26-shared-iteration.md` for demand, proof and layout boundaries.
The shared checkpoint's Clone marker control is separate from executable clone.
Canonical `Iter::clone` now has an exact selected implementation and an ordinarily
checked generic original, including non-Clone elements and independent cursor
state. `tests/iterator-clone-outcomes.json` and `docs/pa26-iterator-clone.md`
record actual execution, lookup and lifetime controls.
Canonical mutable slice/array iteration and `IterMut` now use checked originals
with independent source/item and next-receiver lifetimes. The nonempty next
kernel restores the disjoint remainder before returning its mutable item;
empty/ZST and non-Copy elements require no element Copy/Clone promise.
`tests/mutable-iteration-outcomes.json` and `docs/pa26-mutable-iteration.md`
record source/native, identity, split-loan and cold-demand controls. Concrete
derived/reference Clone targets, owned collection and range iteration, further
iterator methods, question-mark, additional derives and formatting remain
required; PA26 stays planned.

## Maintainer half-open integer range checkpoint

Canonical `core::ops::Range<T>` now has ordinary public endpoint fields and
checked half-open integer iteration for all twelve integer primitives.
Qualified, reified, generic and ordinary loop calls retain their complete
selected proofs; record-only construction leaves integer method bodies cold.
Range is not Copy. Conditional Clone marker evidence is not executable cloning.
The source/native matrix and five retained Rust-positive Range gaps are recorded
in `docs/pa26-integer-ranges.md`; owned arrays remain required separately.

These canonical declarations and factories are bootstrap library contracts.
The next source-declared binding work must allow pinned upstream core to supply
the actual traits, impls and bodies. PA26 still owns its compiler hooks and
remaining iteration/derive/question-mark/formatting behavior. No assignment
release or source-sysroot completeness is claimed by this checkpoint.

## Selected source Location derives

The [source assertion milestone](../docs/source-assert-implementation.md#source-location-derive-consumer)
records real source Copy/Clone expansion for Location, with actual field/WF,
const-policy, ownership and native `31 7` evidence. This bounded module-struct
profile preserves source language/header identities and once-parsed opaque
field types; it does not provide fieldwise/generic/enum derives, marker overlap,
Debug/formatting, caller tracking, PanicInfo or executable Assert. Assignment
status stays planned.
