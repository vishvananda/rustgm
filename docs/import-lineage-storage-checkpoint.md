# Shared import-selection storage

LIB-046's latest original-core recurrence comes from import lineage, rather than
the lint attribute-owner cache. A debugger observation of unchanged immutable
CLI6076cc08 stops on the existing storage-error branch with2,887,435 work steps
and110,103,872 charged bytes. The caller is Work::record_source from Resolver::glob.
Its edge table has229,376 entries/capacity; the next attempted growth charges
55,050,480 bytes and1,146,885 steps. Failed charges precede publication. All722
original source hashes, target, capAllow and finite source-contract limits remain
unchanged. Profiling introduces no compiler/source mutation or limit increase.

The existing optional lineage owner now interns complete immutable selections.
Destination/import edges retain four-byte indices into a checked source vector.
The interning key preserves the selected scope/name/namespace/hygiene context and
the complete binding snapshot: target, visibility, import origin and ambiguity.
Textual Direct terminals remain distinct from selected binding keys. Replacements
keep prior snapshots; another import cannot repair an old edge by changing a
shared record. Missing edges or invalid indices still fail as required information.
Export traversal still authenticates the snapshot against the actual completed
binding table, preserving hidden-glob and source-authority opponents.

Each edge slot changes from60 logical bytes to24; the source pool and intern map
add separately paid storage proportional to distinct selections. The optional
cold owner grows once when demanded; no field is added to ordinary bindings,
syntax nodes, bodies or import Work. Every edge/map/vector growth payment precedes
mutation, including failed attempts. The10M-step/64MiB limits are unchanged.

Private controls exercise missing provenance, exact namespace/visibility export
chains, source/binding mutation, no extra body demand and existing forbid order.
New fan-out inputs require1/8/32/128 distinct sources across1/64/1024/16384 edges,
with bounded paid work and storage. A changed snapshot remains separate and a
failed storage payment publishes no source, intern key or edge.

## Qualification

All gates bind identical compiler/tool fingerprints. Private9 passes in264.85s;
production18 plus all PA6245 and PA8120 text cases pass in230.53s. Both gates
include formatting, architecture/file placement and strict Clippy. No local
release build or whole-suite repetition is used.

| Receipt | SHA-256 |
| --- | --- |
| Private `obj/qualification/fast/1793640380787846845/receipt.json` | `7b8932bf1f6466c7fbdacb0595a2b7feb47564f88a6db297b45f8e4e37dc9b9f` |
| Production `obj/qualification/fast/1793640655373061326/receipt.json` | `9ce0137b27200ce9517b5d8efad82533769f94261b6e2ef095ff7ce8f75afffa` |
| Debugger `obj/parallel/lint-policy-storage-v3/profile-v2/receipt.json` | `d20303cc57de0ac10905550e14dfedcb21644f424bf6e57b1702d3d9801fbc8c` |
| Original core `obj/parallel/lint-policy-storage-v3/core-v1/receipt.json` | `70edb15511083c16b5842914436a3d05867b3155668a318db9a149feed7842c7` |
| ABBA `obj/parallel/lint-policy-storage-v3/perf-v1/receipt.json` | `e7fb6df6d3f8df09b864e4693b75d2b32291b7e90fa2acfdae655bc67112c3e9` |

Fan-out widths1/8/32/128 pay520/14280/187848/2814312 bytes and
12/823/12361/193801 work steps. Existing shared-chain export traversal retains
the same27/120/458/1786 steps and128/1088/4608/18208 bytes. Its import production
now pays1608/2376/6600/22920 bytes versus1232/3152/13712/54512 previously: sharing
has a fixed cold-owner cost and reduces growing repeated payloads. Measured
Lineage88 and Work544 confirm the cost remains inside the existing cold pointer.

Forty immutable-binary observations use four frozen inputs, A/A calibration and
two paired ABBA blocks, retaining every stdout/stderr, raw time/RSS record and
output hash. All outputs match. Width128's median is0.10522s before and0.11325s
after; A/A calibration varies0.12436–0.14185s. Ordinary-control medians are
0.01057/0.01017s. These short wall timings do not establish a speedup or a
significant regression. Width128 median peak RSS is36728/36058KiB; edge-slot
compression is a logical storage result, not a claim of60% lower process RSS.

Immutable successor CLI
7f41f3f47ed57d991448cce03725938a6a5b497e75793548874ca27817c9159a passes
the storage failure and reports `ERROR: file 55: 12164: unresolved value name`.
The28.96s observation preserves all722 source hashes,375 opens, target/config,
capAllow and finite limits; infrastructure succeeds. Source-name resolution is
the next actual requirement. The longer observation reaches further work and is
not a paired performance comparison. Whole assignments and original core/alloc/std
artifacts remain required. Successful generated artifacts are scoped-pruned;
frozen inputs, diagnostics, receipts and current warm caches remain.
