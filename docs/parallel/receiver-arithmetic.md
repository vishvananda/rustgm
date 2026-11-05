# Actual Receiver and wrapping arithmetic checkpoint

Actual Receiver/Target source declarations now complete through the existing
ordinary role/header/associated producer. One actual trait/projection kernel
serves receiver declaration validation and type-only method owner discovery.
Legacy receivers still require real LegacyReceiver/Deref; arbitrary receivers
retain their actual source/macro feature, complete Self arguments and lifetime
proof. Raw pointers require their separate feature. Method selection consumes
the original outer receiver, keeps ordinary safety/loan checks and evaluates it
once. Reaching a target grants neither dereference nor target-field access.

The original blanket Receiver relation exposed a general coherence limitation:
unknown generic arguments do not make a closed nominal root downstream-local.
The shared implicit-absence query now checks orphan-relevant roots without
walking every nested argument. It also checks every nested ordinary trait goal:
a conditional impl with an open downstream trait argument remains possible.
Existing Deferred/cycle/cache semantics retain that uncertainty. Seven genuine
ordinary coherence controls prevent a feature-specific or blanket absence waiver.

The once-evaluation observer retains its genuine wrapping_add counter. Actual
wrapping_add/sub/mul now share the existing typed source arithmetic path, exact
source signature/Copy/call proof, scalar runtime operators and charged CTFE
overflow policy. Modular integer lowering adds no nowrap/overflow assumptions.
The source dependency classifier is the single authority for authenticated direct
operations; dependency discovery no longer duplicates its growing family list.
Unsupported declarations and indirect/reified operations retain ordinary demand.

The focused gate1793879895653700652 passes44 Receiver outcomes, seven wrapping
inputs and actual const slice read9/write91, all1193 PA19 and1389 PA20 text cases,
strict selected Clippy, formatting and architecture in111.82s. Independent Rust
reference-v9 matches all44 Receiver outcomes and three native observers. Wrapping
reference-v3 matches seven metadata outcomes and concrete72/ordinary23/f32 3.0
native results; its original generic object ICE remains unqualified. Earlier
candidate native gate1793879644601072298 passes Receiver3 and wrapping4 observers
in34.19s. It predates the final nested-coherence change, so joined native
requalification is pending. The accompanying evidence binds all receipts.

Original722-input core reobservation, common-control AA/ABBA, joined varargs/
future prerequisite checks and full private CI remain pending. LIB-167 is still
the last qualified raw-source frontier until that observation. Broader shadowing,
trait method discovery, lifetime/const/imported body composition, future state/
poll/drop/await/cancellation and varargs const/ABI requirements remain named work.
No whole PA after24 or core/alloc/std artifact is complete.
