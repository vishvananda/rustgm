# PA27/PA28 remaining release contracts

This reconciles the planned handouts with the accepted development slices after
`9829fd38`, counted/overlapping memory, symbolic relocation and byte-fill
checkpoints. Earlier checkpoint reviews retain their original boundaries and
measurements; later slices supersede those explicitly rather than rewriting the
historical observations. Neither assignment is activated by this inventory.

## PA27

The accepted surface includes lexical unsafe permission and scoped lint policy,
raw versus reference place capabilities, mutable-static authority, packed
alignment, runtime union overlap and destruction, constant union Images over
qualified scalar/aggregate and symbolic-word profiles, exact symbolic raw
reads/stores and reconstructed references, and packed/raw/union closure capture
precision. See the owning handout and the
individual `pa27-*-review.md` records for their qualified profiles.

Open implementation or profile contracts remain:

- Initialized whole `[u8; N]` snapshots now retain ordered symbolic fragments
  through copies and const calls; selected numeric bytes and final publication
  remain separate validity gates. Owned U8 array-rest/range snapshots now preserve
  clipped fragments and batched selected stores; general raw/unsized ranges and
  unused-reference eager validation remain required followups. See
  `pa27-array-fragments.md` and `pa27-fragment-ranges.md`. Canonical const/mut pointer
  `.add` now uses checked byte offsets without granting access permission. Logical field
  and index offsets now reuse original backing coordinates, masks, fragments and
  access checks; this does not implement numeric pointer reconstruction or every
  storage family. See `pa27-byte-access-review.md` for the qualified controls.
- Broader niche enum, opaque/closure and string backing, whole raw-slice values and
  object reference-cast/reconstruction. Qualified thin/function/slice/vtable word
  fragments now preserve independent metadata and incoming write restrictions;
  complete scalar/aggregate Images no longer require every field to be scalar.
  Partial fragments remain transient, and typed reads/publication retain their
  validity gates. See `docs/pa28-relocation-design.md` for exact supported and
  unsupported profiles; no active-member union model is introduced.
- Shared-derived interior-mutable writes. A per-pointer restriction is retained,
  but crossing an UnsafeCell-like boundary needs its own typed permission model.
  Runtime `invalid_reference_casting` policy is a separate outstanding frontend
  obligation; CTFE rejection does not qualify that lint.
- Numeric/null/ZST pointer profiles and Rust-positive unused-reference forms
  outside the current eager initialization/liveness/alignment profile.
  The default `invalid_null_arguments` source lint is independently required;
  an unsupported CTFE numeric address does not establish that lint. Pinned-Rust
  count-zero and zero-size CTFE acceptance is recorded separately from runtime
  validity, without numeric/null native claims.
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

Raw-slice element indexing now selects a sized exact typed element from original
array/rest backing and checks logical metadata separately from physical bounds.
Metadata-only length patterns do not read contents. Complete reference duties,
general raw range reconstruction remains separate. The PA25 DST field consumer
now forwards existing checked tail metadata while preserving physical storage
identities; see `pa25-dst-tail-metadata.md`. This does not provide whole unsized
raw values or new unsizing conversions. See `pa27-raw-slice-review.md` for retained
oracle disagreements and exact raw-slice scope.

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

- General byte transport beyond the qualified scalar/aggregate and symbolic-word
  families. Counted copies and overlap-safe `copy` now preserve invalid scalar
  bits, initialization masks and symbolic pointer/function/slice/vtable fragments
  through exclusive byte backing. The `write_bytes` slice adds one checked
  destination span, one provenance interval edit and batched byte/mask fill.
  Typed reads still enforce validity and complete immutable publication rejects
  partial words. Unsupported niche enum, opaque/closure/string backing and the broader
  provenance families above remain required work. See
  `docs/pa28-byte-transport.md`, `docs/pa28-relocation-design.md` and
  `docs/pa28-byte-fill.md` for exact boundaries and oracle differences.
- Primitive `usize::MAX`/`u32::MAX` preparation is PA26-owned and accepted on
  combined main. Original byte-fill sources retain their independently accepted
  outcomes and historical lookup failures; literal-count counterfactuals remain
  isolation evidence. Combined qualification supersedes the old lookup boundary.
- Numeric, bit and cast intrinsic families, including width/sign behavior and
  u128 controls beyond ordinary language arithmetic.
- Additional atomic widths/types, fences, accessors and update/control methods.
  The current U32 registry is not a claim about the rest of the library surface.
- Aggregate, ZST and wide-pointer volatile transfers. Ordinary copies cannot
  substitute for volatile operations or padding behavior.
- The retained pointer/static/lifetime boundaries above, plus broader atomic
  concurrency qualification. Backend capability inputs alone establish no
  frontend contract.

The counted-copy development checkpoint qualifies exact typed symbolic spans,
snapshots and existing runtime transfer loops; its review is
`docs/pa28-counted-memory.md`. The overlapping-copy checkpoint is reviewed in
`docs/pa28-overlapping-memory.md`. The supported byte fill and relocation profiles
do not complete the broader provenance/storage families above. Full release, immutable
performance measurement, active manifests/oracles and publication remain
integrator-owned. Focused logical work bounds are not wall-time or RSS
neutrality claims.


Explicit integer-repr fieldless enums now use the existing byte backing with a
lazy full-TypeId tag index. Typed reads map signed/sparse physical bits to exact
canonical VariantIds and check current initialization/provenance; raw copies
transport invalid or uninitialized tags without decoding them. Existing ordinary
Product values and all public value/provenance schemas are unchanged. Payload,
default-representation enum families remain required followups.
See `pa27-enum-bytes.md` and the PA27/PA28 `enum-byte-outcomes.json` drafts.


The wider-tag slice extends explicit fieldless enum byte backing to every
integer representation, including signed/unsigned 128-bit and target-sized tags.
Full physical discriminant bits remain distinct from canonical VariantIds.
Payload and default-layout enum storage remain required; see
`pa27-wide-enum-bytes.md` for exact outcomes and measured index retention.


Explicit integer-tag payload enums now reuse retained per-variant ABI fields and
offsets, existing symbolic words and the bounded iterative byte codec. Canonical
field addresses retain physical offsets across tag changes. Inactive payloads
and padding remain outside selected validity; default-layout/niche and other
unsupported backing families above remain required. See
`pa27-payload-enum-bytes.md` and the PA27/PA28 payload outcome drafts.


The inferred C-tag slice also supports repr(C) without a written integer tag,
using the exact retained target ABI tag. At that checkpoint default Rust, tagless and niche layouts remained required;
the original Rust-positive counterexamples retain their historical unsupported
outcomes. Subsequent tagless and thin-pointer Option checkpoints supersede
those exact profiles without closing general default/niche storage. See
`pa27-c-enum-bytes.md`.


The guaranteed thin-reference/function Option family now uses one retained null
encoding for runtime size/alignment/call ABI and constant byte backing, including
eligible transparent carriers. Four original stride wrong-code controls are
corrected; Some/None, C/Rust/reified calls and destructor behavior are qualified.
See `pa27-option-niche-design.md` and the PA26/27/28/30 Option outcome drafts.
Ordinary multi-variant Rust enums now consume their existing retained I64 direct
tag in byte storage, preserving exact discriminant-to-variant mapping and selected
payload validity. The unchanged default-layout required positive now passes;
ordinary typed closure fields retain their previous path. This does not promise
rustc's implementation-selected byte layout or general closure backing. See
`pa27-default-tag-enum-bytes.md`. Wider/other scalar niches and NonZero/NonNull
remain required rather than being excluded from completion.
Default-std compatibility additionally needs AtomicBool and signed/unsigned/
target-sized atomic families; memchr/zmij SIMD/target-feature/inline-assembly
requirements belong to later PA28/PA37 compatibility work. The current U32 and
Option checkpoints do not claim those capabilities.
