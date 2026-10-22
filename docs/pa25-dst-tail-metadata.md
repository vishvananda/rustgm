# Constant DST field metadata

This development checkpoint follows the raw-slice checkpoint `d05c736b`.
PA25, PA27 and PA28 remain planned. It closes the preserved PA26
`early-nominal-value` constant-execution gap without changing source unsizing,
reference permission or layout rules.

## Consumer contract

The typed producer already attaches the correct metadata when an array-backed
record reference is unsized. Constant field projection previously discarded it,
so matching the final slice field failed with `constant slice place lacks length
metadata`. A selected final unsized field now inherits the full existing Length
or Object metadata. Selecting a sized prefix clears only the selected pointer's
metadata; it leaves the incoming pointer unchanged.

The reader validates completed sizedness, canonical struct owner, variant and
field ordinal, or tuple ordinal, and the exact checked logical child type.
Only the final unsized child receives metadata. Unresolved classifications,
foreign fields and inconsistent child identities reject. These are retained
fact lookups, without a new ABI/layout traversal or conversion proof.

For an unsized view over sized backing, `Location.ty` remains the immediate
physical parent's canonical stored child. `Resolved.ty` remains the already
checked logical child. Nested wrappers keep this distinction at every selection.
Physical slice backing requires the exact element type and an independently
sufficient array bound; object leaves require the retained concrete/target
identity. The ordinary lowerer still requires canonical field-type equality
when emitting constant pointer relocations. No full ancestry walk, pointer
rerooting, numeric provenance, new Value/Origin/Pointer variant or reference
permission is introduced. Existing raw-access, bounds, lifetime, initialization,
alignment, static-context and write-authority checks remain in force.

## Independent controls

The original `early-nominal-value.rs` is unchanged, SHA-256
`9a92d10728d3a4506d0083b9043f95fc0b48023d020a40e15f6f080e7e50bf59`.
Its baseline failure and the original peer receipts are retained under
`obj/parallel/pa25/dst-tail-metadata/peer-evidence/`. Runtime-owner IR established
that unsizing already supplied the length; the missing consumer was Field.

The 19 declared source cases cover direct slices, nested records, explicit and
mutable references, empty tails, sized prefixes, published tail/field views,
wrong element/mutability, local escape, bounds and packed alignment. Stable
Rust rejects source tuple unsizing; private tuple representation tests do not
claim that source conversion. PA9's early-view boundary remains explicit for
local escapes and packed references. Contracts are in
`pa25/tests/dst-tail-outcomes.json` and `pa27/tests/dst-tail-outcomes.json`.

Thirteen positive source programs and six additional runtime programs give 19
exact literal Rust/CPPGM native comparisons. The runtime programs exercise
published record, field and nested field pointers, runtime mutable/prefix reads
and actual calls, rather than only already-evaluated scalar constants. Native
tests stay ignored in ordinary CI. The existing CHECK helper in frozen inputs
triggers Rust's default `unconditional_panic` lint; those original diagnostics
remain in `first/`. Explicit `--cap-lints=allow` replay preserves exact bytes and
matches the existing native harness policy. New checked-source API callers must
supply that independent policy explicitly; existing callers keep their default.

The first runtime field-view attempt failed with `constant projection differs
from its canonical field type`, despite independent Rust/native success. Its
source and artifacts remain in `closing-third/` and the neighboring native
folder. Retaining canonical physical field types corrected that required
positive; the lowerer check was not weakened.

## Closing evidence

`obj/parallel/pa25/dst-tail-metadata/closing-fifth/qualification.json` records:

- Six nonempty private invariants, including full Object metadata, wrong
  physical owner/element/length/concrete identity and nested physical/logical
  divergence.
- Nineteen source cases across 76 views, 19 native comparisons and 40
  independently qualified work inputs.
- Product-constant, symbolic-location, reference-cell and raw-slice regressions.
- Strict all-target/all-feature Clippy, audit (1,623 files, zero findings),
  layout, upstream and suite-registration checks.

The work grid varies unrelated prefix width 4/32/128, nesting depth 0/8 and
repetitions 8/32/128, with ordinary sized controls and 64 unrelated types.
Repetition deltas are identical at every width and grow exactly fourfold for
24 versus 96 additional calls. At DST depth eight and width four, the three
counts cost 1,739/6,731/26,699 logical steps. The unrelated-type controls retain
exactly 6,815 steps, 33 frames and 419 peak slots. These are bounded logical
work observations, not throughput or RSS neutrality claims.

All 8,283 branch-active PA1–24 cases ran: 8,233 compare byte-for-byte and the
same 50 stdout-only raw-slice deltas remain. Their actual stdout hashes equal
the previously handed-off raw-slice inventory exactly; no new output, stderr
or status differences were introduced. References and active manifests remain
unchanged; the integrator owns their combined qualification and publication.

The closing selector initially matched zero private tests; the guard rejected
that batch. `closing-fourth/` retains that failed receipt and the fresh nested
native qualification. The corrected `closing-fifth/` private log records all
six matched tests. Earlier test-only compile/lint repairs remain documented in
the qualification receipt. No full Cargo or exclusive performance matrix was
repeated for this slice.
