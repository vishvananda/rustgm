# Method shape diagnostics and completed comparison caches

The unchanged PA20 invalid-unused-signature fixture declares a method returning
`T::Out` and implements it with a `u8` result. Its opaque parameter types can
match rigidly; the result cannot. The compiler now reports an implementation
signature mismatch instead of mapping that completed false answer to an invalid
lifetime identity. Malformed identities and exhausted comparison work retain
their separate errors, source charging and later lifetime proof duties.

The iterative comparator publishes a positive type pair only after its children
finish. A failed comparison discards pending work and preserves completed pairs.
The finish marker uses existing tuple padding, retaining the 12-byte work record.
This avoids both caching an unproved parent and repeatedly clearing a large
historical hash table after small failures. No caller permission is cached.

Qualification-v3 passes formatting, architecture, strict selected Clippy and two
public conformance tests, including pinned Rust acceptance/rejection checks.
The cached build includes the actual CLI, SHA
`37a0ffcc75e149a5dceafca8df349b55c5ba527f7b0dfa56bd9e7c7c8706b86e`.
Its focused steps total 241.50s; resource admission adds about 26s. The outer
combined batch later times out after successfully regenerating the one selected
oracle. That failed combined attempt is retained, not counted as a full pass.

A separate guarded follow-up reuses the exact source fingerprint and CLI. It
passes all 1,389 PA20 text cases, harness/assignment placement, and the 29 pinned
upstream fixtures and 126 Reference files in 24.13s. References come from the
scoped generator, with its original source unchanged. Adjacent evidence binds
the exact receipts, changed sources and generated partial-reference provenance.

Two private controls cover repeated false/error answers and bounded work after
1,024 completed lifetime-erased pairs followed by 128 small failures. Their
compilation/execution awaits successor CI; no measured wall-time speedup or
whole assignment/library completion is claimed. Original-core admission remains
the qualified CLI292c96f2/LIB-169 baseline until the AsyncFn execution work passes
its own required consumers and original-input observation.
