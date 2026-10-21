# Exact-type raw constant stores

This PA27 development checkpoint extends the existing symbolic constant machine
with raw stores to live local storage. The investigation baseline `a4f064ff`
explicitly rejected reached raw stores before the ordinary write path. That was
an unsupported capability, not a demonstrated CTFE validity hole. The original
19-case and eight-case hypothesis sets and their original diagnostics remain
separate from the supported/boundary contract.

`WritePermission::{Writable, SharedDerived}` belongs to a pointer value beside
its location and metadata. Owned allocation construction creates writable
provenance; shared borrows and promotions restrict it monotonically. Projection
takes the complete pointer, and casts, products, copies, calls and returns retain
its permission. A final `*mut` type cannot restore permission. Shared reborrowing
does not mutate the allocation or another existing writable pointer. Static
storage is forbidden independently of this flag. Reader-only pointer
reconstruction from a bare location conservatively grants no write permission.

A reached store checks symbolic lifetime/provenance, the exact stored type,
retained typed physical footprints, bounds and alignment at the actual raw
access anchor. It then uses the existing checked mutable cells. It does not
read the replaced value first. A private invariant starts with an empty cell,
validates the store, writes it, then reads the initialized result. Source E0381
initialization checks remain separate. Packed aggregate pointer field stores
can succeed while a separately formed unaligned field pointer store fails.

`SharedDerived` is not a complete UnsafeCell model. An ordinary shared-derived
store rejects; a conservative typed interior-mutable or unfinished root reports
an explicit unsupported/readiness boundary. The independently accepted
AtomicU32 shared-derived example belongs in PA28. Raw union-field stores,
reference construction through raw places and raw type/byte reinterpretation
remain unsupported. In particular, the invalid-bool byte-store example is still
rejected at reinterpretation; this does not establish bool validity for that
unsupported write family. Existing scalar union Image reads retain their real
bool/char/initialization checks.

The separately qualified default runtime `invalid_reference_casting` lint gap
remains: pinned rustc rejects its default-lint fixture while the frontend accepts.
The explicitly allowed runtime control is compiled but never executed. That
runtime/frontend obligation is not a CTFE permission hole. The original
shared-before-writable hypothesis predicted acceptance; both compilers instead
reject its live shared-loan conflict. The original hypothesis is retained.

The planned constant view adds exactly `write=writable` or
`write=shared-derived` to pointer value rows. Allocation rows, location/origin
identities, scalar-only metadata export and native pointer serialization stay
unchanged. Permission participates in pointer/value equality and hashing for
memoization, so differently restricted values at one location cannot collapse.
Language pointer comparison remains outside this constant profile.

Storage observations before and after agree: Pointer 24 bytes, Constant/Value
32 bytes, Location/Place 64 bytes on the pinned x86-64 host. The new enum fits
existing pointer storage, so execution cell/slot and interning storage bounds do
not grow. Full-pointer projection adds no extra arena entry. The existing bounded
interior query is used only on a rejected shared-derived store, and its work is
charged before the store. Existing layout caches and machine path work limits
remain authoritative.

Qualification uses the predeclared 34-case contract, all three public views,
three independent native groups, private storage/provenance/hash invariants and
8/32/128-iteration raw/ordinary controls. Raw work is 526/1846/7126 steps with
36 retained slots; ordinary work is 343/1159/4423 with 30 slots. This establishes
linear changed-path work and fixed retained execution storage, not wall-time
neutrality. The integrator owns combined release/performance qualification.

Initial checks caught one private projection consumer still passing a location
ID and one oversized expanded invariant; both failures are retained in evidence.
Strict all-target/all-feature Clippy and the 1,355-file audit subsequently pass.
The focused raw-read, scalar-union and constants-view regressions also pass.
No assignment activation, active reference regeneration or CPPGM/backend change
is part of this checkpoint. Exact final source hashes and gate outcomes are in
`pa27/tests/raw-store-outcomes.json`.

The later scalar union-address checkpoint supersedes the historical raw-field
address/store boundary for all-fields primitive scalar Images only. It preserves
canonical typed projection aliases, current-byte overlap and immutable copied
snapshots; pointer-bearing Images, general reinterpretation and raw-reference
reconstruction remain outside that profile. See
[the union-address review](pa27-union-address-review.md) and its separate receipt.
