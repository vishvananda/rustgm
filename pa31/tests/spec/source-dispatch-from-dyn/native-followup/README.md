# Native reference receiver

Expected observation: `7` followed by newline, successful exit and empty stderr
when a C observer calls the exported `observe` function. Pinned rustc and RustGM
compile the same Rust source. RustGM checks the source and original instances,
then uses the locked CPPGM LowIR backend for the object.

This independently authored reducer retains the earlier reference-call source
and adds the complete Freeze declaration from pinned `core/src/marker.rs:904–906`
and its `auto_traits` feature request. The original metadata-only input failed
object generation because Freeze was missing; that observation remains retained.
The initial candidate driver-option failure also remains retained. The successful
successor uses the existing object bridge. See the
[checkpoint](../../../../../docs/source-dispatch-from-dyn-checkpoint.md)
for frozen inputs, receipts, raw observations and remaining upstream scope.
