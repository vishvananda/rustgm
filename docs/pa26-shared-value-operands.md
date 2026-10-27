# PA26 owned-left/shared-RHS Add and Sub

The existing primitive value-method descriptor now admits shared-reference RHS
operands for Add and Sub. The original retained inputs remain byte-identical:
`primitive-add/reference-boundary.rs` (SHA-256 `6f3f18c1524bee64c23f068f6f0a76e735bcb7deaf3b8bc8871a964cd62aaecf`)
and `primitive-binary/reference-boundary.rs` (SHA-256 `b15564b6f442133b804911e28d55a083c80e8e91105daa4882c75dca3b467174`).
Their predicates change only after both unchanged sources pass native execution.
The earlier owned-family outcome files retain the superseded boundary fields.

Candidate admission, receiver validation and canonical Output normalization use
one shared supported-pair query. Existing numeric-domain inference can constrain
an immutable RHS referent from a known primitive left operand; it never converts
an unconstrained general or nominal type into a scalar. Other binary operators,
mutable RHS types and shared-left profiles acquire no new primitive permission.

Every selected method still validates its exact canonical owner, operation,
complete instance, scalar widths, output family, signature, ABI and binder count.
A local shape check verifies the immutable reference's scalar storage type.
Canonical scoped substitution verifies its full region identity. Generic regions
are raised beneath the method function binder, including an empty binder; direct
raw type equality across those scopes incorrectly rejected a valid generic
wrapper during the first native run. The retained failed receipt records that
error. The corrected private control accepts the real generic signature and
rejects a forged static signature with unchanged generic arguments.

The existing body factory already models the owned left value, one immutable RHS
dereference and the ordinary checked binary operation. No body, ABI, registry,
wire or backend field is added. Nonstatic RHS factories retain one independent
universal; concrete static instances retain zero. Existing instance-bound permits,
ownership, loans, output and checked-overflow obligations remain mandatory.
Forged Owned/Mul descriptors, mismatched permits, unsafe/binder signatures and
wrong universal counts remain rejected. Direct scalar arithmetic allocates no
primitive-method registry.

Predictions are frozen in `pa26/tests/spec/shared-value-operands/` and actual
SourceCore inputs in `pa31/tests/spec/shared-value-operands/`, with explicit
frozen outcome plans. Thirty-two independent metadata observations retain one original
prediction disagreement: Rust rejects the early-RHS method item to higher-ranked
function-pointer conversion. The original prediction and diagnostic remain;
static concrete reification and a genuine late-bound wrapper are positive.
Sixteen literal source pairs execute through both pinned Rust and the CPPGM
backend, including generic/dot/alias/reified calls, numeric inference, coercion
from mutable borrow to shared borrow, float/wide scalars, lexical ownership,
left-to-right evaluation and two overflow traps. Negative source controls retain
lexical ambiguity, missing import, wrong widths/result, mutable/deep references,
Bool/Char, an active conflicting loan and non-const method calls.

The actual Add/Sub source language-item inputs are required pending capabilities.
Rust accepts the source-owned profile; the candidate currently stops during
resolution at unsupported source language item Add. The source-absent opponent
is independently rejected. Neither gets bootstrap factory permission. Separate
ordinary same-named SourceCore traits supplement this evidence: real owned impl
bodies return deliberately distinct 19/23, the missing impl fails, and the genuine
bodies compile, link and print `19 23` through Rust and candidate object producers.
The primitive registry stays absent. These ordinary traits do not close the real
language-item debt. The original proposed source-native plan remains retained
beside the explicit ordinary followup rather than being silently rewritten.

Shared-left Add/Sub remains Rust-positive and unsupported, with an explicit
candidate diagnostic and no native execution. Genuine source Add/Sub bindings,
float MAX and other primitive/reference families remain required separate exits.
Original core/alloc/std, cumulative release qualification, oracle publication and
assignment activation are not claimed here.

Evidence is under `obj/parallel/pa26/shared-primitive-operands-v1/`. The final
private nine controls and 16 native pairs include strict selected-target Clippy,
formatting and architecture checks. Failed audit/source-role/test-lint/generic
observations and all prediction disagreements remain indexed in the handoff.
The inherited `compound_calls.rs` hunk is rustfmt-only: the integrated base failed
whole-tree format before this checkpoint; its implicit-assignment tests are
otherwise unchanged. The final native-to-source delta changes only the two old
integration predicates; production and native-test bytes remain identical.

At 1/8/32 pairs of static Add/Sub callers, the existing registry retains two
factories with capacity 3. Definitions 6/20/68 and instructions 23/65/209 grow
linearly; dispatch selections 2/16/64 and inference steps 327/2553/10185 include
ordinary per-call proof. Direct scalar arithmetic retains no registry. Measured
layouts are Descriptor 5, Selected 12, Permit 4, Body 456, semantic Operation 16,
BuiltinTraits 168 and Resolution 2032 bytes. These are bounded-work/layout facts,
not allocator or wall-time performance claims.

The corrected private9 gate is `fast/1793141346256868416` (113.73s), the final
native16 gate `fast/1793141732056984026` (92.10s), and final source3 gate
`fast/1793141885951759408` (44.03s). The final observer verifies the private and
integration binary hashes and exact two-test-file reuse delta, prints work and
negative diagnostics, and verifies 24 pinned upstream fixtures plus 126 Reference
files. Native executables and failed generated binaries are retired; input,
output, diagnostics, timings and hashes remain. The handoff authenticates the
committed source tree, all receipts, independent observations and source-native
producer/consumer hashes under `qualified-v1/`.
