# Finite ineffective unstable trait impl checkpoint

This unpublished PA40 producer supplies PA31's next source requirement. PA31
and PA40 remain planned; imported policy transport, the remaining lint inventory
and original library artifacts remain required.

The producer follows the pinned Rust 1.98
[stability visitor](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_passes/src/stability.rs#L587).
Only an actual direct unstable trait-impl annotation is a candidate. Resolve the
trait and visit the retained self/trait header paths, including generic path
arguments and alias identities. Stable declarations and known local absence
count as stable; primitives also count as stable. Never types suppress the lint,
except a bare function-pointer return Never. An unstable source rust-call ABI
suppresses it. A matching retained unstable feature bound also suppresses it.
Inherited impl instability alone produces no event. Default policy is Deny.

The existing configured attribute owner retains a cold map keyed by the exact
underlying impl, with its real wrapper Tail and item/node witnesses. Optional
effect IDs select the already retained unstable/bound Attribute witnesses;
the actual unstable attribute supplies the warning byte. No general owner-index
aliases, parsed owner fields or dense semantic fields were added. Bounds parse
once, with their complete input trees retained for invalidation. They grant no
API, const or body availability. Export profiles reject untransported bounds.

One paid pass visits actual associated owners. Each candidate walks only its
retained type headers and O(1) resolved declaration slots. A checked completed
local inventory distinguishes absent policy from a missing row. Imported/source-
free or catalog-imported declarations require explicit stability observation;
missing metadata never establishes absence. Qualified/associated paths,
unimplemented generic/type shapes, other callable ABIs and non-impl feature-bound
owners remain explicit requirements. The ordinary bootstrap rust-call fixture
requires source-ABI authority; its separate authenticated source-provider control
qualifies the genuine no-event result. Rust accepts both inputs.

Actual header events use the existing effective expectation identity and sink,
fulfilling before warning deduplication. Successful declaration collection checks
every actual owner before facts escape, so this lint's expectations can close at
that header boundary. No function body is demanded. Crate and untouched nested
function controls prove both event fulfillment and genuine unfulfilled results.
Failed/incomplete headers return an error before finalization.

## Qualification

The [source contract](../pa40/tests/spec/ineffective-unstable-impl/contract.json)
contains 29 authored inputs, with 58 matching independent Rust observations in
editions 2021/2024. Positive and opposing cases cover effective policy, owner
identity, f16, generic arguments/self, unstable aliases, inherited instability,
Never positions, source ABI and matching/combined feature bounds. The warning
source byte is independently fixed at 247 for the warning fixture.

Private gate `1793206087779998597` passes seven exact controls in 125.09s.
Production/CLI strict Clippy, the ordinary no-attribute allocation control and
lexer pass eleven controls at `1793206286402732919` in 169.02s. Metadata,
formatting and architecture checks pass in both gates. Earlier failed receipts
retain adapter/style diagnostics, a missing typed function-parameter arm and
the corrected test's mistaken error-phase assumption. No failing behavior was
blessed or hidden.

Measured x86-64 sizes are Level 1, dense policy ID 4, cold Policy 152,
policy/completion row 160, source Owner 160, Producer 368, annotation row 152 and
annotation map owner 32 bytes. Producer adds one cold optional pointer. No
attribute owner means no producer allocation. Repeated authenticated reads add
no retained bytes; wrong owner/item/effect, missing inventory rows and exhausted
payments reject with attempted work retained.

| Impl width | Charged steps | Charged bytes |
| --- | ---: | ---: |
| 1 | 225 | 5,456 |
| 8 | 1,121 | 22,872 |
| 32 | 4,193 | 82,584 |
| 128 | 16,481 | 321,432 |

Cold/warm/failed annotation observation reaches 96/156/286 steps. Linear delta
controls cover work and storage; these measurements are bounded operation counts,
not a wall-time speedup or full performance matrix.

Frozen CLI SHA-256
`a1f4ada57736d1e6fec14ff2ac2bf6933faee200154b8d6e9a42ecd96d7ff3f4`
passes all 29 declared CLI header outcomes, including the explicit ordinary ABI
requirement. Exactly one unchanged original-core observation verifies all 722
source pins and the same configuration. It opens 56 source files and stops at
`mem/alignment.rs:212`, byte 8375, `unsupported deprecated argument`, on the
existing `suggestion = "as_nonzero_usize"` attribute. Open counts are not compiled
percentages. The loader stops before completed original-core lint production;
this does not prove original expectation fulfillment or produce a crate artifact.

Raw diagnostics, source/binary hashes, independent predictions, work measurements
and terminal receipts remain under
`obj/parallel/ineffective-unstable-impl-v1/`. Full release/independent/native
matrices remain scheduled CI work; this checkpoint activates no assignment.
