# Borrowed source intrinsic Call contexts

The finite source transmute and arith_offset selectors now distinguish the
FunctionItem signature binder from the already opened Body/Code Call signature.
A real Formatter method taking borrowed string names exposed the mismatch:
source/destination and SSA operands agreed exactly, while the item signature
shifted the nested captured lifetime by one binder. The first scalar reducer
passed and was retained; no guessed type relaxation was applied.

One shared readonly canonical lookup lowers item signature roots by exactly one
function binder, preserving nested callable-local binders. Both source operations
use the same helper. Actual opened Calls match exact caller-level input/result
IDs, zero own binders, safety and convention through a shared pure validator.
The raw item signature remains unchanged for complete FunctionItem identity.
Dependency, runtime and CTFE consume those distinct contracts; CTFE reuses its
existing checked_call_arguments_with_signature hook and retains actual Code,
block/position, evaluated callee, result and const checks. No lifetime erasure,
new interning, caller permission or physical-layout equivalence is introduced.

Lookup work is prepaid for the attempt and observed steps are charged on success
and failure. Its existing one-million-step local bound is unchanged. Both roots
share one lookup, avoiding repeated shared-tail traversal. Failed payment returns
no selection. No dense body, node, catalog, descriptor or wire field was added.
Old source report/header/unsafe/const/pre-monomorphization size controls remain.

Final source-bound private9 passes format, architecture, strict private Clippy,
build and all nine exact controls in143.59s. Receipt
`obj/qualification/fast/1793256895369174628/receipt.json`, SHA-256
`9e13617b485b6f4da96bd9fbfe6cc35910d525a27d417afb48d9d9062a4d2c9d`. Controls include actual scalar and captured method calls, actual
constant execution, nested binders, wrong slots/types/flags/arity, failed work,
linear shared DAGs and old complete-call/owned-transmute/wrapping consumers.
Temporary diagnostics and copied Root Debug files were removed before this gate.

Independent const Rust receipt SHA-256
`9fd9734dbc03da2ea69f85a5656a2eb134c01d36fd76ba745762b0374c961492`
and candidate native receipt
`013132bf4d20414ff0c181f417a5ee16e10f2ebc062c1a061f40e9c32f68c70f`
bind exact source and immutable tools; both native traces are `1`. The first
feature/stability failures, scalar non-reproducer and actual signature diagnoses
are retained separately. No original crate artifact is claimed.

Root owns joined Debug source/native execution, current production/CLI and the
next unchanged-core observation. Imported operation provenance, whole original
Formatter/Debug/runtime and every parent assignment/artifact gate remain required.
