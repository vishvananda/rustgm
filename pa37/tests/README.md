# PA37 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Required families: stage 1 against the PA1–PA24 oracles; stage 1 against stage 2
byte identity; strict build with zero bring-up records; reproducible inputs and
deterministic output; bounded build resources.

Follow the [owning brief](../README.md) and the
[self-hosting plan](../../docs/self-hosting-plan.md).
Declare independent outcomes before any stage runs. Register every input in a
nonempty `manifest.json` when the stage is ready for activation; no undiscovered
fixtures or empty-suite success. This assignment adds no oracle of its own for
PA1–PA24: stage 1 is checked against the committed ones. Preserve upstream
checksums, configuration/build provenance and the recorded hashes of both
stages.
