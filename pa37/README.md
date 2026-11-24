# RustGM Programming Assignment 37 — Procedural macro host

The graded suite for this assignment is pending.

Extend [PA34](../pa34/README.md)'s host and target build split to run
procedural macros. Build against the [standard-library profile](../pa36/README.md)
and use the pinned host toolchain for host macro crates. RustGM compiles
the target crates.

## Required implementation

Identify the macro's defining crate and exported entry. Run the macro
through the host boundary and transfer typed token streams with delimiter,
spelling, span, and hygiene facts. Preserve the input's source origins and
associate generated tokens with their expansion. Feed returned tokens into
the ordinary expansion pipeline.

Isolate host macro execution from frontend state. Convert host load failures,
failed execution, or invalid output into diagnostics with the invocation
location. Preserve the selected host, target, and dependency identities
in the build graph.

## Tool surface

Use the crate-build interface to build the host macro and pass the selected
artifact to the target compilation. Use the expansion view to inspect the
invocation and its output. The pinned Rust Reference describes
[procedural macros](../doc/rust-reference/src/procedural-macros.md).

## Check your implementation

Test token and span round trips with a small macro crate. Check nested
groups, retained source locations, and failure diagnostics. Build and
run a target program using the generated syntax. [PA38](../pa38/README.md)
adds derive and attribute expansion on top of this host boundary.
