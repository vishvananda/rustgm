# Checked slice iterator cloning

This adds actual `core::slice::Iter::clone` execution, including dot, qualified,
aliased, reified and generic `T: Clone` calls. The element type need not implement
Clone or Copy. The implementation copies the stored shared slice view and gives
each iterator its own cursor; it never reads, clones or moves an element.
PA26 remains planned. No references or assignment activation accompany this
checkpoint.

## Identity, proof and lazy demand

The resolver extends the existing exact canonical Clone identity with one real
method declaration, one Self parameter and, when needed, a distinct registered
Iter implementation. Source definitions keep their identities. Source/local
lookalikes retain normal lexical lookup, no-prelude imports and ambiguity. A
local trait named Clone does not hide the canonical prelude method for Iter:
the original predicted-positive source-shadow probe was independently rejected
by Rust as ambiguous. Its bytes and mistaken prediction remain in the receipts;
v2/v3 explicitly supersede only that predicted outcome.

The trait member has its normal `Self: Clone + Sized` requirements. The registered
implementation owns lifetime slot 0 and a sized element parameter; its shared
receiver uses independent late slot 1. It has no element Clone requirement.
Method conformance, lifetime requirements and the actual implementation binding
complete through the normal header machinery before the original can be used.
The common source-free trait-owner builder is shared with iteration declarations.

Only selecting/reifying the typed canonical member marks executable demand.
Resolver spelling and candidate metadata are not implementation permissions.
Marker-only programs, unrelated Iter formation, and source/inherent clone
winners leave the generated implementation/body cold. A later exact collection
activation completes an already demanded implementation through the same ordinary
header/conformance facade, supporting generic specialization without a second
proof route. Rigid disjoint subjects exclude the latent Iter header; genuinely
unknown matching subjects remain pending.

Preliminary applicability omits only Clone's own exact canonical trait predicate,
while retaining marker Clone and Sized filtering. The final selected call keeps
its complete ordinary goal. A marker-bound generic parameter may use the scoped
marker counterpart for abstract checking, but that evidence carries no executable
implementation or body. Bound-priority dot lookup does not mislabel this marker
as an explicit user-trait assumption. Concrete dispatch still requires an actual
implementation and rejects marker-only concrete subjects.

The factory validates the exact canonical schema, method binding, signature and
independent lifetime slots. Both incremental and batch collectors submit the
ordinary generic original to region, loan, ownership and drop verification.
Specialized callers retain their ordinary type/formation/receiver proofs. There
is no new IR, Owner/BodyKey identity, primitive Permit or const-call permission.

Known FieldsOnly nominal candidates now use the same declaration possibility
filter as constructed candidates, retaining actual Sized checking. This is not
Clone permission: a private opposing nominal with a non-Clone mutable-reference
field is possible during candidate filtering but fails actual Copy/Clone proof.
Source derives and imported/source nominal policies are unchanged.

## Declared outcomes and verification

`pa26/tests/iterator-clone-outcomes.json` contains 31 supported declared outcomes:
22 accepted programs and nine rejections. Twenty-one accepted programs have
independent native comparisons; the abstract generic control is codegen-only.
The matrix exercises cursor independence before/after exhaustion, live borrowed
items, non-Clone/non-Copy elements, Drop observers, empty/ZST slices, two generic
call spellings, no-prelude visibility, source winners, ambiguity, mutation/escape,
non-Copy Iter, const rejection and foreign-convention rejection. Native maintainer
tests are explicitly ignored when the external backend is unavailable.

Two additional Rust-positive programs remain explicit required work: concrete
derived nominal Clone and reference blanket Clone. They reject concrete dispatch
from symbolic marker evidence. They are not language negatives or substitutes
for the supported Iter implementation. The original 23 collection/range positives
also remain required, including mutable/owned collection and range iteration.

The focused check-v3 batch passed all 31 source outcomes, 21 native comparisons,
the two explicit required boundaries, and six private invariant tests. Those
cover both original collectors, forged owner/subject/parameter/predicate/binder
records, incomplete conformance, cold typed-demand behavior and candidate-only
FieldsOnly admission. The final close-v3 guard passed strict all-target/all-feature
Clippy, architecture audit (1739 Rust files, zero findings), affected iterator
original/declaration invariants, shared iteration and visibility suites, all 31
Clone source outcomes and 21 native comparisons, layout, suite inventory and
pinned upstream checks. No full publication suite was run in the worker.

The single final PA1–24 text pass preserves all 8360 outcomes. Its eight changed
artifact triples (seven PA20 views/diagnostics and PA6 for-binding names) are
byte-identical to the retained b75 shared-iteration observations; Clone adds no
text differences. Exact attribution is in
`obj/parallel/pa26/iterator-clone/text-v1/inherited-delta-review.json`.
The frozen default-debug executable is
`obj/parallel/pa26/iterator-clone/text-v1/candidate/rustgm`, SHA-256
`23412183ab099c33c1928347e6054ead6e41adeab90fe273cb6a39d7528eb3a1`.
Its source map and source archive are retained beside it. Failed check-v1/v2 and
close-v1/v2 observations remain preserved; close-v3 is the final coherent gate.

All independent evidence is under worker
`obj/parallel/pa26/iterator-clone/{declared-v1,declared-v2,declared-v3,qualified-v1,qualified-v2,qualified-v3}`.
Failed/superseded probes and debug traces remain alongside check-v1/v2/v3.
The fixtures retain their independently qualified source hashes.

## Layout and bounded work

On this worker's b75 base, Declarations grows 8472→8480 and Resolution 2080→2088:
one optional lazy pointer in each owner. The demanded semantic registry is 84
bytes and the resolver declaration 40 bytes. Type remains 24 bytes, and Store and
inference Context remain 448 bytes each. No ordinary IR/value/call record widens.
Integrated bases may have unrelated additional fields; the incremental cost is 8
bytes per owner, and integration must retain those separately qualified fields.

For 8/32/128 generic clone calls, inference work is 2058/7746/30498 with exactly one
trait selection. For one call and 8/32/128 unrelated nominal/function pairs,
inference work is 431/527/911 with one selection. Canonical activation adds exactly
8 nominal-requirement and 16 variance work units in every case. These are logical
work observations, not wall-time or instruction performance claims. Full immutable
performance comparison and publication remain integrator-owned.
