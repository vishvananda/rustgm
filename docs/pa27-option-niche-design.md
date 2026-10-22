# Guaranteed Option pointer niches: owning ABI checkpoint

The implementation starts at the qualified `2dede8e4` tagless checkpoint.
This record retains the frozen design, independent hypothesis disagreements and
the focused implementation evidence below; it does not activate PA27 or PA28.

## Independently frozen observations

`obj/parallel/pa27/default-enum-bytes/declared-first.json` preserves 23 initial
hypotheses and exact sources; `first/qualification.json` records every result.
Twenty-two hypotheses matched. The original expected-invalid bool source was
accepted: for pinned Rust's `enum E { A(bool), Empty }`, byte 2 selects Empty.
Eight separately frozen follow-ups all match their declared Rust outcomes and
valid native results (`declared-followups.json`, `followups/qualification.json`).
They include the returned bool/empty result, a rejecting explicit `repr(u8)`
bool counterpart and rejecting niche byte 3. The original disagreement remains.

The bool experiment depends on the implementation-defined repr(Rust) encoding.
It does not prove a portable byte layout or an invalid-Rust source. The current
course ABI deliberately retains a direct I64 tag for ordinary multi-variant
Rust enums. Any future bool niche choice must be stated as an owning target
layout contract, not inferred at a read site from rustc acceptance.

In contrast, the pinned core Option documentation guarantees equal size,
alignment and call ABI for the listed reference/function payloads, and zero None
for sized references/functions and qualifying transparent wrappers. See the
[exact pinned source](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/option.rs#L115-L157).
The four array-stride programs for shared/mutable refs, function pointers and a
transparent reference wrapper are valid ordinary runtime tests. All compile in
all baseline views, but baseline CPPGM executables return 134 instead of the
expected zero: Option stride is 16 rather than 8. Pinned Rust executables return
zero. This is an ABI correctness gap, independently of CTFE byte-family support.

The original default-union program remains an accepted positive gap. A plain
whole-array typed copy and ordinary enum control already pass the baseline and
must not be relabeled new coverage. Thin pointer/function Option from-word,
zero-None, expired/partial/numeric/shared-write controls are independently frozen.

## Canonical declaration policy

Add a small typed `NullableEnum { empty: u32, payload: u32, field: u32 }` policy
in `types::aggregate`, retained in a lazy optional map on Definitions. Register
it atomically for the exact canonical LibraryType::Option during its existing
nominal publication, before any layout query can observe that family. Select
ordinals from that canonical descriptor's empty/one-field records and validate
its exact two-variant/field declaration, rather than interpreting user spelling
or assuming backend tags. An ordinary user enum named Option gets no policy.

The policy grants only layout selection. It grants no source type equality,
lifetime, auto-trait, write capability, const eligibility or solver premise.
Closed type/layout caches keep their existing full TypeId scope.

## Retained ABI shape and cold table

Append an opaque `NicheId(u32)` and this explicit physical shape:

```rust
Shape::Niche { variants: Range, encoding: NicheId }
```

Keep the existing Aggregate/direct Tag and single-variant layouts unchanged.
Add a lazy `Option<Box<Niches>>` on Layouts; a niche shape holds only its opaque
index. `Niches` owns descriptors and bounded cached payload-carrier decisions.
The descriptor for this first family is:

```rust
struct Niche {
    owner: TypeId,
    empty: u32,
    payload: u32,
    field: u32,
    word: TypeId,
    ty: TypeId,
    offset: u64,
}
```

It explicitly means a zero null pointer word selects empty; a nonzero symbolic
word selects payload. No arbitrary scalar niche range is introduced here.
`word` is the actual retained reference/function leaf, including its full type;
it is never an invented numeric pointer. The sole payload and eligible
transparent-wrapper field chain have the same physical address, offset zero.
The logical payload TypeId and physical variant fields stay in existing Variant
records. Empty and payload ordinals need not equal their discriminants.

Expose `Layouts::niche(&Layout) -> Result<Option<&Niche>, &'static str>` with
checked shape/index correspondence and `Layouts::variants` supporting the new
range. No caller may interpret a niche as `tag=None`/first-variant. The explicit
Shape variant makes exhaustive consumers require an intentional decision.

The layout constructor recognizes only the registered Option family, with a
closed thin reference/function payload or an eligible completed transparent
wrapper around it. Raw pointers, same-spelled user types, explicit repr enums,
wide pointers and other scalar-validity niches do not receive this encoding in
this first family. Eligible wrappers use their checked transparent declaration,
physical size/alignment/offsets and unique nonzero stored child, preserving ZST
field rules. Interior-mutability/no-niche representation barriers need their own
typed check; do not infer a valid scalar niche from transparent size equality
alone. This first profile qualifies ordinary transparent wrappers explicitly.
Use a bounded iterative cached carrier walk over touched types,
with no scan proportional to the Store. Ordinary non-Option layouts avoid it.
False and successful completed carrier answers are cacheable; unresolved facts
remain errors rather than negative answers.

Construct empty and payload physical variants from the same retained fields;
payload fields start at zero and the total size/alignment equal the carrier
payload. Publish descriptor and layout together only after all checks succeed.
Roll back descriptor/variant/offset suffixes on error. Charge actual visits,
cache/descriptor growth and temporary traversal before allocation through a
narrow bounded niche query, exposing work/retention counters for tests. Do not
weaken ordinary layout readiness or caller budgets.

The bounded entry must be explicit: a private `NicheBudget { remaining_work,
retained_limit, temporary_limit }` reaches every carrier-walk/cache/descriptor
operation, with consumed work retained on cache hits and errors. Ordinary
`Layouts::demand` keeps its signature and traversal behavior; an additive
caller-limited entry shares that traversal (constant-specialized bounded path)
so CTFE's remaining work reaches niche preparation before it can grow scratch
or caches. This budget covers the new table/traversal, not the canonical Store
or the pre-existing global layout index. Resource exhaustion must be typed
internally and restore unfinished states to retryable status, never cache an
invalid-type result or classify diagnostic strings. Completed valid dependencies
remain reusable. Keep the limit owner transient; do not widen every Layout or
add an optional work-limit branch to the ordinary hot traversal.

Measured baseline sizes are Layout 48, Shape 32, Layouts 144, Tag 16 and Variant
32 bytes. The ID-only shape should retain Layout/Shape/Variant density; the cold
Layouts owner gains one eight-byte optional handle. Measure, rather than assume,
those sizes after implementation. No public Value/View/Origin/Pointer change.

## One encoding for runtime storage and call ABI

Audit and update every Shape consumer, not only byte decoding:

- ABI construction/readiness/variant access, physical field offsets, tail and C
  classification: preserve checked owner/ordinal/child facts and exact wrapper
  eligibility; enums must never accidentally pass a repr(C)-record classifier.
- `lower/function/aggregate`: construct empty by writing the null word; construct
  payload through existing field stores; test variants by comparing the current
  word with null. Preserve actual canonical variant/discriminant mapping. A
  no-tag multi-variant result must not use the first variant fallback.
- `lower/cleanup/emit`: variant decisions use the same retained null encoding,
  retaining ordinary cleanup and selected drop identity.
- `lower/data/{initializer,zero}`: empty stores zero, payload emits its actual
  pointer/function relocation, and zero detection observes the same encoding.
  Constant field selection must use actual selected variant offsets.
- `lower/context` and function entry/call/return/C-boundary plans: matching size
  alone does not establish the required call ABI. Preserve internal object
  snapshots while passing/returning the encoded word by value, like its carrier.

For that last consumer propose `Representation::PointerObject(MemorySpan)`:
internal Selected remains Object, but parameter/result are Ptr. Its payload is
no larger than existing Object; measure density. It applies only with the exact
retained niche/transparent carrier witness, never arbitrary same-size storage.
Allocation/copy/read remain object operations. Entry stores the incoming word
into object backing; argument lowering loads it; return lowering loads it; a
call result stores it into its object destination. Ordinary Object keeps its
hidden-result protocol. C passing classification may select the same carrier
only through this witness, with explicit boundary load/store adaptation, not a
new semantic equivalence. This shared lowering delta needs integration review.

## Reader and constant execution

Append the marker `StoredShape::Niche`, without embedding a large descriptor.
`Reader::enum_niche(TypeId)` returns a borrowed facade over the exact physical
encoding, canonical VariantIds and checked field slices. Extend common physical
enum validation to the explicit niche shape. Validate full owner, two variants,
empty/payload field counts, leaf/wrapper type, offsets and bounds. Preserve
structural union boundaries and the ordinary typed fast path.

Use the existing iterative codec and exclusive backing:

- Raw transport copies bytes, masks and symbolic fragments without decoding.
- Encoding empty writes one zero word with initialized bytes and removes only
  intersecting fragments. Encoding payload visits its actual typed fields and
  preserves complete original pointer/function identities and write restriction.
- A typed decode first requires all carrier bytes initialized. Symbolic
  fragments select payload and then require normal complete-word/type validation;
  placeholder zero bits must not be mistaken for None. A fragment-free zero word
  selects empty and creates no reference. A fragment-free nonzero word selects
  payload and its normal typed decode rejects invalid numeric/provenance cases.
- Selected payload decoding retains actual referent bounds, alignment, lifetime,
  pending static duties and original write authority. Warm shape/carrier facts
  do not discharge any of these. Mutable references still require writable
  provenance; None never creates a null reference or requires such permission.
- Existing decoded-result/duty continuation keys and retry-exactly-once behavior
  remain authoritative. Some references cannot escape their true backing.

Extend byte-family child selection, immutable/thawed-cell encoding, typed raw
field projection and closed-layout subobject indexing for this shape. Canonical
field pointers use checked physical ByteOffset, preserving addresses/capability
across Some/None changes. Include references formed before/after conversion.
Constant views keep ordinary Product/Variant output; Image fragment vocabulary
is unchanged. Dependency discovery/native emission use the same existing words.
PA29 enum/Image export remains explicitly unsupported unless its owning profile
has a typed encoding transport; it must not silently serialize this as a record.

## Qualification and remaining work

Prioritize the guaranteed Option thin reference/function and transparent-wrapper
cases, including the four runtime stride failures. Qualify actual typed
calls/reified functions and structural parameter/result ABI, Some/None copying,
zero fills, pointer restrictions, publication and invalid partial/numeric/expired
words. Add forged owner/ordinal/field/ID, actual null versus reference validity,
wrong wrapper/shape and warm-cache validity controls. Test cached shared wrapper
DAGs, unrelated type growth, depth/repeats 8/32/128 and tiny budgets, measuring
actual table/scratch retention and ordinary no-Option behavior.

Default direct-tag Reader support can follow using its real retained I64 tags;
it does not solve or simulate niches. Other guaranteed library NPO families,
wide pointer/metadata niches and general scalar niche optimizations remain
required follow-ups, alongside remaining PA27/28 backing/provenance and intrinsic
families. No assignment activation, oracle publication, backend edit or worker
full release/performance gate is part of this proposal.


## Implemented focused checkpoint

The typed policy, explicit Niche shape and borrowed Reader witness now drive
construction, matching, drop, static data, byte encode/decode, field addresses,
Rust calls and C classification. PointerObject keeps internal aggregate snapshots
while parameters/results carry the actual pointer word. The descriptor retains
its full closed owner and payload TypeId, and checks its completed owner layout.
None is inhabited even if the transparent payload has an uninhabited zero-size
field. No public Value/View/Origin/Pointer or backend vocabulary changed.

`functional-first/qualification.json` under
`obj/parallel/pa27/default-enum-bytes/` records all 41 frozen programs in each of
`--emit-types`, `--emit-lowir` and `--emit-constants`: every declared outcome
matches. The 34 valid native programs return zero, including the four original
16-versus-8 stride failures, real C provider calls linked with the same C object,
C composite/reified calls, transparent ZST fields and selected destructor counts.
Seven validity negatives retain partial-word, numerical-reference/function,
expiry, shared/static-write, one-past and alignment rejection. No rejecting or
implementation-defined hypothesis gained a native-valid claim.

The immutable candidate is `functional-first/candidate/rustgm`, SHA-256
`bc90b163d1e8dbd8c90cca281aea0768f9bec09a0e2e25426165e4997754d980`.
The exact source snapshots become 38 draft fixtures with per-stage
`option-niche-outcomes.json` in PA26/27/28/30; three already-passing ordinary
counterfactuals remain evidence rather than new earlier-stage oracles. Fixture
bytes and their original hashes are unchanged. The CHECK helper's deliberate
runtime failure function uses the independently declared `--cap-lints=allow`;
ordinary source lint defaults have not changed.

Strict all-target/all-feature Clippy and the architecture audit pass. Ten private
invariants pass, including malformed owner/field/count/encoding, warm validity,
wrong capability, retry after budget exhaustion and shim ABI. Layout/Shape/Variant
remain 48/32/32 bytes; Layouts grows 144 to 152 for one optional cold handle.
Niche is 32, StoredShape 32 and Representation 24 bytes. The release binary's
reduced debug types do not provide a reliable GDB sizeof result; these sizes are
asserted by the compiled private tests instead. Cold wrapper depths 8/32/128 cost
88/346/1378 visits and retain 1088/3104/11168 conservative table bytes. Warm
8/32/128 repeated lookups cost exactly 8/32/128 visits, unchanged by 1024 unrelated
types. Ordinary no-policy layout leaves the niche table absent. These are
logical work/storage checks, not wall/RSS performance neutrality claims.

Full combined qualification and performance remain integrator-owned. General
repr(Rust) direct-tag backing, wide/other niches, NonZero/NonNull and broader
storage/provenance/intrinsic families remain required. The bool-niche oracle
disagreement above remains separate from this guaranteed pointer family.


The terminal closing receipt is `closing-third/`: strict Clippy, 1,725-file
zero-finding audit, assignment layout, upstream verification and grouped suite
registration pass. The two memory source/native tests and C ABI native test pass.
All 8,360 active PA1–24 outputs are byte-identical to the retained qualified
`2dede8e4` baseline outputs. Existing earlier reference differences remain the
baseline's reviewed history; no sidecars are changed. Source-map SHA-256 is
`ecd5e312ad5262070e1b8d10cb1b6e13fc6859de3775cdf87571773df0d6c922`.
`handoff.json` records exact immutable compiler/backend and fixture-contract
hashes. The first two closing attempts retain their test-only helper-registration
and repeated-filename findings; neither reached a new functional assertion.

Focused integrator commands (under the normal resource/storage wrappers):

```sh
cargo test --manifest-path dev/Cargo.toml --release --locked --lib nullable -- --nocapture
cargo test --manifest-path dev/Cargo.toml --release --locked --test suite_memory option_storage:: -- --include-ignored --nocapture
cargo test --manifest-path dev/Cargo.toml --release --locked --test suite_linking option_abi:: -- --include-ignored --nocapture
```

The uninhabited-wrapper program qualifies ordinary None storage/stride, not its
byte conversion: unsupported inactive payload child families still need their
own byte-family investigation. It is not a semantic rejection contract.
The next small implementation recommendation is default multi-variant enums
using their already-retained direct tag and canonical variants, preserving the
separate implementation-defined-layout observation and original positive source.
