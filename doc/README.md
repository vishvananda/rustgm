# Pinned language reference

[The Rust Reference contents](rust-reference/src/SUMMARY.md) are available
offline as searchable Markdown, just like the language reference retained in
CPPGM. Start with [tokens](rust-reference/src/tokens.md) and
[literal expressions](rust-reference/src/expressions/literal-expr.md) for PA2.

This is the Reference source selected by **Rust 1.98.0**, not a download of
the moving stable website. Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea` pins its `src/doc/reference`
submodule to Reference commit `86635e30bf861a038dc197d7e16fd09e7e514e7a`.
The unchanged `src/` tree, upstream README, book configuration and both
licenses are retained. `rust-reference/upstream.json` records every checksum.

Run `make test-upstream` to verify this copy without network access. The
Markdown includes upstream mdBook grammar/rule markup; it is readable and
searchable without building mdBook. Build tooling and rendered HTML are not
vendored. The [upstream tree](https://github.com/rust-lang/reference/tree/86635e30bf861a038dc197d7e16fd09e7e514e7a)
contains that tooling if a rendered edition is needed later.

The Reference is MIT OR Apache-2.0, with original notices in its retained
[MIT license](rust-reference/LICENSE-MIT) and
[Apache license](rust-reference/LICENSE-APACHE). RustGM's profile and assignment
contracts determine what is implemented; storing the full Reference is not a
claim of full language support.
