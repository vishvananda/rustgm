# Named-pattern dependency readiness

This draft checkpoint extends the existing declaration-time constant query graph.
It does not activate PA25 or change the closed selected-initializer profile.

A completed checker transfers its deferred work into `PreparedScope` objects.
Each object retains the original semantic body, module, owner/binder count,
ordered value requests, range checks, irrefutability roots and match checks.
The existing stable dependency ordering runs once for the batch. A value query
then advances a cursor in the same `Queries` Active/Ready graph used by ordinary
constants and discriminants. It does not recursively drain a fresh evaluator.

Typed frame IDs identify pending publication and nested initializer scopes.
Before entering another scope, the frame moves its dispatcher, pattern scope,
value scope and complete instance context into saved state. Suspension moves the
whole catalog into the existing continuation. Coverage and range publication
consume a prepared object only after every requested value completes. Successful
publication clears that body's pending marker; failure restores the saved query
metadata without publishing a replacement range or execution permission.

A parked dispatcher reserves its existing free-region interval before another
owner can allocate. On resumption it advances through the declaration allocator's
actual frontier and records foreign gaps in its proof. Publication uses the same
reservation and exact body/source. Disjoint owners do not acquire one another's
free identities, assumptions or selection caches.

All completed source scopes transfer together into the query-owned catalog,
indexed by exact semantic body. `PatternOwner` actions distinguish queued,
active and completed source scope states. A selected value may consume a later
body's queued scope before its scheduled turn; that later action then observes
completion without repeating publication. The former one-scope legacy evaluator
is removed, so every live pattern dispatcher has an active typed owner before
member inference can allocate foreign IDs. Source scope completion does not
supply instantiated CFG or caller proof authority. Ungapped, unquantified
reserved evidence publishes the same ordinary facts/origins as the previous
flat path; retained quantified evidence preserves foreign gaps.

The new catalog is lazy. Its 64 MiB retained-storage charge includes batch map
capacity, arena and active-stack capacity, prepared child vectors, moved dispatcher
payload and embedded frame headers. The dispatcher facade documented in
`pa25-pattern-retention.md` counts fresh owned caller payload and newly appended
proof children once. It excludes only the documented declaration-shared immutable
pointees and allocator/control metadata. Catalog snapshots inspect owner totals;
they do not rescan old proofs. Growth checks run before adding arena/stack storage
or partitioning a pending batch, followed by defensive aggregate checks. Inspection
visits consume the existing query work budget.

Pattern validity remains a property of the original source declaration. Pattern
requests reset the value-specialization context while selecting the source pattern;
entering the selected initializer restores its own full instance identity. A
concrete outer query cannot make an opaque generic `T::N` pattern valid. Preparing
an associated dependency in a skipped scalar branch also enters its exact value
owner, checks its named patterns and restores the parent before continuing. It
does not execute skipped arithmetic merely to prepare eligibility.

Declared outcomes and original source hashes are in
`pa25/tests/pattern-readiness-outcomes.json`. Accepted families include named
patterns in enum discriminants, forward and nested constants, const-function
callees, associated values and ranges, distinct enclosing instantiations, array
bounds, and preparation-only skipped associated dependencies. True value,
pattern and enum cycles reject through the same active graph. The original
cross-body pattern fixture is promoted without changing its source bytes.

Two independently accepted Rust inputs remain uncovered positives: a reached
callee cast to an unfinished enum discriminant, and checked CFG execution of an
associated initializer with open caller lifetime arguments. The opposing missing
lifetime-debt fixture currently rejects at the same closed-materialization
boundary; it is not evidence that its intended outlives proof was discharged.
These are remaining PA25 requirements, not excluded assignment scope. The next
slice must carry exact caller/context proof authority and discriminant readiness
without weakening `arguments_are_concrete`, mapping lifetimes to static, or
publishing placeholder discriminants.

Qualification is retained under `obj/parallel/pa25-pattern-readiness/`. The
`readiness-second` run passes the three private frame tests and all nine source/
native constant-pattern functions, then fails strict Clippy on three small
findings. The preparation probe retains a new valid-Rust skipped dependency that
exposed a missing owner frame. Its correction and the subsequent closing gates
are recorded separately. The third guard was canceled while queued (exit 130,
no functional observations) after spotting an invalid test-only private API use.
No references were generated or assignment status changed in this lane.

The fourth guard retains passing production evidence for all nine constant-pattern
functions, including 15 newly accepted native values, after fixing preparation-
only ownership. Its interval suite stopped at a test-only private-field access;
that test now reads through the public initializer/Reader interfaces. The fifth
guard was canceled while queued (130, no observations) to consolidate the
exterior-owner audit fix and its controls into the closing sixth batch.

The sixth run passes four private frame tests, then the concrete exterior source
exposes the still-external PreparedScope batch (two source/native failures).
A guarded GDB trace in `exterior-trace/trace.log` identifies the original
Checker→one-scope evaluator path; no production diagnostic logging was added.
The seventh batch qualifies the complete ordered-batch transfer and exact-source
catalog, including both foreign-ID interleaving invariants and that unchanged
source. Earlier passing evidence remains retained, not relabeled as a pass for
the revised source.

The seventh all-target check passes. Its private matrix identifies one stale
test assumption: the ungapped child now correctly publishes ordinary flat facts
rather than a quantified container. The eighth run checks the same production
source with that assertion corrected, preserving exact child equalities and its
Inference origin while the parent still retains the foreign-gap proof.

The eighth batch passes all four private controls and all nine pattern functions,
including the concrete exterior regression and sixteen accepted native cases.
The graph/reuse family records `(scopes, graph visits, discriminant visits)` as
`(8,24,8)`, `(32,96,32)`, `(128,384,128)`. Repeated value demands do not add graph
publication or CFG executions. These are graph/entry visits, not total scheduling
instructions; heap comparisons, sort work and other query operations are not
claimed linear from this counter. The old interval callee fixture still exposes
provider preparation before its pending source scope is completed. The ninth
source revision gates provider preparation on that marker and consumes the exact
existing body requirement through the owning Queries catalog before retrying the
suspended machine. It also leaves unfinished checker requests in the saved State
until body completion, charging their capacities without transferring them early.

The ninth guard was refused before launch on shared disk reservations (280.1 GiB
free versus 284.2 GiB required); one authorized retry was likewise refused
(276.9 versus 284.2 GiB). No compiler or functional observation ran, and no limits
were lowered. Those receipts remain infrastructure failures, not language results.


The admitted ninth batch passes all-target/all-feature checking, four private
frame controls, all nine constant-pattern functions (including seventeen new
paired native values), and all five interval/reuse functions. Its only stop is
a stale discriminant test that still expected the now-supported root-pattern
case to fail. The exact positive source remains unchanged in the pattern suite;
the tenth run corrects only that expectation and passes all seventeen related
query/discriminant/selected-initializer functions. Strict Clippy then reports
one function-size finding. Extracting the existing selected-instance preparation
sequence into `prepare_instance` resolves that finding without changing its
owner/context ordering.

The eleventh batch passes the four private controls, strict release all-target/
all-feature Clippy, architecture/file audit (1,483 files, zero findings), layout
and upstream verification. Its intended targeted source filter accidentally
matches zero tests; this empty result is not functional evidence. The final
corrected `readiness::` run is recorded separately before checkpoint publication.
The exact final compiler-source digest is
`85797e56a5776901881fa04846b63f6d432a7eadffc5a4134fea2b5cd4e474b8`
from 869 sorted SHA-256/path records under `dev/src` and `dev/bin`.

Across the focused suites, thirty-five test functions pass: four frame
invariants, nine pattern functions, five interval/reuse functions and seventeen
related functions. Six backend-dependent groups independently execute forty-seven
paired Rust/CPPGM programs; the seventeen new accepted fixtures are included in
that count. The twenty-five new sources are independently object-qualified:
seventeen accepted, six rejected and two preserved Rust-positive implementation
boundaries. The missing-lifetime-debt negative remains masked by the documented
closed-materialization boundary. No full release matrix or performance comparison
was run in this worker for this source checkpoint; the integrator owns combined
qualification, and prior unresolved scalar RSS evidence remains unchanged.

Resource receipts preserve every admission and failure. The eleventh successful
scope is `58685a0e9f6e405eac306ec8e873eb4c`; its generated storage was
17,638,981,632 bytes within the unchanged 50 GiB worker budget. Earlier stale-test
and lint failures remain recorded under scopes
`9df19dc721764578be2b45b2cd382180` and
`86f60ef0234842ebbf35239675728e2b`. No lease/quota/reserve failure was reported
after the two pre-launch reservation refusals above.

The corrected final `readiness::` filter runs all three intended functions,
including independent objects/three views, retained positive boundaries and
seventeen paired native executions. All three pass on the final source in
3.62 seconds (test time). Its successful receipt is `764ee742b004437791ec14f97c644ac6`. The previously
empty filter remains recorded as an empty run, not counted among passing tests.

### Generic default value caller scope correction

The active PA20 `generic-default-dependency.rs` exposed a regression introduced
by the value-owner frame: evaluating trait default `C = Self::D` for `S<T>`
installed the original initializer's parameter environment as the selection
caller. The substituted rigid `T` belongs to the requesting generic function,
so nested selection incorrectly reported `shape parameter is outside its
environment`.

`begin_pattern_value` now retains the actual caller `PatternScope` while its
`ValueScope::Initializer` retains the exact declaration identity. The original
queued `PreparedScope` still supplies the declaration's own environment when
checking its patterns, then restores the caller scope. No foreign-parameter
fallback, concrete specialization, or extra proof assumption is introduced.

The isolated comparison is recorded under
`obj/parallel/pa20-default-context/first`: the unchanged source rejects in all
three views with c9377070's binary (`2ea7997768ab394e5eedb00cd0326d95eb7458f21ffb6a680000effcb6b6f952`)
and accepts with the fully qualified frozen PA24 binary
(`c408b1205c7815a004309b471ee115db532c7dd3c3a070c0ca2e2904d858cd10`,
c7fe46c0 / production 0b89bf31). The corrected binary is
`ce4abc52d50a5c54d0a772515b3c942a10c224e8357a9d6cc6a8974f829df49d`;
its sorted production source-map digest is
`668b9bafdfe2c038afa809238c0d1a84223d5ec36f9de6e6ac7a78b0ab0e81ca`.

Six private frame controls and five focused source/native test functions pass.
The original generic default and bounded-owner fixtures accept all three views
and execute paired native value checks. Missing caller bounds, missing Copy,
original opaque declaration patterns, and the existing lifetime-debt rejection
remain negative. Existing closed-readiness outcomes and eighteen paired natives
also pass. All 7,788 PA1–23 text comparisons pass unchanged, including PA20's
1,381 cases. Strict release all-target/all-feature Clippy, the file/architecture
audit (1,518 files, zero findings), layout and upstream verification pass.
Receipt `5b40d8170b644008b95f03f2bcd57620` closed with status zero and
19,670,597,632 bytes retained within the unchanged worker budget. No reference
or expected-outcome changes were needed.
