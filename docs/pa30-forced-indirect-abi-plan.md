# Original core forced indirect ABI requirement

The current compiler is qualified through source `c6369af04` and the
[joined Cell/conversion checkpoint](pa31-cell-advisory-checkpoint.md). Its
unchanged core types attempt reaches LIB-111, the local marked repr(C) struct in
ffi/va_list.rs:416. Work is paused at the user's request. Resume with Root alone;
both former worker handoffs are already integrated into public main.

The [owning draft packet](../pa30/tests/spec/forced-indirect-abi/README.md)
freezes 28 source cases and five native predictions. Pinned Rust independently
matches 28 metadata and 27 object outcomes and four native results. The remaining
marked-return program succeeds in metadata but triggers a real pinned codegen
ICE; its required positive/native7 stays unqualified. This is a reference
limitation, not a valid expected rejection or a completed ABI implementation.

Implementation order on resume:

1. Authenticate actual struct, source owner, feature or macro envelope, word,
   duplicates and representation conflicts during the existing attribute walk.
   Retain the semantic policy in typed nominal representation. Measure common
   record sizes and prepay failed work/storage; avoid a second source scan.
2. Derive forced indirect argument mode from completed layout, including actual
   transparent payloads. Keep return classification and ordinary Rust ABI
   separate. The current ABI type cache is shared between arguments and returns;
   its authority must account for their distinct modes. Cache relevant typed
   work once, preserving linear shared-wrapper and repeated-call behavior.
3. Lower incoming and outgoing parameters through real object snapshots and
   existing LowIR pointers/copies. A pointer argument consumes one integer
   register or normal pointer stack slot; it is not SysV stack-byval MEMORY.
   Preserve actual argument order, Copy/move/drop/loans, callbacks and variadic
   classification. Qualify with the independent C observers.
4. Fence older metadata profiles before publishing a marked type or callable.
   Implement authenticated representation transport for real cross-crate
   consumers; absent wire intent cannot establish ordinary ABI authority.
5. Run exact focused source/ABI regressions with strict Clippy and audits, then
   repeat the original 722-source core attempt under its unchanged profile.
   Continue full semantic checking, metadata/object production and a real core
   consumer. Core, alloc and std artifacts remain unbuilt.

Pinned primary implementation is available in the local Rust source checkout at
revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`. The packet binds full-file
and interval hashes for codegen_attrs.rs, layout/ty.rs, x86_64.rs, check_attr.rs
and the original VaList source. The original Rust UI ABI test independently
documents C indirect arguments, normal Rust convention and transparent conflicts.
