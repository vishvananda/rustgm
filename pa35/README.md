# RustGM Programming Assignment 35 — Upstream crate acceptance

This is a draft handout. The assignment's grading suite will accompany
its release.

Use the [PA34 build graph](../pa34/README.md) to compile original upstream
packages with RustGM's source-built sysroot. Build separate consumers and
execute the linked programs against the pinned CPPGM backend.

## Core-only milestone

Once [PA32](../pa32/README.md) produces `core`, build the original
`itoa` 1.0.18 and `serde_core` 1.0.229 with default features disabled.
Run consumers that check integer formatting and Serde visitor behavior.
You can complete this milestone before starting `alloc`.

## Allocation-dependent milestone

After [PA33](../pa33/README.md) produces `alloc`, build the original
`serde_json` 1.0.151 with default features disabled and `alloc` enabled.
Build every selected transitive dependency, including the locked
`serde_core`, `itoa`, `memchr`, and `zmij` versions. Preserve source
checksums, feature selections, and actual target configuration.

## Tool surface

Build the complete selected target graphs through the PA34 driver.
Use the unchanged sources and the real target configuration. Retain
dependency identities and the settings needed to reproduce each build.

## Check your implementation

Run consumer checks for parsing and serialization, integer and floating
boundaries, escaping, malformed input, nesting, and round trips. Complete
both milestones with their original dependency graphs. Record whole-build
time and memory and check the result of each executed consumer.
[PA38](../pa38/README.md) adds procedural Serialize and Deserialize derives.
