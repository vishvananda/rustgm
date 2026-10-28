# PA16 fixture ownership

Every source is explicitly registered in `manifest.json`, with independently
declared acceptance and pinned-rustc outcomes. These are original RustGM tests,
including cases promoted from the maintainer invariants; `maintainer_origin`
records their source. No upstream reference text is copied.

`loans/`, `pattern-loans/` and `promotion/` cover borrow liveness, aliasing,
storage, guards and persistent provenance. `thin-native/`, `wide-native/`,
`pattern-native/` and `persistent-native/` run declared executable outcomes through
rustc and CPPGM. `former-boundaries/` preserves the earlier executable boundary
inputs under their owning proof stage. The 329 cases include 123 native executions and two library object producers;
inspection and rejection cases do not count as native runs.

Only the explicit `ref-test` target regenerates sidecars. Review exact output,
diagnostics and status together with independent evidence before accepting a
new reference. Missing cases, missing oracles and infrastructure failures fail.

`shared-call-reborrow/` declares ordinary shared call, annotation, field, branch,
function-pointer and reference context outcomes before compiler changes. Its
PA31 source-provider counterparts intentionally omit reference Copy/Clone impls.
The native C consumer observes repeated calls and returned-reference operand
effects; see the [checkpoint](../../docs/pa16-shared-call-reborrow.md).

Type-generic referents/value arguments, Copy bounds and mutable generic moves
belong to PA18's `shared-call-reborrow/` lane. The original freeze and
`ownership-relocations.json` retain their byte-identical relocation; unused
helper generics were removed from the plain PA16 inputs before re-observation.
