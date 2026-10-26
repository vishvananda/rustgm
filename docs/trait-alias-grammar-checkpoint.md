# Earlier isolated trait-alias grammar checkpoint

The [source-site successor](trait-alias-sites-checkpoint.md) now adds genuine
resolver identity, whole-item authority and disabled-syntax diagnostics. The
following records the earlier grammar-only result and its original limits.

This checkpoint was on `pa31-trait-alias-grammar-v1`. It is not an implementation
of typed aliases and has not been merged into the public compiler. Active aliases
retain an explicit unsupported semantic boundary. Feature/token authority,
disabled-syntax diagnostics, actual alias definitions and typed bound expansion
remain required before the complete frontend contract can be qualified.

Hosted run40244740261 passes five exact grammar/ordinary controls, strict
production/private/integration Clippy, the global architecture audit, formatting
and all230 existing PA5 text cases without reference changes. Its helper takes
113.79 seconds; the source snapshot has2,643 fingerprints. Syntax changes reuse
the existing parser, bound/where grammar, arena traversal and transfer. The
ordinary trait handler is a cohesive helper within the existing parser module.
The alias is a real `ItemKind::TraitAlias`, preserving bounds and constness.

The qualified CLI SHA is
`daf1abd5587c1885cf7a16a3b8a6e8a0055ea471a17f05ccff1a6df238a601d8`.
Artifact12618177491 has ZIP SHA
`eb7aff7430a4ecc973e372b91ddef1a56916f72a2724300909e80b50015809ce`;
binding SHA
`55713cc8986d5cc773ec15fe5c80a9f67d2f138fbb7603d68aed316f3e90e4dc`.
No local build, formatter or compiler hydration was used. Hosted formatting
precedes the qualified snapshot; integrator import checks the input and final
source hashes. Earlier static failures remain retained: one missing enum case,
the item dispatcher exceeding its function limit, and two strict Clippy findings.
No tests ran after those failed static checks.

The additional pinned parser review and six new metadata observations in
run40243370192 correct an initial design assumption: const trait aliases are
accepted with `const_trait_impl`, including aliases over ordinary traits. Without
that feature the same syntax is rejected. The failed RustGM static run does not
invalidate these independent observations, whose sources/tool/raw outputs remain
frozen separately. They were not repeated in the passing grammar run.

Pinned rustc parses alias constness and gates the whole item span, starting before
visibility and ending after its semicolon; it does not gate only the alias name
or trait keyword. See the [pinned parser](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_parse/src/parser/item.rs#L1048).
Actual macro transfer/source identity must preserve that authority in the next
phase. No source-feature permission is inferred from this syntax checkpoint.
