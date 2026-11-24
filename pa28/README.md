# RustGM Programming Assignment 28 — Intrinsics and atomics

Extend the typed places and safety checks from [PA27](../pa27/README.md)
with memory intrinsics, volatile access, and `AtomicU32`. Use Rust 1.98.0,
edition 2024, on Linux x86-64 with checked arithmetic and aborting panics.
The [PA28 manifest](tests/manifest.json) defines the memory and atomic
kernels. PA44 carries the broader intrinsic families and symbolic transport
requirements.

## Memory operations

Select the canonical `core::ptr` and `std::ptr` operations by declaration
identity. Keep their safety and constant eligibility separate. Direct and
reified calls must select the same typed operation.

- Implement `read` and `write` at their selected `Sized` types, including
  owned non-Copy values. A read takes an owned snapshot without marking the
  source place moved. A write consumes the incoming value and does not
  destroy the overwritten value. The caller must observe the operation's
  ownership preconditions.
- Implement `read_unaligned` and `write_unaligned` with the same transfer
  semantics. Relax alignment for that one operation. A later dereference
  or reference formation checks its own alignment.
- Implement `copy_nonoverlapping`, overlap-safe `copy`, and `write_bytes`
  as counted transfers. Evaluate their operands once, preserve source
  bytes when the regions overlap in `copy`, and leave overwritten storage
  undropped. A transfer of zero bytes performs no load or store. Preserve
  the address and alignment rules for the selected operation.
- For supported constant transfers, use symbolic storage. Check the selected
  range, lifetime, write permission, and const context. Preserve initialized
  byte masks and symbolic provenance on raw transfers. A typed load checks
  that the selected bytes are initialized and form a valid value.
- Support the graded scalar `read_volatile` and `write_volatile` types,
  including thin pointer values. Keep ignored reads observable. Volatile
  calls require unsafe permission and are runtime-only, even in an
  unevaluated branch of a const body. Reifying a callable and calling it
  during constant evaluation are separate checks.

For a source-declared `transmute`, diagnose raw-pointer to integer
transmutes in const functions and associated constants according to the
scoped `ptr_to_integer_transmute_in_consts` level. Keep the transmute's
safety, size and constant-evaluation checks in force.

## AtomicU32

Resolve the canonical `AtomicU32` and atomic `Ordering` identities. The
atomic has its private `u32` storage and a const `new` function. It is
neither Copy nor Clone. Shared references can access the atomic while
ordinary storage and promotion rules still apply. A match on the external
non-exhaustive `Ordering` needs a catch-all arm.

Translate Rust's ordering values explicitly to the backend operation's order
encoding.

| Operation | Allowed orderings |
| --- | --- |
| `load` | Relaxed, Acquire, SeqCst |
| `store` | Relaxed, Release, SeqCst |
| `swap` and all graded `fetch_*` methods | Relaxed, Acquire, Release, AcqRel, SeqCst |
| Compare-exchange success | Relaxed, Acquire, Release, AcqRel, SeqCst |
| Compare-exchange failure | Relaxed, Acquire, SeqCst |

Apply the scoped `invalid_atomic_ordering` lint to direct method calls
using disallowed order variants. An invalid order that reaches execution
aborts. All atomic memory operations in this assignment are runtime-only.
Keep normal receiver alignment and argument-evaluation rules.

Return the previous value from exchange and fetch operations. When the backend
returns the new value for add-and-fetch, recover the old value using wrapping
`u32` arithmetic on the captured result. Use the compare-exchange operation's
observed value to construct the selected `Result<u32, u32>` variant. These
operations need no extra read of the atomic. The weak form may use the strong
operation for this profile.

For bitwise and min/max fetch operations, keep the expected value in the
compare-exchange scratch. A failed attempt updates it and retries without
reevaluating source operands. A successful attempt returns the observed
old value. Even unchanged desired bits require the atomic operation. Min
and max compare unsigned `u32` values. The
[compare-exchange example](tests/spec/atomic-cas-native/strong-matrix.rs)
exercises the legal success and failure orders.

## Run the tests

Use PA11's `rustgm --emit-lowir` interface. Run the PA28 test target from
the repository root.

```sh
make -C pa28 test
```

Use [the PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
for executable cases. The manifest selects the expected diagnostics and
LowIR, including input preludes for memory observers.
