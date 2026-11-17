# PA33 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Required families: allocation and reallocation; Box Vec String and collections; allocation failure; drop and ownership across crates; allocator ABI and symbols.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.

Staged for rung M2 of the [self-hosting plan](../../docs/self-hosting-plan.md),
not graded while this stage is planned: `spec/alloc-consumers/` holds 12 `no_std` programs over alloc with their own global allocator.
`alloc-consumer-outcomes.json` declares each program's inputs, exit status and standard error;
`NAME.stdout` beside each source is what pinned rustc prints. Regenerate or
compare with `scripts/consumer_oracle.py m2` (see the
[runtime tracker](../../docs/self-hosting-runtime.md)). They are not in a
manifest, so the stage still fails as planned.

`spec/vec-collect-inference/` adds four complete source-profile regressions owned
by `vec-collect-compile-outcomes.json`: inferred and explicitly typed mapped Vec
collection, a wrong collection item, and a floating non-Step range. The original
Vec consumer harness prefix is preserved and attributed. This separate compile
outcome manifest leaves PA33 planned; it does not create a graded suite or a
maintainer integration root.
