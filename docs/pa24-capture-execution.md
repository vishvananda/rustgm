# Direct captured closure execution

PA24 remains planned. This checkpoint executes direct, same-kind shared,
mutable and consuming closure calls through ordinary body, region, ownership,
loan and destruction proofs. The public callable signature remains unchanged;
captured bodies use the separately retained execution signature and hidden
`&Self`, `&mut Self` or owned `Self` parameter.

Captured source places project into physical environment fields. The logical
capture boundary follows the dereference of a stored capture reference and
retains source binding replacement authority through owned descendants. Source
reference dereferences clear that restriction, including dereferences already
inside a precisely captured prefix. Actual reference capabilities still govern
referent access. Unique immutable formation creates an immediate exclusive loan
without making the source binding replaceable.

A sparse body-owned entry table retains the complete closure instance,
requested call trait and selected execution signature. Its constructor is
private to canonical selection. Structural matching keeps nested callable
binders rigid, matches body universal slots at their actual binder depth, and
adds repeated-slot equalities to the existing caller contract. Ordinary region
proof must establish these equalities. Callee proof requires its exact retained
execution signature; no primitive factory permission or fabricated source
template is involved. Native lowering retains receiver argument zero and checks
that selected and proved execution signatures have compatible LowIR ABIs.

Environment fields participate in the existing symbolic and executable cleanup
recipes. Consuming calls can move one capture and destroy the remaining fields
without changing physical move paths or duplicating cleanup.

Wildcard capture mentions emit `CheckInitialized`, independently of ordinary
`Mention`. It checks only initialization and contained-reference validity at
closure creation, without reading, moving, borrowing or executing the mentioned
place. Typed projection records carry this proof without runtime projection
actions. A reference is required to remain valid until creation, but an unused
wildcard reference does not remain live until invocation. Ordinary wildcard
mentions retain their previous initialization behavior.

## Qualification

The focused gate covers 28 independently qualified direct outcomes: 16 accepted
programs and 12 Rust rejections. All 16 accepted programs execute successfully
through both rustc and the pinned CPPGM backend. Controls cover precise fields,
unique immutable captures, mutable references, immutable owned fields, consuming
partial cleanup, initialized and moved wildcard paths, and reference validity
before versus after closure creation. Four internal invariants check structural
mismatches, nested binder escape, retained caller equalities and exact execution
contract pairing. Forty existing closure, lifetime and guard test functions,
including their native groups, pass unchanged.

Evidence is under `obj/parallel/pa24/capture-execution/`. The final functional
record is `final-fifth.json`; earlier failing observations remain alongside it.
Those include the corrected direct-assignment shortcut, ordinary direct-call
capture classification, missing wildcard initialization, an absent environment
record on a noncapturing closure, and the independently disproved hypothesis
that an already invalidated reference could be mentioned at closure creation.
The final lint and placement record is separate so passing functional gates are
not repeated for a formatting-only lint correction.

Two additional Rust-accepted output lifetime cases remain explicit producer
boundaries in `execution-outcomes.json`: returning a captured reference and
returning one borrowed capture while destroying another. Their source and
diagnostics are retained for the semantic lane. Nested captures, adjusted
receivers, reference conversions, complex patterns, generic captured body
specialization and stronger-trait receiver adapters remain separate work.
Combined release qualification and an immutable performance comparison remain
integrator work. No assignment activation or text reference changed.
