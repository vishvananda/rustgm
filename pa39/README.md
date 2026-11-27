# RustGM Programming Assignment 39 — Derive and attribute macros

The graded suite for this assignment is pending.

Extend the [PA38 host interface](../pa38/README.md) to expand procedural
derives and attribute macros. Use ordinary typed expansion and name
resolution for their output.

## Required implementation

Apply attribute macros in source order, replacing the attributed item
with the selected macro's output. Register permitted derive helper
attributes and append the items emitted by derives. Preserve nested
expansion order, source provenance, delimiter-none groups, and punctuation
jointness across host token conversions.

Build the original `serde_derive` host crate and its locked dependencies.
Compile a target application that uses its real `Serialize` and
`Deserialize` expansions. Use the host/target separation from
[PA34](../pa34/README.md) and the runtime environment from
[PA36](../pa36/README.md).

## Tool surface

Use the expansion inspection mode for token and item replacement checks.
Use the crate build and LowIR pipeline to build, link, and run the
complete application.

## Check your implementation

Exercise helper attribute lookup, order-sensitive replacements, nested
derives, joint punctuation, and groups without delimiters. Run native
serialization and deserialization round trips with the original macro
output and the selected Serde dependency graph.
