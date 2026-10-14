# PA16 fixture ownership

Every source is explicitly registered in `manifest.json`, with independently
declared acceptance and pinned-rustc outcomes. These are original RustGM tests,
including cases promoted from the maintainer invariants; `maintainer_origin`
records their source. No upstream reference text is copied.

`loans/`, `pattern-loans/` and `promotion/` cover borrow liveness, aliasing,
storage, guards and persistent provenance. `thin-native/`, `wide-native/`,
`pattern-native/` and `persistent-native/` run declared executable outcomes through
rustc and CPPGM. `former-boundaries/` preserves the earlier executable boundary
inputs under their owning proof stage. The 293 cases include 113 LowIR executions;
inspection and rejection cases do not count as native runs.

Only the explicit `ref-test` target regenerates sidecars. Review exact output,
diagnostics and status together with independent evidence before accepting a
new reference. Missing cases, missing oracles and infrastructure failures fail.
