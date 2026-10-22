# Checked place cache and address replay

The integrator attributed an ordinary-record regression to d3's blanket address
cache bypass: depth8/32 raw-address controls increased instructions by
8.91%/12.35%, and the depth32 peak heap increased from 963,635 to 1,618,995 bytes.
The frozen source and profiles remain in the integrator's
`obj/parallel/real-crate-prerequisites-performance-v2` and `attribution-v1` roots.
Those are historical measurements, not a measurement of this correction.

One existing Frame map now has typed Common/Value/Address keys. Both consumers
reuse ordinary common ancestry; every enum field conservatively distinguishes
its path and descendants. Classification follows a successful canonical
projection and inherits ancestor distinctions in the existing iterative walk;
it does not scan ancestry again or classify a failed projection. Once a path is
specific, descendants do not repeat the owner classification. No second Frame
map, semantic type equality, field authority or pointer permission is added.

Both Project and Select explicitly evict all three keys for their ProjectionId.
Common pointers preserve the original full Resolved value, including logical
type, metadata, raw alignment anchor, union selection and incoming restriction.
Physical enum selections cannot reuse logical enum selections. Cached addresses
still undergo actual read/write initialization, validity, provenance and lifetime
checks; a cache hit grants no access permission. Ordinary typed fields do not
force whole-parent byte materialization merely to fill the cache.

On the qualified target Frame remains 312 bytes, Resolved 64, and the map 48.
The key changes4→8 bytes, while the aligned `(key,Resolved)` entry remains80.
There is no extra bucket-entry or Frame density in this profile. Each checked
projection has at most three possible keys; repetition cannot add fresh keys.

Private controls alternate value/address demand at depth1/8/32 with reuse8/32/128.
A warm pair costs two logical steps, with entries2/9/33 and capacities3/14/56,
unchanged across reuse. A warmed common path remains usable after conversion to
byte backing while shared-derived writes and expired reads still reject. Exact
dispatcher controls exercise both invalidation actions and all cached modes.
These deterministic work/storage bounds do not claim wall-time/RSS neutrality.

The permanent seven-source PA27 inventory preserves independently declared bytes:
enum demand alternation, explicit-tag mutation with stable physical field
address, typed-record to byte transition, changed index and dereference inputs,
and restricted/expired negatives. The five positive inputs were independently
compiled and executed by pinned Rust before candidate qualification. Source
expectations are in `pa27/tests/address-cache-outcomes.json`; the existing grouped
ignored enum native harness consumes their literal bytes.

Exact failed setup receipts, layout probes, frozen original depth inputs and the
focused closing evidence live in `obj/parallel/pa27/address-cache-reuse`.
The first two private attempts found only fixture visibility and non-const owner
setup issues; production visibility and const permissions were preserved.
The integrator owns full release and comparative timing; closure5954 remains an
independent immutable checkpoint.

The closing batch passed strict all-target/all-feature Clippy, the 1,828-file
audit with zero findings, all four cache invariants, the existing 18-case enum
replay grid, 30 declared enum source cases plus its separate retained-layout
observation, 24 enum native positives, 17 closure source cases and 12 closure
native positives. All three views of the unchanged ordinary depth1/8/32 inputs
are byte-for-byte equal to the pinned worker baseline. All 8,362 PA1–24 text
cases, 98 harness tests, layout checks and 24 upstream/126 Reference pins passed.
The successful receipt is `closing-first/qualification.json`; terminal scoped
cleanup removed 109 generated files (0.15 GiB), recorded in
`obj/cleanup/1792704835299470137.jsonl`. Final source/binary hashes and the retained
evidence index are in this slice's `handoff.json`. No worker full release or
comparative timing was run.
