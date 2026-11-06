# Typed CStr constant backing: compiled prefix

This prefix joins Root's actual CStr producer/profile API with one shared literal
value kernel in source constants, promotion and the existing constant machine.
The logical type remains the genuine CStr reference. Its actual immutable sized
`[c_char; N + 1]` backing retains `Length(N + 1)` and `SharedDerived`; no unsized
allocation, nominal-as-array value, new value variant or second interpreter is
introduced. Plain string and byte-array value paths retain their existing forms.

The existing literal cache now keys actual text and sized carrier type. A byte
literal and C literal with identical bytes cannot reuse a location with the wrong
stored type. Physical signed-byte storage uses the existing raw-bit codec and
lowering view. The actual source role/nominal/field/transparent representation is
authenticated by Root's shared profile query, including ordinary shared mutation.

The existing ABI metadata and unsized-field queries gain read-only completed
accessors. They retain errors, distinguish misses, allocate nothing and add no
fields. The requesting literal evaluator demands their genuine producers and
charges actual metadata/tail/layout work, including failures. Tail projection
authenticates the real owner, ordinal, substituted field and cached zero offset;
only the checked sole transparent byte tail may reuse the original sized backing
pointer. Ordinary tail projections retain their existing physical-field path.

The real to_bytes_with_nul body casts `[i8]` to `[u8]`. The existing raw word,
sequence and byte codec admits that finite one-byte bit reinterpretation while
retaining source cast/unsafe/loan, metadata, bounds, alignment and write checks.
Ordinary public subsequence formation still requires the same element type.

Qualification evidence is under
`obj/parallel/source-transport/cstr-backing/static1` through `static3`.
Static1 stopped at a repeated filename stem; Static2 stopped at ordinary import,
error-adapter and Root Option-comparison compile errors. Static3 passed format,
architecture and strict library Clippy (97.396 seconds). Its public candidate
built and ran: interning and original source intervals pass; the other three tests
fail at real source-method scalar lowering and a narrower mutability diagnostic
assertion. Native execution is ignored and was not run. The script stops there,
so legacy constant/tail controls remain pending. Root owns source-method/lowering
integration; the original failures remain. New private additions were not executed.

Actual compiler layout output measures References 160 bytes, Place 64, Pointer
24, LiteralProfile 16 and transient TailStorage 8. The existing References field
layout is unchanged; its literal map's typed key is the cold carrier distinction.

The actual encoded fat-reference roundtrip is a known remaining consumer gap:
the current word reconstruction path must preserve the genuine CStr logical view
while validating its real sized array through the existing slice validator.
That independently frozen positive remains required; this prefix does not claim
general symbolic CStr roundtrip, all CTFE/reference profiles or PA completion.
Nonzero/subextent and wider nominal DST storage profiles remain explicit duties.
Root owns declaration/literal typing, body/lowering and source-method integration.
The follow-up constant-lane consumer fix retains the same profile/ABI/backing
authority and does not alter any source expectations.
