# Scalar byte transport development checkpoint

This checkpoint extends constant `ptr::copy` and `copy_nonoverlapping` beyond
initialized typed snapshots. It is a bounded scalar/aggregate profile, not the
completed PA27/PA28 memory contract. No backend or public Constant, Pointer,
Origin, Image or View schema changes are made.

## Storage and access

A canonical local allocation can transition once from typed cells to one mutable
byte backing with an initialized-byte bit mask. The original allocation identity,
projection ancestry, liveness and pointer-value write authority remain unchanged.
After conversion every typed, projected, range and metadata access uses that
backing; the former cells cannot become a second mutable authority. Untouched
ordinary roots retain their typed fast path and allocate no byte backing.

Raw transport snapshots the selected source once, preserves both bytes and mask,
and commits the destination interval once. Partial writes clear destination mask
bits where the source was uninitialized. Overlapping copies retain snapshot
semantics. Raw transport does not decode bool or char; selected typed reads do,
and reject invalid representations or uninitialized selected bytes. Ordinary
field stores and range assignments repair the same backing seen by existing
aliases. Length and empty-range controls do not decode unrelated invalid bytes.

The profile accepts primitive scalar fields, tuples, arrays, structs and existing
scalar-only union Images. Retained `StoredShape` facts describe fields without
reconstructing ABI layout in the interpreter; projection offsets still use the
existing canonical layout facade. Enums remain distinct from products, including
single-variant enums. Pointer-bearing, enum, opaque and closure storage cannot be
converted by this checkpoint. Ordinary typed pointer-bearing transfers retain
their existing path. An unrelated pointer field is a retained Rust-positive
boundary, not a language rejection.

Scalar union fields read directly from the current backing. Actual by-value union
snapshots reuse the existing immutable Image format and its independent mask;
there is no active-member tracking. Native padding choices do not initialize
constant bytes. General union families and symbolic pointer relocations remain
required follow-ups.

Allocation preflight charges bytes, compressed masks, retained headers and work
before allocation/publication. Normal backings have no new 64 KiB cap; existing
machine limits apply. Existing union Image publication retains its prior bound.
A 131,072-byte ordinary allocation is independently qualified. Repeated typed
fills are encoded once and replicated as bytes/mask, including zero-stride
handling; this optimization must be extended to relocation metadata before
pointer families can use it. Constant and Cell remain 32 and 48 bytes on this
host; the private Backing header is 56 bytes.

## Qualification and retained failures

`pa28/tests/byte-transport-contract.json` declares source hashes and independent
outcomes. The final focused inventory has 22 pinned-Rust object cases and three
checked frontend views per case, plus 15 paired Rust/CPPGM native value controls.
It includes invalid bool/char transport versus demanded reads, partial masks,
identical and overlapping copies, projected/range aliases, ordinary repairs,
metadata-only access, a large backing and the pointer-neighbor boundary.

The first uninitialized-local hypotheses were wrong: Rust rejects forming their
raw addresses with E0381. Their exact original sources, expected outcomes and
failed receipts remain; the final contract records source-initialization
rejection, independently rechecked before functional execution. Two proposed
native positives are retained as rejected hypotheses and make no runtime claim.
The interpreter does not weaken source initialization to admit them.

Four matched private invariants cover mask clearing, overlap/range aliasing,
shared-derived versus original writable aliases, expired accesses and budget
failure before publication. Ninety-nine logical work controls cover backing
width 64/512/4096, repeated accesses 8/32/128, unrelated-type noise 0/64, and a
27-case grid varying zero-sized field width 4/32/128 independently of repeated
element count 8/32/128. The latter's count deltas are independent of field width
for ordinary typed copying, byte-backed copying and range assignment.

The ordinary union-store control preserves a preexisting immutable-Image
width-times-repetition cost; it is not reported as neutral. Converted scalar
union accesses and small ordinary writes do not republish whole Images. These
are logical work/storage checks, not wall-time or RSS measurements.

Evidence is under `obj/parallel/pa28/byte-transport/`. The eighth batch passed
strict all-target/all-feature Clippy, the 1,570-file audit with zero findings,
four private tests and all work controls, then retained the incorrect E0381
hypothesis failures. The ninth batch independently confirmed all three E0381
cases, passed the corrected source/native functions, seven counted/overlapping
transfer regression functions, fifteen rest/Image/direct-union/unaligned
regression functions, layout and upstream verification. Earlier compiler and
test wiring failures remain in their versioned evidence directories.

This explicitly supersedes the old counted `partial-field-profile` and overlap
`invalid-byte-profile` constant boundaries. Their unchanged source bytes now
succeed; the unchanged overlap demanded-invalid-bool input rejects for actual
scalar validity. Historical reviews retain the original observations.

## Required continuation

Symbolic relocation-bearing storage, fragmented pointer transport and restoration,
byte fills, general offsets/reinterpretation, numeric/null provenance and the
source lint contracts remain open. A future relocation design must preserve the
complete original pointer metadata and write restriction, invalidate provenance
on partial bit writes, and never invent a pointer from numeric bytes. It must
cover typed reads, constant graph inspection, immutable Image publication and
native relocation emission before widening public schemas.

After the isolated checkpoint, public `2f76b1c1` was merged cleanly and the new
root registered in the grouped memory suite, importing its existing shared
helpers. The grouped follow-up passed strict all-target/all-feature Clippy,
the 1,586-file audit with zero findings, all 89 Python harness tests, exact
registration (337 roots / 13 binaries), layout, four private invariants, all
three regular byte tests and the ignored native test. Every one of the 8,283
active PA1–24 text comparisons matches committed references. Evidence is in
`obj/parallel/pa28/byte-transport/grouped/`.

Focused commands use the ordinary resource/storage wrapper around
`python3 scripts/maintainer_suites.py run byte_transfers` and the same command
with `--ignored`; private checks use `cargo test --manifest-path dev/Cargo.toml
--locked --lib byte_backing_`. No full maintainer or exclusive performance
matrix was run by this lane. The integrator owns combined release/performance;
neither assignment is activated and no text reference is regenerated here.
