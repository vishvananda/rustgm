# Canonical reference comparison checkpoint

This PA26 checkpoint supplies the canonical reference PartialEq, Eq, PartialOrd
and Ord implementations through complete selected instances and ordinary checked
method bodies. PA26 remains planned; this is not an assignment release.

PartialEq permits all four shared/mutable outer reference combinations when the
exact pointee PartialEq condition holds. PartialOrd permits matching outer
mutability only. Eq and Ord require the corresponding pointee property. These
are implementation contracts, not conclusions inferred from reference coercion.
Pointees may be unsized; no extra Sized condition is imposed on generic users.
The original `fn compare<T:PartialOrd>(a:&T,b:&T)->bool{a<b}` gap is a positive
source/native control, including the analogous prior equality and Ordering
boundary fixtures without changing their source bytes.

A compact Operands enum distinguishes scalar operation types from reference
mutability. Operation::References retains the comparison family and canonical
member identity. The selected full Instance carries actual Self/Rhs types and
lifetimes. Reference candidates descend through the existing iterative trait
search one layer at a time and preserve the selected child proof. An intermediate
source override remains a candidate; no chain is flattened to its final pointee.
The existing lazy pending-search slot serves both callable and comparison
conditions, rather than adding storage to every search frame.

Source reference implementations compete with the canonical blanket. The
header pass asks whether the blanket's exact existential referent condition can
hold, using the existing declaration-only environment. Only completed absence
without a cycle/cutoff proves disjointness. Pending or unresolved conditions are
possible overlap. This query exports no evidence or assumptions, and a fresh
query-local cycle counter prevents unrelated earlier searches from affecting it.
Source order, generic conditional bounds and opposing Sized conditions have
independent controls.

The private factory independently validates the canonical member, complete
Self/Rhs instance, reference permissions, two separate method-borrow slots and
exact pointee prerequisite. Complete operand lifetimes are abstracted together;
only the blanket's declared condition supplies its selected lifetime witnesses.
The body projects through the outer shared method borrow and stored reference,
then makes a shared pointee reborrow. It never reads/moves an intermediate &mut
value. It calls the same canonical member on the pointees, preserving ne and
relation overrides, then enters the ordinary dispatch, region, ownership, loan
and drop pipeline. Owned Ord helpers retain their checked provided originals;
pointee overrides of owned min/max/clamp are not incorrectly forwarded.

The draft `tests/reference-comparison-outcomes.json` declares 61 source cases,
with 24 selected native programs. Forty-five are independently Rust-positive;
42 are frontend positives and three remain explicit required implementation
gaps. Sixteen language negatives cover mutability-family mismatches, missing
pointee properties, float Eq/Ord, unpaid lifetimes, invalid reification, orphan
competition, active loans and const eligibility. Nested references, custom and
heterogeneous pointees, aliases, reified methods, NaN behavior, tie identity and
mutable-reference reuse are covered. All source hypotheses are independently
qualified before frontend comparison.

Three Rust-positive boundaries remain required before PA26 release: concrete
str comparison implementations; elided early lifetime arguments for a user
reference impl with a reference trait argument; and dot-call RHS autodereference
inference. The latter two original fixtures and digests remain unchanged, with
frontend failure recorded rather than reported as passing. Both failures also
reproduce at identical locations on immutable equality checkpoint 45870765
(binary SHA-256 4a12a4fa2a797a8338eb01c9fb642c115e33a3c539c48ba774b06db895b12bc8).
A direct `str:PartialEq` control is independently Rust-positive and fails at the
same location on both binaries too, distinguishing the missing concrete impl
from the new reference blanket. Supported owned-argument lookalike and generic bound dot forms exercise the
opposing path; they do not replace these completion requirements. Failed
alternate hypotheses and all preparation observations remain under
`obj/parallel/pa26/reference-comparisons/`.

Fifteen factory invariants pass, including all earlier primitive/Ordering
controls and rejection of forged operand shapes, primitive tokens for nominal
reference instances, missing referent conditions and erased/injected lifetime
witnesses. Repeated calls with 8/32/128 unrelated declarations retain exactly two
factories, five demanded definitions and two selections; instructions are
47/119/407. Descriptor storage decreases from 6 to 5 bytes; Operation, Selected
and selection Source remain 2/12/16 bytes. Search Frame/Pending/Node remain
1392/80/160 bytes, matching the retained baseline receipt.

Remaining PA26 work includes the three boundaries above, structural tuple/array/
slice comparisons, ordinary array/slice len, primitive reference/unary and Deref
hooks, unsuffixed assignment-method RHS inference, iteration, question-mark,
required derives and formatting. PA30 remains a separate lane. No references
are regenerated or assignments activated by this checkpoint.

All 61 declared source outcomes, 24 independent Rust/CPPGM native programs,
existing equality/ordering/helper/Ordering source and native groups, four
provided-original invariants, the query-local cycle control and existing callable
forwarding scaling invariant pass. Strict all-target/all-feature release Clippy,
architecture audit (1,471 files, zero findings), layout and pinned upstream checks
pass. All 7,787 committed PA1–23 text comparisons agree exactly. The 868-file
production source manifest SHA-256 is
`a0a998147d72930c66bf3da2329ce12582c1e6d7e97cbd0a01ffdc51c708b2b3`.
The retained qualified-checkpoint receipt records exact commands and observations;
full Cargo publication and exclusive measurement remain integrator work.

The combined checkout includes the earlier closure-signature placeholder cache
from `7c9f1b17`, which the worker's isolated base did not contain. Its one lazy
pointer adds eight bytes to each inference Context. Frame owns two such contexts
(its inputs and its optional trial), so the integrated Frame is 1,408 bytes;
Pending and Node remain 80 and 160 bytes. The integrated layout assertion uses
this independently identified base change. Reference forwarding itself adds no
Frame storage. The first combined check retained the failed 1,392-byte assertion
as evidence rather than claiming the worker's old-base size for this checkout.


The impl-lifetime-sealing follow-up supersedes the omitted-lifetime boundary
above without changing its fixture bytes: all 61 classifications now include
43 frontend positives, two required gaps and 16 language negatives; 25 native
programs pass. Concrete str and dot-call RHS inference remain required. See
[the sealing checkpoint](pa19-impl-lifetime-sealing.md) for its independent slots,
ordinary proof path and separately retained FunctionItem inference boundary.
Slice len was subsequently completed by its own checkpoint.
