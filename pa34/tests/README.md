# PA34 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

`foreign-edition-outcomes.json` declares 32 paired source outcomes and 13 native
outcomes for extern blocks, member safety and supported linkage attributes.
These [development controls](../../docs/pa34-extern-editions.md) pass independent
and focused candidate qualification. They do not enable the Rust 2021 driver
profile or establish assignment acceptance; combined checkpoint qualification
remains required before publication.

`temporary-edition-outcomes.json` predeclares 20 source and 18 native outcomes
for tail and if-let temporary scopes, unchanged scope controls and opposing
borrow lifetimes. Independent Rust and focused candidate source/native
qualification pass; combined checkpoint qualification remains required.
Native assertions check the actual destructor trace in each edition. See the
[temporary scope checkpoint](../../docs/pa34-temporary-editions.md).

Required families: per-crate editions; feature and dependency resolution; host versus target artifacts; build script outputs; reproducible build provenance.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.

`never-edition-outcomes.json` declares sixteen paired source outcomes, twelve
local native outcomes and four independently checked cross-edition macro native
outcomes. All hypotheses match pinned rustc. The candidate implementation and
its focused qualification are in development; no driver profile is activated.
