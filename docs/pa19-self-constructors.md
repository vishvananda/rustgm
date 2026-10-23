# Value-form Self constructors

This prerequisite closes the source `Copy` reducer that failed at `Self(self.0)`.
It selects the actual tuple or unit struct constructor after the impl header is
available, retaining type, const and lifetime arguments. The sparse existing
constructor-use index distinguishes a syntax prefix from an exact Self type.
Resolution tags retain the owner until elaboration; the completed reference is
the ordinary constructor declaration. No fabricated AST, source identity,
per-body flag or reparsing is involved.

Ordinary constructor inference and lifetime proofs handle calls and function
pointer coercion. Visibility is checked from the impl's module. Record/enum
calls, trait-default Self, extra arguments and captured outer impl parameters
are rejected, including in unused nested functions. Ordinary aliases do not
gain value constructors. Closed outer Self constructors remain accepted in
nested functions, matching the pinned compiler's lint-only behavior.

The initial independent plan wrongly predicted that last closed case would be
rejected. Its accepting rustc observation is retained in
`obj/parallel/self-constructors-v1/independent.py` and accompanying receipts.
The pinned compiler's `rustc_hir_typeck/src/fn_ctxt/_impl.rs` explicitly separates
parameter-bearing outer Self (error) from closed outer Self (compatibility lint).
The corrected positive and both generic-capture negatives are permanently
registered; the original failed hypothesis was not rewritten as rejection.

The maintainer suite independently qualifies eleven positives and eleven
negatives, then executes the positives against rustc through the CPPGM backend.
The PA19 manifest adds twenty cases with ten executable positives. Existing
pattern and ordinary-alias controls remain in the maintainer suite without
duplicating their earlier assignment ownership.

Development qualification is retained under
`obj/parallel/self-constructors-v1/qualification-v4`: formatting, all-target strict
Clippy, architecture audit, harness, focused Self/qualified-method/instance
constructor batches, native Self cases and all 8,362 pre-existing text cases
passed. Earlier exhaustive-match and formatting failures remain in v2/v3.
The unchanged original nominal Copy scaling inputs at widths 8, 32 and 128 now
pass the constants inspector; `closing-v1/original-repro.json` binds their exact
source and candidate hashes. This does not claim merged native execution of
those three scaling inputs.

The twenty new fixture outcomes were declared before generation in
`fixture-declaration.json`. The full updated PA19 independent source/native
suites passed before the scoped oracle generation; the updated 1,171-case PA19
text suite, harness and upstream attribution verification then passed.
`fixture-qualification-v1/complete.json` binds the same unchanged development
binary and receipts. The generated positive LowIR and rejection diagnostics were
reviewed; pre-existing sidecars remain unchanged. Combined release and measured
performance qualification remain separate gates before this successor is pushed.
