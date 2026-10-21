# PA26 constructor promotion policy

The selected constructor table now records a variant identity and a typed
`ConstructorKind::{Variant,ConstFunction}`. Ordinary struct/enum constructors
and the canonical Option/Result variants remain genuine variants.
`ManuallyDrop::new` is recorded as a const-function factory through its exact
library metadata identity. Future constructor-backed const functions, including
atomic `new`, must retain this distinction.

Promotion admits genuine variant calls. A const-function factory remains an
ordinary call whose temporary cannot acquire a static runtime lifetime merely
because its result uses aggregate storage. Constant evaluation and runtime
constructor factories still use the selected variant identity and remain
eligible independently. This does not suppress promotion based on a nominal
result type: named ManuallyDrop constants and user lookalike tuple constructors
retain their ordinary permissions.

Twelve declared source outcomes cover three formerly accepted invalid returns
(ManuallyDrop holding u32, Option or unit), an aliased library path, an ordinary
user const function, constant reference initialization, named constants,
genuine Option/Result variants, tuple constructors, lookalikes and runtime/const
construction. The first seven fixture bytes retain the independent PA27 probe
hashes. Independent Rust 1.98 lifetime checks and five native controls pass;
existing nominal field privacy, destruction and layout controls remain covered.
Draft outcomes are in `pa26/tests/constructor-promotion-outcomes.json`; no
assignment activation or oracle regeneration is part of the correction.

The earlier PA30 linkage layout check is resolved after merging the reviewed
main reference receipts: all 38 assignment layouts now pass. Its earlier
baseline receipt failure remains recorded in the original evidence.

Strict all-target/all-feature Clippy and 32 focused test groups pass, including
promotion DAG/cache bounds, deferred proofs, generic constructor bodies, constant
destruction, nominal privacy/destruction and option_env execution. The source
audit reports 1,249 files and zero findings; all 38 assignment layouts, 24 pinned
upstream fixtures and 126 Rust Reference files verify. Final resource receipt
`c58bdfc4245b4e74925fa14a9749e3d5` records the batch. Evidence is under
`obj/parallel/pa26/constructor-promotion/` and
`obj/parallel/constructor-promotions/`. PA26's remaining hook families are still
listed in `pa26-environment-checkpoint.md`.
