# Sparse initialization eligibility producer

This finite checkpoint implements the initialization side of
[source-const-variant-design.md](source-const-variant-design.md). It does not
connect selected leaves to const proof. Actual `prknow` and `prmove` remain
Rust-positive required compilation work at the existing selected-enum fence.
Actual interpreter cleanup also remains required at its independent Code fence.

The existing optional refinement owner holds a paid structural DAG, sparse owned
occurrences, tag/freshness/history bits, actual-path indexes and one typed transfer
index. The same initialization fixed point and replay capture source bits before
consumption/replacement and publish destination bits afterward. Constructor
coordinates come from the shared actual member lookup; the new Constructor query
checks complete inventory and ordinal/owner identity, while Variant stays enum
only. Arrays use one merged element descriptor, with no array-length expansion;
merged element tests cannot restrict every element's tag.

Each relevant owned coordinate has its own eligibility provenance. Contradictory
tag edges clear only that impossible payload's history. Fresh independent values,
call results, replacements and wrappers with their own destructor remain
conservative. Partial payload moves retain parent history while whole root moves
clear their former root. Mutable/raw exposure and shared interior mutation make
the affected root unknown and sticky until actual storage reset. These bits confer
no read, initialization, loan, native-plan, trait or interpreter permission.

Activation is explicit. Existing symbolic Physical mode retains its old behavior;
source Defining mode adds eligibility metadata. Native source-original staging
uses EligibilityOnly and never starts the old physical payload-exclusion pass.
An earlier test gate caught that accidental activation before semantic wiring;
the final gate restores all seven unchanged cleanup controls. No enum means no
partition/tag allocation. Legacy and ordinary paths retain their existing mode.

The final guarded checkpoint passes formatting, the cached architecture audit,
strict CLI/traits Clippy, three actual constructor/forgery tests, three const effect
tests, seven unchanged cleanup tests, suite registration and upstream checksums.
The immutable Rust packets preserve original predictions, corrections, syntax
setup failures and semantic errors separately. In particular the invalid `dunk`
name collision is retained alongside corrected `dunk2`; it is not an intended
semantic rejection. Public tests do not yet establish enum leaf proof parity.

Measured cold Refinements is120 bytes versus104 before this producer: an optional
partition pointer and activation policy add16. Facts664 and Point24 retain their
layouts. Partition416, Tags296, Parts152 and each variable record's actual width
are bound in the evidence JSON. The owner prepays DAG/index/vector/map capacity,
full block tables, tag ranges, transfer scratch and cold inline storage before
replacing the incoming table. Fixed point operations and metadata queries remain
bounded by the existing work/storage limits. Changed-path scaling and broader
consumer outcomes remain pending; no performance claim is made.

The next join owns Selected coordinates/rows in this already paid cold owner,
resolves full semantic fields through the existing shared Bound::joint fold, and
pays exact Body/Whole/OwnDrop claims before the same sealed loan completion.
Physical cleanup, live_type/drop-check, declaration validation and Code activity
remain independent. No persistent Body/Plan/Proved/Code or wire field is added.
