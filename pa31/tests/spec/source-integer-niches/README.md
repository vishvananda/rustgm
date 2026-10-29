# Unsigned zero niches

The finite profile accepts closed unsigned `1..=MAX` carriers for u8/u16/u32/u64/u128/usize. The authored declaration macro shell and invocations retain the exact original `new`, `new_unchecked` and `as_inner` block from pinned `core/num/niche_types.rs`; `source-provenance.json` identifies its bytes. The minimal actual Option lang-role protocol is authored. Complete original declarations/derives/generic projection graphs remain required.

`source-runtime.rs` exercises None/Some1/MAX, source checked construction, indirect C calls, layout, CTFE transmute in both directions, nested transparent wrappers and mixed/composite C signatures. `standard-reference.rs` separately checks the genuine standard NonZero guarantees. The observer is a separately compiled C object. Source/oracle profiles explicitly cap diagnostics at allow without weakening hard requirements.

`cleanup/` extends the same carrier with a non-Copy transparent payload and actual source destructor. The minimal Drop declaration reuses the established source role protocol and exact pinned regular/const-stability header; the original Pin/pin_drop fallback body graph remains required. None has no destructor trace; Some1/255 and a local Some7 do. The original first frozen attempt lacked const stability and its independent failure is retained.

Some0 is invalid and must never silently encode as None. Partial ranges are valid Rust and remain an explicit unsupported nullable profile; their ordinary physical storage is unchanged. Pointer provenance, initialization and reference validity are separate from the integer encoding. Signed/char/partial niches, metadata and full core/alloc/std artifacts remain required.
