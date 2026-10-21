# Shared captured-callable entry selection

`body::invocation_entries::select_execution` selects a captured execution entry
from an actual receiver and an already opened physical call signature. Ordinary
Invoke consumes the same core as future callable-object wrappers. Selection is
not a body proof or permission to erase the receiver.

The API takes mutable Declarations and Contract, the defining caller NodeKey,
requested CallTrait, full actual receiver TypeId and selected Function TypeId.
The caller must equal Contract.source. The signature must be safe Rust with
bound_regions=0 and its first parameter must be exactly the actual receiver.
Higher-ranked input/output binders must first be opened through the ordinary
scoped call proof; object-local or environment regions cannot be replaced by
static to pass selection.

The existing complete instance/receiver matching, repeated-slot equalities,
projection normalization debts and checked generic source-condition images are
retained. Conditions are caller obligations. Generic selection fails without
the checked source catalog. Entry construction stays private; source body and
wrapper proofs remain mandatory before ResolvedCallables and native lowering.
The ordinary noncaptured path avoids interning unused physical signatures.

This extraction changes no generic adapter boundary. Generic stronger-trait
and borrowed receiver adapters remain explicitly unsupported until their
wrapper proof consumes the exact sealed specialized source while preserving
the original checked template and full Instance. The later generic receiver
slots must not be shifted when an obsolete source receiver slot becomes unused.

The merged dfff66d4 source passes all-target/all-feature check, eleven entry
invariants, and all thirty-two environment_execution source/native groups,
including generic capture scaling. Strict all-target/all-feature release Clippy,
architecture audit (1420 files, zero findings) and assignment layout pass.
The new malformed-header invariant rejects wrong caller, receiver, closed
binder, unsafe signature and C convention. Receipts, including the corrected
import/name audit attempts, remain under
obj/parallel/pa30/aggregate-design/selector*.json.
