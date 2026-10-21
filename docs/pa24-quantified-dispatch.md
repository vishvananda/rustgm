# Quantified captured callable conversions

Captured closures with higher-ranked public signatures now select executable
object entries under an explicit lifetime scope. PA24 remains planned.

The conversion reserves fresh universal aliases for the public call binder,
opens that signature once and selects the ordinary checked Fn/FnMut entries.
The temporary contract retains the real caller identity and diagnostic owner,
but copies no caller assumptions, output conclusions or body vectors. Selection
only emits obligations. Its normalization proofs are grafted below the new
scope, preserving private witnesses and nested quantifiers; actual outer
lifetimes cross that scope through the ordinary binder substitution. The final
caller still pays every lifetime, source-condition and loan requirement.

The existing lazy proof cursor owns the allocation frontier. Opening a public
binder and retaining subsequent normalization proofs share that cursor, so
each conversion does not scan all preceding caller proofs. Projection-free
ordinary entries retain their prior lazy path. Physical vtable sharing still
occurs only after the source body is sealed; it supplies no source permission.

An independent borrowed-result control exposed a separate object-requirement
bug. In `fn run<'a>(x: &'a u8)`, converting `move |_: &u8| x` to
`dyn for<'b> Fn(&'b u8) -> &'a u8` retained the caller lifetime as a bound slot
when entering the local query. Opening the object's binder then made that slot
look local to the proof. The fix aliases escaping inferred conversion inputs
through the current body's existing universal mapping before constructing
requirements. Object-local and nested callable binders retain their identities.
Bodies without caller binders skip this inspection. Ordinary nonclosure
conversions without an object quantifier retain their prior diagnostic ownership;
only quantified objects and closure demand templates need the aliasing boundary.
Inherited callable demands
consume the same aliased inputs before ordinary requirement registration.

## Independently declared controls

`pa24/tests/quantified-dispatch-outcomes.json` registers 22 new controls: sixteen
positive programs and six language rejections. The existing
`captured-dispatch/quantified-capture.rs` becomes an executable acceptance.
The nonclosure associated-output control belongs to PA23's conversion family,
bringing that assignment to 400 cases and 86 native executions. No PA24 text
oracles or activation changes are included.

The controls cover Fn/FnMut and weaker permissions, identity and external
reference results, distinct input and outer lifetimes, nested callable
signatures, generic owners and consumers, projected captures, zero-sized
captures, upcasts and a non-consuming FnOnce view. Opposing controls retain
local/static escapes, wrong outer output lifetimes, missing call permission,
conflicting loans and a stronger requirement after an earlier valid conversion.

Every declared outcome was checked with pinned Rust 1.98 object compilation
before promotion. Eighteen positive programs then executed with both rustc and
the CPPGM backend; two frontend runs also produced identical LowIR bytes for
each input. Evidence is in
`obj/parallel/pa24/quantified-objects/native-second/`. The initial borrowed-result
failure, direct-call/unused-object contrasts and debugger proof observations
remain under `first/` and `followup/`. They distinguish the ordinary Output
proof failure from the new conversion-entry proof.

Two private proof tests require the caller's actual outlives assumption and
reject local universals escaping to static or an outer caller. Grafted private
witnesses and nested universal aliases keep their original ownership.

## Development scaling and integration

At widths 8/32/128, repeated quantified conversions retain one physical table,
two captured targets, one receiver adapter and one virtual-call forwarder.
Table planning takes 257/881/3,377 steps. Distinct generic specializations retain
8/32/128 tables and adapters, with 16/64/256 captured targets; planning takes
168/480/1,728 steps. Both measured series are affine. These are table-planning
counters, not total frontend work or a wall-time comparison.

The combined candidate includes the separately qualified nested receiver,
non-Copy pointer transfer and inherited object-inference checkpoints. Its strict
Clippy check, file audit and all 594 library invariants pass. The first combined
audit rejected a seventh test filename with the same prefix; the tests now
belong to the existing captured-dispatch module. That failure is retained under
`combined/`. An initial broader lifetime aliasing change shifted four existing
PA23 diagnostic locations. The narrowed boundary preserves all 399 prior case
outputs byte for byte; that failed comparison remains under `combined-finish/`.

Final focused integration under `combined-third/` passed formatting, strict
Clippy, the 1,457-file audit, all 594 library invariants, 62 source tests,
21 native tests, upstream verification and all PA1–PA23 text suites. The full
PA23 receipt refresh was required by its legacy format and ran only after the
399-case comparison passed; it adds the independently checked new conversion
case and per-case contract metadata. Existing text oracles are unchanged.
Projected closure probes at widths 8/32/128 also passed rustc and candidate
compilation. Full release and immutable performance checks remain publication
requirements.
Reference-Self captured objects and the documented inherited quantified-input
inference boundaries remain separate implementation work. Unsized consuming
FnOnce execution is not supplied by the non-consuming object view.
