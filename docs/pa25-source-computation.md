# Open-lifetime initializer source computation

The initial checkpoint computes a selected associated initializer's primitive
source value when its lifetime arguments remain open in the original caller.
The closed-result extension below admits additional already canonical results. It is a
bounded PA25 step, not completion of the assignment or of all associated-value
contexts. Ordinary materialized `Target::Associated` still requires concrete
arguments and a uniquely validated canonical result.

`Target::SourceAssociated` carries an opaque `SourceComputation` and the exact
expected result. Only the semantic minting path can construct the witness. Its
full selected `Instance`, original initializer source and caller source are
validated against the actual parked caller dispatcher, its reservation, the
current value frame and its original `PreparedScope`. No dispatcher is cloned.
The caller's complete scope identity includes body, declaration owner, binders
and pending request boundary. Success, suspension and errors preserve the same
caller allocator and foreign gaps. No extra debt ledger is introduced.

The initial family requires a checked original const initializer with a closed
primitive result. Captured lifetimes can remain open; other type/const arguments
must already be closed. The actual original declaration still checks its own
patterns. Substituted value selection uses the actual requesting caller, as
established by the preceding PA20 correction.

A fresh dispatcher cannot claim existing inference aliases merely because they
have the same source owner. The first private probe rejected that incorrect
hypothesis: argument region 1 was linked to alias 2 and body slot 0, while the
fresh dispatcher owned only region 7. The trace is retained in
`obj/parallel/pa25-source-computation/caller-gdb.log`.

The original `PreparedScope` now retains exact ranges for the main solved and
late canonical equality publications. Checked index+1 endpoints use four
`NonZeroU32` values; the optional record occupies 16 bytes. Both ranges are sealed
after late publication and before the scope escapes. The first source request
validates complete ranges and body identity, then lazily reuses the existing
`RegionEquations` and `Bound` machinery to recover only static or original body
slot anchors. Missing anchors, foreign owners and stale ranges reject. Genuine
nested binders remain local. The original equalities remain ordinary obligations;
this transport does not prove them. One folder belongs to the exact parent frame,
with its capacity and traversal work charged through the existing continuation
owner. Preflight bounds precede graph construction.

The provider checks the original initializer contract, specializes the full
instance with required premises, and runs the ordinary const eligibility,
dispatch, promotion, region, ownership and loan checks. Its private execution
code is marked source-only and is not registered as a materialized associated
body. Return validation remains unchanged, but source-only return cannot publish
a typed completed value. Query forwarding retains the established source result
with `ty=None`. Inspection reports `status=source-complete result=deferred`,
without manufacturing a typed value root. Cache reuse preserves each caller's
mandatory proof obligations; unpaid caller bounds still reject in checked views.

The seven new declared programs are independently compiled to full Rust 1.98.0
objects: five accept and two reject for real lifetime requirements. Their three
frontend modes retain the staged distinction: type inspection can defer the
negative callers, while constants and LowIR reject them. All five positives have
paired Rust/CPPGM native value checks. The unchanged original
`interleaved-lifetime.rs` is promoted from a recorded implementation boundary to
an accepted/native case. Its opposing unpaid-bound source remains unchanged;
only its explicit staged diagnostic contract changes. Opaque original generic
patterns, missing caller bounds and forbidden static equivalence remain rejected.

Qualification across the fourth/seventh batches covers 31 private functions,
including original initializer cache/publication controls, exact restoration,
foreign/stale source witnesses, unchanged ordinary concreteness, and range
adversaries. The final seventh batch passes all 19 affected integration functions,
including six backend-dependent groups executing 40 paired native programs.
Strict release all-target/all-feature Clippy, file/architecture audit (1,525 files,
zero findings), layout and upstream checks pass. All 7,788 PA1–23 text comparisons
and all 467 published PA24 comparisons pass unchanged. PA24 is read-only export
66455db6; archive SHA-256 is
`2e0304e1a97aaeb3ef4bfc8172dac115f6dea0f62ee3c880527467c07e076612`.
No generated references were changed.

The final source-map digest is
`0b0924af39babf6254ccb98771797ce74dc8aaa12ffc3b022f1928420108c3b7`
(890 sorted production Rust files). The debug candidate is
`22f7a26122bd930f2f51ac3d4c27f082bc1f2ccad451ad4c28fba4baf525b0a4`.
Evidence lives under `obj/parallel/pa25-source-computation/`. The final functional
receipt `d28df7c2d4ac4722b0e31352870918f1` ended zero with 20,424,278,016 bytes
retained within the unchanged worker budget. Earlier compile, ownership,
lint/placement and launch failures remain recorded and are not passing evidence.

Measured layouts: source witness 24 bytes; Target 28 to 32; Request 56 unchanged;
roots-map entry `(Target, usize)` 40 unchanged; Catalog 192 unchanged; Code 328
unchanged. Pending-pattern Scope grows 64 to 80, PreparedScope 160 to 176, and
Frame 1,168 to 1,192. The folder allocates only on a source request; bodies without
pending patterns allocate no Scope/Frame catalog. The immutable ordinary512 and
constcall64 controls produce identical inspection bytes with the frozen PA24
binary and this candidate. This is output/layout evidence, not timing or RSS
neutrality. The next combined immutable performance study must cover those
ordinary controls and changed-path demands. Prior unexplained scalar RSS evidence
remains unresolved.

Required remaining work includes open-reference or projected results,
open type/const argument contexts, and source computation outside the prepared
pattern-caller path, including nested non-pattern contexts. Those need exact
existing caller proof integration and result normalization debts; no static
substitution or source selection alone can authorize them. PA25 allocation,
validity and sequence-view work owned in the other lanes remains separate. This
checkpoint does not activate PA25 or broaden PA29 metadata.

## Closed results

The next bounded family accepts already canonical tuples, fixed arrays and
nominals, plus shared references whose declared result region is static. The
original initializer's exact result TypeId must still equal the request. Cached
Store properties must show no parameters, projections or outer region context;
this check performs no result substitution or speculative normalization. A
caller-bound reference, including one inside a tuple, remains outside this
family. Standalone mutable references and raw pointers are not added.

Nothing changes the source witness, original/template/instance proof, structural
pattern equality, final escape checks, caller obligations or untyped completed
payload. Full witness validation still occurs before cache reuse and on external
resumption. No sealed reuse cache is introduced. The helper adds no retained
field, map or allocation.

Declared fixtures under `lifetime-initializer-results` cover tuples, arrays,
derived records/enums, promoted/static shared references and a tuple containing
a static reference. Seven positives and two negatives are independently compiled
to full Rust objects before changing the result gate. The dangling result fails
its lifetime check, and the nonstructural record fails the existing PartialEq
pattern requirement. All seven positive values have paired native checks. The
original paid/unpaid caller, foreign witness and cache/publication tests remain
part of the focused qualification.

Performance inputs under
`obj/parallel/pa25-source-computation/performance/inputs` retain distinct open
caller counts 16/64/256 and a wide-argument/forward-callee matrix. The qualified
manifest records every input and output hash. Common ordinary512/constcall64
inputs remain unchanged under `pa25-inline-const/performance`; root owns the
combined immutable measurements. The original wide forward-callee family has one source advance and no external
suspension, with 8/32 internal machine preparations. A separate pending-pattern
family retains widths 8/32/128 and 8/32 callee scopes: every width records exactly
9/33 source advances and 8/32 external preparations. A private invariant delegates
to the actual checked provider and verifies those counts. All witness checks stay
in place. These are preparation counts, not timing or RSS claims.

The closed-result functional batches pass all 11 selected-initializer integration
functions, including four native groups with 24 paired programs (seven new).
Four private functions cover witness rejection/layout, the closed-result gate and
the external resumption matrix. The closing batch reruns the two new source/native
functions after the lint-only nested-pattern correction. Strict release
all-target/all-feature Clippy, file/architecture audit (1,526 files, no findings),
layout/upstream and all 8,255 active text comparisons pass. Every new scaling
input also passes pinned full-object Rust, constants and LowIR. Failed author
syntax, expected-diagnostic and lint observations remain retained separately.

Closing evidence is `obj/parallel/pa25-closed-results/final-second/`; the merged
15-input qualified manifest has SHA-256
`dc177517a9bc79ffb77ab27e33522cb07caa7662373789bfe9b5bf9f64f89092`.
The immutable default-feature debug candidate is `pins/rustgm`, SHA-256
`c19e4bac987cba4be6f54a88fb71d8a50c619421cd5524d4a8888da50fce0c08`.
The 890-file production source map is
`81e33b28c8b44cb574e1101c3c0e86d17e56fec49e335c6b8f5f7043d1b26a87`.
Receipt `4058ce5c61e542a78afcdf3e41874bd5` ended zero with 20,827,602,944
retained bytes within the unchanged worker budget. Common-control output hashes
remain equal; no performance neutrality or RSS improvement is claimed.

## Open results in prepared pattern callers

The projection-free source path also accepts canonical shared-reference results
and aggregates containing them when the selected arguments retain caller-owned
lifetimes. This is a source computation, not a materialized constant. The full
selected instance and the exact original initializer remain mandatory; substituting
the declared result through that instance must produce the exact requested type.
Projection normalization is not attempted by this gate. The prior closed-result
path keeps its exact equality check and does not gain an extra contract obligation.

The request's selected arguments and result first pass through the same retained
PreparedScope equality folder and parked Dispatcher normalization. Reservation
restoration and all caller debts remain unchanged. Source witness validation is
repeated on cache reuse and every external resume, including its arity, argument
kind, original source and caller-region ownership checks.

Initializer specialization imports the requested type through the same
`Bound::universals` fold used for its selected arguments. It preserves the actual
body result in the original initializer's universe and adds an ordinary required
coercion from that result to the imported requested type in the actual instance
Contract. A private result interface records the request, imported type and exact
obligation index. It verifies identity and the unchanged coercion before returning
the instance body to the provider. It carries no proof authority: the provider
still verifies the original initializer and the complete instance, including the
captured-lifetime identity assumptions and all original debts, before installing
Code or publishing a cached value. Ordinary Associated requests retain their
concreteness and exact materialized-result gates. Callable permission is unchanged.

The completed source payload continues to have `ty=None`; inspection displays
`status=source-complete result=deferred`. It cannot become a typed value root merely
because a reference result was computed. Source escape and structural pattern
equality checks remain in their existing consumers.

Seven declared sources in `lifetime-initializer-open` are independently compiled
to full pinned Rust 1.98 objects before implementation: shared-reference and
tuple results, paid lifetime-relation and static bounds, plus dangling storage
and opposing unpaid bounds. The four positive cases have paired native value
checks. Dangling storage rejects in every view. The two unpaid caller cases retain
the existing deferred type-inspection behavior and reject constants/LowIR at the
actual lifetime-outlives proof; unsupported-profile diagnostics do not count as
these negative checks. The private result test removes captured identity
assumptions, removes the indexed coercion and substitutes a different requested
result; each must fail at its owning identity or ordinary proof check.

Projected result types, open type/const arguments, and nested or non-pattern
source computations remain required followups. Projected results must retain
normalization debts through selected original/instance proof and typed escape
validation. Nested/non-pattern paths must use their actual retained caller proof
owner; a fabricated PreparedScope or source key alone cannot supply authority.
No new caller ledger or global scope scan is introduced here.

The open-result batches pass all 13 selected-initializer functions: eight ordinary
source/inventory functions and five explicitly ignored native groups, executing
28 paired native programs (four new). Six private functions cover the result
interface, full witness identity/layout and external suspension matrix, plus
ordinary source-debt preservation. Strict release all-target/all-feature Clippy,
the file/architecture audit (1,578 files, zero findings), layout/upstream and all
8,255 active PA1–24 text comparisons pass unchanged. The first private attempt
failed because its manually prepared body lacked promotion facts; that failed
setup and the corrected ordinary promotion/proof path are retained separately.

The request and persistent catalog layouts remain unchanged: witness 24 bytes,
Target 32, Request 56, roots entry 40, Catalog 192 and Code 328. The private
ResultInterface is 16 bytes; its source-only wrapper is 768 bytes versus 744
for Materialized. The existing specialization entry now delegates to a shared
optional-result helper with `None`; no retained field, map or ordinary-path
allocation is added. Common ordinary512/constcall64 inspection hashes remain
equal. This is output/layout evidence, not timing or RSS neutrality; root owns
the combined measurements and the prior scalar RSS limitation remains open.

Evidence is under `obj/parallel/pa25-open-results/{independent,first,second,third,final}`.
The final immutable default-feature debug candidate `final/pins/rustgm` has SHA-256
`36aa6f93fc00ea726a381508278224277416aefabdf9a0318c3edd3aa3101fd0`;
the 909-file production source map is
`e671f1fb5d66c1a5f13d7822f262856025fc673973f487bae08d4dd92c7bdf7b`.
Closing resource receipt `d198de70c10242ffa22dd923d791e24b` ended zero with
21,365,092,352 retained bytes. No limits were changed, no new performance
observations were selected, and no generated reference was edited.
