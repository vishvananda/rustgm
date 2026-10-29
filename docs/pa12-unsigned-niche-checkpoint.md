# Unsigned zero-niche ABI checkpoint

The candidate retains physical carrier identity in the existing paid cold nullable
cache: non-null pointer and exact unsigned zero-excluded range are distinct answers.
A valid wider scalar range remains distinct from proven ineligibility; ordinary
partial-range/char objects keep their representation while a nullable request
reports its wider profile. No Store/range-order semantics or wire profile changes.

The finite unsigned profile requires the complete `1..=MAX` range for u8/u16/u32/
u64/u128/usize, actual transparent storage and a source-owned nullable policy.
None stores zero and skips inactive refinement; Some retains and validates its
real payload. Integer carrier bits never become pointer provenance. Scalar ABI
type is retained once in naturally aligned `ScalarStorage`, preserving the existing
representation cache width and memory-span APIs. Runtime calls, C argument/result
and composite classification, shims, variant tests/construction, cleanup, data
materialization and CTFE encoding/decoding consume that same typed encoding.

Fixtures and predictions are in
[the source cluster](../pa31/tests/spec/source-integer-niches/README.md). The selected
original constructor/accessor block is unchanged. Minimal Option/Drop protocols
and closed range invocations are explicitly authored; standard NonZero evidence
is separate. Source Rust metadata/object/C-link/native and standard Rust staticlib/
C-link/native passed before candidate execution. The initial standard plain-object
link lacked real core panic runtime; the separately retained staticlib successor
supplies the real runtime. A cleanup successor adds the actual const-stability
header after the preserved first prediction failure.

Candidate private/source/native qualification is pending. Cold/hot widths and work
observations will be bound to the final source receipt. Cold build timeout after
strict checks is retained as a failure with zero runtime tests, never a pass.

Original open range helpers, full NonZero/ZeroablePrimitive/Option/Drop source
contexts, signed/char/partial niches, metadata and full core/alloc/std artifacts
remain required. This checkpoint does not close an assignment or original library
artifact goal.
