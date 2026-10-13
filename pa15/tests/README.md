# PA15 fixture ownership

Every source is explicitly registered in `manifest.json`, with independently
declared acceptance and pinned-rustc outcomes. These are original RustGM tests,
including cases promoted from the maintainer invariants; `maintainer_origin`
records their source. No upstream reference text is copied.

`declarations/`, `contracts/` and `provenance/` cover lifetime identity,
elision, explicit/implied bounds, variance, callable quantifiers and local versus
referent storage. `former-boundaries/` preserves earlier lifetime cases under
this assignment's actual contract proof. All 191 cases inspect `--emit-regions`;
none is counted as a native execution.

Only the explicit `ref-test` target regenerates sidecars. Review exact output,
diagnostics and status together with independent evidence before accepting a
new reference. Missing cases, missing oracles and infrastructure failures fail.
