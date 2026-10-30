# Source MIR-inline prohibition

PA31 validates the pinned `rustc_no_mir_inline` compiler attribute: actual rustc_attrs permission (or authenticated macro allowance), word form, one active marker per canonical function/method, and a genuine non-foreign function identity. Free functions, inherent and trait methods including declarations qualify; closure, foreign, and non-function targets do not. Invalid forms and duplicates are hard errors, independently of unused-attribute lint caps.

The current frontend has no Rust MIR inliner. Its typed executable body IR is not Rust MIR, and a marked function therefore cannot undergo that prohibited transformation. Native/backend inlining remains independent: the adjacent original `#[inline]`, and authored always/never hints, retain their actual meanings. No const, unsafe, API, layout or noalias permission is granted.

Current metadata consumers likewise have no MIR inlining pass, so this word requires no new wire field or transport denial to preserve current behavior. No API classifies an unencoded imported word as checked absence or permission. A future MIR-like inliner must first add authenticated local/imported contract support rather than infer permission from its absence.

`original-swap-chunk.rs` preserves the exact pinned core function and its preceding rationale. It uses genuine Rust core MaybeUninit only for the independent metadata/object observation, and is not a candidate claim for the complete original pointer/MaybeUninit runtime graph. `contract.json` binds the original interval and each observation. The initial extern-body syntax mistake and independent closure syntax-feature prerequisite remain in predecessor raw evidence.

The focused component passes 14 actual tests with strict private Clippy,
formatting and architecture checks in 221.94 seconds. Root's exact-source
production/CLI gate passes 35 tests in 133.70 seconds. All 29 candidate CLI
outcomes match the declared contract. The complete unchanged v7 provider prefix
with the added marked function exports through the production CLI; a fresh
genuine NoCore client emits metadata/LowIR and returns 7 natively, matching an
independent pinned Rust provider/client build and execution. Receipt:
61b2f1a066565877f9c352deb4866c127188f2494fdbb6f967adbee75568642c.

Qualified CLI 49f16ba0 clears the original marker and next stops at
convert/mod.rs801 rustc_reservation_impl. All 722 original input hashes, target
and finite limits are unchanged; 375 source opens are loading evidence, not a
compiled-file count. No original crate artifact or complete assignment is claimed.
