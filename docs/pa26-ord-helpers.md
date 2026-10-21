# PA26 owned Ord helpers

PA26 remains planned. This checkpoint supplies canonical Ord `min`, `max` and
`clamp` methods through checked provided originals. Ordering's own comparison
trait implementations remain a separate follow-up.

## Contract and execution

Exact canonical member IDs distinguish the helpers from `cmp`. Min/max use
`fn(Self, Self) -> Self`; clamp uses `fn(Self, Self, Self) -> Self`. They have no
public borrow binder and each requires method-level `Self: Sized`. Ord itself
retains its unsized Self contract and existing Eq/PartialOrd prerequisites.
Qualified, reified, generic and ordinary dot calls use normal selection.

Provided bodies have the actual canonical method declaration as their owner.
Min/max borrow their owned inputs and call selected `cmp`: min keeps the first
value on equality, max the second. Clamp calls selected `le(min, max)` before
selected `lt(self, min)` and `gt(self, max)`. User overrides of these comparisons
and of the helpers keep their ordinary dispatch. Mathematical ordering laws
remain implementation promises, not compiler assumptions.

Returning branches move one value and leave ordinary cleanup to drop the others.
The existing factory ownership recorder now records actual CFG blocks and each
return's transfer before its own cleanup exit; one-block constructors retain
their behavior. Both original contract collectors, promotion analysis, region,
ownership, loan and destruction checking remain required. Primitive Ord evidence
still proves the selected trait, but a helper uses its checked provided template
instead of obtaining a primitive factory token. Existing scalar factories and
canonical result contracts are unchanged.

A false clamp precondition uses the general typed `Terminator::Panic`, lowered
through the existing shared abort block. Unreachable remains distinct. This is
the current abort contract: panic payloads and PA31 unwinding are not supplied,
and failure does not run unwinding cleanup. Constant execution reports a reached
panic as a fault; helper calls gain no const eligibility.

## Qualification

`pa26/tests/ord-helper-outcomes.json` declares 28 independently qualified outcomes:
21 native programs (including three checked failures) and seven rejections.
Controls cover ties, min/max/clamp cleanup, contained references and rejected
escape, owned operand order/moves, generic selection, qualified/reified calls,
helper overrides and comparison overrides. Failure controls cover inverted bounds,
an overridden false le, and a manual Ord wrapper whose NaN comparison is unordered.
Floating primitive Ord, wrong helper results and const calls reject.

The unchanged earlier `ordering/ord-helpers.rs` boundary is superseded as accepted
and native. Four provided-original invariants and seven existing factory
invariants pass, including both contract collectors, method-only Sized, zero
public helper slots, the promotion gate and malformed-original rejection.
Both helper and ordering source/native test groups pass. CPPGM-dependent native
tests remain explicitly ignored by default and were exercised with
`--include-ignored`.

Strict all-target/all-feature release Clippy, architecture audit (1,463 files;
zero findings), layout and pinned upstream checks pass. Measured sizes before
and after Panic are identical: Terminator 16, Block 28, Body 464, Rvalue 20 and
Value 24 bytes. All 7,787 committed PA1–23 text comparisons pass exactly.
The three final object-backend failure probes terminate through libc abort
(SIGABRT); a valid clamp/drop probe also passes. Evidence is retained under
`obj/parallel/pa26/ord-helpers/`, including `ninth`, `active-text`, `object-final`
and the before/after size receipts. References are not regenerated.

All failed/preparation observations remain available. The first moved-value
hypothesis used a wildcard that performs no read; the corrected negative reads
the moved field. Initial clamp failure bodies reached Unreachable and produced
SIGILL in object probes; the final typed Panic path passes the existing checked
panic harness. Missing owned-value records and promotion-analysis prerequisites
were caught by the original-proof invariant before final qualification.

## Remaining scope

The subsequent `docs/pa26-ordering-implementations.md` checkpoint supplies
Ordering's own comparison traits. Its convenience methods and structural/reference
comparison implementations remain outstanding, alongside primitive reference,
unary and Deref hooks, iteration, question-mark, required derives and formatting.
Unsuffixed assignment-method RHS inference remains an explicit gap. Free cmp
functions and broader library surface are not supplied by these trait methods.
PA30 remains queued separately. Integrated release and throughput qualification
remain integrator-owned; neither assignment is declared complete.
