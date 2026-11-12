# Computed matches in capturing closures

Pinned core filter_map_fold fails at filter_map.rs:39, file141:1253, on
`match f(item)`. The existing capture collector has no Match case. It now uses
the already selected match decision and arm facts to visit the computed input,
guards and arm values. Callback receiver selection, stored fields, callable
kind, ownership and branch execution retain their existing consumers. No new
engine, cache field, permission, backend, storage policy or compiler limit is added.

A match directly inspecting a captured place has a different contract: a tag
read cannot consume its non-Copy payload, and wildcards or independent fields
may require no capture. Those positive contracts remain explicitly required.
The new path rejects such places rather than silently treating them as value
moves. It does support local scrutinees and computed callback results, including
the actual generic filter_map_fold and source try-fold helper.

Forty-nine independently declared Rust1.98 outcomes are41accept/8reject. Forty
complete API outcomes qualify, including20 actual-source outcomes, both callback
variants, guards, copied/mutable/owned arm uses and repeated-owned-call opponents.
Nine entire positive outcomes remain required: four captured-place contrasts per
ordinary/source owner and one ordinary generic Try import. The original public
helper enum native input also retains a required CLI metadata artifact: complete
API lowering and actual native execution pass, while exposed enum publication
hits the existing scalar metadata boundary. Its bytes remain serialized; a
separately declared private-helper variant isolates Match and completes the CLI.
No valid source is reclassified as a language rejection.

Focused receipt1794457800427825291 passes four exact tests/two targets in129.39s;
the private-helper follow-up1794458273040726694 passes its affected test in31.93s.
Both include full formatting, architecture and strict selected production/CLI/
test Clippy. The new ordinary and actual-source programs execute through CPPGM,
matching independent Rust/C stdout `match-capture 7 9 12314`. The first callback
returns captured C input7 through Some, the None branch retains accumulator9,
and the fold callback runs once; C observes exact operation order12314. The
previous actual try protocol7/43/23/123 also executes successfully.

Widths8/32/128 retain one field. Ordinary inference349/733/2269, canonical115/
427/1675 and trait selections0; source inference1466/1850/3386, canonical1521/
1833/3081 and trait selections5. Adding128 unrelated types changes no counter.
These bounded-work observations make no optimized or original-core speed claim.

Nineteen new ordinary course outcomes (15accept/4reject) complete the CLI and
normal text contract. All15 positives independently cross the Rust/CPPGM object
bridge before generation;14 explicitly empty objects contain no defined code or
data in either compiler. Native execution is separately established for the one
real exported program. Normal scoped generation and exact text checks then
produce19 reviewed triples. All28,145 prior reference hashes remain unchanged.
PA24 now has661 cases,474accept/187reject,396 runtime comparisons,5 codegen-only
controls and73 object controls. No planned assignment is activated.

Initial setup failures remain in raw evidence: missing bool Copy in the source
foundation caused an independent Rust ICE, and missing whitespace caused a native
syntax failure. Revised foundations/declarations were checked independently before
candidate execution. The first ordinary public-enum CLI attempt exposed the retained
metadata requirement; a standard pre-generation native-check attempt could not
read not-yet-created warning references. No oracles were produced by either failed
attempt. Direct declared-warning and object checks established all positive results
before normal generation. References are neither hand-edited nor blessed to hide
an unsupported positive.

Upstream, maintainer harness and cleanup all pass;28,203 current reference hashes
remain unchanged across those checks.

CLI 623ff5bc248b5f8d722ade9e71cf2ed486328cc6b2271fa8a46b1aef3b220204 has the local
source/text/native evidence. Clean freeze, immutable accepting comparison and the
unchanged120s original observer are next. No core, alloc or std artifact exists.
Full43154313816 remains active on exact01e7013/sourcec388: fast, lint/architecture/
harness and host graph pass; release/course is running and later stages remain
pending. Do not restart it for this descendant. Preserve all older requirements,
stable5d until full success and optimized21f. Preceding e8 has been retired after
publication and comparison; its producer/raw evidence remain, its executable is absent.

The [machine record](pa24-match-captures-evidence.json) binds the raw checkpoints.
