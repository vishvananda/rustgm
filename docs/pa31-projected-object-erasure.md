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

The predecessor source-coercion full CI42900791503 on8ac237506 passed every stage.
This semantic successor still requires its own full CI. Original source hashes,
profile and observation window remain fixed; no original core/alloc/std artifact
is claimed here. Earlier optimized nested-proof controls remain pending.
