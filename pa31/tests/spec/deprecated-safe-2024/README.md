# Edition-dependent unsafe calls

The pinned Rust checks run on the original fixtures with the declared
edition for each case. RustGM checks the same local declarations after a
minimal Source provider prefix. Physical source calls to an actual marked
function use the compatibility rule before 2024. Function pointer calls
and Rust 2024 calls retain their ordinary unsafe checks. The edition of a
call assembled from macro tokens requires a separate provenance proof.

The installed std cases exercise cross-crate behavior in pinned Rust.
RustGM source publication rejects this contract until metadata can preserve
the declaration marker and the call's edition provenance.
