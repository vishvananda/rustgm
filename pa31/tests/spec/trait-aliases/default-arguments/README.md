# Unused alias parameter defaults

These are planned PA31 declaration controls. `contract.json` preserves the
initial predictions and frozen source hashes. Independent Rust 1.98.0 metadata
compilation in both Rust 2021 and 2024 accepts all three original inputs: the
two rejection predictions for `invalid-transitive.rs` were wrong. Its historical
filename remains unchanged. Alias default checking does not expand inherited
alias clauses into additional declaration obligations.

The existing `semantic-v1/unused-invalid-default.rs` independently rejects a
written `T: Copy = String` default. RustGM's bootstrap prelude does not supply
String, so the two `no-copy-*` controls use an actual local non-Copy struct to
exercise the same distinction without requiring alloc. Their separate contract
declares outcomes before execution; all four independent outcomes agree: reject
the direct bound, accept the inherited bound, in both editions. No library name
or copied trait proof is invented for the test.

Valid inputs currently reach RustGM's explicit unfinished alias execution/export
boundary. That is declaration validation evidence, not successful compilation or
an alias release. Invalid direct defaults must report the actual default-bound
violation before that boundary. No text references have been generated.
