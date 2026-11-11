# Source reference coercions through Deref

The frontend now converts references through actual selected Deref/DerefMut calls.
It preserves the expression's original operation, retains a separate paid call
chain, and feeds the final returned reference to the existing lifetime proof.
Existing access-call lowering evaluates the operand and each call once, then
forms the final borrow. Mutable consumers select real DerefMut; source effects,
formation, argument bindings, defining clauses, captures and loans retain their
existing checks. No associated Target, layout equality or source role grants call
permission. There is no new proof engine, global cache or dense node/frame field.
The existing sparse reference-adjustment record holds the checked conversion ID.

A known result expectation now reaches an otherwise unknown call result before
its inputs are checked. This lets `accept(&mut wrapper, t)` use its declared
return type. Pinned Rust rejects the corresponding unhinted statement/unit calls;
those are negative controls. An incompatible Deref destination cannot replace a
constructor's own nominal identity or field-input context. Existing compatible
factory and unsizing hints stay in force.

Built-in reference conversion/unsizing is tried first. A new regression proves
that a valid direct unsize cannot owe an unrelated conditional Deref bound:
Rust and immutable baseline05cc accept; an exploratory local version rejected an
unpaid Deref target. The corrected full checked-body case passes. Existing source
unsizing, source upcasts and their lifetime/equality opponents also pass.

## Independent and candidate evidence

The earliest packets contain57 required independent object outcomes,
43accept/14reject: PA26ordinary16 (11/5), PA30native2 (2/0), PA31source39 (30/9).
All28 previous input bytes remain unchanged. The38 const-Deref source prefixes
and the separate original Unsize prefix remain exact. Every new input and C
observer is listed in its owning manifest. All39 source outcomes now pass complete
checked-body/prepared-root construction. The two source lifetime negatives pass
typing and reject during actual lifetime/loan checking. Ordinary18 still stop at
core::ops imports: these remain REQUIRED outcomes; an import error qualifies no
language rejection. The two ordinary native artifacts remain REQUIRED POSITIVES.

Final public1794409791154946495 passes15 exact tests across2 targets in104.59s,
including new source contracts/scaling, old receiver/const-adapter/callback
contracts, PA18 factory-context controls and existing source Unsize/upcast groups.
Selected strict production/CLI/test Clippy, full formatting and architecture pass.
Final native1794410023347977560 passes3 exact tests in38.29s. Four new real source
Rust/CPPGM executions match independently checked Rust/C results:
`borrow-coercion 23 41 1342`, `borrow-guard 23 41 1345`, and two `borrow-const 41`
outputs. The guard observes the actual replaced value41 and drops after the call.
Existing receiver23/41 and three callback initializer/CTFE executions also pass.
The backend bridge/contract remains17b024f69; storage.py and compiler limits stay.

At widths8/32/128, inference steps are3653/13253/51653 and trait selections
56/224/896. Method selection steps/probes are0. Every counter is identical with
128 unrelated types. A temporary read-only harness copy bound to the final test
binary recorded these counters, then was retired; raw output and hashes remain.
The [machine evidence](pa26-borrow-coercions-evidence.json) records qualifications.
This is a bounded-work result, not an optimized or whole-core speed claim.
Upstream128/Reference126 and315 harness tests/468roots13bins44stages pass.
No new course oracle or whole-assignment activation occurs.

Two draft no-context positives and six draft unhinted scaling positives were
incorrect author predictions: pinned Rust independently rejects them with E0308.
The original draft bytes/diagnostics remain recorded. The two real inference
opponents and separately declared, accepted explicit-type scaling inputs are
reviewed against Rust before candidate qualification. No Rust-valid unsupported
positive was inverted, and all original28 language outcomes remain fixed.

## Core and remaining requirements

The implementation addresses the reduced shape of original RefCell::replace,
`mem::replace(&mut self.borrow_mut(), t)`. The original source has not yet been
reobserved with a frozen compiler for this checkpoint. Do not claim that the
original blocker is cleared until that observation reaches the real function.
The immutable producer, matched comparisons and unchanged120s original observer
are next. Prior callback full43034322443 on exact357d0bc5e/source5798 PASSED every
stage. A semantic successor is due after this source is published and no active
full run exists; never restart it for documentation/input descendants.

Core/alloc/std artifacts, complete source/library transport, ordinary bootstrap
imports and broader Deref/unsizing compositions remain required. Keep every
paid-alias, projected-premise, symbolic-index native, depth128, factory, tuple and
optimized requirement in handoff.md. Whole PA25–PA44 remain planned.
