# Runtime source callback selection checkpoint

The qualified local v3 candidate selects the evaluated runtime callback of an
actual source-declared `const_eval_select`. It passes twelve runtime source and
native comparisons, six semantic rejection controls, fourteen private invariants,
and explicit CTFE, malformed-header and full-staged-provider boundary checks.
PA28 and PA31 remain planned.

The handoff adds a reviewed lazy pointer-only path, diagnostic-specific rejection
checks and four frozen scaling contexts. That final delta is deliberately
**unqualified locally**: the integrator owns its fresh merged audit, strict,
source, private and native qualification. The local v3 results below are retained
for their exact source snapshot, rather than described as a pass of that delta.

## Retained source contract

The compiler recognizes the actual validated intrinsic declaration, with four
owned Sized type parameters ARG/F/G/RET, three owned inputs `(ARG,F,G)`, RET result,
safe Rust convention and const declaration eligibility. It checks ARG:Tuple,
F:const FnOnce<ARG,Output=RET> and G:ordinary FnOnce<ARG,Output=RET>. The source
sizing hierarchy supplies genuine default MetaSized clauses; only its exact
completed Sized/MetaSized/PointeeSized identities, Ordinary effect, empty external
arguments and these four owned subjects are admitted. The legitimate internal
Bound::Sized representation remains supported. Other defining constraints fail.

Header identity is structural data. The actual caller still proves the complete
source call, tuple, callable, lifetime, ownership and destruction requirements.
The runtime profile supports const function items for F and safe Rust function
items or function pointers for G. It introduces no general closure profile.

The private non-Copy RuntimeCall borrows the actual Lower body and Call ValueId.
It retains exact logical inputs/result and the opened/public signature relation.
Operands have already executed in source order; cached values are consumed once
and the owned tuple is spread through the existing ABI writer. Non-Copy fields
move through ordinary ownership facts, and Copy behavior remains intact.
The unused F callback retains its obligations without acquiring an executable
demand merely because it is an operand. The callbacks themselves are dropless
in this first profile.

## Actual value origins

The preserved mapping hypothesis was wrong: in the original source observation,
logical G instance115 maps to itself. Its actual operand is Constrain(v5), whose
FunctionItem instance121 maps to the proved runtime target117. The old dependency
consumer demanded logical115 while emission preserved evaluated117. Both the
failed hypothesis and failed candidate receipts remain retained.

Demand and emission now use one data-only index of actual body value origins.
An earlier FunctionItem Constrain operand with the same declaration inherits its
indexed origin. The already-proved full relation remains the caller's authority;
the index does not reconstruct it or install a global logical115-to117 mapping.
Each consumer derives the raw actual origin and crosses that body's existing
callable_target mapping once. Runtime emission strictly compares the evaluated
Selected::Item with that actual mapped target.

The v3 observer records actual target117 exactly once, no demanded logical115
body and no unused actual F target. Its source is the real runtime callback
Node284, full instance117 and public/internal signature pair89/89 in that local
type store. These numeric IDs are evidence coordinates, not external identities.
The existing reached-body check retains source, key, owner, full instance,
public/internal signatures and actual canonical owner index. No Body, Code,
IRFrame or backend opcode was added.

The index is one forward O(V) pass with four-byte ValueId rows and O(1) origin
queries. Demand also charges the selector-call count in its cumulative dependency
work owner. Emission retains a bounded local cold work/storage owner behind one
optional Lower pointer. Preparation charges before allocation, checks actual
vector capacity against the payload bound before filling, and keeps sticky
failure without publishing partial rows. Vector payload excludes Vec/Box and
allocator metadata. The final handoff defers demand's index until its first
FunctionItem G; ordinary and pointer-only bodies require no origin table.

## Completed local evidence

All commands used the configured resource/storage guard without reducing CPU,
memory, task, disk reserve or worker limits. The v3 source snapshot, tool/build
receipts, exact inventories and outputs are under
`obj/parallel/pa28/const-eval-select/source-origin-focused-v3/`.

| Check | Local v3 result |
| --- | --- |
| rustfmt, architecture/file audit, all-target/all-feature strict Clippy | pass |
| Source runtime contexts | 12 accepted |
| Semantic opposites | 6 rejected at intended trait/projection/type/lifetime rules |
| Private identity, signature, budget, origin and storage invariants | 14 passed |
| Runtime native comparisons against frozen Rust observations | 12 passed |
| CTFE selector | explicit unsupported callback execution |
| Full-staged original provider | valid Rust; explicit unsupported source-core feature |
| Malformed intrinsic header | safe candidate error; pinned rustc ICE retained |

The v2 harness and upstream provenance checks passed. V3 retained them after
reviewing its sole pure predicate-extraction delta; their receipts are in
`source-origin-focused-v3/retained.json`. Final fixture additions invalidate that
retention for the merged checkpoint, whose harness/upstream checks remain due.

The independent corrected18 and mapping2 batches have no hypothesis
disagreements and twelve native observations; the unchanged owned-drop context
retains its earlier independently qualified native observation separately.
The full-staged source and its positive Rust observation are unchanged. It is
not classified as a Rust rejection. The malformed-header input causes a pinned
rustc ICE in both metadata and object lanes, representing one compiler-failure
context rather than two language rejections.

Original failures remain under runtime-focused-v3, runtime-focused-v4 and
mapping-diagnostic-v1. Source-origin-focused-v1 was cooperatively interrupted
with infrastructure status130 before its gate; v2 stopped on the dependency
scan's Clippy function-size finding. Those failures were not blessed away.

## Work and storage observations

Completed work and retained payload are reported separately. The local v3
chain/shared-prefix index observations are:

| Shared prefix depth | Completed rows | Actual vector payload | Warm rebuild work |
| --- | --- | --- | --- |
| 1 | 3 | 12 bytes | 0 |
| 64 | 129 | 516 bytes | 0 |
| 2048 | 4097 | 16388 bytes | 0 |

The local v3 record sizes are Lower720, Scratch360, RuntimeCall64 and Origins72
bytes. ValueId is four bytes. The ordinary-static-target control constructs zero
indexes; item-empty constructs two indexes, one during demand and one during
emission, for ten completed rows total. Actual source scaling in the final delta
also reports maximum single-index vector payload, separately from completed row
visits. These are structural/counter observations, not a wall-time speedup claim.

The four new frozen rows are scale-mapped-8/32/128 and scale-pointer-128. They
preserve the qualified mapped-provider declaration prefix, the qualified record()
observer protocol and the qualified pointer-selection protocol. The C observers
require outputs `7 8`, `7 32`, `7 128` and `9 128 13 128`, with final newlines and
empty runtime stderr. Every selected callback must therefore execute. Candidate
source/native execution includes all four rows; pointer128 must construct no
origin index. The declarations, generator and source/client hashes are frozen
under `obj/parallel/pa28/const-eval-select/source-scaling-v1/declared.json` before
independent execution. All four rows independently pass pinned rustc metadata/object and linked native
execution in source-scaling-v1/independent-v1. All expected outputs and runtime
stderr match, without a hypothesis disagreement. Tool versions/hashes, complete
commands, source/client hashes and raw observations remain in binding.json,
results.json and summary.json. The plan SHA-256 is
403c70a90ef5c1068f3a9543520d10cf17eb5f1f37750a26d373f183b0f21473;
the independent results SHA-256 is
bfe46cc548e95a62eab2fa14affff87386fcccf6629c6317eda43e7ec38aecea.
This independent-only run made no RustGM/backend invocation. Actual candidate
source/native execution and merged scaling measurements remain integrator-owned
and pending; they are not inferred from the smaller v3 contexts.

## Remaining boundaries

Constant callback selection is explicitly unsupported, including a body-bearing
placeholder. General callable objects/closures, callback destruction and wider
intrinsic families remain separate work. The original fully staged declaration
retains active API/const-stability requirements; local policy integration may
advance its exact unsupported diagnostic, and the integrator owns that review.
No source attribute is silently stripped to claim support.

Whole-core/alloc/std acceptance, assignment activation, generated references,
combined release qualification and controlled performance publication remain
integrator responsibilities. This checkpoint uses the existing pinned backend
and supplies no new backend capability claim.
