# Ordinary shared call reborrowing

Established reference expectations now use the existing typed reborrow owner for
shared references as well as mutable references. This includes ordinary calls,
method arguments, function pointers, explicit generic reference arguments,
annotations and contextual groups/blocks/branches/loops. Generic referents,
aliases, nested mutable references and wide slice/str heads retain ordinary type
and loan proof. Contextual unsizing reborrows before the existing conversion.

The compiler change is confined to `sema/bodies/references.rs`. Its existing
`identity_reborrow` witness is passed into reference selection; type joins pass
false. An unknown generic argument or inferred binding cannot acquire a reborrow
retrospectively. Existing place construction, fresh inferred regions, source
ancestry, promotion, Copy requirements, contracts, loans and original/instance
proofs remain authoritative. Source unsizing additionally authenticates the current
CoerceUnsized declaration/image and enqueues its ordinary trait goal on the
converted reference view. Existing selection must prove the actual source impl
and its Unsize/formation premises; the original place conversion still proves
lifetimes and loans. No source reference Copy/Clone impl, bootstrap
permission, role, retained field/cache, wire or backend operation is added.

The original `source-format-macros/display/source-mixed.rs` is unchanged at
SHA256 `a21e9e3546226c831113cc7b4a9bf2337555911697395ba4ccc65c46296208a8`.
Its genuine constructor graph now runs without a reference Copy impl and matches
Rust's `e1;e2;t1;p0M9;p1D7;p2D9;d9;d7;3` trace: inputs evaluate once, constructor
pairs retain their real formatter identity, and drops preserve order. The
separate genuine shared-reference Copy/Clone successor remains exercised; it
never waived the original reborrow obligation. Full Formatter/write/PanicInfo/
handler execution and original core/alloc/std remain required.

The ordinary and PA31 source controls were frozen before compiler edits. Plain
reference controls belong to PA16; four byte-identical type-generic controls
belong to PA18. Unused generic helpers were removed from plain PA16 inputs after
reading the owning handouts, with original bytes/observations retained. The existing generic
loan-view construction fence was observed rather than waived; PA18 controls use
its supported defining-body/LowIR library pipeline. Three accepted unused-body
libraries have object-only evidence; their empty LowIR claims no native execution.
Fifty-three original metadata observations retain one prediction disagreement:
the first source unsizing profile lacked its actual Unsize/CoerceUnsized
foundation. A separately frozen complete foundation passes without adding a
reference Copy impl. Twenty-four separate ownership-correction metadata commands
all match; these do not replace the original observations. Three additional
foundation probes retain an Unsize-only positive prediction disagreement: both
real roles and the applicable shared implementation are required. Five explicit
cast probes match independently. A candidate negative caught structural source
unsizing without those prerequisites, so the ordinary source trait owner now
checks the full goal. No-role, no-impl and mutable-only profiles reject; the
complete shared implementation passes. In total, 85 metadata commands are
retained across the separately frozen packets. Generic value/inferred binding moves, late input inference,
Copy bounds, shared-to-mutable conversion, a live shared loan, local-reference
escape and owned moves reject through their actual owning diagnostics.

Four general native profiles match independent Rust object/link/execution:
ordinary/source tuple and function-pointer reuse prints
`1:9;2:9;3:13;4:13;7`; effectful returned-reference operands additionally print
`5:9;6:9` exactly once in order. The unchanged formatter profile reuses its
already qualified independent eight-command evidence. Fifteen candidate native
commands cover all five profiles; objects/executables are hashed and retired,
while source, LowIR, diagnostics and raw observations remain.

Private controls construct and promote every defined function before checking
its actual source Copy, region and loan contracts. The initial helper's omission
of nongeneric bodies and missing promotion was caught and corrected; failed
receipts remain. Existing array/match coercion representation assertions also gained
the newly required shared operand borrow; its mutable-pointer read count and
independent Rust acceptance stay unchanged. Required native producers and sparse-row scaling already pass.
The 1/8/32/128-call matrix retains exactly one reference adjustment per contextual
call and none for its scalar control, with no new persistent owner. Body remains
456 bytes and NodeFacts 80. Final work observations and affected private/CLI
receipts are bound in the handoff under
`obj/parallel/pa16/shared-call-reborrow-v1/qualified-v1/`.

These focused checks do not claim a whole PA release or a broad performance gain.
The integrator publishes reviewed PA16/PA18 text oracles and runs cumulative release
and upstream library gates. The historical no-Copy failure is retained rather
than rewritten as a Rust language rejection.

Final guarded qualification passes 13 private controls and 11 production controls
plus PA6. All24 CLI outcomes match their independent predictions; six additional
CLI/backend/native commands match the two general C traces. The unchanged
five-profile candidate native run contributes15 commands. Upstream verification
passes for the pinned fixtures and Rust Reference bytes.

| Calls | Reference rows | Scalar rows | Charged reference work | Charged scalar work |
| --- | --- | --- | --- | --- |
| 1 | 1 | 0 | 764 | 794 |
| 8 | 8 | 0 | 799 | 822 |
| 32 | 32 | 0 | 919 | 918 |
| 128 | 128 | 0 | 1399 | 1302 |

These are complete per-fixture canonical-work counters, including their common
source foundation, not a wall-time speed comparison. The source/runtime code is
unchanged between the final native and observation checkpoints; the exact delta
is an integration-test row count and the correction from an unsupported generic
loan view to PA18's existing LowIR contract. The reverted temporary mode change,
failed launchers, failed generic inspection and bridge filename setup failures
remain in the raw packet. Official20 PA16 and4 PA18 sidecars remain unblessed
until integrator review/publication.
