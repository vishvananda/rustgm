# PA6 maintainer validation

PA6 adds modules, NFC interning, namespaces, lexical name facts, imports and
visibility to the maintained frontend. It leaves PA1–PA5 text oracles unchanged.
The active total is 825 cases: 51, 139, 88, 181, 213 and 153 by assignment.

All 153 PA6 cases have independent pinned-rustc checks. One deliberate difference
is recorded in the manifest: a resolved enum variant cannot denote a type, which
PA9 will diagnose. No PA6 fragments are excluded. Macro expansion, type-directed
associated selection and external library metadata remain explicit later-phase
boundaries, as specified in the handout.

Validation includes 63 debug/release Rust invariants, 14 harness tests, Clippy
all/pedantic with warnings denied, the syntax-aware file/architecture audit,
all earlier course suites and independent checks, and verification of 24 imported
fixtures plus 126 vendored Reference files. Reference regeneration changes old
receipts' compiler hashes, not earlier expected stdout/stderr/status bytes.

The implementation reuses configured typed syntax and keeps compact source
ranges for borrowed literals. Sources and syntax can move without self-references;
decoded bytes share one pool per source. Name interning has a borrowed ASCII path
and reusable Unicode normalization scratch. Dense owner lookup uses four-byte
optional nonzero indices. Declaration and expression traversals use work vectors;
a 20,001-reference left-associative chain is an invariant test.

Import watchers record the bindings a path depends on, including provisional
successful/ambiguous results. Glob subscribers consume changed-binding journals.
Per-import contributions let a later explicit binding replace a transient glob
result throughout downstream aliases. Invariants check final definition identity,
not just acceptance. A finite retry budget diagnoses pathological cyclic work.

The harness requires normalized relative fixture identities and explicitly
listed auxiliary sources. It hashes those sources in receipts and recreates
independent rustc source packages from declared inputs, avoiding stale files.

## Performance evidence
Frozen A/B snapshots, identical checksummed inputs, four A/A calibration samples
and three paired ABBA blocks were retained at each scale. Full rendered output
and telemetry matched between the snapshots. Final runs were sequential. Earlier
overlapping exploratory runs are retained with `NOT_USED.txt` and are excluded.
| Stage | Input bytes | A median ms | B median ms | B peak RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Crate syntax | 316000 | 29.01 | 28.91 | 7848 |
| Crate syntax | 1264000 | 108.65 | 104.76 | 24768 |
| Crate syntax | 5056000 | 402.19 | 417.69 | 93752 |
| Names | 242920 | 49.77 | 50.18 | 10596 |
| Names | 980920 | 205.58 | 192.86 | 35948 |
| Names | 3956920 | 817.69 | 812.75 | 136928 |

Raw crate syntax evidence: `obj/perf/1791769528300027713/samples.json`.
Raw names evidence: `obj/perf/1791770054631251350/samples.json`.

The crate-syntax baseline is committed PA5 (`ec57b7b`). Its paired medians
varied from −3.6% to +3.9%; the largest input also had a roughly 24% A/A
calibration spread. This does not establish a significant regression or speedup.
Names had no PA5 implementation; its A snapshot is an immutable earlier PA6
implementation and B is the final implementation. These measurements establish
scaling/output equivalence, not a comparison with rustc or a fixed speed claim.
