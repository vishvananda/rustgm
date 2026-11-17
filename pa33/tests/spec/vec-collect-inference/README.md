# Vec collect inference

These four complete programs preserve the allocator, output buffer, panic handler
and exported C main prefix from the existing authored
[`alloc-consumers/vec.rs`](../alloc-consumers/vec.rs). The manifest records the
original source hash and exact preserved prefix hash; the original is unchanged.

The inferred mapped range and the explicit-u32 control both require actual Vec
contents `[1, 4, 9, 16, 25, 36]` and zero live heap blocks/bytes after run. A closed
wrong collection item and a floating range without Step remain semantic rejections.
[`vec-collect-compile-outcomes.json`](../../vec-collect-compile-outcomes.json)
declares source outcomes and native expectations before execution. Signals,
resource failures and unrelated driver errors cannot satisfy a negative case.

PA33 remains planned. These source-profile regressions do not activate its graded
manifest or qualify a whole library. Native execution uses unchanged original
core/alloc sources with the fixed library-only bring-up skip list; no app body may
be skipped. Pinned rustc is an independent oracle only.

All four source outcomes now pass on immutable RustGM `e1071bbeab4c`
(checkpoint `2b2db1cac`). Both positives compile through CPPGM and run with exact
output in an empty environment and fresh directory; both independent pinned
rustc native outputs match byte for byte. The fixed library list still has twenty
alloc stubs and no core stubs, with no application stubs or unproven bodies.
The manifest records the separate host oracle link harness and retained initial
link failure. The complete original Vec program advances to its line258 closure
lifetime-identity blocker; full alloc/std qualification remains open.
