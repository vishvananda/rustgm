# Actual stdarch array-field provenance

Pinned revision: `88d9e12ae178fab0fb5cc050a94da85685d449ea`.
`original-types-macro.rs` preserves `library/stdarch/crates/core_arch/src/macros.rs`
lines 67–164, and `original-m128i.rs` preserves the actual two-i64 declaration in
`src/x86/mod.rs`. `original-debug-helper.rs` preserves the real helper from `simd.rs`.
Their exact byte hashes are in the frozen contract; upstream licenses and attribution
remain those of the source repository.

`macro-input.rs` invokes the actual macro's final arm directly, with an authored
stable-attribute normalization. The typed observer inspects its actual transcribed
array and stored literal; it does not infer generated coordinates from the physical
original file offset. `reference-rust.rs` runs the full actual macro/helper with real
pinned Rust core providers, including splat, Debug and conversion dependencies.

`source-native.rs` is a distinct finite candidate: the genuine selected expanded
`repr(simd)` header, array field and real source Clone/Copy are checked through LowIR
and a separate C vector object. All 128 bits must survive the by-value call/return.
The missing-feature source is a hard gate opponent. This finite consumer does not
claim the full original macro's splat/Debug/conversion source graph or a new ABI/backend
contract. The original complete source graph remains required.
