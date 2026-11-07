# Constant modules of const and static annotations

Three no_core source-core libraries for the physical module of a constant in a
const or static item's own annotation. Such an item has no generic binder, but
its array-length constant is still typed in the item's lexical module, not the
crate root, so a `pub(super)` type of an enclosing private module is visible.

The controls cover the core `io/error/repr_bitpacked` shapes, an unnamed and a
public const naming that type in a length directly and through the
`static_assert!(@usize_eq: ...)` macro, and an opponent whose crate-root const
initializer names it through a public alias (rejected without an error code).
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json` with the fixture amendments made before the re-runs. rustc
does not check type privacy in anonymous constants of item signatures, so the
v1 opponent using the alias in a crate-root length was retired; see the second
amendment.

The lang-item prefix is the exact pinned Rust core fragment from the census
batch 2 controls, with its attributes and comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/static_constant_modules.rs` checks every case. See
`docs/parallel/census-batch4.md`.
