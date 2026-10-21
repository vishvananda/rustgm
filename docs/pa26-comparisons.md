# PA26 canonical equality checkpoint

PA26 remains planned. This checkpoint supplies canonical `PartialEq`/`Eq`
contracts, selected equality operators and ordinary method materialization. It
adds no course text references or assignment activation.

## Typed contract

The neutral comparison family keeps the trait identity distinct from its member
identity. Demand-loaded `core::cmp`/`std::cmp` metadata registers actual trait,
Self/Rhs parameter and `eq`/`ne` declaration IDs. `PartialEq<Rhs: ?Sized = Self>`
uses two independent late borrow slots:
`for<'a, 'b> fn(&'a Self, &'b Rhs) -> bool`. `Eq` retains the canonical
`Self: PartialEq<Self>` prerequisite and has no ordinary methods in this profile.
No operator `Output` projection or prior language-hook identity changes.

Ordinary lookup handles prelude membership, imports, aliases, shadowing and
ambiguity. Bare qualified and reified `u8::eq`/`u8::ne` requests load the same
metadata; a same-named source function retains its own identity. The method
visibility index is built once. Prelude insertion reuses its sorted candidate
list and accounts for lookup/insertion work.

Custom/generic `==` and `!=` retain selected ordinary method-call facts. Their
operands are evaluated and shared-borrowed once, LHS before RHS, without moving
non-Copy values. Physical borrow types remain distinct from aliased callable
parameter types. Existing receiver facts retain the LHS conversion; a sparse
comparison-only fact retains the RHS borrow. Both use ordinary coercion and
`Constrain` checks. A private invariant verifies each conversion enters its owning
lifetime contract exactly once. Closure capture collection consumes the selected
member identity and uses shared capture permission for both operands.

Provided `ne` has one typed original CFG under the actual canonical method ID:
read the two reference parameters, call canonical `eq`, negate its Boolean result
and return. Its signature/parameter environment and contract come from the
registered declaration. The template and contract are appended before ordinary
original-body region, ownership, loan and destruction checks. Its diagnostic
origin is never a source-body owner or fabricated AST. Specialization then uses
the existing checked-template/demand pipeline. User `ne` overrides dispatch to
their actual source method. No primitive factory permission exempts this generic
original from checking.

Primitive method selection extends the existing private descriptor with an
explicit comparison operation/form. Successful canonical selection still owns
the exact complete instance token. Factory validation checks the exact eq/ne
member, safe Rust signature, both shared borrows, independent slots, operand
domains and Boolean result before the ordinary CFG proof route. `Eq` is
selection evidence only, excludes floating-point primitives, and cannot produce
a method factory. Existing arithmetic/assignment `Output` validation is retained.

One unresolved numeric operand can be equated to an exact known primitive only
when its Integer/Float domain permits that primitive. This happens inside the
isolated candidate; ordinary competing candidates still determine ambiguity.
Rigid nominal, Any-domain and both-unknown inputs gain no such equation. Controls
exercise a legal primitive Self/local nominal RHS alongside builtin scalar
candidates, plus mismatched suffixed types and unresolved competing candidates.
Existing scalar syntax retains its direct arithmetic/comparison path.

## Qualification

`pa26/tests/comparison-outcomes.json` declares 52 outcomes: 32 implemented
positives, 18 language rejections and two independently accepted implementation
boundaries. All 31 native controls pass with Rust 1.98.0 and the pinned external
CPPGM LowIR backend. The other implemented positive is the no-prelude source
control. Coverage includes NaN, signed/unsigned values, user `ne` overrides,
an eq body that traps if the override is bypassed, unsized/projected RHS,
independent input lifetimes, default-ne lifetime transport, operand order/loans,
prelude/shadowing, reified calls, Eq prerequisites and rejected floating Eq.
Manual equality does not grant structural constant-pattern permission.

Six primitive descriptor/factory invariants and two provided-original/coercion
invariants pass. They include wrong form/member/marker/borrow-slot admission,
nonzero real selections, ordinary factory/template gates, malformed default-body
rejection, and incremental/batch default-contract collection. The full 23-group
language_operators suite (including native controls), strict all-target/all-feature
release Clippy, architecture audit (1,457 files; zero findings), layout and pinned
upstream checks passed. The final loader/lazy-registry follow-up also passes its
52 comparison outcomes, 31 natives, strict Clippy and static gates.

Evidence is retained under `obj/parallel/pa26/comparison/`. Hypotheses v1/v3/v4/v5
bind the final fixture bytes; v2 retains the corrected hypothesis that a shadowed
prelude name would suppress a competing method. Rust instead reports ambiguity.
The independently accepted static-counter control is retained only as evidence;
the committed PA26 fixture uses a trapping eq method to avoid requiring PA27.
The sixth/seventh batches record the complete regression/final gates. Early
compile and wiring failures remain described in development-history.json; its
note identifies the early reused log filenames. No reference outputs were blessed.

## Remaining scope

Reference, tuple, array, string and function-pointer builtin trait impls, derive
execution and Eq's deprecated hidden assertion member remain follow-up library
surface. The two accepted boundary fixtures keep reference forwarding and derive
execution explicit. Mathematical equality laws are promises, not compiler proofs;
the deliberately nonreflexive manual Eq control remains accepted.

Primitive AddAssign dot lookup also still rejects an unsuffixed RHS such as
`let mut n=3u32; n.add_assign(4)`, although Rust 1.98 accepts it. Both matched
performance pins reproduce this gap; an explicitly typed `4u32` succeeds. Its
future correction should reuse isolated numeric-domain candidate inference and
retain ordinary competing-implementation ambiguity. The original performance
input and failed receipt are preserved alongside the separately frozen control.

PA26 still needs the remaining primitive/unary/reference and Deref hooks,
PartialOrd/Ord, iteration, question-mark, required derives and formatting/runtime
hooks. PA30's accepted scalar/foreign-static and bounded repr(C) aggregate ABI
checkpoints remain separate; this change expands no ABI, metadata or linkage
surface. Combined release, text comparisons and publication belong to integration.
