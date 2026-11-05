# FusedIterator source and ordinary consumers

The full pinned `library/core/src/iter/traits/marker.rs` now completes as actual
source, with its original FusedIterator:Iterator declaration and &mut blanket
implementation. The existing canonical header validator and paid role ledger
retain source identity, real trait parameters and declared supertraits/members.
Unused alternative headers preserve their Rust legality. Source target, arity,
duplicates, ordinary implementation, associated Item and full body obligations
remain checked. There is no new builtin, dense record, allocation or query engine.

Role recognition grants neither a blanket implementation nor fused runtime
behavior. The pinned compiler supplies its special FusedIterator relation only
for actual gen coroutines. General gen body/state, yield, exhaustion, cancellation
and authenticated transport remain PA42/43/29 work. This checkpoint uses real
ordinary source implementations and actual `next` calls throughout.

## Qualification

[Evidence](fused-iterator-evidence.json) binds source/fixture hashes, the frozen
independent predictions, reference, public gate, native/core observer and all54
performance samples. All17 new full source/body outcomes match pinned Rust.
The public gate passes4 exact tests, including46 AsyncFn and14 Future controls,
with full formatting, architecture and selected strict Clippy. Two native
observers pass through the locked CPPGM backend: actual generic Iterator calls
return first/second/third values `23 0 0`, and an ordinary same-name trait returns
`23`, both followed by LF, exit0 and empty stderr. Harness/upstream checks pass.
Reference setup failures and one doc-markdown Clippy failure remain retained;
foundation repairs preserved original source bytes and every predicted outcome.

Immutable CLI8d4a4ff3 clears FusedIterator and reaches actual AsyncIterator at
`library/core/src/async_iter/async_iter.rs:17`, file108 byte636. Original-core
observation takes19.6464s/375 opens and exits1. All722 input hashes, six pins,
fifteen configuration values, options/capAllow and finite limits remain unchanged.
This is source admission, without a checked/emitted original library artifact.

Exclusive immutable A/A+ABBA preserves54 observations and byte-exact output
identity. Candidate medians are0.249s/1.046s for ordinary1024/4096 and0.0546s for
source Deref; paired B/A ratios are.925/1.229/1.066. Calibration is noisy
(4096 AA.786,1024 AA up to1.204). The23%4096 ratio remains an unresolved
regression/noise observation, not an absence-of-regressions or speedup claim.
No measurements are discarded. A stable-host follow-up remains part of the next
meaningful performance checkpoint. Current/predecessor pins are8d4a4ff3/5fc8c666;
292c96f2 is explicitly retired/pruned after preserving its earlier receipts.

The old private unimplemented-vocabulary test now excludes the independently
qualified eight AsyncFn roles and FusedIterator from its unsupported-role list.
Its successor private compilation/execution remains pending. Full42131050558 on
4ab1926bb targets the preceding checkpoint and is still running at publication;
do not treat its later result as qualification of these additional source changes.
No whole assignment after24 or original core/alloc/std artifact is complete.
