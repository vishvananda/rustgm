# Actual source destruction helper

This sized runtime consumer has focused source/native qualification on the
Freeze/DropGlue prerequisite `e9b002f2`. Central combined release qualification
remains separate; no assignment is activated. It follows Rust 1.98.0 commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`.

The actual helper in
[core/src/ptr/mod.rs822–829](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/mod.rs#L822-L829)
is const unsafe Rust, takes one `T: PointeeSized` and `&mut T`, returns unit, and
requires `T: [const] Destruct`. Its empty body is a compiler placeholder. The
ordinary `drop_in_place` source forms the reference through its existing unsafe
raw-place rules. Neither a source spelling nor an empty body grants execution.

## Typed runtime boundary

The actual sealed source role supplies only declaration data. Ordinary original
and materialized body/contract, region, loan and caller checks remain mandatory.
Mechanical lowering matches the full selected Instance and T, actual retained
body index/source/owner and both public/owned signatures. Its selected reference
ABI must be thin for the first sized family. It does not erase lifetime binders,
reconstruct a signature from counts or infer an owner from matching layouts.

The existing destruction Proof and `Recipes::exposed(T)` authorize the exact
recipe. Missing entries remain errors even if another recipe has an equal
layout. Existing cleanup emission retains custom-method then active fields in
declaration order and ascending array order, including actual ZST calls.

Only the exhaustive completed-empty registry has a separate no-cleanup route:
a bounded concrete query must produce exact false for the same full T, after
fresh actual-role/registration, proved-body and completed sized-layout checks.
Missing heads, partial registration and budget failure never become unit return.
One lazy query is shared within the emission invocation and binds both the
canonical Store and immutable aggregate Definitions. Ordinary paths allocate
nothing; no Context or Frame field is added. Structural answers carry no caller
permission. Error locations come from the retained proved header, not a rejected
claimed source.

## Independent contracts retained

`obj/parallel/pa28/source-drop-glue/` retains every original declaration, source
hash, default-lint diagnostic, object/native observation and failed hypothesis.
The planned PA31 fixture contract copies exact source and C-observer bytes.
Observers pass valid borrowed storage, and never create a second Rust owner or
drop the referent after a helper call.

Direct and generic calls, the unchanged pinned wrapper body, named-lifetime
reification, custom/field order, selected enum payload, reference nonownership
and primitive/zero-size cases independently execute. The original cleanup-count
source remains byte-for-byte hash
`633442327cf88bd6f30745d737706695860e5ea96cdbfd2662a81cdb25e1d110`
with native output seven; it exercises ordinary scope cleanup rather than an
explicit helper call. The first reification hypothesis was invalid because its
function pointer quantified an extra inner lifetime; both that rejection and the
distinct named-lifetime positive are preserved.

Nonempty ZST arrays first accepted metadata but required genuine source
Unsize/CoerceUnsized declarations for rustc-generated glue. Distinct completed
contexts independently execute 1/8/32/128 logical calls. Their source hashes are
not interchangeable with the original no-role context. Initial malformed
attribute placement and optional unknown feature names are retained separately.

Actual const cleanup independently returns seven after the genuine
`const_destruct` feature and pinned `const impl` syntax are supplied. Ordinary
Destruct/nonconst Drop cannot discharge const obligations. Rustc accepted the
transient invalid-bool/expired-reference dropless CTFE hypotheses; those are
retained oracle/profile observations, without invalid native executions or
additional reference permission.

## Focused qualification

`obj/parallel/pa28/source-drop-glue/consumer-close-v2/` retains the frozen source
inventory, commands, outputs and hashes. All eleven runtime positives and four
independently rejected opposites pass checked lowering, alongside the eleven
prior needs-drop positives. Four private controls pass; twenty-two native
programs reproduce the eleven new and eleven prior independent observer results.
The unchanged cleanup-count source returns seven. Strict all-target/all-feature
Clippy, the 2,012-file audit with zero findings, 152 harness checks and upstream
verification pass. These are focused internal checked-LowIR/native results,
without a whole-source-library metadata or full-release claim.

The invocation State measures 32 bytes. Context remains 1,656 bytes and Body
448 bytes; no Context, Frame, Code, Value or pointer field was added. Shared
depth-32 type graphs at root widths 8/32/128 take incremental query work
829/99/387, retaining 6,040 logical bytes. The first root includes the shared
cold traversal. Adding 8,192 unrelated types changes neither result; 128 repeated
queries add 128 work units and no retained storage. Tiny-budget, incomplete
registration, unresolved-head, foreign Store and foreign Definitions controls
remain fail-closed. Exact retained-header tests reject changed instance, source
and owned signature independently of the data-only selection cache.

The earlier development receipts retain seven reference-bearing conformance
failures from the unqualified prerequisite; the unchanged inputs now pass after
its checked capture correction. The initially failing registration test used an
unsupported whole-generic body builder; its setup now inspects the genuine proved
materialized instance. No fixture bytes or declared source outcomes changed.

## Required continuation

CTFE still needs its own actual per-execution const proof and a bounded cleanup
continuation. The consumer rejects reached actual helper execution before cold or
warm placeholder-code use. Independently accepted const positives remain required
and are not reclassified as language negatives. Full Drop/Pin/default override,
full attributed wrapper and imported source-role closure also remain required.

The declaration stays PointeeSized. Exact independent slice/dyn sources now
accept metadata, object and native execution (three/one drops); Rust performs
ordinary unsizing internally and C passes only a sized thin-pointer input.
Metadata-aware shared cleanup remains a separate required implementation: the
current internal cleanup ABI is one data pointer, while wide references carry
length or a vtable. Existing emitted vtables already carry their proved drop
entry; missing metadata or proof cannot be replaced by a null assumption.

Actual owned-box behavior is another distinct compiler contract. The pinned
[Box declaration](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/boxed.rs#L227-L237)
uses `owned_box`, and its
[Drop implementation](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/boxed.rs#L1990-L2004)
requires the pointee to be dropped **before** Box deallocation. Ordinary record
method-before-fields cleanup cannot implement that rule. Exact ownership/Unique
identity, drop checking/`may_dangle` and allocator ABI remain required; a pointer
layout or ordinary same-spelled type cannot supply them. The pinned
[Vec drop](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/vec/mod.rs#L4294-L4305)
is a concrete source driver for raw-slice cleanup and later const allocator work.
