# RustGM Programming Assignment 33 — Allocator and alloc bootstrap

The strict self-hosting graph has built the original `alloc` from source.
This assignment qualifies its separate artifact and fresh consumers.
The graded suite is pending.

Build the original Rust 1.98.0 `alloc` crate against the
[`core` artifact from PA32](../pa32/README.md). Use the same pinned source
revision and target configuration. Compile every selected target Rust
dependency with RustGM.

## Allocator boundary

Connect the library to an explicitly selected native allocator provider.
Implement the compiler's allocator hooks and check size, alignment, allocation,
zeroed allocation, reallocation, deallocation, and allocation failure.
Preserve ownership through allocation and cleanup.

Use [PA30](../pa30/README.md) to validate the native ABI at the provider
boundary. The pinned source requires compiler-generated `__rust_*` shims
with Rust ABI, typed `Alignment` and `NonNull` arguments, and an
out-of-memory handler. Keep this interface distinct from the provider's C
functions. Container algorithms, methods, and trait implementations come
from the original library source.

## Tool surface

Produce `alloc` metadata and native objects with the same crate-build
interfaces as PA32. Link complete consumers with the built `core` and
`alloc` objects and the declared allocator provider.

## Check your implementation

Run consumers of the original `Box`, `Vec`, `String`, and collections.
Exercise growth, zero-sized elements, iteration, moved values, destruction,
and allocation failure. Complete the full library build and execute the
linked consumers. Then run the allocation-dependent graph in
[PA35](../pa35/README.md).
