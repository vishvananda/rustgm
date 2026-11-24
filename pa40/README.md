# RustGM Programming Assignment 40 — Editions and attributes

The graded suite for this assignment is pending.

Complete Rust 2015, 2018, 2021, and 2024 behavior beyond the subset used
by [PA34](../pa34/README.md)'s selected crate graph. Extend the typed
attribute checks and diagnostic policies from [PA31](../pa31/README.md).
Features already required by an earlier library build belong to that
build's prerequisites.

## Edition behavior

Retain the edition of the source and each relevant macro token.
Apply edition-dependent rules to parsing, paths and preludes, fragment
matching, captures, and temporary destruction scopes. Check mixed-edition
macro and crate boundaries using the edition at the deciding location.

## Attributes and diagnostics

Validate the placement, form, owner, and required permissions of retained
attributes. Separate documentation and advisory metadata from attributes
that change semantic eligibility. For example, a primitive documentation
label describes an item and leaves the type namespace and body checks
unchanged.

Carry scoped lint levels and expectations through expansion, imports,
and checked bodies. Associate deprecation and visibility events with
their actual source or import. Report unmet expectations after their
scope has completed. A lint cap can suppress optional diagnostics.
Hard errors and semantic checks still apply.

## Tool surface

Use the compiler's edition option and the crate graph's per-crate
configuration. Use the existing typed inspection and LowIR modes to
compare edition-dependent results. The pinned Rust Reference describes
[attributes](../doc/rust-reference/src/attributes.md).

## Check your implementation

Use paired programs that differ only in edition or attribute placement.
Cover capture, drop scope, macro fragments, inherited lint levels,
and expected diagnostics. Check cases with disabled syntax, imported
bindings, and actual macro allowances. Run executable examples when the
edition changes evaluation or destruction behavior.
