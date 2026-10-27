# Char MIN/MAX values through the primitive constant owner

The unchanged primitive-constants/char-required.rs now passes the typed pipeline
and literal-source native execution. Its SHA-256 remains
5ca261b7ead6180554e917836eeb0207ddd3ea6634e69d5a8505f73c3daee44b.
PA26 stays planned. Preserve NOTICE and the pinned Rust source attribution.

The existing lazy primitive value owner now selects char MIN and MAX as typed
Char constants with bits 0 and 0x10ffff. Integer MAX retains its existing target
width/signedness formula. MIN publishes only for an actual normalized Char
prefix; other types keep ordinary lookup. Aliases complete their existing proof,
nominal/shadow paths keep ordinary declarations, and concrete Self preserves
inherent priority. Argument, place, call and generic-const restrictions remain.
The existing scalar validator still rejects surrogates and values above 0x10ffff.

No new owner, cache, body/NodeFacts field, intrinsic or metadata/backend profile is
added. Expression inference, patterns, CTFE and runtime ConstArgument reuse the
existing consumers. Mutable temporary borrows do not modify canonical values.
Char arrays, copies, reference loads, function arguments/results and narrowing/
widening casts retain their existing physical representation and proof duties.

## Source ownership and independent observations

SourceCore still exits before bootstrap primitive selection. Actual associated
declarations/initializers own their values; absent source constants stay rejected.
PA31 provider controls retain the pinned char declarations' stable attributes and
literal initializers in a minimal source definition profile. Authorized independent
source-owned bodies returning 'a'/'z' execute those values, with no primitive Uses
owner. These controls qualify declaration ownership, not the entire original
char/methods module or original core/alloc/std.

Prediction commits 897ec532f and b3f690dcd precede 27+5 pinned Rust metadata commands.
There are no infrastructure failures. Two original positive predictions fail
because the staged-API fixture omitted stability attributes on public foundation
traits and the custom source constants. Those inputs, plans and diagnostics remain
retained. All five followups match, including corrected source-positive/negative
profiles and the array/copy/ref/function width consumer. Byte-identical source
provider inputs moved to their PA31 owner in 9da814d2e. source-relocations.json,
historical input copies and Git history bind original paths to current bytes.
No outcomes were rewritten to hide those original disagreements.

Private seven passes in 164.40 seconds at fast/1793136479409320402. The relocated source
consumer passes separately in 99.04 seconds at fast/1793136973930311380, including
private strict Clippy, format and architecture checks. Production/CLI/native at
fast/1793136673481459392 passes in 100.45 seconds: 16 literal rustc/CPPGM native pairs,
including the byte-identical original fixture, all exit 0 with empty stdout/stderr.
Every pair retains hashes/source/LowIR and retires its generated executables.

Only after actual source/native success does char-required become an accepted
legacy predicate and draft execute/pass outcome. Final production/CLI source two at
fast/1793137167234104150 passes in 37.93 seconds: all 62 inherited primitive outcomes
and 24 ordinary char opponents. Float-required remains an explicit Rust-positive
unsupported case. The exact final private binary also passes four printed controls
and upstream verification of 24 pinned fixtures and 126 Reference files. Native to
final source differs only in two test files: provider fixture paths and the legacy
char predicate. Compiler bytes are identical. No official references/publication
are changed.

## Actual retained costs

Descriptor grows one to two bytes. Selected remains 16, Uses 32, NodeFacts 80 and
Body 456. Existing Char layout is 4-byte size/alignment; [char;3] is 12/4. The ordinary
no-value-use controls retain no Uses owner and char use demands no primitive method
body. The payload column counts existing map key/value slots, excluding allocator
and hash-table overhead; it is not a memory-speedup or wall-time comparison.

| MIN/MAX pairs | Selected uses | Canonical values | Map capacity | Capacity payload bytes | Inference steps |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2 | 2 | 3 | 72 | 9 |
| 8 | 16 | 2 | 28 | 672 | 51 |
| 32 | 64 | 2 | 112 | 2,688 | 195 |

The existing integer control retains one canonical MAX at 8/32/128 uses with
27/99/387 inference steps and 33 definition slots. These are current observations;
historical integer-checkpoint counts are not a same-binary baseline.

Float constants, shared-reference RHS Add/Sub and broader hook/derive closure
remain separate required exits. Other char members, imported source completion,
unit publication and original core/alloc/std remain independently owned work.
