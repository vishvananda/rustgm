# Reborrow implementation prerequisite

This unqualified frontend checkpoint implements the structural coherence checks
from pinned Rust 1.98 `compiler/rustc_hir_analysis/src/coherence/builtin.rs`,
`reborrow_info` and `coerce_shared_info` (lines503–794). PA31 owns the actual
compiler/library declarations. Root's admission dependency is c4f27122d; this
worker adopted main afcc7d4a8 without changing prior reference evidence.

Actual matching source roles require their completed catalog declarations.
Ordinary implementations return before touching a reborrowing field payer.
The existing prepared impl schema supplies the owned lifetime count, including
anonymous `impl ...<'_>` inputs. Each actual struct application independently
has exactly one lifetime argument. The data-only field view checks complete
member coverage and actual field ordinals. Only genuine PhantomData is excluded.

One paid existing joint Bound folds both existing normal and unreduced field
lists. Raw absence has the existing producer's exact-normal meaning; a present
raw list must match the complete actual field count. No source text, AST reparse,
new type kind, registry or dense body record is introduced. The ephemeral field
row retains its real selection, normal type and declared raw type. Its expected
width is16 bytes; the view is expected32. These are predictions pending the
combined metadata build, not measured results.

Every Reborrow data field proves actual Reborrow or actual Copy, with the pinned
mutable-reference exception. CoerceShared permits equal counts of zero or one
data field; its alternatives are the original reference exception, equal rigid
shape plus actual Copy, or an actual nested CoerceShared obligation. Parent
Reborrow and Target:Copy requirements remain ordinary impl obligations. Nested
selection and normalization debts stay in the same owner Proof and region gate.

The early builtin alternatives deliberately publish no temporary field-lifetime
equation as a global impl assumption or debt. Independently frozen Rust outcomes
accept unused differing-field-lifetime impls but reject their demanded local
escape. Actual operations must prove directed complete application and field
lifetimes, source-place loans, ownership and physical layout separately.

Intrinsic eligibility uses the original declared outer reference and its raw
peeled structure, before normalization. The unchanged associated-projection
opponent and its direct-reference spelling both reject in pinned Rust; simple
reference aliases accept. A blanket alias rule would be wrong. Existing
unreduced field provenance preserves this distinction without adding an AST
classification flag. Source owns those frozen references and retains initial
disproved predictions and demanded ICE cases as unqualified observations.

No worker build or RustGM candidate run has qualified this commit. Root requested
the coherent checker and API before its actual operation consumer is joined,
avoiding unused-API lint allowances and duplicate compiler builds. Strict lint,
format, architecture, public coherence outcomes, actual layouts and native
operation checks remain pending that combined gate. No implicit operation,
runtime, CTFE, transport or whole-assignment completion is claimed here.
