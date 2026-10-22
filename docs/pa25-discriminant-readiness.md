# Discriminant value readiness

This draft continues the same typed dependency graph as named-pattern readiness.
It does not activate PA25 or alter ordinary closed selected-initializer authority.

`Requirement::DiscriminantValue(VariantId)` identifies one canonical enum value.
The read-only `Reader::discriminant` validates the member/enum owner and returns
no scalar while that tag is pending. Both AST cast operations and the interpreter
consume only a completed tag and its actual representation. The interpreter's
reached cast guard preserves its exact statement/frame before yielding. The
provider retains that session; the owning `Queries` schedules the same variant
through its existing Active/Ready graph and then retries the request. A true
cycle encounters the original active query; it never consumes a provisional zero.
Post-declaration queries have no alternative evaluator: they must see canonical
completed data or report missing declaration readiness.

Nested variant requests also retain their own anonymous value scope through the
existing typed frame mechanism. A later tag initialized by a named match or a
second const-function call can complete its own source patterns and obligations
without using the waiting caller's value owner. Success, suspension and error
use the same reservation/restoration paths as other nested value owners.

Independent qualification exposed a distinction from reached arithmetic. Pinned
Rust rejects a dependency cycle caused by a direct `F::B as isize` retained in a
skipped branch. It accepts the corresponding skipped cast through a local,
named constant, parameter or factory, and accepts source casts discarded after
an unconditional return. It also accepts a cast to B when only a different F
variant depends on the caller. These results rule out whole-enum layout demands,
source-wide cast scans, value propagation through locals/calls, or evaluating
skipped arithmetic as an implementation of this rule.

The checked snapshot therefore generalizes its existing ordered inline-operand
vector to `PreparedOperand::{Inline, Discriminant}`. During the existing retained
CFG statement walk, an integer cast whose immediate operand is a direct empty
variant aggregate contributes only that variant's dependency. Parentheses and
blocks are handled by their actual typed operand, not source spelling. In particular,
the grouped block `({F::B}) as isize` is accepted by Rust in the skipped-cycle
control, while grouping only the constructor retains the dependency. The same
preparation cursor checks this operand before branch execution. It does not
compute the skipped cast or its arithmetic. Dynamic operands retain the reached
cast guard. Ordering with inline operands is preserved in statement order.

The vector retains a single owner and preflights its actual count/entry size
before allocation. The measured prepared entry and former inline tuple are both
28 bytes; the Code header is 328 bytes. Replacing the Vec element type keeps its
header and the frame's existing usize cursor unchanged. A scalar body has zero
operand length and capacity. Nested value-frame storage remains in the existing
64 MiB continuation accounting. Each preparation visit consumes interpreter work.

Declared original sources and object/view hypotheses are recorded in
`pa25/tests/enum-dependency-outcomes.json`. The prior
`constant-pattern-readiness/callee-enum-cast.rs` positive is promoted without
changing its bytes. Open-lifetime associated initializer CFGs remain a separate
required PA25 follow-up; no concreteness gate, source/materialized cache state,
const permission or execution proof is weakened here.

Evidence lives under `obj/parallel/pa25-enum-dependencies/`. The first build/check
and existing pattern/discriminant source/native controls pass, but two new tests stop at the wrong
Rust-positive skipped-cycle hypothesis (Rust E0391). Its original bytes and all
observations remain in the first/probe/formation-probe/operand-probe evidence.
Probes distinguish the exact retained constructor cast from the opposing cases;
no production logging or source-text implementation was added.

The second batch passes checking, five private frame/reservation/malformed-request
controls, the retained-storage invariant, nine pattern functions, six discriminant
functions, four prior discriminant-readiness functions and both Ordering dispatch
source/native functions. It includes twenty independently accepted new sources
and nineteen paired new native executions. That batch still used the earlier
two-cycle iterator, leaving five new negative rows unexecuted. The closing batch
corrects that iterator and verifies the complete declared matrix rather than
counting unexecuted rows as evidence. It also rejects a non-enum member explicitly
at the read-only boundary. Full release/oracle/performance qualification remains
integrator-owned; no worker references or assignment status were changed.

Two guards were refused before launching on conservative shared reservations:
282.9 GiB free versus 283.2 GiB required for an operand probe, and 278.0 versus
282.6 GiB for the closing checks. Neither ran a compiler observation, changed
limits or supplied a language result. The operand probe subsequently ran after a
peer lease closed; its exact binary hash is retained with the observations.

The third batch passed five frame tests, exact graph-visit scaling and all nineteen
paired native controls, then stopped when independent object emission disproved
the declared block-cast-cycle negative. Its original source bytes and diagnostics
remain retained. That control is now an accepted opposing case; the matrix has
twenty-one accepted and six rejected sources. Strict audits had not started in
that failed batch.

The fourth run confirmed the grouped block positive was a candidate regression.
Its Aggregate ValueId had been forwarded through the block, losing the source
formation distinction. The retained Cast now carries `direct_variant`, set only
from the semantic layer's exact selected Construct fact through Group/Attributed
wrappers. Ordinary/manual producers set false. The fifth run restores that block
positive and passes seven private controls with unchanged Rvalue/Value/Body
layouts (20/24/464 bytes), then exposes the same missing prerequisite in the
legacy direct-root AST path. The existing typed preparation walk now uses that
same classifier to enqueue only the exact Variant prerequisite. It does not
execute a skipped cast, arithmetic or any after-return source operation.

The earlier declaration scheduler remains unchanged: append-time implicit-value
classification, retained entry cursor/used-tag set, and canonical tag publication
still avoid rescanning or accepting provisional layouts. Its earlier evidence is
`obj/pa25-discriminant-readiness/{0..6}.log`, source-map digest
`567273bf2f1cb3ac12d2b38231b8a6b39f1c6a0fcf550a2ccbd58efd91e1f547`,
and successful receipt `942c3022c65d4b67bfd8dd94174fbfee`. The two forward/pattern
boundaries described by that earlier checkpoint are now supported by the scoped
pattern graph and this exact variant continuation.

Final focused qualification passes on the same production source:

- Five private frame/reservation controls and two storage/malformed-Cast controls.
- All 27 declared new sources: 21 Rust accepts and six true cycles, independently
  object-qualified with default lints and checked in types, LowIR and constants.
  Nineteen include paired Rust/CPPGM native value checks. The two unmodified empty
  main controls intentionally have no native value claim.
- Nine existing pattern functions, six discriminant-execution functions (including
  the three new functions), four discriminant-readiness functions and both
  Ordering source/native functions. This is 28 unique focused test functions
  including the seven private controls; the new trio also ran separately.
- Strict release all-target/all-feature Clippy, file/architecture audit
  (1,518 files, zero findings), layout (38 stages) and upstream verification.

The repeated-callee family at widths 8/32/128 records finish-entry visits
10/34/130, one CFG, and one index/publication per actual variant. Three subsequent
queries for every constant leave expression/entry/CFG counts unchanged. This
linear invariant covers the measured graph-entry visits, not total interpreter,
allocator or hash-table work. Rvalue/Value/Body remain 20/24/464 bytes before and
after the new Cast fact, measured by guarded GDB and asserted by the private test.
The exact source-map digest over 886 production Rust files is
`bfcfb871732eb64a3c56d0ba37a0f550251ad43cded737d0b0c2ba17aa928a8b`.

Final functional evidence is `sixth/{frames,storage,reached,patterns}.log`.
That wrapper ended 101 solely because its next cargo command mistakenly named
Ordering functions as test targets; no test was launched by that command. The
corrected `tail/` batch runs `language_operators ordering::` (two actual tests),
then all strict/audit gates and ends zero under receipt
`02955ea92a9f4d6b9c860ce8c9ced321`. Its final worker storage is 19,507,040,256
bytes under the unchanged 50 GiB quota. Every prior failed hypothesis, launch
failure and reservation refusal remains separate evidence. No broad release,
performance neutrality, public reference or PA25 completion claim is made.
