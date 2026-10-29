# Source Debug field-array checkpoint

LIB-073 extends actual source Debug expansion beyond the pinned five-field
cutoff. One emitter handles structs and enum variants, preserving the existing
small helpers. Larger records use a static field-name array; larger record and
tuple shapes use a typed `&[&dyn crate::fmt::Debug]` values array and the actual
`debug_struct_fields_finish` / `debug_tuple_fields_finish` calls. The final field
keeps an extra reference for possibly-unsized values. No original formatter body
is replaced and no new ordinary syntax/body record fields are added.

The pinned compiler recipe is `rustc_builtin_macros/src/deriving/debug.rs` at
Rust88d9, retained with exact source/helper intervals under
`pa31/tests/spec/debug-field-arrays`. Independent Rust evidence covers5/6/7
record/tuple/enum recipes, DST, raw/Unicode labels and missing Debug. All27
command statuses and six ordinary native lines match. An observer initially
compared Unicode escapes as literal bytes; the failed raw receipt is retained,
and a read-only corrected review verifies those same immutable outputs.

The source candidate passes8 exact expansion/receipt/regression controls in
122.19s: `obj/qualification/fast/1793247830230542366/receipt.json`, SHA-256
`ebe2c0e2fabf54c72253db9d15b70e339adfc02cdbef70f843ae1c173eb25aa0`.
Normal production and CLI cap controls pass2 cases in155.60s at
`1793248037640598690`, SHA-256
`1db0f631c9daf5c66ab7f7c12f169faabff17c87890479e212838a1f76537683`.
Both include their strict Clippy targets, full formatting and architecture
checks. Work/storage controls subtract fixed foundation costs before comparing
8/32/128-field growth; warm receipt reads retain no additional bytes.

That pre-UnsafeUnpin-join CLI `b7e814a0` preserves all722 original source hashes
and effective flags. Explicit Allow passes original FnPtr Debug7 and stops at
`mem/type_info.rs363`'s actual Abi enum Default derive, file61:10239. Allow
receipt SHA-256 `19dfd69576317e791e2e28ad4f66da3e1facceec73a7d35e6a0059a71293544b`.
Strict still opens68 at the deferred pointer-cast lint expectation; its receipt
SHA-256 is `901fdc79a7b10593be6a040de474847c151117b518933ee8617aef26c2b8b23e`.
These pre-join observations do not qualify a later compiler revision. Loading
counts never count as compiled files or original artifacts.

A separate explicitly authored NoCore observer exercises dynamic Debug calls,
array-to-slice coercions, all seven values/labels and both enum shapes. Pinned
Rust metadata/object/C link/native pass all4, receipt SHA-256
`8b37d41ee6da1cb2b08329a8ebc050bbafa14622563b9efa760cf6f96a809b40`.
The observer's unit Result, Formatter and seven-slot tracing bodies are test
code, not original core helpers. Earlier fixture prerequisite failures are
preserved. Original helper/Formatter/Result/Index execution remains required.

The joined source implementation now passes the complete Debug consumer. The
actual captured string name exposed two real intrinsic boundaries: FunctionItem
signature roots retain their binder, while opened Body Call roots already live
in the caller. The [borrowed-call correction](pa31-borrowed-source-calls-checkpoint.md)
uses exact readonly canonical lowering and caller-root signature validation for
transmute and wrapping offsets, preserving all current source/callee/unsafe/const
checks. No recipe, source signature or lifetime proof was weakened.

Joined45 receipt1793257363048411345 passes149.22s, SHA-256
`2b80d5dc4ccdbbb030cfd04356f03e21a6753b8381f01dbb083938049733e5ed`.
Seven actual CPPGM source-native programs pass along with upstream, registration
and layout checks; receipt SHA-256
`e7e17d565c057982e5bb0ea10a58871c56d3b60e88c42303fad8ed99d55c53ae`.
The Debug record/tuple/named/positional observer preserves all seven labels and
values and matches the independent v6 Rust trace exactly. The other programs
retain Default/Self construction, associated equality and signed wrapping.

The newer associated-equality CLI1b626052 passes original Abi/DecodeUtf16, then
reaches a measured source macro attempted-storage overcharge at FromCoroutine.
All722 original hashes/effective flags remain unchanged;373 opens are loading,
not artifacts. The [capacity repair](pa31-derive-image-storage-checkpoint.md)
keeps every limit unchanged. Final joined48 passes148.85s (receipt1793257759839985457,
SHA-256 df7441f64f18b258136b0771a3d9e6d0fae0d457a04995a4ce243ca5fd24af9c),
and production/CLI2 passes109.78s (1793257949980303029,
SHA-256 1d32b71a30ddcf22cd3233f58873f0bc088f90f94665a4905cb9923b7183c673).
All7 native inputs remain byte-identical; equality receipt SHA-256
23cb88d09d859c515bb57180827030917a1f050ea147d9aa8756269726ff7eef
reuses the earlier native evidence without another native run. Current original
CLI8ee48985 reports a remaining work/storage boundary at StepBy; exact counter
work continues with unchanged sources, flags and limits.

The earlier deadline, partial-role/intrinsic and exact signature failures remain
retained evidence; they were never counted as native or crate successes. Full
original Formatter/Result/builders/Index/helper execution, source transport,
whole parent assignments and original core/alloc/std artifacts remain required.
