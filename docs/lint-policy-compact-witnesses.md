# Compact source lint witnesses

The unchanged original core now passes the LIB-046 storage failure with the
existing 10M-work/64MiB-storage limits. Its next rejection is actual source
formatting-argument construction, in a `write!`/`format_args!` expansion. All722
pinned files verify before and after;372 files open in8.25s. There is no core
artifact, and this observation does not prove all later lint queries fit.

## Measured cause and representation

The preceding immutable CLI9d3ea4f6 debugger observation stopped at the actual
cold storage failure before diagnostic retention. It recorded23,964 owners,
169,844 witness rows in257,732 slots, and60,713 tree pairs in234,878 slots.
111,193 witnesses had no trees;57,698 one-tree vectors each reserved four slots.
The existing payer had charged67,109,064 bytes and2,940,826 combined read/work
steps. Compositions were zero. This established retained-record pressure;
warm policy composition was not the cause of this failure.

The same `Parsed` owner now holds a boxed header array and one boxed flat tree
array. Each header retains checked u32 effect/capture/count fields and its full
Attribute. The old72-byte witness becomes44 bytes; Owner grows104→112 bytes.
No new map, global cache, source copy or common Body field is introduced.

A charged counting pass validates actual retained effect/capture/range facts,
then prepays exact array storage and fill work. Exact pushes preserve the pinned
allocator's requested capacities; conversion into boxes needs no shrink copy.
Lint changes are still interpreted once in source order. Warm reads authenticate
the same source incarnation, configured owner, effect, capture, attribute and
ordered typed trees. Per-header coverage and the final flat cursor are checked.
Zero-tree/doc/nonlint attributes remain present, including positive-empty owners.
Failed counting/storage payments publish neither an owner nor an index/tail;
attempted work remains charged on retries.

Actual source controls retain five attributes and eight tree pairs per owner:

| Owners | Header/tree payload bytes | Existing payer bytes | Work steps |
| ---: | ---: | ---: | ---: |
| 1 | 412 | 1,572 | 106 |
| 8 | 3,296 | 8,376 | 519 |
| 32 | 13,184 | 31,704 | 1,935 |
| 128 | 52,736 | 125,016 | 7,599 |

These are actual Rust payload and logical payer counts, not whole-process heap
or allocator bucket measurements. Source and syntax storage have separate owners.

## Qualification

Five new exact controls cover linear payload, complete empty witnesses, shifted/
corrupt counts and order, missing/trailing trees, malformed source ranges, failed
payments, sticky retries and forbid priority/source offsets. Together with eight
prior producer/frame/permission controls, all13 pass with private strict Clippy,
formatting and global architecture in145.15s. Production/CLI strict Clippy,
twenty ownership/lexer controls and all120 PA8 text cases pass in103.40s.
All2,956 source fingerprints agree between these gates and the frozen candidate.

Evidence in the root-owned `rustgm-pa25-work` checkout:

- Private: `obj/qualification/fast/1793126171886283937/receipt.json`.
- Production/CLI: `obj/qualification/fast/1793126666402242284/receipt.json`.
- Exact layouts/scaling: `obj/parallel/lib046-lint-budget-v1/compact-costs/`.
- Prior actual failure attribution: the sibling `debugger-observation/` directory.
- Unchanged original core: the sibling `compact-core/result.json`.

The candidate SHA-256 is
`6a99faf35ba4044de38567682fcbdb7089534d5fc9eeaf40e3a289831d0a9063`;
its source/qualification binding SHA-256 is
`b7fbcd41dcac8e0e5a23cb51ef9f3ea9f3718e9cad82afa530cb51f701afac8c`.
The next source construction failure belongs to the existing real formatting/
panic consumer lane. Genuine helper bodies, full semantic checking and original
library publication remain required. This checkpoint releases no whole PA.

## Bounded performance observation

Immutable previous CLI1b5d6518 and candidate6a99faf3 ran four frozen inputs
under an exclusive resource lease, with A/A calibration and three ABBA blocks
per input. All full expanded views and count outputs match byte-for-byte.

| Input | Previous median ms | Candidate median ms | Previous / candidate peak RSS KiB |
| --- | ---: | ---: | ---: |
| attributes-128 | 14.57 | 16.19 | 34300 / 29908 |
| attributes-512 | 38.58 | 39.02 | 35724 / 31540 |
| attributes-2048 | 124.04 | 126.89 | 39960 / 35224 |
| ordinary-512 | 20.10 | 17.95 | 33444 / 30292 |

The counting pass adds real work; this is a storage repair, not a demonstrated
compile-time speedup. The2048-owner baseline calibration ranges115–154ms, so
these brief timings do not establish small throughput changes. Process RSS also
includes binary/source/parser memory and is not the witness payload delta.
All72 observations, exact hashes and calibration rows remain in
`obj/parallel/lib046-lint-budget-v1/compact-perf/receipt.json`.

Receipt SHA-256s:

- private: `b8061df26c0bc045174c7105021b49e98e49ba648da2b9c223989af03adf33b3`.
- production: `201e4b0fea740e91b8ba6c06bd9d51d82de915c33fc0bbd644e7c77bbb006618`.
- core: `d33a73ef00e61adbb29a20e6c19a8db083cba65f1264920a8a5a134446d478da`.
- performance: `ca9c78a6a6c5e920abd54448ab150ec30b6b3434ce9e34668afcf296c466962b`.
