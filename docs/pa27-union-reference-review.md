# Raw-derived references into scalar union Images

This bounded PA27 slice permits shared and writable mutable raw-reference
reconstruction for existing scalar-Image union field projections. It removes the
profile rejection in the private reference helper and the obsolete query. The
canonical `storage_target` consumer now recognizes the retained UnionScalar
projection for ordinary references as well as raw accesses. A reconstructed
reference correctly loses RawAccess on dereference; physical Image classification
must still follow its typed projection, independently of access permission.
Exact nominal owner, field ordinal and declared type checks remain intact.

Both field ordinals address the same current Image bytes. Reads decode the
selected scalar with existing initialization masks and bool/char validity;
stores replace the current immutable Image through the existing union write
path. No active member, product-field cell, pointer bits or new representation
is introduced. Existing lifetime, provenance, bounds, raw anchor versus actual
leaf alignment, source unsafe/packed checks and per-pointer authority remain.
Shared reconstruction restricts only its result; mutable reconstruction requires
existing writable authority and initialized ordinary nonstatic storage.

Twenty-seven independently compiled object outcomes distinguish fourteen
supported cases, nine independently rejected controls and four accepted-Rust
unsupported cases. Demanded invalid bool/char/uninitialized reads, expired access,
escaping storage and physically unaligned references retain their own rejection
reasons. Shared-derived mutable write stops at the existing explicit profile
gate, so it is not evidence of a reached later write check. Local products,
transient calls, physically aligned packed subobjects, and immutable static
readiness plus a later demanded constant dereference are supported.

The four accepted-Rust boundaries are direct source union Borrow, pointer-bearing
union families, write-only initialization through a mutable reference and repair
of invalid bool bytes through one. The last two independently succeed in Rust
but stop at this profile's eager initialization/validity proof. Forming a reference
is not inherently a contents read. The demanded-read rejection tests must not be
used to claim otherwise. Original preimplementation and first-candidate evidence
remain, including the static demanded-reference Product classification failure.
The fast semantic contents consumer is corrected independently: only completed
Image projections use the private typed helper; Product/Bytes retain their fast
path. It validates base/child types, canonical UnionScalar owner/field/ordinal,
zero offset and registered family/size, then charges before the existing scalar
codec. Structural Reader.field still rejects unions. A private invariant checks
valid/invalid bool, uninitialized bytes, wrong type/ordinal/owner, budget-before-
decode rejection and unchanged Image retention. Demanded static invalid-data
and two-ordinal alias controls are independently qualified.

Five ignored-by-default native groups compare sequential cross-field borrows,
valid scalars, static/nested/packed references, transient mutable calls and scaled
constant values with pinned rustc. Alternating borrows use separate scopes; a
later borrow through the other ordinal observes the latest store. The scaling
native group checks Image and ordinary values at N=8/32/128. Five private
reference/contents invariants and eleven affected source/scaling regressions pass.
Three earlier raw-derived union-reference fixtures become supported unchanged.

The first candidate exposed the raw-only physical classifier as a real missing
consumer. Its failed native result and subsequent all-case observations remain.
Other retained failures were test-only: a fixture-folder typo and an ordinary
control with two named loop bindings, which exceeded the initially declared
N+96 slot bound (60/108/300 slots at N=8/32/128). The corrected predeclared loop
uses one reference binding, preserving the original failed observations.
A test-only missing-semicolon lint failure and a guard regex that misread Cargo's
singular “running 1 test” output are retained. The corrected private selector
verifies one actual passing contents test, alongside four raw-reference tests.
Ordinary work is 538/1882/7258 steps and 49/73/169 slots; Image reconstruction is
1020/3684/14340 steps and the same slot counts. The test bounds linear work and
constant changed-path slot overhead, without claiming throughput neutrality.
Ordinary scalar output equality is checked against the retained baseline hashes.

Strict all-target/all-feature Clippy and the 1,370-file audit pass. The final
receipt records fresh object/view hashes, native evidence, upstream verification,
source and binary fingerprints and every partial gate. The physical classifier
now performs constant-time location inspection for ordinary accesses too; no
ancestry walk is added. Combined release performance remains integrator-owned.

Remaining PA27 boundaries include direct source union borrows and the deferred
reference profiles above; mutable/foreign/cyclic-static reference formation;
numeric nonnull ZST provenance; general reinterpretation and pointer offsets;
pointer/reference/function/aggregate union Images; shared-derived interior writes;
the default runtime invalid-reference-casting lint; and complete packed/pattern/
release qualification. PA28 still lacks the other memory/numeric, broader volatile
and atomic families/release contracts. No backend, active oracle, assignment
activation or full-release claim occurs here.
