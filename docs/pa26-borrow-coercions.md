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

Source7c7b1b132 freezes CLI2b7a8d17, empty tracked patch and3905 Rust hashes.
Thirty exclusive A/A+twoABBA observations retain complete output equality on an
ordinary import, accepted builtin-before-Deref conversion and actual priority
LowIR graph. Candidate/base wall ratios1.044/.998/1.029, CPU1.043/.999/1.029,
A/Awall.850/1.084/.739 retain noise and slower values. There is no optimized,
neutrality or original-core throughput claim.

Its first unchanged120s original observation ended at122.069393s with no item
events or failure records. This was an infrastructure deadline, not a language
rejection. Bounded stack snapshots reached different declaration/body phases;
they establish no repeatable regression or cause. A diagnostic hint-revert is
non-equivalent and was not adopted. The
[progress extension](census-progress.md) exposes implementation conformance and
projects all item locations through existing diagnostic provenance.

The qualified diagnostic successor6556387ca/CLI0912e08f binds an empty patch and
3905 Rust hashes. Its unchanged120s original observation ends at121.226504s,
retaining141 partial records and14,554 item attempts through function9261,
unicode/unicode_data.rsfile244:58785,line998. Both actual **RefCell::replace**
(function3753,body112:35560) and **clone_from** (function3771,body112:49846) are
reached without their prior failure records. The first remaining failure is
**RefCell::clone**,cell.rs112:49655,line1449, at
`RefCell::new(self.borrow().clone())`, requiring an unambiguous trait selection.
partial_cmp/cmp remain separate failures. ProgressSHA
1444d340ebf3b801acb112bdb9c3369326fbe53138c102c7aace338c83ef5bcf.
Postcheck1794414625686080575 verifies all722 source hashes,375opens,sixbindings,
the immutable/current3905 source hashes and unchanged profile/limits. No complete
census or original artifact exists; partial counts are not throughput evidence.

Prior callback full43034322443 on exact357d0bc5e/source5798 PASSED every stage.
This checkpoint is published at eae11a06a; full43059556041 finished FAILURE on
that exact head/source6556387ca: the unchanged result-self-context negative was
accepted. The [clone checkpoint](pa26-deref-clone.md) repairs the result-input
boundary and passes that exact regression; a fresh semantic full successor is due.
Comparison05cc and superseded2b7/d77/diagnostic4f executables were explicitly
retired after publication (prune1794414887003043616); raw producers/observations
remain. Current0912 and the separately pending optimized baseline remain. Reduce
the actual clone expression next, retaining its ordinary caller bounds and
the actual Deref call rather than inferring a cause from method names.

Core/alloc/std artifacts, complete source/library transport, ordinary bootstrap
imports and broader Deref/unsizing compositions remain required. Keep every
paid-alias, projected-premise, symbolic-index native, depth128, factory, tuple and
optimized requirement in handoff.md. Whole PA25–PA44 remain planned.
