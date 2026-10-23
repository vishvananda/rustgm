# PA34 external declaration and linkage editions

The typed frontend accepts an implicit `extern` block when its parsed
introducer originates in Rust 2021. Rust 2024 requires `unsafe extern`.
Both editions require an explicitly unsafe block before a member may use an
explicit `safe` or `unsafe` qualifier. Default members remain unsafe to call or
access; accepting older syntax does not grant permission to use them.

The existing `no_mangle` and `export_name` attributes accept their bare spelling
in Rust 2021 and require `unsafe(...)` in Rust 2024. The attribute name token
owns this edition, independently of the item and surrounding attribute wrapper.
`link_name` still rejects an unsafe wrapper. This checkpoint does not implement
`link_section`, expose a Rust 2021 CLI profile, or activate PA34.

Only edition-sensitive syntax demands provenance. The implementation follows
the existing bounded token-origin ancestry and consumes parsed member flags;
it adds no token, AST or semantic record fields, source rescans or new caches.
An explicit unsafe wrapper needs no edition lookup. A demanded missing name
origin fails instead of falling back to the package's edition. Existing ABI,
linkage, call permission and static-access checks remain in the same pipeline.

The pinned Reference's [external blocks](../doc/rust-reference/src/items/external-blocks.md)
and [ABI attributes](../doc/rust-reference/src/abi.md) provide the language rules.
The pinned rustc attribute safety implementation additionally selects the
attribute path span, because a macro can supply a wrapper from another edition.
Private controls construct actual expanded attributes with independently owned
name origins and cover all four name/wrapper edition combinations.

## Focused qualification

`pa34/tests/foreign-edition-outcomes.json` declared 32 paired source outcomes
over 16 original fixtures before execution. Pinned Rust 1.98.0 agrees with all
32 outcomes. All 13 accepted programs execute successfully, and the RustGM
typed frontend plus CPPGM backend independently reproduces their behavior.
Controls cover member qualifiers, ordinary unsafe-call rejection, static imports,
bare and wrapped exports, macro-generated declarations and opaque forwarding.

Receipts remain under `obj/parallel/pa34-foreign-editions/` in the root
implementation checkout. `independent-v1/complete.json` binds the plan hash
`a0f86184484bf522cd9807329545037be49e270391474771a77792c05eea9b1a`.
`development-v3` passed 17 focused tests and six native tests, including the
32 source and 13 native cases above, prior pattern/package edition cases,
foreign linking and mutable static controls. Formatting, strict all-target
Clippy, architecture, harness and upstream checks pass. The strict result is
retained across an exact test-file/module rename; the audit and both focused
lanes ran after that rename. Earlier helper error-conversion and audit filename
failures remain recorded rather than being counted as successful checks.

The debug batch compiled once in 143.98 seconds. Its native continuation reused
the same artifacts and completed in 7.93 seconds, including inventory and
runner overhead. These shared-machine development costs are not performance
measurements of the compiler. Scoped cleanup removed generated artifacts while
retaining original inputs, observations and binary hashes.

Full combined checkpoint and performance qualification remain required before
publication. Cross-crate macro transport and the remaining edition behavior
needed by the actual library graph still require completion.
