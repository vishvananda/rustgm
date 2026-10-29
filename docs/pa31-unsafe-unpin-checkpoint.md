# Source UnsafeUnpin identity and proof checkpoint

The source role appends `LangItem::UnsafeUnpin` at index66, preserving every
existing0–65 ordinal. Its source-only `AutoTrait` follows Freeze; the five-entry
bootstrap `AutoTrait::ALL` is unchanged. Actual role headers require their exact
unsafe auto trait, no type/lifetime formals, members, bounds or where clause.
Structural children retain that actual ordinary TraitApplication, explicit impl
identity and current caller scope. Raw internal marker goals cannot replace it.
No ordinary library method or optional noalias optimization is added.

The original trait and negative UnsafePinned impl are retained under
`pa31/tests/spec/source-unsafe-unpin/`, with pinned byte intervals and hashes in
`contract.json`. The original UnsafePinned type has its distinct `unsafe_pinned`
language role and remains Required. The structural reducer has a clearly
attributed authored generic payload type; it grants no original cell/pin or
aliasing contract. The exact negative source impl supplies its behavior.
Pointer/reference/PhantomData exceptions come from explicit source impls; an
Unpin-style pointer exemption is not substituted. Existing metadata profile
role IDs and whitelists are unchanged; explicit new-role export fencing prevents
silently dropping this source contract.

Pinned Rust1.98, edition2024, Linux x86-64 independently accepts actual/renamed
source roles and the caller producer, and rejects the intended payload,
zero-array, missing pointer impl, unpaid caller/formation and header/unsafe impl
controls. A custom unmarked auto trait is a valid Rust positive with an explicit
candidate profile boundary. Corrected13-case independent receipt
`obj/parallel/unsafe-unpin-v1/oracle-v2/receipt.json` has SHA-256
`f1dfdbb069ffd985ef03c5732d201a257df6110951722d5f6491e9d256cb85ab`.
The initial authored const Clone foundation's missing const stability remains
preserved in v1; original trait/type/negative intervals were never rewritten.

The first real positive exposed an existing source sizing bug: implicit Self
`MetaSized` was added to auto traits, forcing an unproved metadata tail while
forming the genuine negative impl. Pinned
`compiler/rustc_hir_analysis/src/hir_ty_lowering/bounds.rs169–177` explicitly skips
that default on auto traits because the added supertrait could not be relaxed.
The correction excludes parsed auto traits from that implicit insertion. It
preserves all written bounds and the existing defaults on ordinary traits,
parameters and aliases. The exact failed predicate/type/application and underlying
“source sizing requires a proved metadata tail” are retained in diagnostic
receipt1793249654211747633. All temporary source observations were removed.

Final receipt `obj/qualification/fast/1793250151664957828/receipt.json`, SHA-256
`dbeff02e84a9be8bf62d8eae28e74d76e6e13d6c6214a3cb37835ba6f24891c9`,
passes formatting, architecture, strict private Clippy, build and10 exact tests
in125.46 seconds. Actual/renamed source callers reach checked LowIR; the negative
payload, zero-array, missing pointer impl and unpaid caller fail real trait
obligations. The unpaid nominal fails its formation prerequisite. Malformed
headers and unsafe impls reject at their own checks. Captured-negative lifetime
debt cannot be discarded, a good source cycle proves, and its negative sibling
prevents a false positive. Raw marker and exhausted-work controls reject.
Existing Freeze formation/debt, ordinary source, alias sizing and cold-width
controls pass. This supersedes the retained partial common-prerequisite failures.

At8/32/128 nested nominals, charged query work is871/3415/13591 and retained edges
90/354/1410. Cold Catalog1528 and Reservations2224 are measured; six old exact size
seals were updated from those values. Ordinary sources still allocate no source
catalog. Syntax Node36 and Body IR456 are unchanged. The separate ordinary cold
measurement receipt SHA-256 is
`25674b2eb1cc74aa319eacf6df7926c7852ceb515b5ab2205fbdf604c550fcd3`.

Root owns joined production/CLI, genuine source Debug runtime/native and unchanged
original-core observations. This finite role/trait proof checkpoint does not
claim that those consumers or original core already pass. Imported UnsafeUnpin
transport, the full original UnsafePinned type/cell/pin contract and optional
noalias optimization remain Required. Other unknown language roles stay closed.
