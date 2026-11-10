# Associated-result input regions during source coercion discovery (PA31)

Original error.rs request_by_type_tag constructs TaggedOption<'a,I>, passes it to
an Error receiver and returns the projected payload. An ordinary control passes
the call; actual source CoerceUnsized is necessary for the failure. Minimal
payload and free-function forwarding reductions retain it without TypeId, dyn
objects or the separately unsupported as_request cast. Therefore PA31 owns them.

The bounded trace has Opt<Projection(I, fresh-region1)> and
Opt<Projection(I, bound-region0)>; both result occurrences are independently
normalized. The source nominal filter interprets their different argument-layout
IDs as a forced constructor mismatch and requests an actual CoerceUnsized impl.
Existing normalized shape comparison can recognize a region-only difference
without equating types or lifetimes. Full ordinary equality, parent, formation,
region, move and cleanup proofs remain authoritative.

All inputs and the C observer have manifest hashes. Outcomes are independently
object-checked before implementation. Native positives must return9 and23,
including a real borrowed representation and complete generic forwarding.
The original upstream files and provenance are unchanged. No text oracle is
created; whole PA31 remains planned.

The call-width cases check eight counters at8/32/128 and128 unrelated types.
A separate typed boundary test grows a chain of independently completed opaque
operations at depths8/32/128. Comparing a complete canonical input shape must not
repeat the same traversal at each nested operation. Remaining queued siblings and
ordinary equality/parent/lifetime proofs keep their existing requirements.
