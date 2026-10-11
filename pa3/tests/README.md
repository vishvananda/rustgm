# PA3 fixture ownership

88 required inputs cover delimiter forests, doc desugaring, attribute syntax,
cfg predicates, custom options and cfg_attr expansion. Their expected outcomes
were declared before reference generation. See the handout for syntax ownership.

77 fixtures have independent rustc whole-program or predicate checks; 11 are
explicit diagnostic-stream fragments. The latter include opaque macro input
and general custom attributes whose resolution belongs to later assignments.
Configuration cases record target/profile and custom arguments in the manifest.

Put required regressions here, in the earliest assignment owning all their
features. Register every input in `manifest.json`; no undiscovered fixtures.
