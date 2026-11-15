# PA32 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Required families: complete core source build; real generic bodies across crates; layout and const trait protocols; slices iterators and formatting; deterministic sysroot metadata.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.

Staged for rung M1 of the [self-hosting plan](../../docs/self-hosting-plan.md),
not graded while this stage is planned: `spec/core-consumers/` holds 18 `no_std` programs over core.
`core-consumer-outcomes.json` declares each program's inputs, exit status and standard error;
`NAME.stdout` beside each source is what pinned rustc prints. Regenerate or
compare with `scripts/consumer_oracle.py m1` (see the
[runtime tracker](../../docs/self-hosting-runtime.md)). They are not in a
manifest, so the stage still fails as planned.
