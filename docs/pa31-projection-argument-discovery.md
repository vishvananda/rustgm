# Associated-result regions during source coercion discovery — PA31

Original core's request_by_type_tag constructs TaggedOption<'a,I>, calls
provide(as_request()) and returns the projected payload. With genuine source
CoerceUnsized present, an unannotated construction incorrectly requires a nominal
conversion. Ordinary Rust and the control without that source role pass the call.
An explicit expected type also passes. PA31 owns this protocol reduction.

A bounded trace of the source reduction identifies the exact query: source
Opt<Projection(I, fresh-region1)> and target Opt<Projection(I, bound-region0)>.
Both occurrences have independently completed opaque normalization. Their member,
Self and complete input shapes agree; only their lifetime IDs differ. Comparing
argument-layout identities makes this appear to be a different constructor, so
the filter requests Opt<…>: CoerceUnsized<Opt<…>> without an implementation.

The unsizing exclusion query now reuses the existing normalized shape relation
for these independently completed associated results. A region-only difference
cannot establish a forced constructor mismatch. The ordinary comparison and
raw equality queries retain their prior behavior. No type variable, lifetime,
parent application, trait or conversion permission is inferred by this filter.
The eventual ordinary equality, lifetime, formation and source proofs still run.
Once a complete canonical opaque input shape is compared, the outer paired walk
continues with its other queued siblings rather than revisiting that shape's
nested inputs. Known sibling mismatches remain discoverable. Invalid identities and exhausted
work remain errors. No hot fields, caches, limits or backend contracts change.

The iterative paired graph walk and constructor compatibility dispatch are now
separate functions in the same module. This keeps each operation within the
strict function-size limit and preserves the old constructor cases and traversal.

## Assignment reductions

[The PA31 packet](../pa31/tests/spec/projection-argument-discovery/README.md)
registers26 outcomes:19 accept and7 reject. Pinned Rust1.98 independently checks
every object outcome and executes all19 positives with output `9 23` before the
compiler edit. All sources and the observer have manifest hashes. Cases retain
real generic payload movement, free-function/method forwarding, nested fields,
borrowed representations, trait and GAT lifetime arguments and mixed type/const
inputs. Missing parent proof, wrong result/member/Self/type/const inputs and an
escaping borrowed value remain rejections.

The minimal payload/forwarding cases need neither TypeId nor trait objects.
The fuller original-shaped probe separately reaches as_request's scalar-cast
boundary, which remains required. No fake runtime helper or unreachable body is
used by the active native packet. Original upstream bytes and provenance remain
unchanged; whole PA31 remains planned.

## Focused evidence

Evidence is in B's `obj/parallel/tagged-request/`:

- `independent-final.json` binds all26 final inputs to Rust object outcomes and
  all19 Rust native results. The earlier source probe's mechanical name replacement
  and a missing CoerceUnsized prerequisite for its DispatchFromDyn control were
  caught independently; original setup bytes and errors remain retained.
- `public4.log`, receipt1794349472569058821,91.03s: six exact tests pass across
  projections and frontend. All26 complete LowIR/status outcomes, eight-counter
  width8/32/128/noise128 bounds, raw normalization/equality/identity/budget guards,
  previous projected erasure and independent property/noninjectivity controls pass.
- Raw normal-form guards preserve Self/type/const/member and trait/family group
  distinctions, leave region facts unchanged and retain ordinary equality failure.
- `native.log`, receipt1794349618413134907,52.96s:19 new programs and the existing
  nominal-wrapper transfer/cleanup observer execute through CPPGM and match Rust.
- `controls.log`, receipt1794349754737734661,48.25s: six exact controls pass across
  frontend, traits and memory. These retain the earlier source unsizing and coercion
  outcomes, pointer region/ownership and receiver reference/slice behavior, plus
  the separately required tuple-factory cleanup boundary.

`harness.log` passes315 harness tests,468 roots/13 binaries/44 stages,128 pinned
upstream fixtures and126 Reference files. The initial implementation passed functional checks but its nested-depth guard
failed:205 steps at depth8 and2,737 at depth32 (receipt1794350289219220965).
The paired walk was repeating complete shape comparison at every nested opaque
operation. `public5.log`, receipt1794350414882768771,88.83s, passes seven exact
tests after stopping that duplicate descent, including all26 outcomes and the
8/32/128 depth invariant. Query work stays at most64×depth+128 and4×previous+128;
other queued siblings, raw fences and the work budget remain checked.
`native-final.log`, receipt1794350534890245310,54.12s, repeats all20 real native
programs after that algorithm correction. The before/after receipts are retained.

Strict selected production/CLI/test
Clippy, full formatting and architecture pass.
The initial name exceeded the projection filename-stem ownership limit; later
attempts stopped at the constructor dispatch's103-line limit and a raw fixture's
u32/u16 region-index mismatch. They executed no tests and remain failed receipts.
No blanket lint allowance, repeated private-harness build, release build, oracle
change or backend-contract change was added. Temporary tracing was restored before
implementation; its binary is analysis-only.

Full42909829598 on3273d5032 passed every stage for the preceding erasure
checkpoint. This newer semantic change requires its own full successor after
publication; never cancel a live full run by redispatching. Original core remains unbuilt.

## Immutable controls and original core

Final dev-fast CLI
`e06cd0d9d8cb3cfc0835c2f766f5f2a014e38cbcf853e7cc90e4fd54a3c2b4dc`
binds compiler96548ba413a3555d2acf5ff73cd29de213462341, an empty patch and3,888
compiler/CLI/test Rust hashes. Forty immutable A/A and two paired ABBA observations
retain complete ordinary LowIR, two accepted source-control type outputs and74
frozen outcomes. Ratios are1.008/1.056/1.016/0.982; A/A ratios0.648/0.841/1.005/0.999.
These preserve output equality, without a speed claim. The nested and call-width
work invariants above cover the actual changed path. The initial7892 candidate
was retired by scoped prune1794350742473176755; its metadata remains retained.
The diagnostic02bd trace was also retired after its exact sources were restored.

The unchanged120-second original observation fails its infrastructure deadline at
121.1873seconds. Its partial stream has229 failure records and4,426 item attempts,
ending at function8996,file232 byte110334,node2429 in str/mod.rs. All722 original
source hashes, six configuration bindings, profile, compiler limits and observer
window remain fixed. Progress SHA-256 is
`832a38f90433b1590380edf7a68880c08a9ac646a3ba93de793f61ad353d8ef0`.

No request_by_type_tag failure remains in the recorded prefix. The first failure
is now as_request at error.rs,file78 byte34589: operation requires a scalar type.
Its explicit cast from &mut Tagged<TaggedOption<'a,I>> to
&mut Tagged<dyn Erased<'a>+'a> already fails an independently accepted ordinary
reduction; implicit erasure passes. First reduce nominal DST reference casts
using an ordinary slice tail, then retain trait objects or source roles if needed.
This determines the earliest PA rather than assigning the whole core call toPA31.

The prefixes end at different items, so raw counts are not whole-core performance
evidence. No complete census, checked-body count or original core artifact exists.
