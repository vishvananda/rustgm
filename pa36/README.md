# RustGM Programming Assignment 36 — Std and platform bootstrap

The strict self-hosting graph has built the original `std` from source.
This assignment qualifies its separate artifact and fresh consumers.
The graded suite is pending.

Build the original Rust 1.98.0 standard library for Linux x86-64 with
aborting panics. Use [PA32's core](../pa32/README.md) and
[PA33's alloc](../pa33/README.md), plus the selected dependency graph
and build inputs from [PA34](../pa34/README.md).

## Required implementation

Support the original library's platform and runtime contracts, including
initialization, thread-local storage, threads and synchronization, I/O,
errors, process entry, and symbol visibility. Keep the library algorithms
in upstream Rust. Use declared native providers through the C ABI for
host services.

The selected source can require compiler-private features, target
operations, and configuration. Implement the active subset needed by this
profile. Build on [PA32's target prerequisites](../pa32/README.md#required-target-support-for-core)
and check reached operations against the pinned backend. Unwinding is a
follow-on assignment.

## Tool surface

Build the original Rust sources and all selected target Rust dependencies
with RustGM. Link native objects through CPPGM and the declared platform
providers. Retain platform libraries, build-time inputs, and linker
configuration for replay.

## Check your implementation

Build default-feature `serde_json` and run consumers against the new
standard library. Exercise runtime initialization, thread-local state,
I/O, error propagation, and resource destruction. Repeat the build with
the same inputs and compare artifacts. Record whole-library compile time
and memory. The `alloc`-only graph in [PA35](../pa35/README.md) remains a
separate check.

After the library bootstrap, complete the [PA37 self-hosting milestone](../pa37/README.md).
