# PA27/PA28 remaining release contracts

This reconciles the planned handouts with the accepted development slices after
`9829fd38`. Earlier checkpoint reviews retain their original boundaries and
measurements; later slices supersede those explicitly rather than rewriting the
historical observations. Neither assignment is activated by this inventory.

## PA27

The accepted surface includes lexical unsafe permission and scoped lint policy,
raw versus reference place capabilities, mutable-static authority, packed
alignment, runtime union overlap and destruction, scalar-only constant union
Images, exact symbolic raw reads/stores and reconstructed references, and
packed/raw/union closure capture precision. See the owning handout and the
individual `pa27-*-review.md` records for their qualified profiles.

Open implementation or profile contracts remain:

- General pointer offsets, byte reinterpretation and provenance reconstruction.
  Exact-type symbolic locations and canonical scalar union projections do not
  supply arbitrary byte-addressed memory.
- Pointer/reference/function and aggregate union field Images, including
  relocation-bearing byte storage. Current Images require every concrete field
  to belong to the primitive scalar family and have no active member.
- Shared-derived interior-mutable writes. A per-pointer restriction is retained,
  but crossing an UnsafeCell-like boundary needs its own typed permission model.
  Runtime `invalid_reference_casting` policy is a separate outstanding frontend
  obligation; CTFE rejection does not qualify that lint.
- Numeric/non-null ZST pointer profiles and Rust-positive unused-reference forms
  outside the current eager initialization/liveness/alignment profile.
- Mutable, foreign and cyclic static reference formation, and mutable-static
  initializer contents reads. Reference validity and by-value access eligibility
  remain separate; successful immutable-static retry is already supported.
- Reference-bearing aggregate storage whose nested lifetime structure differs
  beyond the exact outer-reference-cell compatibility rule. Source lifetime
  equality, variance and escape proofs must remain unchanged.

Release qualification also needs the complete packed/pattern and cross-feature
inventory, one reviewed phase/diagnostic contract and the planned manifest/text
publication. Existing accepted packed-reference source controls do not waive
alignment, and rejected reinterpretation profiles are not semantic validity
proofs. The integrator owns that release work.

## PA28

The accepted surface includes scalar/thin-pointer volatile transfers, Sized-only
ordinary ptr read/write including non-Copy ownership, AtomicU32 storage and
Ordering, load/store/swap, strong and weak CAS, add/sub/bitwise/extrema RMWs,
reification and const/runtime eligibility. Weak uses strong CAS; deterministic
retry instrumentation is separate evidence from uncontended source execution.

The unaligned slice adds canonical `read_unaligned`/`write_unaligned`. It reuses
ownership and exact symbolic memory, relaxing only the selected raw operation's
alignment requirement. It does not create a reference or implement general
reinterpretation. Its outcome contract lives in
`pa28/tests/unaligned-memory-contract.json` and its review is
`docs/pa28-unaligned-memory.md`.

Open operation families remain:

- Counted copies, overlapping moves and byte fills, with explicit byte/count
  overflow, overlap, initialization and provenance rules.
- Numeric, bit and cast intrinsic families, including width/sign behavior and
  u128 controls beyond ordinary language arithmetic.
- Additional atomic widths/types, fences, accessors and update/control methods.
  The current U32 registry is not a claim about the rest of the library surface.
- Aggregate, ZST and wide-pointer volatile transfers. Ordinary copies cannot
  substitute for volatile operations or padding behavior.
- The retained pointer/static/lifetime boundaries above, plus broader atomic
  concurrency qualification. Backend capability inputs alone establish no
  frontend contract.

The next substantial operation family after unaligned transfers is counted
memory, subject to an agreed typed storage/count interface. Full release,
immutable performance measurement, active manifests/oracles and publication
remain integrator-owned. Focused logical work bounds are not wall-time or RSS
neutrality claims.
