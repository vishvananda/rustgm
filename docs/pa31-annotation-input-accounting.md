# Owner-indexed closure annotation regions

This corrects a newly introduced quadratic path in source-provider checkpoint
`5fb7bb86`. It does not expand source bindings or make upstream core compile.
The [whole-core census](pa31-whole-core-source-census.md) remains blocked at its
original source-feature policy; its byte4062 location is line88, correcting the
initial doc-only line94 transcription.

## Independently qualified source and measured visits

The frozen sources contain const-function chains whose uncalled closures have
an input annotation `&[u8; next_const_fn()]`. Preparing that array bound demands
the next function's local annotations while the outer input continuation remains
open. An opposing family adds three unrelated local reference annotations at
each level. Independent width controls contain separate closures instead of a
chain. Deepest-function type errors are separate declared negatives.

All twelve original sources match pinned Rust1.98.0
`88d9e12ae178fab0fb5cc050a94da85685d449ea` and the immutable pre-correction pin:
nine accepted, three rejected for the actual local type mismatch. No source was
trimmed to make a failing program pass. The counters observe actual loop visits,
including work before errors; they are not timings or estimated operation counts.

| Source family | Size8 before→after | Size32 before→after | Size128 before→after |
| --- | --- | --- | --- |
| Nested annotations, no extra foreign regions |64→8|1,024→32|16,384→128|
| Nested annotations, three extra foreign annotations/level |232→8|4,000→32|65,152→128|
| Independent closure width |8→8|32→32|128→128|
| Deepest invalid local type |1→1|1→1|1→1|

The first family was exactly n² visits, the second4n²−3n, despite only n owned
input regions. Both now produce, validate and transfer exactly n IDs; foreign
annotations do not enter the input collector. The errors occur before outer
continuations finish, so their measured one visit is retained rather than
pretending they exercised the full successful chain. Existing actual global
region counts remain15/63/255 without extra annotations and60/252/1,020 with them.

## Production ownership and accounting

Declaration Checker now owns one lazy input-scope index. Begin opens the exact
closure owner and retains its produced-ID cursor. The existing annotation-region
producer records a RegionId only when that same owner has a pending input scope.
This constant-time index operation is covered by the annotation node's normal
work charge. Foreign owners never contribute to that vector.

Finish consumes only the recorded owner range, validates every actual retained
BodyRegion owner and Annotation origin, and charges each transferred ID to the
existing declaration work budget before publication. Invalid/stale owner/range,
zero or foreign RegionId, and exhausted work controls reject without replacing
previously completed inputs. A repeated memo hit with no new regions preserves
those inputs. Nested scopes of the same owner retain their own cursors. Result
annotations are scheduled only after input completion, as before. No new
lifetime assumption or callable permission is introduced.

This removes the repeated foreign-region traversal instead of adding a quota
around it. There is no new global region scan, second proof ledger or retained
source/body schema. The normal completed-input vector is moved from the pending
scope into the existing closure input map; the ordinary final scope does not
copy that vector.

## Storage and scope

Measured local sizes are Checker12,952→12,960 (+8 for the cold index pointer),
Work80 unchanged, execution Pending96 and Phase1 unchanged. Declarations,
Body and Code records are unchanged. The index is absent before any closure
input demand; recording without an active scope does not allocate it.

The cold map header is48 bytes; an active scope's Vec/count row32 and keyed
entry40. At depths8/32/128 the peak pending owner/ID count is8/32/128,
map capacity14/56/224 and region-buffer capacity32/128/512 RegionIds. The
extra three foreign annotations per level do not change any of those values.
Independent widths retain just one pending owner/ID, map capacity3 and four
RegionId slots. Successful completion leaves no pending IDs; the empty map's
capacity can be reused by later closures. These are payload/capacity observations,
not allocator-header or process-RSS measurements. On errors, pending rows remain
only until the aborted Checker is dropped; the error observations record that
actual pre-drop state.

## Verification and provenance

The focused before/after corpus and source hashes are under
`obj/parallel/pa31/annotation-accounting-v1/declared.json`,
`independent/results.json`, `old-instrumented/{observations.log,source-map.json,source/}`
and `new-indexed-v3/annotations.log`. The private test asserts the linear
visited/produced counts for both depth families and width, plus the real error
counts. Its observer is cfg(test)-only and uses thread-local counters so parallel
unit tests do not mix observations.

The indexed code passed all-target/all-feature strict Clippy, two focused
annotation/scaling/storage tests, five readiness/malformed/memo/budget tests,
and three existing execution-storage tests (including early lifetime-bearing
closures and nested readiness budgets). The initially mistyped execution-storage
filter selected zero and is explicitly superseded by `storage-tail/selected.json`
and its three actual test results, using the same just-built unit binary.
The audit's sole generic filename-stem finding was corrected by naming the leaf
`input_regions.rs`; its module identity and bytes otherwise remain unchanged.
Final strict/audit/layout receipts and source map are under `closing/`.

The first repair compile's private-visibility failure and the next test-only
Clippy generator findings remain in `new-indexed/` and `new-indexed-v2/`; neither
is counted as successful validation. The coherent code introduces no native or
backend operation. No full worker release, complete active-text rerun, exclusive
performance measurement or assignment activation was performed. Integration
owns the one corrected combined release and performance qualification.
