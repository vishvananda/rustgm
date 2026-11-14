# Conditional supertrait calls in const defaults

Clean original-core candidate30 stopped at `cmp.rs:1037`, inside the unchanged
`Ord::max` body. Its `<` operation calls `PartialOrd<Self>` through Ord's declared
conditional supertrait. The defining validator incorrectly required every call
to name only the defining trait's own application.

The scoped validator now borrows the completed defining environment. Its existing
own-trait fast path retains the exact local conditional Self premise; other
applications require the same defining Self and an exact indexed unconditional
or conditional const premise. Canonical application identity retains the complete
trait arguments. Source operation, target/signature/result, current caller effect,
selected implementation, borrow and execution checks remain separate. There is
one preparation scan per body, no per-call clause scan, new cache, persistent
AST/body/value field, wire change, backend change or storage-policy change.

Concrete execution of the exact reduced `Ord::max` exposed a second omission:
its method-local `Self: [const] Destruct` bound can have genuine builtin proof
without an implementation definition. That exact authenticated source role now
consumes completed selection evidence, retaining actual Drop/field, lifetime and
size debts. Other traits preserve their existing closed-implementation
certification and reuse. The method itself still needs its real concrete const
implementation and selected default.

Ordinary const-trait minima belong to [PA25](../pa25/tests/spec/default-supertraits/manifest.json).
Actual source-role counterparts, the byte-exact original cmp.rs, and the unchanged
Ord::max item belong to [PA31](../pa31/tests/spec/default-supertraits/manifest.json).
Predictions preceded execution. Pinned Rust1.98 agrees with all standard-core
counterparts: conditional/always-const, generic and transitive supertraits,
method-local bounds, ordinary effects, missing/wrong arguments, const bound
implementations and runtime-only destructors. There are18 source outcomes
(13 accept/5 reject), all qualified through prepared lowering and standalone CLI.
Of16 ordinary inputs,14 qualify (10 accept/4 reject); the two importing
`core::marker::Destruct` stop at unresolved import and remain required/unqualified,
including the negative whose actual const rejection has not been reached.

Evidence is under `obj/parallel/default-supertraits/`:

- Initial `inputs/declaration.json` and `baseline/` freeze nine paired predictions
  and reproduce five valid-default failures with immutable clean candidate30.
  `bounds-declaration.json`/`bounds-reference/` independently reproduce the
  builtin Destruct omission while preserving both rejecting opponents.
- `reference/` qualifies additional scaling and actual native results41 and
  `41 41` for the original Ord::max constant/runtime pair. Complete original
  method bytes and the existing comparison foundation have separate hashes.
- fast5,1794678879779995705, passes six exact tests in140.36s, strict selected/
  production/CLI Clippy, complete formatting and architecture checks. fast1/2
  corrected test-helper/diagnostic-owner setup; fast3 exposed Destruct proof;
  fast4 exposed the separately retained ordinary import boundary.
- native1794679030907816509 passes two groups/eight actual programs in41.78s:
  three new observers (41,41,`41 41`) and five existing cleanup controls.
- `replay/` authenticates3964 compiler sources and all packet input hashes,
  checks32 complete CLI outcomes plus the two unqualified imports, and reruns
  the exact lowering/scaling controls. Post-lowering inference/canonical work
  for8/32/128 calls is1754/1473,6962/4741,27794/17765. No timing/speed claim.
  All315 harness tests,495 roots/13 binaries,44 stages/one component,
 128 upstream and126 Reference checks pass.

The preceding full43618987950 on4a80bfb4a passed every stage, including3080
private release/debug tests (4.79s/46.67s). The new checkpoint's full successor
and clean original-core retry are next. Original v13/v14 metadata/LowIR/native
consumers, ordinary bootstrap, executable futures and every previously retained
valid unsupported contract remain required. PA25–PA44 stay planned as whole PAs;
the separate113-case PA25 const-machine component is unchanged.
