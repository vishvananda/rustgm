# Source negative literal patterns

Five authored no_core reductions. For a negated literal pattern, pinned rustc
1.98.0 requires the `neg` language item and registers `T: Neg` for the literal's
type. It calls no method. Positive ranges need neither.

- `missing-range` and `missing-literal` reject: "requires `neg` lang_item".
- `unimplemented` declares the item without an `i32` impl and rejects (E0277).
- `positive-range` and `ordinary-impl` accept. A non-const impl is enough.

`contract.json` binds the declared plan and the receipt. The first plan's
positive controls had no `i32` Clone/Copy and hit a rustc borrowck delayed-bug
ICE. That setup failure is kept as the predecessor. The corrected v2 inputs
match every declared outcome. RustGM's bootstrap (non-source) model keeps its
builtin negative patterns.
