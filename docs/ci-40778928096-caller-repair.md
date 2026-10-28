# Caller and metadata regressions from full qualification

Full CI40778928096 on `5d47f7925` passed2142 private controls but failed two;
four additional tests failed in the closures/traits integration targets. Its
release step failed, so later full-course/native/debug/independent qualification
is not established. These are compiler regressions, not stale reference output.

The new sealed caller port treated compiler-generated comparison bodies as
ordinary source functions and rejected closure adapters before consulting their
real source. A sparse map owned by the append-only proof builder now records
only successfully proved factories/adapters. A factory retains its exact permit
and supplies no lexical feature environment. An adapter refers to its earlier
immutable proved closure; ordinary checked source lookup supplies that closure's
actual environment. Source/instance/owner checks remain hard, and an unbridged
wrapper still follows the existing rejecting path. There is no source-anchor or
callee-derived permission fallback.

Map growth is paid before proof/publication in the existing aggregate64MB demand
budget; failed attempts retain work and storage charges. No Body/Work/node record
grows. Existing wrong-source/signature/lifetime opponents remain, with an actual
exhausted-budget check proving no body or map publication and an exact original
source-pointer check after successful adapter proof.

Separately, newly retaining completed body scopes made a cache-only environment
serialize as Declaration instead of Contract. Metadata classification now first
authenticates the actual retained Arc, then distinguishes a real declaration
environment from a cache-only scope. Feature-bearing scopes stay explicit.
Existing provided-v0 first-encoding bytes and the original Contract expectations
are restored without editing a golden or generated reference.

| Qualification | Result | Receipt SHA-256 |
| --- | --- | --- |
| Private10: both original private failures, exact caller/feature/owner controls, actual handler and adapter provenance/payment | Pass,155.56s | `65f016f761a6517a0859cdbfa53a6b75aa441fcaa80863d9b35cff19614e183c` |
| Four original closure/sequence-comparison integration failures | Pass,125.41s | `99b844844215a64ebdffc19fe1479c78e464443699fd5db3631fff4ac5799646` |

Receipts are `1793230291794372228` and `1793230480095623963` under
`obj/qualification/fast/`. Both actual launchers pass, including their strict
private or production/CLI Clippy, formatting and architecture checks. The first
static attempt exposed ten private adapter call sites needing the new real
budget argument; those were updated and the complete affected controls rerun.
Raw full-CI output remains in `obj/parallel/ci-40778928096/failed.log`.

Peer review found no material authority or cache-erasure issue. This is focused
development evidence; a new full-CI result remains required, and original
core/alloc/std and assignment qualification remain unfinished.
