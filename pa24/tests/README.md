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
including quantified input/output and pending generic target arguments. See
[the implementation contract](../../docs/pa24-object-input-inference.md) for the
ordinary required-goal protocol, cache identity, independent/native evidence and
remaining scope. It does not activate a grading suite or generated references.

`quantified-dispatch-outcomes.json` declares 22 additional captured callable
controls, with 16 native positives and six language rejections. The existing
`captured-dispatch/quantified-capture.rs` now accepts and executes; its prior
unsupported observation remains documented. A nonclosure caller-lifetime
regression belongs to PA23, and no PA24 text oracles are generated yet.

`capture-copy-outcomes.json` covers unresolved reference pointees and the opposing
move/call-kind/lifetime rules. The typed outer-shape contract and qualified
source/native evidence are in
[reference capture classification](../../docs/pa24-reference-copy-inputs.md).

Reference-Self captured objects are predeclared in
`reference-objects-hypotheses.json`; qualified hashes and results are recorded in
`reference-objects-outcomes.json`. Their exact frozen bytes live in
`spec/reference-objects/`; ten positive programs exercise shared/mutable nested
references, generic and quantified calls, outer results and unused FnOnce views.
Four opposing permission/loan/escape inputs retain rejection. This draft has no
released manifest or generated oracle. See
[the implementation review](../../docs/pa24-reference-objects.md).

`signature-slot-outcomes.json` covers independent public frames and opposing
outer/local lifetime ownership. The exact registration, import and scoped equality
contract is in [signature ownership](../../docs/pa24-signature-slot-ownership.md).
No generated references or assignment activation accompanies these draft cases.

`pending-object-outcomes.json` retains unresolved early type/const targets, actual
object lifetime inputs, unused public binders and opposing type/escape controls.
Declaration clauses create required goals; no pending caller terms enter a cache.
