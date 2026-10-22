# Actual-parent source port onto the frontend performance checkpoint

This is a source-only adaptation of `73353ab9dde5fc5c6b5091f5a6710f68537997de`
onto `46d8b82a`. The original remains on `pa25-alias-demand-work`; the successor
is `pa25-parent-perf-port-work`. No build, test, probe, benchmark or resource job
has run for this adaptation. Formatting and source review are complete; compiler
acceptance, audits, layouts, work measurements and all source outcomes remain
unvalidated until the integrator releases the resource hold. The earlier
[paid-parent receipts](pa25-paid-parents.md) qualify only the original checkpoint.

The six textual conflicts preserve both sides' semantics: layout-result caching
and scoped constant Code indexes; current demand and final assembly; structural
verification reuse and actual paired proofs; the layout executor and parent
scopes; compact NodeId access; and Fx-backed inline values with cold journals.
All new compiler-owned hash collections use the existing Fx aliases. No central
typed API, SourceLayout permission, hot Value/Target representation or opaque
publication boundary is added or broadened by the port.

The structural cache still supplies only substitution-invariant CFG/storage
facts. `PairedProved` contains the full actual `incremental::Proved` produced by
`Verifier::prove`, the immutable paired Contract and the exact completed group
lineage. Context-only original rows, a structural `Proved`, and equal numeric
region IDs cannot activate a child. Actual ownership, initialization,
destruction, region, loan, escape and eligibility checks remain mandatory.

The private Pending row now records its parent's actual numbering unit once.
Preparation checks each exact child's unit; scope entry and every resumed child
request revalidate the recorded parent and child ownership using
`BodyRegions::unit_of`. The bridge still comes only from that exact retained
Contract, original Summary and the fold-produced slot map. No allocation-log,
suffix or numeric-frontier scan supplies ownership. Unit lookup counts are added
to canonical work on success and failure; these count logical indexed owner
queries, not instructions or the internal parent-link hops of `unit_of`.

The new unrun private controls cover coincident IDs in foreign units, a changed
shared parent/child unit, valid nested shared units with unrelated allocations
interleaved, cold/warm exact requests, and unrelated allocation suffixes of
0/128/4096 at 1/8/32/128 ports. Their preparation invariant is
`values + bridge slots + 2 * ports + 1` logical visits for the neutral fixture.
No measured counts from the old checkpoint are substituted for port results.
The added retained unit is one NodeKey per conditional Pending row; actual
record sizes, padding and the full pipeline's affine work must be remeasured.
The ordinary path still allocates neither Pending nor its scoped sidecar.

After release, the focused gate should compile all targets, run the original
group/ancestor-failure/port/journal/index/retirement controls plus these three new
numbering tests, and retain the existing numbering-unit and structural-cache
equivalence controls. Then rerun the unchanged closure and paid-lifetime
positives through checked constants, LowIR and native execution, their unpaid,
foreign-owner and faulting-child opposites, selected_initializers and both legacy
caller/lookup suites through the registered maintainer runner, strict Clippy,
architecture/registration checks and the current active text fixtures. Root owns
the combined full release and performance qualification. Any actual layout or
diagnostic delta must be recorded before central reference review.

Generic/SourceInitializer and additional conditional-parent families remain
required followups. This source port makes no assignment activation, oracle,
performance or end-to-end acceptance claim.
