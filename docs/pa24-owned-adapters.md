# PA24 owned closure receiver adapters

This checkpoint builds on the integrated compiler `f8939df8`. PA24 remains
planned. It adds checked receiver wrappers for captured closures whose enclosing
owner has no type or const parameters: `Fn` to `FnMut`/`FnOnce`, and `FnMut` to
`FnOnce`. A move capture does not by itself require `FnOnce`.

## Typed execution and proof

`types::closure::Adapter` retains the complete canonical closure `Instance` and
requested call trait. Its constructor validates complete compatible permission;
identity alone grants no permission. `Owner::ClosureAdapter` and
`BodyKey::ClosureAdapter` distinguish the wrapper from its original source body.
The sparse selected `Entry` retains the actual caller execution signature.

`body::adapters` constructs a typed CFG with an ordinary hidden receiver
parameter. It borrows or reborrows that receiver, invokes the strongest source
entry and uses the existing storage, move, loan and drop machinery. The owned
`FnOnce` wrapper drops its environment after invocation. It preserves all source
public/environment universal slots, including unused ones, and introduces a
private local borrow instead of reusing the removed source receiver slot.

The dedicated incremental proof route requires the exact immutable `Proved`
source body and a private derived template that validates the complete instance,
public/source execution signatures and wrapper signature. The source proof stays
in final assembly. Ordinary instance proof cannot authorize an adapter; primitive
factory permission and conditional nested-output summaries supply no authority.
The wrapper contract contains its own signature and obligations, with no copied
source-local assumptions. Selection equalities remain caller proof obligations.

Demand caches use the complete adapter identity. Existing body/instance budgets
charge wrappers and their retained contracts. Lowering indexes checked wrapper
bodies once and emits ordinary function calls with a checked execution ABI.
No generated source or fake definitions are involved; ordinary instances retain their source-template requirement.

A supporting correction applies the existing declared callee-premise proof even
when substitution leaves no projections requiring normalization. Only declared
`Trait` and `ProjectionEqual` clauses enter the existing scoped witness and
quantified-condition path after complete substitution. Manual `with_contract`
keeps requiring the debts. Equal projection results do not imply equal inputs.

## Qualification

The declared matrix is `pa24/tests/receiver-adapter-outcomes.json`: 22 exact
fixtures, 13 accepted and nine rejected by pinned Rust 1.98. Nine accepted cases
execute through the external CPPGM backend with value/drop self-checks. The
remaining four accepted cases are explicit implementation boundaries at this
frozen baseline: three blanket-reference forwarding cases and the contextual
quantified-identity producer gap. The original root declaration/qualification
receipts are retained; these boundaries are not Rust rejections.

Focused evidence is under `obj/parallel/pa24/receiver-adapters/`. `fourth.json`
records all 22 independent outcomes, all 22 frontend outcomes and nine native
positives. `invariants-second.json` records seven private proof/cache invariants,
17 existing premise invariants and the existing premise source/native controls.
`final-functional.json` records 73 passing affected proof/source/native groups,
including the final forged owner/key invariant. `final-static.json` records strict
release all-target/all-feature Clippy, file audit 1359/0 and pinned upstream
24+126 agreement. The unchanged PA16 receipt mismatch on this older baseline is
retained for the root merge; no references were regenerated. Earlier failed
diagnostic and lint observations remain available.

Private controls reject ordinary proof of a wrapper, a wrong source index,
forged signature, forged matching owner/key, different environment and an unpaid
private-borrow-to-static debt. Sealed-caller versus manual specialization and
noninjective-result controls keep the premise authority explicit.

At widths 8/32/128, distinct generic callers of one closure demand exactly two
receiver wrappers. Total checked bodies are 13/37/133 and retained CFG bytes are
16,216/48,664/178,456. Both grow by constant increments per additional caller;
the original proved closure remains present. The source also contains an ordinary
generic identity call as a shared control. This is bounded structural evidence,
not a wall-time performance claim.

## Remaining contract

Reference forwarding needs separate typed blanket callable evidence and receiver
ABIs; an owned closure key cannot stand for `&Closure`. Generic-owner captured
adapters and dynamic callable adapters remain explicit boundaries. The root's
contextual producer checkpoint will supersede only its accepted fixture's
frontend boundary and add that unchanged source to native execution. Assignment
activation, complete release qualification and reference publication remain with
the integrator.

## Contextual producer integration

After merging root `33f10048`, the unchanged contextual quantified-identity
fixture is accepted and joins the native group. Its draft record explicitly
supersedes the frozen `f8939df8` producer boundary; the earlier failed receipt
and original Rust acceptance remain intact. The matrix now has ten implemented
native positives and three accepted reference-forwarding boundaries.

`contextual-merge.json` records all ten affected adapter/contextual/reference
source/native groups passing on the combined source, with layout38 and file
audit1363/0. The contextual fixture itself retains SHA-256
`10f5c7500069184324b240379cd026952fccfb14e71cf268f93abb71a6374ef1`.
