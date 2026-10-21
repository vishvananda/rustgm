# Packed field copies through raw aggregate pointers

The first raw-read checkpoint checked the alignment of the final copied field. That rejected valid packed field copies: `(*p_to_packed).field` requires the raw aggregate pointer's target alignment, while dereferencing a separately formed `*const u32` requires u32 alignment. Independent full Rust objects confirmed this distinction before correction. Original hypotheses and pre-correction observations remain under `obj/parallel/pa27/packed-raw-base`.

Private `RawAccess { location: Option<LocationId>, ty: TypeId }` now anchors each raw dereference at its actual symbolic target. Ordinary field/index projections preserve that anchor, including internal binding views; a subsequent raw dereference replaces it. Final reads still validate the selected leaf's type, bounds, initialization and allocation lifetime. Alignment uses the dereferenced target's type and physical offset. Numeric address cancellation remains address-only. Raw writes and reference creation through raw places keep their independent boundaries, and packed-reference rejection remains qualified through the existing result-storage tests.

| Control | Rust status | Previous frontend | Corrected frontend |
| --- | ---: | ---: | ---: |
| Packed base field copy | 0 | 1 (unaligned read) | 0 |
| Pointer formed to packed scalar field, then dereferenced | 1 | 1 | 1 |
| Nested packed field copy | 0 | Not probed | 0 |
| Pointer formed to nested packed scalar field, then dereferenced | 1 | Not probed | 1 |
| Packed pointer-field copy, followed by aligned pointee read | 0 | Not probed | 0 |
| Packed pointer-field copy, followed by unaligned scalar pointee read | 1 | Not probed | 1 |

All 35 source controls pass independent Rust 1.98 object qualification and both frontend modes. These include existing expired storage, initialization, bounds, direct/indirect static initialization and rejection controls. The raw cell invariant, reachable-storage suite, strict all-target/all-feature Clippy and file audit (1,297 files, zero findings) pass. All four native groups pass; the new packed-origin group checks values for the three accepted controls above. Full input and artifact hashes are in `pa27/tests/raw-read-outcomes.json`.

No Value/View/Origin or IR enum changes, backend changes, reference generation or assignment activation occur here. Broad combined validation remains integrator-owned.
