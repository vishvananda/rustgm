These unchanged independently declared inputs exercise the actual pinned
`#[lang="drop_glue"]` declaration and source Unsize/CoerceUnsized roles.
`original-zst-slice3.rs` is the preserved original Rust/native-positive program,
SHA c04f576414232867b062ae3587805059010b1434315d7f84bcf86b60be212eb9.
The next14 inputs were declared before pinned Rust metadata/object/native
qualification; three later sized-array regressions have their own independently
settled declaration. Exact outcomes and C observers are in contract.json.

Declaration/header cutouts derive from Rust1.98.0 commit
88d9e12ae178fab0fb5cc050a94da85685d449ea:
`library/core/src/ptr/mod.rs`822–829,
`library/core/src/marker.rs` and `library/core/src/ops/unsize.rs`.
Rust is Copyright The Rust Project Developers, licensed MIT OR Apache-2.0.
The local observations/borrow controls are maintainer-authored. All native C
boundaries use thin pointers to valid live sized storage; Rust creates every
wide reference. They do not claim portable C fat-pointer ABI, general concurrency,
const cleanup, full original Drop/Pin/default bodies, dyn or owned-Box cleanup.
The original dyn positive remains required in the retained independent study.

Grouped `glue_provider::unsized_elements` registers every input, with the native
programs explicitly ignored like other backend maintainer tests. PA31 is not
activated and there are no text oracles to regenerate here.
