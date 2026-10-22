# Stored arrays across crate boundaries

This extends PA29's aggregate metadata envelope with completed array values and
static element addresses. PA29 remains planned; generic body transport and the
other owning requirements remain required. Focused source/native qualification
passes, while combined release and performance qualification are still pending.

## Representation and complexity

The shared typed value graph retains explicit products, one-child repeat recipes
and bounded byte payloads. Repeat length comes from the array type, so a large
extent does not allocate or serialize one value per element. The importer reuses
the existing constant arenas and interned byte storage; no second evaluator or
rendered-text transport is introduced.

Static addresses keep their defining external storage identity. A projected path
distinguishes u32 record-field ordinals from u64 array indices and validates each
transition against the current aggregate type, bounds and exact final referent.
Mixed record/array paths cannot be relabelled as all-field paths. Hidden backing
stays private, and arrays of references retain the original target symbols.
All-field paths keep the earlier encoding; new value tags require the existing
profile-seven aggregate envelope.

Completed value nodes are postordered and shared. Decode checks preceding child
edges, element types, lengths, byte-array element type, static targets and final
reachability before semantic IDs are allocated. Quotas charge retained edges,
path steps and byte payloads, not the numerical array extent. Export memoizes
typed value identities; import remaps each retained node once. No generic/const
body permission is implied by receiving completed data.

## Independent controls and retained evidence

`pa29/tests/array-storage-outcomes.json` declares 13 original sources whose hashes
and Rust outcomes were frozen before candidate observation. One provider and
seven positive clients exercise explicit/repeated/byte/empty/nested arrays,
constant imported reads, hidden record/array projections and address identity.
A zero-sized array of 8,589,934,593 elements and a reference at index 8,589,934,592
exercise full-width indices without a large allocation. Five negatives preserve
privacy, length, immutable storage, non-Copy moves and constant bounds failures.

The source runner builds full pinned-rustc provider/client artifacts and compares
repeat/reordered RustGM LowIR and metadata. Seven clients independently execute
through both rustc and the frozen CPPGM object backend with their original value
and identity assertions. No backend code or lock changed.

Four new codec tests cover explicit/repeat/byte round trips, all truncations,
wrong extents/types/edges, invalid projection kinds/bounds and fixed wire size
through u64::MAX. Those wire tests do not claim every such array has a valid
target layout. All 41 codec tests passed in
`obj/parallel/array-storage-focused-v2`; the later diagnostic-only edit does not
change their implementation or inputs.

`obj/parallel/array-storage-functional-v4` passes formatting, strict all-target/
all-feature release Clippy, the 1,748-file audit with zero findings, all 13 source
cases and seven native clients. Earlier failed receipts are retained: the length
negative predicted a different wording for the same type rejection, and indexed
immutable static writes reached a misleading unsafe-checking diagnostic. The
latter message now identifies immutable storage. Actual mutable statics have a
separate typed access classification and do not use that error branch. Original
sources and all independent success/rejection outcomes remain unchanged.

Candidate/source hashes and source/native runner receipts are retained beside
the focused evidence. Combined earlier-suite comparison, full release checks,
legacy metadata compatibility and affected performance review remain publication
gates. No references were generated or assignment status changed here.
