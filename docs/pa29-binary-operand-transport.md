# Retained binary operand source facts

This is an executable generic-body transport prerequisite on combined base
`b6e30beddbc1b27afb514ca63b2f9fc19bbb3792`, not a generic-body import feature.
Body ownership and the wire format are unchanged at this checkpoint.

Rvalue::Binary now retains a compact typed per-side lexical classification.
Original source construction peels only Group, Attributed and empty Expansion
wrappers, recognizes literal/name/path forms, and consumes the actual typed
Unary.literal fact for a negative literal. Division/remainder acquire these facts
once. Other operators use NotRequired and perform no new operand traversal.
Synthetic binary producers explicitly specify Neither; they never default to
immediate source syntax. Unary.literal and Cast.direct_variant are unchanged.

Promotion reads these retained facts alongside actual operand readiness, values,
types and checked promotion restrictions. It no longer scans all statements to
index source nodes or reads operand AST. Both syntactically immediate operands
can still be runtime/unpromotable, and signed MIN / -1 and zero divisors retain
their existing behavior. NotRequired at a demanded division is malformed data.
Transport must validate supported operation/provenance consistency; no wire tag
or imported permission is assigned here. Original/instance eligibility, caller,
region, loan and escape proof remain required.

The existing source view bytes do not display the extra typed field. Rvalue stays
20 bytes and Value 24; the focused retained-Code storage/layout control passes.
The promotion engine drops its source-node Vec and Report reference. BodyWork
adds a single cfg_operand_visits Cell counter; visits include errors, and each
wrapper walk is bounded by the actual source AST node count. Exact construction
visits are 32/128/512 at 8/32/128 repeated grouped divisions, with zero visits for
ordinary addition controls. No per-use graph or retained cache is added.

Qualification uses `obj/parallel/pa29-executable-originals/binary-second/`:
strict all-target/all-feature release Clippy, fmt, architecture audit 1918/0 and
registration pass. The new batch runner built the selected debug library and
ownership target once, then executed 8 exact tests: three new source/data/work
controls, one existing Code/layout control and four existing promotion regressions.
All passed; the receipt records source map, executable hashes, exact inventory,
commands and timings. No full release/native suite or repeated rustc matrix ran.
The first strict failure was a missing test-independent statement semicolon and
is retained under binary-first. This step does not close the source-free owner,
trait/header or executable generic/const transport requirements.
