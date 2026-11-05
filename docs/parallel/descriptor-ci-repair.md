# Descriptor registry and atomic ABI test repair

This test-only checkpoint repairs the four owned failures from CI42075839657.
The generated-registry budget fixture previously used primitive `+=` syntax,
which does not select generated methods. It now calls the genuine canonical
AddAssign members for u8/u16/u32/u64. Every existing cold allocation, warm hit,
map growth and sticky byte/work limit assertion remains required.

The atomic controls previously expected Ordering to be loaded through a pointer.
The existing scalar enum ABI repair passes its byte value directly. CAS now
checks both actual scalar parameter slots and all15 comparisons per slot; RMW
and fetch_sub check their actual scalar parameter and all5 order comparisons.
The tests reject extra pointer loads and retain their atomic operation, ordering
pair, scratch read and reconstruction requirements. No compiler code changes.

## Evidence

Focused gate1793902943960158076 passes all3 exact atomic tests, formatting,
architecture and strict Clippy in54.21s. The earlier gate1793902794546979944
failed because the first test draft used equality on Predicate, which has no
PartialEq implementation; the corrected tests use variant matching.

Independent predictions were frozen before candidate observation. Rust1.98
accepts both exact old and revised fixture sources. A typed public pipeline
client using the gate's actual library confirms zero canonical scalar shims
for primitive syntax and exactly U8/U16/U32/I64 shims for the four UFCS calls
(the actual LowIR64-bit integer vocabulary is I64). It uses the genuine entry
and prepared lowering pipeline, and reads typed function signatures.
The first client draft incorrectly named a nonexistent LowIR U64 type and
failed to compile; its source/diagnostics remain archived. The corrected client
builds and passes in5.55s. This validates the producer/setup, without claiming
execution of the private counter assertions: that exact test remains pending
full CI. No repeated private harness link or policy bypass was attempted.

Raw receipts, predictions, sources and diagnostics remain under
obj/parallel/future-runtime-v1/descriptor-repair; the compact tracked evidence
records hashes and exact scope. The approved future runtime draft is preserved
separately in stash d2961a553585d05f798556f103942389abbbd69b.
No new future execution, native future or assignment completion is claimed.
