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
- Broader niche enum, missing opaque physical heads and unsupported capture-child backing, whole raw-slice values and
  object reference-cast/reconstruction. Qualified thin/function/slice/vtable word
  fragments now preserve independent metadata and incoming write restrictions;
  complete scalar/aggregate Images no longer require every field to be scalar.
  Partial fragments remain transient, and typed reads/publication retain their
  validity gates. See `docs/pa28-relocation-design.md` for exact supported and
  unsupported profiles; no active-member union model is introduced.
- String data/length word transport now uses the existing exact U8 range, mask,
  provenance and write-authority rules, including empty/one-past references. This
  is raw storage support, with no eager UTF8 validity rule or compiler string API.
  Actual UTF8 library validation/conversion routines remain Rust source and need
  their source intrinsic prerequisites. The two unchanged original panic-macro
  sources remain required positives blocked on ordinary source-library bindings;
  their separately frozen helper reductions qualify storage only. See
  `pa27-string-byte-words.md`. Missing opaque heads remain unsupported until the
  actual checked original publishes them; completed-head backing and the scoped
  defining-head readiness family are qualified separately.
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
  partial words. Unsupported niche enum, opaque/string/capture-child backing and the broader
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
target-sized atomic families. SIMD, target-feature and inline-assembly contracts
actually required by selected core/crate sources must work at PA32/PA35;
the broader target closure belongs to PA41. The current U32 and Option
checkpoints do not claim those capabilities.

The source-bound `needs_drop<T: ?Sized>` kernel now uses exact actual Copy/Drop
roles, exhaustive destructor registration and a fresh checked const-call witness
before every result-cache access. Ordinary runtime wrapper calls specialize their
existing const block; direct runtime intrinsic codegen remains unsupported.
The focused provider qualifies eleven original bool-result/native controls and
thirteen query-reuse/scaling controls. This does not execute full upstream
Drop/Pin, Freeze/drop_glue or conditional destruction, and does not close the
public source-library metadata/entry boundaries. See
`pa28-source-needs-drop.md`; all original required controls remain retained.

Actual source destruction also requires metadata-aware slice/dynamic/tail cleanup
and a const cleanup continuation with real per-call const obligations. Independent
slice/dyn source controls now execute three/one drops; they are required positives
beyond the qualified sized runtime-helper consumer. The original full
Drop/Pin/default override sources remain required. Actual `owned_box` additionally
requires pointee destruction before Box's allocator/deallocation method, unlike
ordinary record method-before-fields cleanup. Preserve genuine ownership/Unique,
`may_dangle` and allocator ABI protocols; no pointer-layout/name inference. See
`pa31-source-drop-glue.md` for the exact pinned source paths and current boundaries.

Retained closure environments now use their exact completed capture tuple layout
and the existing iterative Product codec. Owned/shared/mutable captures preserve
ownership and pointer restrictions, including fresh selected-reference validity;
zero-byte nested closure products are still validated without expanding a Repeat
extent. Full demanded Closure TypeId remains callable authority. This supersedes
the exact explicit-closure counterexample, while function-item and other
unsupported capture children remain open. See `pa27-closure-byte-storage.md`.

Prepared opaque storage heads now drive the physical byte codec and readonly
product inspection without changing source equality, traits, ownership or
lifetime authority. Shared-head family validation is reused. Reference/owned
opaque applications now retain conservative source cleanup and replay after an
exact defining-source proof publishes their head. Ordinary UTF-8 source routines,
unprepared or otherwise unsupported defining heads and the other families above
remain required; see
`pa27-opaque-byte-storage.md` and the exact desired/observed outcomes.

The source-bound `needs_drop<T:?Sized>` kernel now evaluates its independently
qualified concrete instances and ordinary original const-block wrapper. Its real
intrinsic identity and bool signature remain separate from the layout provider.
Actual source Copy/Drop readiness, complete destructor preparation and fresh
reached-caller proof precede result caching. Full Drop/Pin/default-method,
const-drop and unresolved-storage continuations remain required as recorded in
`pa28-source-needs-drop.md`; missing roles never imply false.
