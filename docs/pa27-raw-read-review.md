# PA27 symbolic raw constant reads

This slice executes typed raw reads through the existing symbolic allocation graph. It does not add a byte storage format or change public value, origin, projection, or execution-requirement enums. Raw address cancellation remains address-only.

A private resolved place retains its address, checked type and raw derivation across cached projections and internal binding views. Reads resolve the original allocation and ordinal projection path, preserve storage lifetime checks, verify the requested stored type, and use retained ABI offsets to check physical alignment and allocation bounds. Local cell snapshots provide initialization checks without reading neighboring fields or padding. Footprint preparation checks typed field readiness before using the ABI cache and charges its work to the constant machine budget.

Static reads use canonical completed definition values rather than snapshots in address records. A reached incomplete immutable static requests its existing initializer body. Once that body is registered, a private initializer frame runs before the parent statement retries; it publishes the definition only after successful result validation. It has no destination value in the parent. Existing initializer cycle, frame, step and storage limits remain authoritative. Foreign static contents and mutable static reads reject independently of unsafe permission.

The source expression evaluator distinguishes raw pointer dereferences from ordinary reference dereferences using the retained operand type. Raw reads therefore use the same checked machine rules with or without surrounding statements. Safe reference evaluation keeps its existing path.

## Declared controls

`pa27/tests/raw-read-hypotheses.json` fixes independent Rust outcomes before each qualification. The first preparatory batch accidentally used `main() -> i32` in positive controls; that contaminated batch is retained under `obj/parallel/pa27/raw-read-prequalification`. Corrected source signatures were qualified separately under `raw-read-qualification`; their semantic hypotheses were unchanged.

The focused source suite checks full Rust 1.98 objects, then both frontend inspection and LowIR modes. It distinguishes supported reads from accepted Rust operations that remain explicit compiler boundaries. It includes local scalar and mutable-pointer reads, tuples, arrays, raw base projections, const calls, direct and indirect immutable static reads, and raw-address cancellation. The alignment controls distinguish a packed allocation with insufficient root alignment, naturally aligned packed storage nested in an aligned aggregate, and a nested field at a misaligned byte offset. Negative controls cover expired allocations, numeric pointers, index bounds, initialization, mutable statics and initializer cycles.

The ordinary uninitialized-local control fails Rust source initialization checking (E0381) and frontend borrowed-place checking. It does not establish uninitialized-byte validity. A separate machine invariant constructs an empty symbolic cell, checks raw-read rejection, initializes and reads it, checks an invalid projection ordinal, then expires its allocation and checks lifetime rejection.

The native controls check computed values for local products, function/static reads and physically aligned packed storage. The reusable native helper supplies the same checked failure function to Rust and RustGM; the locked CPPGM backend is unchanged. Qualification results, exact diagnostics, complete qualified native input hashes and artifact hashes are recorded in `pa27/tests/raw-read-outcomes.json`.

## Remaining contract

| Area | Implemented | Remaining |
| --- | --- | --- |
| Unsafe source rules | Lexical blocks/functions, operation policies, closure inheritance, nested-item boundaries, unsafe calls and static access | Complete PA27 release matrix and remaining pattern interactions |
| Raw constant execution | Typed reads of resolved symbolic storage, liveness, physical alignment, projection bounds and existing cell initialization | Byte reinterpretation, raw writes, reference creation through raw places, pointer arithmetic and broader provenance rights |
| Unions | Runtime layout, selected-field initialization, overlapping loans, field eligibility and destruction | Constant byte overlap, initialization masks, pointer-word provenance and scalar validity |
| Packed storage | Value copies, primitive compound assignment, raw addresses, qualified reference-pattern rejection and raw read alignment | Complete packed/pattern release qualification |
| Constant result validity | Reachable result graph, expired symbolic storage, zero-length repeat reachability and interior-reference escape rules | Invalid union boolean/uninitialized-byte rejection for validity reasons rather than unsupported-union errors |
| PA28 | Scalar/thin-pointer volatile operations, AtomicU32 construction/storage, Ordering and load/store rules | RMW, compare-exchange, further intrinsic families and final bounded atomic performance study |

The original demanded and unused invalid-union fixtures remain unchanged and currently reject through the explicit unsupported union boundary. Required initializer completion has closed their earlier acceptance hole; these rejections still do not prove boolean validity or uninitialized-byte enforcement. A future union implementation needs initialized bytes and symbolic pointer-word provenance; a product with an active field cannot model overlapping storage.

No assignment activation, oracle regeneration or completion claim is part of this slice.

## Checkpoint validation

- All 29 independently declared source controls match Rust 1.98 full object outcomes and both frontend modes: 17 Rust acceptances and 12 rejections; 14 frontend acceptances, 12 corresponding rejections and three explicit accepted-Rust capability boundaries.
- The direct machine invariant passes empty-cell initialization, successful typed reads, invalid projection bounds, cache reuse and expired-allocation rejection.
- Six affected maintainer binaries pass across the retained batches: atomic_storage, constant_locations, initializer_lifetimes, provenance_reads, reachable_storage and unsafe_unions. Three new native comparisons pass with status 0 and empty streams under Rust and the locked CPPGM backend.
- Strict all-target/all-feature Clippy passes. The new test was subsequently renamed without changing its bytes to satisfy filename-stem ownership; the complete file audit then passes with 1,297 files and zero findings.
- Assignment layout and upstream pin verification pass. All 7,786 active PA1–PA23 text comparisons match committed references; no reference output was regenerated.
- A bounded immutable performance study for this compiler checkpoint remains coordinator-owned; cache reuse is covered here by an invariant rather than a throughput neutrality claim.

The earlier partial gate passed atomic_storage and constant_locations before stopping at a diagnostic expectation changed by direct raw-read routing; later gates completed the remaining suites. Strict lint and filename-stem failures were corrected and their original logs retained under `obj/parallel/pa27/raw-read-final-gates`.

Integration must preserve the no-destination static-initializer return path when adopting PA25's `returns.rs` extraction. The initializer frame publishes only after validation, then resumes the parent's unchanged value slots at its decremented statement cursor.

The later scalar union-address checkpoint supersedes the historical raw-field
address/store boundary for all-fields primitive scalar Images only. It preserves
canonical typed projection aliases, current-byte overlap and immutable copied
snapshots; pointer-bearing Images, general reinterpretation and raw-reference
reconstruction remain outside that profile. See
[the union-address review](pa27-union-address-review.md) and its separate receipt.
