# Open-lifetime initializer source computation

This checkpoint computes a selected associated initializer's primitive source
value when its lifetime arguments remain open in the original caller. It is a
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

Required remaining work includes nonprimitive/reference or projected results,
open type/const argument contexts, and source computation outside the prepared
pattern-caller path, including nested non-pattern contexts. Those need exact
existing caller proof integration and result normalization debts; no static
substitution or source selection alone can authorize them. PA25 allocation,
validity and sequence-view work owned in the other lanes remains separate. This
checkpoint does not activate PA25 or broaden PA29 metadata.
