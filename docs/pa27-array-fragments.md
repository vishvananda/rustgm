# Initialized byte-array fragment snapshots

This development checkpoint follows pointer-add `c84569ee`. It closes the original
unchanged `pa27/tests/spec/byte-access/offset-word-roundtrip.rs` requirement: a
pointer into a physical subobject can pass through an initialized whole byte
array, return through a const function, reenter a union and retain its original
address and access restrictions. It does not activate PA27 or PA28.

## Representation and validity

The existing immutable Image payload now also admits exact `[u8; N]` values.
Retained type identity, byte extent and layout must agree. The producer requires
every selected byte initialized, then preserves the initialized mask and ordered
symbolic fragments. No integer bits are fabricated for pointer, function or
vtable words. The existing complete pointer targets retain their metadata and
write restrictions. Value, Constant, Pointer, Origin and View layouts do not
change; no backend vocabulary changes.

Whole-value transport and const-function returns may retain partial words.
Reencoding the array into physical backing copies its bytes, mask and fragments
together. Scalar element selection checks initialization and rejects any selected
provenance byte. It uses the sorted immutable fragment index and charges bounded
logarithmic lookup work. Numeric bytes beside symbolic words remain readable.
Union structural field reads remain forbidden through Reader's ordinary field
interface; their typed union capability still selects the physical storage.

Final array publication rejects **all** remaining provenance, including complete
words. Union publication retains its existing complete-word rule. Reference
validation decodes the selected referent's scalar values, including a frozen
array previously observed as a permissive whole-value snapshot. This storage
check does not create or discharge source lifetime or unsafe obligations.

Union snapshots retain their 64 KiB limit. Ordinary byte-array snapshots are not
subject to that union-specific limit; the existing 64 MiB Image arena and machine
work/storage budgets still bound retained and temporary buffers. Capacity is
charged before cloning bytes, masks and fragments. Ordinary typed products,
repeats and numeric byte arrays keep their existing fast paths.

## Consumer audit

- Reader and Evaluator fields expose only initialized numeric array elements.
  The Machine wrapper charges indexed selection; source contents selection also
  checks the exact physical owner and child type.
- The physical codec snapshots fragment-bearing arrays without scalar decoding;
  ordinary reference validation retains scalar decoding. Encoding uses one
  selected byte/mask copy plus the existing indexed fragment transport.
- Result validation distinguishes array publication from complete union words.
  Promotion/interior-mutation observation consumes the array's exact type and
  length without treating pointer bits as owned reference fields.
- Native initialization, zero observation, instance dependency discovery and
  constant-view graph rendering reject invalid final array publication before
  emitting or observing its bytes. The existing union Image text is unchanged.
- Source equality is not implemented by comparing Image arena IDs. Scalar
  demand must use the checked field/byte decoder. PA29 Image export remains an
  explicit unsupported metadata profile; this checkpoint adds no wire encoding.
- Fragment-bearing owned rest/range snapshots remain required work. Existing
  product/byte-range fast paths remain available; this checkpoint does not
  silently drop fragments or materialize them as scalar integers for a range.

## Independent source contract

Twenty-nine frozen inputs were declared before candidate checking. Their original
hashes, pinned Rust object outcomes, baseline three-view failures and valid native
observations remain in `obj/parallel/pa27/array-fragments/{first,followups,producers}`.
The draft companion contracts are `pa27/tests/array-fragment-outcomes.json` and
`pa28/tests/array-fragment-outcomes.json`. Fixtures using ptr::read or reified
function calls belong to PA28; ordinary unsafe union/array cases belong to PA27.
The original offset-word fixture remains unchanged in its earlier cluster.

The initialized positives cover thin/function/slice words, same-value fragments
from distinct cells, partial-word array transfer, const-function return, direct
union initializer and a selected ordinary byte beside a pointer. Negatives cover
final byte-array provenance, scalar pointer-byte reads, mixed pointer fragments,
numeric overwrites, demanded expired pointers, incomplete final union words and
published references to invalid byte-array storage.

Two transient uninitialized-array programs are accepted by pinned rustc while
this compiler rejects their typed array reads. The initial hypothesis of Rust
rejection was disproved and retained. They are oracle differences, **not** native
validity controls or permission to transport invalid typed values. The pinned
Reference at commit `86635e30bf861a038dc197d7e16fd09e7e514e7a`,
`doc/rust-reference/src/behavior-considered-undefined.md`, lines 52–53, 129–139
and 163–166 distinguishes producing invalid values, initialized scalar/array
elements and const integer/provenance restrictions. Unused invalid reference
formation remains a separate eager-validity obligation; its acceptance is not
used to weaken demanded reference checks.

## Focused verification

The candidate replay and permanent source harness completed all 29 cases in all
three views: 17 accept and 12 reject, including the two documented oracle-only
uninitialized-array differences. Fourteen exact literal native programs match
pinned Rust execution. Sixteen byte-storage invariants and the affected image
and union-address source groups pass. Strict all-target/all-feature Clippy,
file audit (1696 files, zero findings), layout, upstream and suite mapping pass.
Three private tests cover exact type/extent/initialization, unchanged union limits,
ordinary arrays larger than 64 KiB, restriction-preserving reencoding, frozen
reference validation and selection bounds. For 8/32/128 fragments, measured
snapshot work is 282/1056/4152; charged selected-byte lookup work is 5/7/9 per
read, independent of repeated count 8/32/128. These are logical work invariants,
not wall-time or RSS neutrality claims. Focused receipts are under
`obj/parallel/pa27/array-fragments/closing-first`;
private work observations also appear in the preceding guarded run. Active-text
comparison and exact final hashes are recorded in the handoff. Full release
validation stays central.

The unchanged original byte-access cluster also passes. All 8,360 active
PA1–24 outputs exactly match immutable published `0b995f43`; the 51 locally stale
sidecars are the same reviewed baseline differences, with zero unexpected
changes. No references were regenerated. The final production fingerprint is
`59ca77bfb105c32ba30e7560a8e2fad4c64565127e61f67ffa063852fa1d654e`;
the pinned candidate SHA-256 is
`b0f43ca9d0d731c8be6b2d2aced29767ceeab086a3c7cf3bafb4bc50cc7e9bd1`.
The exact text replay receipt is `obj/parallel/pa27/array-fragments/text-first`.

The subsequent owned-range checkpoint supersedes the fragment-bearing U8
array-rest/range value boundary above. It preserves the original byte-array
fixtures and scalar/reference/publication validity gates; see
`pa27-fragment-ranges.md`. General raw/unsized ranges remain required work.
