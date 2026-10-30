# PA29 fixture ownership

Planned: no graded suite or generated reference outputs yet.

`scalar-outcomes.json` declares the maintainer checkpoint's multi-crate inputs,
identities, dependency aliases and independent outcomes. The explicit Cargo
`crate_linking` target checks these inputs, including backend-dependent native
controls when run with `--include-ignored`. This inventory is not an activated
course manifest: the layout check rejects `manifest.json` for planned stages.
The `spec/values/` family adds scalar named constants, exact bit preservation,
imported patterns/discriminants and consumer const-generic bounds. Its producer
identities and dependency edges are distinct from the function-only family.

Required families: multi-crate dependencies; symbol identity; visibility; deterministic exports.

Put required regressions here, in the earliest assignment owning all their
features. Register every maintainer input in the checkpoint inventory. The
reviewed assignment release must publish `manifest.json` and references together;
no undiscovered fixtures or empty-suite success is permitted.

`local-dyn-owner-outcomes.json` declares a real source provider and two fresh
clients: bare imported dyn formation succeeds, while a foreign inherent impl
rejects after actual trait resolution. It preserves the ModuleContracts-v2
provider foundation and grants no foreign permission.
