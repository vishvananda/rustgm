# Typed field macros in source derives

Commit b3ef989e2 copies an actual `Kind::Macro` in Type context through the
existing derive field API. It recursively transfers the selected path and
copies only that occurrence's captured forest. Prepaid vector growth, forest
allocation and tree work precede mutation. Existing typed grafting rebases the
capture, contexts and spans; opaque parsed-pattern handles remain owned by the
real registry. No source spelling is reparsed, unrelated arena cloned or macro
permission inferred from a name. Invalid captures, contexts and failed payment
remain errors. Already-expanded unadmitted field descendants stay explicit.

The byte-identical original niche_types macro contains a type macro whose
path maps to original byte546. Its selected forest has four trees and retains
an actual opaque inclusive-range AST. The source consumer exercises nested
type macros through genuine source Clone obligations and LowIR/native output
`12345678`, matching pinned Rust. Wrong result type and missing feature remain
hard errors. The old generic-constraint copier and ordinary non-Copy Clone
controls pass. The original range remains a parser frontier, not compiled Rust.

Private6 passes30.06s; receipt1793267280890699221 has SHA256
`4155ca71e7f325f8082c95de56872c4ef1d538a01440e0ad31f4f8762a77069a`.
All3151 compiler/test/harness fingerprint entries exactly match the merged Root
tree, so those actual tests are reused instead of rebuilding them. Frozen
fixture hashes also match. Native proof SHA256 is
`53ca7468e61c9c3d5d4ada4f01fb4d2cae0d92692406ea3b8e819a8dcf34643c`;
independent Rust6-operation proof is
`00cf7ce5321e914115460eb696a2f3d0e8f94da2959c711146eab619fee44536`.
No new Root private/native execution is claimed. Root production/CLI3 passes
111.79s (1793267605838310309); both strict Clippy partitions, formatting,
architecture, harness272 and pinned upstream verification pass.

At8/32/128 forest entries, work21/69/261 and bytes776/1736/5576 are linear.
Only two selected nodes and one forest are copied. Unrelated source arenas do
not enter the copy; warm authenticated reads add no retained allocation.
Node36/Body456 remain unchanged. The frozen contract and retained authored
observer/import mistakes are in `pa31/tests/spec/derive-type-macros/` and
B's `obj/parallel/derive-type-macros-v1` evidence.

Qualified CLI64e2f5e6d67e8ed0db7f16bdeb52d13cd19465cbd8c5169abe4b9f61366d5d3b
reaches the actual range parser at niche_types.rs byte3237 in15.22s. Its first
error is `expected punctuation`; all722 original hashes and target settings
remain unchanged under the same explicit finite resource profile. There are
374 source opens, which do not establish semantic completion or an artifact.
Receipt SHA256 is
`ef70a04c75a1b57aefe31fb80591053073fda4e61463a0317a20bc06c8dc51c9`.
Strict separately retains the deferred pointer-cast expectation at68 opens,
receipt`d415dd0a449c0823cc0df37593c28bf0b3515e7f759242981ad186f0e8168b57`.

Real scalar-range formation, constants, validity, base ABI, genuine open/exclusive
RangeMin/Max/Sub source protocol, refinement metadata and all original
core/alloc/std artifacts remain required. This checkpoint does not implement
those semantics or release a whole assignment.
