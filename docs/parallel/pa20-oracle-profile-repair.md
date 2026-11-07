# Explicit PA20 unstable oracle profiles

Full CI42314235480 passes release/course, native and debug qualification, then
fails independent Rust at inherent-associated/unused.rs with E0554: the stable
Rust driver was given a feature-requesting input without its frozen bootstrap
profile. The independent packets already declared bootstrap1. No compiler
acceptance, expected status or fixture text needed changing.

The21 published inherent-associated cases and one custom-auto ordinary primary
now explicitly declare RUSTC_BOOTSTRAP=1 in their own case environments. Every
other case keeps its existing profile. The missing-feature input still rejects
E0658; bootstrap permits requesting a feature, not using it without permission.
All44 metadata/object observations match the original statuses with no E0554.
A clean-environment control reproduces the old E0554 failure.

Scoped course comparison first verifies all22 old output triples. Generation
then updates only their case-profile provenance; every oracle byte is asserted
unchanged. The complete PA20 independent suite passes in67.19s, followed by282
harness checks and upstream integrity. Local strict all-target metadata passes;
full successor CI remains required for the new compiler joins.

The exact commands, diagnostics, profile predictions, byte hashes and generated
batch identity are retained in obj/parallel/cold-creation-field-default-join-v1
and the owning pa20/reference-batches receipt. This is a test-profile repair,
not another assignment release or a change to stable-language feature gates.
