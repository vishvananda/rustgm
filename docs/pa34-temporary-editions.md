# PA34 temporary scope editions

The typed scope walk now selects Rust 2021 or Rust 2024 temporary endpoints
from the syntax's defining edition. A Rust 2021 block tail inherits the enclosing
temporary scope; Rust 2024 gives it the narrower tail scope. Rust 2021 if-let
scrutinee temporaries similarly survive the else branch, while Rust 2024 drops
them before entering else. Binding scopes, ordinary boolean conditions and
explicit lifetime extension retain their separate rules.

The existing typed conditional-chain set identifies when an if expression needs
edition selection. Its parsed introducer supplies the origin. For a block, the
matched delimiter pair supplies the edition independently of a captured tail
or an `unsafe` qualifier. The parser widens qualified block spans at the start,
so the scope walker uses the exact closing delimiter byte. Expansion preserves
paired delimiter provenance; no source scan or generated-text parse is needed.
Existing bounded parent-origin lookup resolves that byte. Missing demanded
provenance fails instead of selecting the current crate's edition.

Only the temporary endpoint changes. Existing lexical scopes and storage-exit
events remain available to borrowing and cleanup. No AST, scope fact, work item
or persistent body record grows; the temporary walker borrows the package for
origin lookup. Scoped scratch reuse and the existing work limit remain intact.
The pinned Reference's [temporary scope rules](../doc/rust-reference/src/destructors.md)
and rustc's `rustc_hir_analysis/src/check/region.rs` define the hypotheses.

## Development evidence

`pa34/tests/temporary-edition-outcomes.json` declares 20 source outcomes and
18 native programs before implementation. Pinned Rust 1.98.0 agrees with every
outcome. Each native program contains an actual destructor updating a static
trace and a predeclared final assertion; RustGM and rustc compile identical
bytes for that selected edition. No cleanup instrumentation replaces compiler
behavior.

| Control | Rust 2021 | Rust 2024 |
| --- | --- | --- |
| Failed if-let: destructor 1, else marker 3 | 31 | 13 |
| Function/nested tail: local 1, temporary 2, later marker 3 | 123 | 213 |
| Explicitly extended borrow: marker 2, destructor 1 | 21 | 21 |
| Tail destructor borrows a block local | rejected | accepted |
| Call argument borrows a temporary from a block | accepted | rejected |

Receipts remain under `obj/parallel/pa34-temporary-editions/`. The independent
plan hash is `47fa13cab5c04164814a82de3a9e2707c2856e1ac7df9e10b4cb501f1865138b`.
The combined focused run passed the prior 28 ownership/storage controls, two
prior edition tests and two initial private origin controls. Its new source
test correctly rejected the old-edition tail borrow but failed a diagnostic
assertion that omitted the word `loan`. `development-v3` preserves that failure,
corrects only the assertion and passes the new source suite plus all four
selected native tests, including existing cleanup-order and pattern/linkage
edition controls. The focused recovery took 7.78 seconds and the native
continuation 14.78 seconds, reusing compiled artifacts.

Formatting, all-target strict Clippy, architecture, harness and upstream gates
pass for that production snapshot. Subsequent private test preparation binds
both delimiter origins to the same donor edition, preserving the matched-pair
invariant while leaving the qualifier and captured tail in their own edition.
That test-only refinement is recorded in `paired-origin-review.json` and awaits
the combined checkpoint's private execution. Production, fixture bytes and
native helpers are unchanged. Full release and performance qualification
remain required before publication.

This does not expose a Rust 2021 driver flag or activate PA34. Cross-crate macro
transport, opaque capture rules, prelude/method selection and never-type fallback
remain part of the actual crate-graph edition contract.
