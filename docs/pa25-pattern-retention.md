# Retained pattern-query state

This prerequisite preserves the exact pattern/value owner and dispatcher when a
constant query yields to declaration or checked-body preparation. It does not
remove the general named-pattern evaluation boundary. Dynamic scope scheduling,
coverage publication, and callable-to-forward-enum readiness remain subsequent
work; the independently qualified draft readiness inputs are not completion
claims.

`Continuation` moves `PatternScope`, `Dispatcher`, and `ValueScope` together with
the existing query graph, work context, inference state, and instance catalog.
Resumption restores those objects rather than reconstructing selection caches.
Suspension is fallible: its dispatcher must fit the 64 MiB logical retained
budget, and allocation-inspection visits consume the existing query work budget.
An over-budget dispatcher cannot become a published continuation.

`Dispatcher::retained_allocation` reports bytes and collection visits. Owner-local
facades cover inference terms/edges, optional inference catalogs and watcher
payload, substitution caches and nested folds, selected targets, constant and
expansion caches, lifetime proof vectors/blocks, and their boxes. It inspects
capacities without resolving inference or acquiring proof authority. The common
`Context` gains no fields; an empty context still reports zero retained bytes.

The first dispatcher snapshot measures existing nested proof payload once.
Subsequent evidence additions measure only the incoming proof children; current
root vector capacities are read directly. Repeated snapshots do not walk old
proof children. When tracking is active, concrete override normalization consumes
only a new root proof and the existing allocator frontier, then appends its debts
through the same accounting owner. Extension preserves free IDs and rebases new
scope IDs. The ordinary untracked normalization path remains unchanged.

Caller accounting distinguishes declaration sharing from copied ownership. The
caller retains the original declaration environment Arc alongside its mapped
environment. Each owner compares corresponding Arc identities: unchanged tables
remain shared; a fresh environment wrapper and copied map/vector/box payload are
charged once. Fresh immutable clause indexes and their later parameter lookup
caches are charged once when observed. Repeated snapshots inspect lazy-cache
presence without traversing old candidates. The observer belongs to the same
immutable caller/index for its lifetime; it is not a cross-context positive cache.
`Caller` grows from 16 to 24 bytes and retains one extra source Arc for this pattern
path. An ordinary dispatcher still has no Caller, and its retention observer is
allocated only at the first snapshot.

The charge is a retained-storage policy, not a heap/RSS measurement. It counts
actual vector/hash entry capacity and owned boxes, excludes allocator/hash/Arc
control metadata and declaration-shared immutable pointees, and does not charge
shared type-store/report graphs again. Quantified proof binder tables retain the
existing immutable shared-table exclusion. A moved object is charged once; these
facades include dispatcher/header storage so an enclosing frame must subtract
that header if its own header charge already includes it. No reduced-RSS claim is
made; the prior unexplained scalar RSS observation remains in
`pa25-core-performance.md`.

Qualification evidence is under
`obj/parallel/pa25-pattern-readiness/retention-{fourth,sixth,owned,final,source}`.
The final resource-only gate passes six dispatcher invariants, strict release
all-target/all-feature Clippy and the architecture/file audit (1,476 files, zero
findings). Earlier qualification also passes both continuation invariants and all
20 constant private tests. The dispatcher matrix covers empty inference,
reserved empty capacity above 64 MiB, old-proof reuse at widths 8/32/128, new proof
payload, and a projected override with pre-existing evidence/nested lifetime
obligations. Both ordinary and tracked invalid lifetime proofs still reject.

Snapshot collection visits at widths 8/32/128 are respectively 66/138/426 on the
first inspection, 30 on repeated inspection, and 45 after one new proof. The
copied-caller control measures 1,772 bytes before lazy indexes, 2,136 after its
mapped trait index, and 2,360 after parameter lookup caches; further snapshots
remain 2,360 bytes with 33 visits. These are collection visits, not total machine
instructions or a heap-size claim.

The final strict/resource receipt is
`obj/parallel/resources/c3839103f4414bc4af3c407efa0dd917.json` (exit 0, about
15.8 GB accounted generated storage). Source digest
`67427a2e0e811de81c4a7eabfdea62e1973a30fc87dd9b3ba325d70ea5aece57`
covers 864 sorted `dev/src` and `dev/bin` Rust source records. Earlier failed
fixture/setup and lint runs remain named evidence; the fifth guard was canceled
while queued and has no functional observations.

Final exact-source checking additionally passes all six constant-pattern tests
(including its explicitly ignored backend-dependent native test) and both
projected-override tests. The latter independently object-qualifies 26 outcomes
(17 accepted, nine rejected) and executes all 17 accepted values with both
toolchains. The new override control retains an actual returned reference.
Resource receipt: `obj/parallel/resources/49ba9d7ef7da4b50adb4b0f404769d35.json` (exit 0).
No assignment activation, fixture oracle generation, or full release matrix was
performed in this lane; combined qualification remains integrator-owned.
