# Source MIR-inline prohibition

PA31 validates the pinned `rustc_no_mir_inline` compiler attribute: actual rustc_attrs permission (or authenticated macro allowance), word form, one active marker per canonical function/method, and a genuine non-foreign function identity. Free functions, inherent and trait methods including declarations qualify; closure, foreign, and non-function targets do not. Invalid forms and duplicates are hard errors, independently of unused-attribute lint caps.

The current frontend has no Rust MIR inliner. Its typed executable body IR is not Rust MIR, and a marked function therefore cannot undergo that prohibited transformation. Native/backend inlining remains independent: the adjacent original `#[inline]`, and authored always/never hints, retain their actual meanings. No const, unsafe, API, layout or noalias permission is granted.

Current metadata consumers likewise have no MIR inlining pass, so this word requires no new wire field or transport denial to preserve current behavior. No API classifies an unencoded imported word as checked absence or permission. A future MIR-like inliner must first add authenticated local/imported contract support rather than infer permission from its absence.

`original-swap-chunk.rs` preserves the exact pinned core function and its preceding rationale. It uses genuine Rust core MaybeUninit only for the independent metadata/object observation, and is not a candidate claim for the complete original pointer/MaybeUninit runtime graph. `contract.json` binds the original interval and each observation. The initial extern-body syntax mistake and independent closure syntax-feature prerequisite remain in predecessor raw evidence.
