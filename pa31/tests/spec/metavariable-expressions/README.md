# Metavariable expression protocol

LIB-041 owns the source-library feature; PA7 owns template/binding grammar and
PA8 owns transcription and hygiene. PA31 remains planned with no graded manifest.
The private expression controls consume these sources directly.

`contract.json`, `additional.json` and `native-owned.json` preserve separately
frozen source predictions. There are 104 pinned Rust metadata observations in
editions 2021/2024 and twelve immutable old-compiler observations. The original
`execute-counts.rs` predictions were wrong in both editions because the provider
omitted Copy. That source and the failed predictions remain; the separately
declared `execute-counts-owned.rs` supplies genuine usize Clone/Copy impls.

The protocol covers count depths and empty first branches, ignore-driven
repetitions, index/length, escaped-dollar nested definitions, malformed operands,
unused unknown variables and crate-only feature authority. Internal feature
allowances do not replace the actual crate request. Numeric tokens retain the
definition's edition, context and source anchor.

`native-predictions.json` and `observe-counts.c` preserve the library consumer.
Pinned Rust prints `3 3 2 33`; candidate library publication still rejects
unimplemented source macro/semantic-name provenance transport. The first
candidate invocation also omitted `--crate-type=lib`; both failures are retained.
The separately frozen `execute-counts-bin.rs`, `observe-counts-bin.c` and
`native-bin-predictions.json` exercise the same four functions through an actual
executable entry. Both pinned Rust and the candidate with the locked CPPGM
backend print exactly `3 3 2 33` in edition 2024.

See [the checkpoint](../../../../docs/metavariable-expressions-checkpoint.md)
for qualification, bounded-work measurements, observed development-build
slowdowns and the next unchanged-core failure. No full library or assignment
completion is implied.
