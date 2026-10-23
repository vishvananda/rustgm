# Runtime source callback selection

These frozen sources use actual source sizing, Freeze, Destruct, Drop, Tuple and
callable declarations. They are reductions of Rust core revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, under its MIT OR Apache-2.0 license.
`literal-attributed-declaration.rs` retains the attached source attributes and
Fn/FnMut/FnOnce methods; the other providers isolate semantic contracts.

The intrinsic declaration comes from `library/core/src/intrinsics/mod.rs`
2407–2416; the runtime move/spread rule comes from
`compiler/rustc_mir_transform/src/lib.rs`239–318. The transform chooses the
already evaluated runtime callback and spreads the owned tuple into its call.
It introduces no cleanup of the unused callback.

`contract.json` records 21 original independent source contexts and their
source/client hashes. Thirteen independent native observers cover runtime items
and pointers, generic and reference binders, argument effects and non-Copy
argument/result destruction.
Twelve semantic-provider observers are candidate runtime targets. The unchanged
full-staged provider remains valid Rust with an explicit unsupported source-core
feature boundary in the candidate; its API/const-stability support is pending.
The six language opposites reject under pinned rustc. The malformed declaration
produces a rustc ICE; it is a compiler robustness control, not a Rust rejection
contract. The accepted CTFE source remains required even while the candidate
explicitly denies constant callback execution.

Four additional declared scaling rows repeat actual mapped selector calls at
8/32/128 and a control with 128 pointer calls. Their C observers count every selected
callback; source/native runs include the largest rows directly. Their independent
observations are pending, with source/client hashes and expected outputs frozen
before execution. See the [checkpoint](../../../../docs/pa31-runtime-const-select.md)
for exact qualification and the retained mapping-hypothesis failure.

The original 26 observations and first 19 provider hypotheses remain unchanged
in retained development receipts. Their missing-Freeze and staged-attribute
failures are separate from these corrected provider contexts. Candidate
v3 validation covers the original twelve semantic-provider runtime contexts;
the final lazy-index/scaling delta remains pending merged qualification. No PA31
activation or whole-core acceptance is claimed.
