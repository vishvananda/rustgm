# Shared source call-use prerequisite checkpoint

The compact producer passes its focused debug qualification. This checkpoint
records unmet source context data; it neither authorizes execution nor enables
source-client metadata. Full combined qualification remains integrator-owned.

Branch `pa29-call-source-use-20261023` starts at development base
`14c2fe3fe8714e80fc32c44cf3e4ddfd348a58e1`, followed by root's isolated Scope
layout correction `26f72f73` (original `b2e25417`). The producer delta is separate
from that test correction. Capability handoff `77993d8d` and previous completed
branches and receipts remain preserved.

## Representation and ownership

`Operation::Call` retains its actual signature plus
`source_use: Option<SourceCallUse>`. The sealed four-byte NonZeroU32 handle names
a row in the current source Catalog. It contains no effect, epoch, eligibility
or paid witness. One lazy boxed table retains each actual semantic body's full
NodeKey and producer-derived Constness once; all its calls share the row. No
second owner map or per-call allocation is added.

`use.effect_for(&Declarations, actual_semantic_body)` checks the current catalog,
row bounds and exact full owner, then returns only current descriptor data.
`use.owner(&Declarations)` is likewise a checked data lookup. Neither establishes
proof. The actual query owner must charge successful and rejected read-only
lookups in its mutable work budget; the shared table has no interior counters.
Coincident rows from independent catalogs cannot carry an effect across them:
real runtime and defining-const controls resolve Ordinary and Conditional from
their respective current catalogs even with identical numeric row/owner IDs.

The existing successful-call publication uses the actual state.body and existing
callee_use_effect accessor. It validates targets, completed signatures, existing
row identity and freshly derived effect before any publication. Repeated
publication must agree and reuses the row. Two bounded passes over existing call
ports precede infallible indexed commit; descriptor growth and commit work are
reserved first. No ancestor/whole-body scan, speculative table or proof graph is
introduced. Ordinary publication keeps its existing one-pass allocation-free
path and has source_use=None.

The producer charges existing mutable body steps for actual inspected/committed
ports, repeated-row queries, row append and copied growth rows. It retains the
existing 1,000,000 body-work limit. Table limits are 1,000,000 rows, 16 MiB retained
(optional slot, boxed Vec header and actual capacity), and 32 MiB transient
(old/staged capacity plus temporary Vec header). Checked preflight precedes
allocation; actual staged capacity is checked before installation. Failure
publishes no row or Call, and consumed work remains charged.

Existing ForPlan publication still skips a Call for the For node; its separate
into_iter/next plans remain intact. A source body containing For must not pass
future descriptor publication until exact context and separate per-Call coverage
are implemented and tested. Missing context cannot be inferred as permission.

## Fresh qualification and layout

Guard93220 (`producer-fourth`) passes format, architecture audit, all-target and
all-feature strict Clippy, one debug library build, exact inventory and all seven
private cases. It runs no full release, native, active-text or broad performance
gate. Evidence is in
`obj/parallel/pa29-source-client-staging/producer-fourth/` with checks.json,
source-map.json, observer-layout.stdout and immutable-binary.json.

The retained immutable private binary is pins/rustgm-lib-tests, SHA256
`7edeec904f12f5d634ddea7fb8be59552e57d2e8bbfaa8af469d3731a7a353da`.

Measured layouts: SourceCallUse4, Option4, Operation16, NodeFacts80, Value24,
NodeKey8, Option<NodeKey>8, Owner32, OwnerKind20, Declarations7064, cold Catalog536,
boxed table payload24 and descriptor12. Operation/NodeFacts/Value size contracts
are const assertions so future changes fail the early static gate. The optional
source Owner snapshot remains32; dense DeclarationOrigin Owner would be36 and
is not adopted. No production Owner representation changes in this checkpoint.

Tests cover direct/dot/UFCS/FnOnce calls; runtime, defining const, actual closed
initializer, closure and inline owners; wrong parent/file, missing catalog/table
and out-of-range rows; current versus foreign effects; exact repeated publication;
invalid target, changed effect, storage/work failure; ordinary absent state and
unchanged type-view bytes. Independent axes 8/32/128 show N calls in one body use
one row, while N bodies use N rows. Repeated publication charges exactly 3N units;
the N-body table's retained bytes are 128/416/1568, including slot and box header.

## Preserved failures and continuing scope

The first copied-enum layout attempt stopped at audit (guard73072), before a
compiler ran; layout-first retains its failed source. Guard1362 was refused by
storage before executing any check. Guard56235 passed format/audit then found a
missed integration-test Call pattern; its sole mechanical recovery adds `..`.
Guard12613 passed strict/build and three producer cases, but deliberately failed
the real dense layout measurement: embedded owner/effect12 widened Operation to24
and NodeFacts to96. producer-second/failed-source retains those exact source
bytes and patch; the rejected measurements are not replaced. Guard40739 passed
format/audit then stopped on test NodeId conversions and forbidden expects; no
private tests ran. Guard93220's corrected results are fresh, not grandfathered.

The source-client neutral row/scope drafts remain parked byte-for-byte under
`obj/parallel/pa29-source-client-staging/pre-call-producer/`, with files.json and
restoration paths. They have not passed compilation or validation. No wire
descriptor/tag, SourceClient policy, Package policy, SourceCore capability or
assignment activation changes here. The full graph in
[pa29-source-body-graph-review.md](../pa29-source-body-graph-review.md), unchanged
clone_from/identity targets and complete remaining metadata matrix remain
required. Root owns the single cold library-request/stability inventory; client
activation must coordinate that authenticated inventory rather than duplicate it
or silently fall back to bootstrap roles.
