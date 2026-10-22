# Fixed arrays and record representation across crates

This PA29/PA30 metadata extension has passed focused source/native qualification
and all active text suites. Combined release and performance qualification
remain pending. The earlier public-name checkpoint is committed separately.
Generic bodies, nominal parameters, trait contracts, richer value graphs and
the other remaining assignment requirements stay in scope.

The declared source contracts are `pa29/tests/array-metadata-outcomes.json` and
`pa30/tests/aggregate-metadata-outcomes.json`. They retain original source hashes
before independent execution. Array arguments/results, nested array fields,
borrowed arrays, empty arrays and non-Copy element moves work across crates.
The C cases require mixed INTEGER/SSE records, nested arrays, MEMORY-class
records, function pointer conventions, private fields and ordinary move checks.
A separate C caller checks actual interoperability with exported definitions.
The original sources qualified against pinned Rust before candidate observation.

## Typed transport

The existing artifact type graph gains a fixed-array node containing one
element TypeRef and a u64 length. Its element remains a preceding canonical
node. Depth and lifetime-slot summaries are computed once per node; canonical
first-use postorder and shared graph storage allow one import remap per node.
Length describes the type, so decoding allocates no per-element entries. Record
storage cycle checks follow array edges, including zero-length arrays.

Record declarations retain their actual representation independently of field
visibility. This extension transports the existing default Rust and plain C
representations. Alignment, packing, transparent and enum representations remain
required subsequent contracts and are diagnosed explicitly.
Import publishes the retained representation into ordinary aggregate facts, then
uses existing layout, size and C ABI classification. No wire-provided offsets,
register classes or invented syntax nodes can replace those computations.

The additive profile-seven envelope is selected only when array or C record
facts are present. It composes with optional profile-six names and the original
payload profiles. Array tag three carries a preceding u16 element reference and
u64 length; extended record rows carry a checked representation byte. Complete
bounded tables validate before resolver or semantic identities are allocated.
Unknown tags, forward edges, invalid representation flags, cyclic storage and
redundant envelopes are rejected. No const-body, generic-body or trait permission
is added. Stored C record values and foreign array declarations use existing
storage machinery; defined array value graphs remain an explicit later boundary.

## Qualification

Qualification v6 passed strict Clippy, the 1,745-file audit, all 37 codec tests,
34 linking tests, nine native linking groups, 90 harness tests, layout and
upstream checks. The new contracts contain 15 original library/client sources,
six RustGM-versus-rustc native clients and one C caller. Repeated exports retain
identical LowIR and metadata. Negative clients fail for their actual length,
element, lifetime, move, privacy or calling-convention violation.

All 8,360 active PA1–PA24 text cases pass without oracle changes. Candidate
SHA-256 is `d82b9137f2c72558c6886efbba800c97f0e46664227dc9303bcd173dd57a4bbe`;
receipts are in `obj/parallel/aggregate-metadata-{qualification-v6,text-v1}`.
The manifest status fields were updated after qualification; original declaration
bytes, hashes and every source/outcome are retained in the qualification folder.
One old C-record unsupported-boundary test now checks successful export, decode
and actual retained representation using its unchanged original source.

Widths 8/32/128 establish one remap per shared graph node. Extents through u64::MAX
remain fixed-size wire facts, without claiming that every such type has a valid
target layout. Codec tests cover truncation, graph cycles, bound-lifetime escape,
public aliases and combined aggregate/storage/name envelopes. The independent
compatibility batch independently qualified eight original Rust libraries, then
confirmed exact legacy metadata bytes for profiles zero through six against the
frozen pre-extension compiler. Its receipts are in the main checkout under
`obj/parallel/aggregate-metadata-compat-v1`; the results SHA-256 is
`92062c7e1d0355d3bdf572f14a0705a4328aeff49a1c1321637e93f409a01cba`.
