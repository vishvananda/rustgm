# PA29 fixture ownership

Planned: no graded suite or generated reference outputs yet.

`scalar-outcomes.json` declares the maintainer checkpoint's multi-crate inputs,
identities, dependency aliases and independent outcomes. The explicit Cargo
`crate_linking` target checks these inputs, including backend-dependent native
controls when run with `--include-ignored`. This inventory is not an activated
course manifest: the layout check rejects `manifest.json` for planned stages.

Required families: multi-crate dependencies; symbol identity; visibility; deterministic exports.

Put required regressions here, in the earliest assignment owning all their
features. Register every maintainer input in the checkpoint inventory. The
reviewed assignment release must publish `manifest.json` and references together;
no undiscovered fixtures or empty-suite success is permitted.
