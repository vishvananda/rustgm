# Projected subjects in source object erasure — PA31

The original core failure is `Discriminant<T>::fmt` at mem/mod.rs byte45756:
`DebugTuple::field(&self.0)` needs the associated representation to implement
Debug. Ordinary associated-bound erasure works. The actual source Unsize role
exposes an earlier head-normalization fence, so PA31 owns the reduction.

The source Unsize planner now generates ordinary object requirements when its
source is a pending projection and its target is a known object. Sized, principal,
associated equalities, auto markers, object lifetime and both endpoint formations
must all prove before conversion evidence can complete. Failed requirements leave
this unknown-head route deferred; they cannot publish a completed absence for a
possibly hidden upcast. Known object upcasts still require intrinsic principal
bounds. Raw head, equality and property queries retain their normalization fences.
No hot fields, caches, limits or backend contracts change.

The paid Send control exposed a second boundary: an internal auto-marker request
reached structural fallback without selecting the actual source application.
Abstract internal requests now retain that application as an ordinary child proof.
Its selector owns declared-result and caller premises and their parent debts;
physical shape grants no new abstract source-trait permission.

## Assignment reductions

[The PA31 packet](../pa31/tests/spec/projected-erasure/README.md) registers26 active
inputs:19 accepts and7 rejects. Pinned Rust1.98 independently object-checks every
outcome and executes every positive with output `9 23`. The cases cover ordinary
and compiler-provided DiscriminantKind representations, explicit assumptions,
field shapes, explicit/default Sized, borrowed lifetimes, associated equalities,
paid/unpaid markers, missing parent/principal proofs, escaping lifetimes and an
overlap that must not infer false absence from an unknown source head.

Three original intrinsic inputs remain byte-for-byte in `required/`. Rust accepts
and executes them, but RustGM's generated generic tuple factory reaches a const
cleanup consumer that requires an authored function header. They remain valid
unsupported requirements, separately manifested and tested at the retained
boundary. Active intrinsic counterparts form the same tuple aggregate with a
struct literal, isolating borrowed-field erasure without demanding a separate
factory body. Neither form adds a wrapper Copy implementation.

## Focused evidence

Evidence lives in B's `obj/parallel/projected-erasure/`:

- `independent-current.json` binds all26 active inputs and3 required factory
  inputs to independent object/native results and their actual source hashes.
- `public4.log`, receipt1794347113478192623,136.62s: six exact tests pass across
  frontend and traits. These include every complete LowIR/status outcome,
  retained constructor requirements, source auto-trait controls and existing
  intrinsic-only source upcasts. Strict selected Clippy, formatting and
  architecture pass.
- Width8/32/128 work is at most4x+1024 per fourfold growth across inference,
  canonicalization, trait/projection selection and steps, unsizing inputs and
  method steps/probes. Adding128 unrelated types changes none of those counters.
- `controls.log`, receipt1794347310678698728,41.64s: four existing source
  unsizing/coercion groups and raw comparison/equation/identity guards pass.
- `harness.log`:315 harness tests,468 roots/13 binaries/44 stages and all128
  upstream fixtures/126 Reference files pass.
- `native.log`, receipt1794347259020498447,45.36s: all19 new programs execute
  through the pinned CPPGM backend and host C observer, matching Rust's `9 23`.

The first packet run retained four positive failures. The tuple-factory
requirement was separated without deleting its original bytes or acceptance
contract; the Send query was repaired. A later run stopped at formatting and
another at the selected Clippy function-size guard before executing tests. All
failed receipts remain. Temporary diagnostic code was restored before the
production implementation. Diagnostic binaries and traces are analysis-only.
No text oracles were generated or changed. Whole PA31 remains planned.

Immutable dev-fast CLI
`0107a6901003ea9f5c9bcefa2810d414ddb0508bfc774caccd75c7de585d808d`
binds compiler b642ca4a8f626df0c42d6b3169cbeeace3daf619, an empty patch and3,887
compiler/CLI/test Rust hashes. `measure/passed.json` records40 immutable A/A and
two paired ABBA observations: complete ordinary LowIR, two accepted source-control
type outputs and74 frozen outcomes remain equal. Candidate/baseline ratios are
0.981/1.000/0.999/0.991; A/A ratios0.821/0.981/0.859/1.026. No speed claim.
Changed-path growth is covered by the eight-counter source invariant above.

The predecessor source-coercion full CI42900791503 on8ac237506 passed every stage.
Full42909829598 on3273d5032 is active for this semantic successor; its remaining
stages are pending. Do not cancel it with a new full dispatch. Original source hashes,
profile and observation window remain fixed; no original core/alloc/std artifact
is claimed here. Earlier optimized nested-proof controls remain pending.

## Original core observation

The unchanged120-second observation reaches its infrastructure deadline at
121.1915seconds. The retained partial stream has230 failure records and4,354
item attempts, ending at function8923,file232 byte10709,node466 in str/mod.rs.
All722 original hashes, six configuration bindings, compiler limits and profile
remain unchanged. Progress SHA-256 is
`b377219abcf86a5d6d31960dbc657f2dcf82c91923e612a9954504eeb445ed7b`.

No mem/mod.rs failure remains in the recorded prefix: Discriminant<T>::fmt's
previous object-proof fence is absent. The first retained failure is now
request_by_type_tag at error.rs,file78 byte15853, again an unambiguous-trait-goal
error. It constructs Tagged<TaggedOption<'a,I>>, calls provide(as_request()) and
returns the projected value. The following as_request body separately reaches a
scalar-operation fence. Reduce the actual tags::Type/Reified bounds and selected
receiver requirements before changing another shared kernel.

Prefixes end at different items. Their raw failure counts are not whole-core
performance evidence. This remains an incomplete census and produces no original
core/alloc/std artifact. Both temporary diagnostic binaries were retired by scoped
prune1794347695205404555; their sources, hashes and raw traces remain retained.

After publication, the superseded6549 comparison binary was retired by scoped
prune1794347786790155449. Its metadata and observations remain; rerunning a new
comparison requires restoring and verifying the recorded binary, not assuming
the old executable still exists.
