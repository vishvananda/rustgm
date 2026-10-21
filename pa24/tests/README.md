# PA24 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Required families: capture precision; Fn/FnMut/FnOnce; moved captures; closure coercions.

`capture-outcomes.json` retains the 32 initial source controls and their declared
language outcomes. The source bytes come from the original independently
qualified PA24 plan. They cover captures, call permissions, field precision,
loans, destruction, returned and nested closures, and pointer coercions. This is
a draft outcome manifest, not an activated grading suite or a claim that the
maintainer implements every positive case. In particular, rejection through an
unsupported-capture diagnostic does not prove a negative borrow/call rule.

Put required regressions here, in the earliest assignment owning all their
features. Keep draft inputs in their outcome manifests. At assignment release,
register every required input in `manifest.json`; no undiscovered fixtures.

`object-input-outcomes.json` declares inherited callable-object inference controls
and explicit quantified/unresolved-target boundaries. See
[the implementation contract](../../docs/pa24-object-input-inference.md) for the
ordinary required-goal protocol, cache identity, independent/native evidence and
remaining scope. It does not activate a grading suite or generated references.
