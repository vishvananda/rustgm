# Actual source dereference contracts

The 37 complete authored inputs in `contract.json` and `cases.tsv` freeze outcomes
before observing RustGM. Pinned Rust1.98.0 (88d9e12ae) independently confirms the original 32 and five added controls;
`foundation.rs` and the C observer are authenticated auxiliary inputs. Preserve
repository NOTICE and the inherited source-foundation attribution.

The packet checks actual Deref/DerefMut/DerefPure/Target identity and unused
alternate headers, shared/mutable/generic operator consumers, original method
bodies and impl conformance, ordinary lookalikes, mutable-place permission,
Copy/move/borrow/escape obligations and destruction. The native observer checks
real method effects, shared/unique selection and one-time place evaluation.
Expected stdout is literal contract data, not inferred from candidate output.

The actual canonical member and signature feed existing method instantiation,
trait goals, late-binder opening, receiver loans and selected calls. Mutable
places refine that selection to the actual DerefMut method. Lowering consumes
those typed calls through the existing borrowed-place path. No replacement
source body or physical Copy fallback is involved.

`deref_contracts` in the frontend maintainer suite owns execution. The
[checkpoint](../../../../docs/pa31-deref-receiver-checkpoint.md) binds its candidate,
independent/native controls and remaining contract. Wider demanded signatures,
const/metadata, implicit autoderef/coercion profiles and whole PA26/31/43 release
requirements remain required. Local source completion is not imported permission.

`nested-borrow.rs` adds a separately frozen actual `Pin<&mut &mut S>`
regression. Its callback borrows the real nested operand, mutates the referent,
and ends the borrow locally; the expected C observer output is `23\n`. It uses
existing Pin/Deref/DerefMut roles and no AsyncIterator declarations.

`receiver-alias-depth.rs` retains 128 named receiver layers without introducing
a parser nesting limit. Its full body-proof test checks depths8/16/32/64/128 and
requires canonical work to grow by at most two for each doubled depth. The four
conjunction controls preserve genuine missing-bound and escaping-lifetime
rejections alongside their independently accepted counterparts. Shared roots
retain every formation and lifetime obligation in their actual caller scope.
