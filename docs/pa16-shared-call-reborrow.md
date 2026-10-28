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

## Identical shared values and capture formation

The root integration found a real false acceptance in the unchanged PA24
formation-truncation/reference-field.rs: `||x.a` returned as a static closure
must retain its actual shorter source storage. The new contextual reborrow had
introduced a synthetic dereference and incorrectly improved capture precision.
An identical shared-reference type now takes the existing value path, even
with a reference expectation. Exact inference roots are the fast path; determined
generic views reuse the existing charged, memoized canonical type query. An
unresolved pointee annotation keeps its real reborrow. A fresh expected call region
still permits its genuine reborrow. Mutable, unsizing and generic-value rules
remain separate; no Copy role, capture exception or new map is added.

Independent standard probes in both editions preserve four wrong initial
hypotheses: `||read(x.a)` and `||read(x)` legitimately reborrow and may capture
the static referent. Blanket suppression of implicit capture precision would
reject those positives. The owning PA24 fixtures distinguish them from the
identical returned value, explicit dereferences and move closures. No-Core
static-call/static-let negatives and an elided-call positive separately confirm
ordinary moves versus fresh lifetime opening. Their first setup lacked
RUSTC_BOOTSTRAP and retained E0554; the corrected identical inputs all match.
See source identity-contract.json and the PA24 shared-reborrow-captures contract.

## Contextual final selections and full course qualification

The final identity relation must replace an earlier inferred reborrow. Previously
ordinary identity publication kept the stale runtime adjustment when the final
target matched the source node's solved type. Branch/quantified merge locals
then expected one lifetime view but received another; seven existing valid
programs failed the strict body store check.

Contextual shared no-ops now retain the full ordinary type relation and publish
an explicit identity reset. Capture preparation folds resets in source order
into its two existing transient selection tables, with one prepaid linear pass.
It adds no wrapper rescans, source algorithm, retained map or permission owner.
The temporary return bundle merely transfers those existing maps. Runtime store
checks, actual source Copy/unsize claims and ordinary loan proof remain strict.
The two independently rejected conditional capture cases prevent reset handling
from retaining a stale synthetic dereference through a shorter-lived field.

All39 private controls pass126.57s at fast1793162155422015038. Production/CLI
strict plus18 lexer/reborrow controls and PA6 pass107.88s at
fast1793162545175933087, on the identical source map. The combined observer
reruns40 current private producers,15 native programs,24 exact old worker CLI
views and10 capture views. Original mixed formatting still matches Rust;
all headers/roles/layout and five-minute development limits remain enforced.

All1032 affected course outcomes match their existing manifest declarations.
The410 non-native views pass exact independent Rust classification in18.65s;
all622 affected LowIR cases pass the official native runner in34.14+36.02s.
Those are603 accepted native executions and19 independent/candidate rejection
checks. Only after that review were exact1032 source-bound references generated
through the official scoped writer. All generated channels match the previously
qualified observations. Existing26 changed diagnostic messages retain their
category and use updated source anchors; two new negative fixtures had no old
oracle. No expected outcome or source was rewritten to hide a failure.

The native runner now honors the declared independent rustc.default_lints policy
with the same cap as the syntax runner. It changes only rustc's oracle command;
candidate warning expectations stay strict. The622 cases exercise both declared
policies (222 false/omitted), and full Python harness validation follows.
This script-only change follows the compiler producer receipts and is separately
hashed in the independent qualification plans; no producer history is rewritten.

Current CLI3053cbd133f345f88201e332196f55edddd6e4c94b2db4d617d5e5d8f016fdc2
still reaches expect(deprecated) in unchanged core, with no crate artifact.
Evidence lives in obj/parallel/reborrow-flow-integration-v2. The broader ordering
of other non-reset replacement adjustments in capture collection remains a
separate preexisting review item; this checkpoint specifically synchronizes
contextual identity resets and preserves its tested opponents.

The final active PA1–PA24 text run passes all8605 cases in60.19s after scoped
reference generation. Full harness253, source/placement/assignment registration
and upstream checks pass. Scaling retains exactly N sparse reference rows for
N=1/8/32/128, work764/799/919/1399 versus ordinary scalar794/822/918/1302,
with Body456/NodeFacts80 unchanged. Warm unsizing work is independent of8192
irrelevant declarations and its selected-child storage is reused across calls.
No O(1) claim is made for genuinely unresolved canonical prefixes.
