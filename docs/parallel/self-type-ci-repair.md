# Trait Self lookup and wrong-provider diagnostics

Full CI42299342081 on main359613f98 failed68 tests across seven targets:67
encountered the same inherent-type feature error; the remaining weak-header
opponent correctly rejected a foreign provider, then panicked while constructing
a diagnostic with that provider's stale source node.

Ordinary trait-impl Self projections again use their existing proof route with
the extension disabled. Enabled lookup first checks actual type-name presence,
before shape/applicability/privacy filters. Genuine aliases keep precedence;
private and wrong-specialization aliases remain errors. The probe reuses real
owner/hygiene checks and prepaid existing name buckets, with no new records or
cache. Provider identity now authenticates before origin-aware diagnostics.

Eight exact tests across two targets pass in123.59s, including original shorthand
and retained lifetime/projection controls. Selected strict Clippy, full formatting
and architecture checks pass. Receipt:
`obj/qualification/fast/1794003865341743010/receipt.json`.
The independent [Self-path packet](../../pa20/tests/spec/inherent-associated/self-resolution-reference-v1.json)
qualifies six positives and four negatives in both metadata/object modes.
Two additional stable regression controls independently produce four artifacts.

The new fixtures are auxiliary-only under the existing unused.rs primary.
Scoped generation with current CLI0269798b preserves all three existing oracle
channels byte-for-byte. All282 harness tests,44-stage layout checks,127 upstream
fixtures and126 Reference files pass; the final guarded batch takes15.69s.
Receipt: `obj/parallel/inherent-routing-ci42299342081/final/receipt.json`.
The earlier missing scoped-provenance failure is retained in the sibling harness
receipt; its Python tests passed but layout correctly rejected the stale manifest.

Private runtime and the full release/native successor remain CI work. These
repairs neither advance the original-core observation nor complete an assignment.
The current unchanged-source frontier remains LIB187/UnwindSafe; Field operations
and actual custom-auto proof work continue in their existing isolated trees.
