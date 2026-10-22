# Half-open integer ranges: bootstrap contract

This checkpoint adds real canonical `core::ops::Range<T>` record identities and
half-open integer iteration. The record has ordinary public `start`/`end` fields,
uses the existing typed construction operation, is not Copy, and retains only
its actual conditional Clone marker. That marker does not authorize executable
Range::clone. The current factories are bootstrap library contracts supporting
range syntax, `for`, and ordinary selected calls. The endpoint is to obtain
Range/Iterator/IntoIterator declarations and method bodies from pinned upstream
core source through validated source-declared compiler bindings; these factories
are not a direction to encode ordinary library convenience APIs in the compiler.

The fixed integer catalog has twelve real impl/Item/next identities. Its checked
ordinary next bodies test start<end before incrementing, update start, and return
typed canonical Option variants. They use normal scalar arithmetic and source,
lifetime, ownership and drop verification. There is no new body/LowIR enum,
backend opcode, primitive Permit, fake syntax or callable identity.

Record-only construction and source lookalikes keep the integer bodies cold.
Actual loop/method/qualified/generic demands prepare only the exact reserved
headers through existing readiness and conformance machinery. Pending is never
absence. Disposable applicability probes do not become execution authority.
Fully paid selected Instances and Item/IntoIter/Option relationships are still
required before publishing ForPlan.

An unsuffixed range can need its Item type before numeric fallback. The bootstrap
inference facade checks the complete applicable validated canonical header
family, verifies every actual Item definition equals that header's Range element,
and retains the shared equality beside the original projection/call debts.
Pending or competing headers/caller assumptions prevent the shortcut. It does
not select a primitive from the expected result or prove Iterator for a generic,
float, bool or nominal element.

Independent qualification: `obj/parallel/pa26/integer-ranges/qualified-v3` retains
52 Rust1.98.0/2024 outcomes (39 accepted with native self-checks, 13 rejected),
using SHA-matched earlier observations and fresh observations for nine additions.
The original v1 mistaken Copy hypothesis and its Rust E0382 rejection are retained;
v2 corrects only that prediction. Original source bytes and immutable IterMut
baseline dc8f0ee1754e03366bbe004a74f140c883c827062cec3f60c1042f11d87bb855
are preserved. No reference/oracle generation is performed by the worker.

The implemented target is 47 declared sources:34 accepted and13 rejected. Five
independently accepted originals remain required: inclusive-max, from-break,
char-inclusive, range-char-half-open-required, range-clone-required. Their source
bytes, Rust acceptance/native observations and current frontend boundary are
retained separately; they are not language-negative tests. Owned arrays also
remain required (21 predeclared sources in owned-array-iteration/declared-v1,
not qualified or implemented in this checkpoint). Other remaining PA26/30
requirements are retained in the existing milestone records; no activation is
claimed. Further ordinary library API coverage should be assessed against the
upstream core→alloc→std bootstrap plan rather than additional convenience-method
factories.

## Qualified checkpoint and limits

The focused `functional-v20` guard passes all 47 implemented source outcomes,
all 34 native self-checks, seven ordinary factory/collector/layout/scaling
invariants and two inference/header invariants. Both batch and incremental
contract collectors prove the actual originals. Wrong registered member,
subject, Item, predicate, binder and completion controls reject; member IDs
cannot masquerade as pending impl headers, and a forged canonical registry
cannot enter factory publication. The original failed readiness traces and
superseded test-only/harness failures are retained beside the passing receipts.

The `closing-v21` guard passes strict all-target/all-feature Clippy, the
1,801-file syntax/architecture audit with zero findings, assignment layout and
grouped test inventory. Ten existing custom/shared/mutable/Clone iterator
source/native controls pass, including generic public-signature storage.
Its upstream invocation used an unsupported `check` argument and stopped
before text; `tail-v22` corrects only that harness argument, verifies pinned
upstream bytes and completes the one active-text pass. It does not repeat
already-passed functional checks.

All 8,360 active PA1–24 statuses match. The sole committed-reference stdout
difference is the inherited PA6 `for_binding` names view, byte-identical to
the qualified 39c9f541 IterMut pin: SHA-256
`4452135d25c2bb541cc415d071ea9924c1aec6758bd7654ced6dde432e25620e`.
Its unchanged source SHA-256 is
`5875118f51d4d8f8f7dcab79451764f76f6b74eea721c505aa361a6e50d09521`;
the prior independent Rust receipt remains in
`obj/parallel/pa26/mutable-iteration/text-v1/names-review.json`.
There are no new active-text deltas from this Range checkpoint. References
were neither generated nor edited.

The immutable candidate and full Rust source map/archive are under
`obj/parallel/pa26/integer-ranges/text-v1`; executable SHA-256 is
`d57783b3efe7b11439240c00a8f527e6df48be6b3c9d834c1db615f70ff3c7d9`.
Full combined release, independent fixture/oracle publication and immutable
performance measurements remain integrator-owned. This is not a release or
assignment activation.

## Storage and logical work

On the isolated 39c9f541 base, ordinary Declarations remains 8,552 bytes,
Resolution 2,152, Type Kind/IR Value 24 each, and inference Context/Store 448
each. The optional demanded iteration Registry grows from 312 to 320 bytes
for its Range owner; RangeRegistry is 232, resolver Ranges 204 and an Original
40. The cold pending Heads is 56 with a 52-byte RangeHeads allocation, while
dense Pending remains 152. Scalar-only sources have neither range catalog;
record-only construction retains reserved identities but no integer method
contracts or bodies. Existing canonical nominal IDs are preserved.

At 8/32/128 loops, inference work is 2,308/9,100/36,268 (`44 + 283*n`) and
trait selections 16/64/256. One loop with 8/32/128 unrelated nominal/function
pairs costs 359/455/839 inference steps (`327 + 4*n`), with two selections.
Added nominal-requirement and variance work is zero throughout. These are
charged logical-work observations, not full-pipeline timing or instruction
claims. The exact generator is
`progression::range_activation_and_unrelated_source_work_remain_linear`;
source copies and hashes are in
`obj/parallel/pa26/integer-ranges/scaling-v1`. Those copies have not yet had
separate independent rustc observations; they are ready for integrator review.

## Next compiler boundary

The next priority is validated source-declared lang-item/intrinsic binding
and no_core/self-core ownership, with pinned SizedTypeProperties layout
constants as the first consumer. Ordinary upstream trait declarations,
blanket impls and method bodies must replace these bootstrap factories.
This preserves the still-required PA26 desugaring/selection/derive/formatting
responsibilities and all outstanding PA30 ABI/release work. It does not move
ordinary library convenience methods into a new compiler inventory.
