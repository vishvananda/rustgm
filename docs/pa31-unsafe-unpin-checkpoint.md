# Source UnsafeUnpin checkpoint (partial)

The source role port appends `LangItem::UnsafeUnpin` at index66, preserving every
existing0–65 ordinal. Its source-only `AutoTrait` follows Freeze; the five-entry
bootstrap `AutoTrait::ALL` is unchanged. Actual role headers require their exact
unsafe auto trait, no type/lifetime formals, no members, bounds or where clause.
The existing source auto-trait selector keeps each child bound on that actual
ordinary TraitApplication, explicit impl identity and current caller scope. Raw
internal marker goals cannot replace a source trait. The existing nominal
formation, coinduction, private lifetime debt and work-budget paths are reused.

The exact original trait and negative UnsafePinned impl are retained under
`pa31/tests/spec/source-unsafe-unpin/`, with pinned byte intervals and hashes in
`contract.json`. The original UnsafePinned type has its separate `unsafe_pinned`
language role and remains Required. The finite reducer has a clearly authored
generic payload type; it grants no original cell/pin or aliasing semantics.
Pointer/reference/PhantomData exceptions use explicit source impls, not a default
Unpin exemption. No ordinary library method or optional noalias optimization is
added. Existing metadata profiles retain their role whitelist/IDs; an explicit
new-role export fence prevents silently dropping this source contract.

Pinned Rust1.98, edition2024, Linux x86-64 accepts actual/renamed source roles and
the structural caller producer, and rejects the intended negative payload,
zero-length array, missing pointer impl, unpaid caller/formation and malformed
header/unsafe impl controls. A custom unmarked auto trait is a valid Rust
positive with an explicitly unsupported candidate profile. The authored const
Clone foundation initially missed const stability after staged_api was enabled;
that failed v1 observation is retained. Corrected independent13-case receipt
`obj/parallel/unsafe-unpin-v1/oracle-v2/receipt.json` SHA-256 is
`f1dfdbb069ffd985ef03c5732d201a257df6110951722d5f6491e9d256cb85ab`.

This is not completed UnsafeUnpin qualification. Candidate receipt
`obj/qualification/fast/1793248611560166786/receipt.json` passes formatting,
architecture, strict private Clippy and build, then executes six controls.
Existing source Freeze nominal formation and captured-negative debt pass; header
and negative errors are observed. All three new positive/direct-query controls
fail during declaration formation of the authored `UnsafePinned<T:PointeeSized>`
negative impl, before reaching the actual auto goal, with “invalid or exhausted
canonical trait selection” at file0:1088. The shared prerequisite also masks
payload/caller negatives; their denial is not yet proof of the intended reason.
The exact underlying canonical goal must be isolated without weakening bounds.

The same compiled source passes the existing ordinary-no-catalog control and
measures cold Catalog1528 and Reservations2224 bytes. The known old width seals
are updated only from those actual values. The other named storage/sizing and
arithmetic occurrences are diagnostic prints. Measurement receipt
`obj/parallel/unsafe-unpin-v1/cold-measured-v1/receipt.json` SHA-256 is
`25674b2eb1cc74aa319eacf6df7926c7852ceb515b5ab2205fbdf604c550fcd3`.
Ordinary sources still allocate no source catalog; no hot record field is added.
The final cold assertion edits await joined qualification.

Two static-only Clippy attempts and the corrected exact-selector failure are
retained. The last unchanged-source six-test run uses real inventory paths;
there is no empty-selection or timeout pass. Root can integrate this partial
role prerequisite to expose independent Debug runtime consumers, while C retains
source trait/nominal proof isolation. Body/inference/array/lowering work remains
Root-owned. Actual source role positives, negative/caller/formational debt and
coinduction/payment controls remain mandatory before a complete role claim;
imported transport and the full original UnsafePinned type stay Required.
