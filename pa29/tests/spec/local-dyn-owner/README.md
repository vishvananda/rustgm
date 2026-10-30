# Imported dyn-principal ownership

The provider preserves the qualified ModuleContracts-v2 root provider foundation
and its real empty `Open` trait. The bare client must resolve that imported trait;
the inherent client must reject foreign ownership after resolution. This is not
an unknown-name rejection and grants no Has/Allow permission. The private control
checks the actual Trait DefId with no source node and the exact ownership error.
See `pa29/tests/local-dyn-owner-outcomes.json` for frozen source hashes and genuine
pinned Rust metadata/object outcomes. PA29 remains a planned assignment; this
checkpoint declares maintainer fixtures without activating a graded suite.

The actual module-profile export/reload control passed: the bare client resolves
its imported trait with no source node, and the foreign inherent impl reaches
the explicit ownership diagnostic. No unknown-name error is counted as proof.
