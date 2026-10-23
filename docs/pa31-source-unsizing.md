# Actual source Unsize and CoerceUnsized

This prerequisite adds actual source roles 18/19 on the sized destruction-helper
checkpoint `29f7ae1`. It does not activate PA31 or implement unsized cleanup.
The pinned source is Rust 1.98.0 commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`:
[Unsize](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/marker.rs#L225-L236)
and [CoerceUnsized with its reference/raw implementations](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ops/unsize.rs#L34-L73).

## Typed boundary

Reservation retains the actual defining identity. Sealing validates the complete
safe ordinary marker header, its actual PointeeSized/Sized supertrait and one
PointeeSized target parameter. Unsize requires explicit-implementation denial
and its pinned dyn-incompatible claim. The latter is enforced for this sealed
role; general use of the attribute remains an unsupported source prerequisite.
No spelling, matching layout, role reservation or cached classification proves a
goal. Missing and reserved-but-unvalidated roles remain distinct.

Ordinary array-to-slice selection proves element equality, including zero extent
and nested callable binders. It cannot substitute lifetime shortening. Concrete
to dyn selection retains every principal, associated equality, auto, Sized and
outlives requirement through ordinary scoped child selection. Both endpoints
also retain actual PointeeSized formation duties. This trait-only formation does
not pay nominal lifetime requirements. The Unsize consumer reads the existing
completed `symbolic_requirements` facts for each resolved endpoint and retains
region/type-outlives debts in source=None prefix nodes. Projected debts use the
existing whole-clause lift and callable-depth rules. The actual caller must pay
these debts; physical shape supplies no lifetime permission. Child proofs, region
equalities and allocated frontiers survive inference suspension. The fresh exact-role gate
precedes query memo reads; completed Builtin classification alone is not exported
as a reusable obligation-free Unsize proof.

A shape mismatch or deferred semantic child still checks both endpoint formation
goals. A known malformed peer cannot hide behind an unknown head. Unpaid prefixes
block completed absence, including inherited projection/region prefixes on warm
negative-cache hits. A forced Deferred result retains known endpoint debt without
exporting an Unsize proof or completed absence. Positive results keep the complete
transitive prefix. The correction reuses ordinary body/annotation lifetime facts;
it does not redefine PointeeSized as a lifetime-validity or access permission.

CoerceUnsized remains a real declared implementation. Its reference/raw conformance
consumer proves the exact referent Unsize relation in that implementation's
actual environment, rejects mutability strengthening, and checks the reference
outlives relation through the ordinary region solver. No const obligation or
pointer access permission is inferred. There is no backend, Frame, Source,
Owner, Machine or CTFE-cleanup change.

## Declared qualification

`pa31/tests/spec/unsizing-roles/contract.json` preserves 46 exact sources and
independent expectations. The four predeclared oracle plans are under
`obj/parallel/pa28/source-unsizing-roles/declared-v1`, `declared-v2`,
`declared-v3` and `declared-v4`. All 46 metadata/object outcomes matched pinned Rust;
22 independent native observers returned 7. Nineteen positive sources and 24 language negatives
form the bounded candidate family, with 19 corresponding ignored native observers.
The other three Rust-positive sources remain required: unsafe marker headers,
nominal Unsize tails and nominal CoerceUnsized implementations. They are never
used as language rejection contracts.

The source cases include generic extents, element inference, equal versus merely
outliving lifetimes, scoped object escape, associated-type mismatch, callable
binder correlation, explicit Unsize implementations, invalid Coerce impls, and
same-spelled ordinary/foreign traits. Native goal observers establish successful
checked execution; they do not assert a portable C fat-pointer ABI.

The width/caller matrix uses independently qualified dyn associated-requirement
sources at 8/32/128. Private counters separately record cold preparation and warm
query work, selected goals/children capacities and suspended rows, with 8192
unrelated types. The original cold-equals-warm work hypothesis was falsified and
retained. The pre-lifetime-fix counts were cold 671/2255/8591 versus
warm 617/2081/7937. Final corrected counts are cold 684/2292/8724 versus
warm 630/2118/8070; the additional existing lifetime-fact visits remain linear. Repeated warm queries are exact multiples; unrelated Store growth
has no effect. Initial goal capacities are 16/64/256; selected child capacities
are 16/64/256 and suspended capacity is zero for these closed controls. These are
selected container-capacity measurements, not whole-heap claims. Failed plans
also consume the existing cumulative inference work budget; the exhaustion and
ordinary no-role controls retain no language payload allocation.

Catalog/Reservations grow 464/472 to 504/512 bytes for two 20-byte declaration slots.
The ordinary Declarations 7048, Frame 1312, Node 160, Selection 136, Body 448 and
Store 464 layouts remain unchanged. Existing iterator/collection aggregate size
invariants are included. The private Pending is 72 bytes. Its compact phase and
checked u32 cursor preserve the existing 80-byte Mechanical owner. Every goal is charged under the existing
1,000,000-step inference budget; row-count and host-index conversions plus cursor
increment are independently checked; earlier 96/88-byte observations are retained.

Focused commands use `scripts/storage.py run --no-prune --` with
`cargo clippy --manifest-path dev/Cargo.toml --all-targets --all-features --locked
-- -D warnings`, the file audit, and `scripts/maintainer_suites.py batch
coercion_provider` plus exact private cases. `focused-v1` through the successful
`focused-v9` retain all outputs, binary hashes and source inventories; the handoff receipt names the
final successful snapshot: 43 source outcomes, 18 private controls, 19 native observers,
strict Clippy, audit 2021/0, 152 harness checks and upstream verification. Root owns
full combined release and performance gates.

## Required continuation

Nominal tails/coercions, dyn upcasts, quantified object requirements and general
attribute support remain required. The unchanged slice/dyn drop-glue sources
still need metadata-aware runtime recipes/ABI adaptation. Their independently
accepted CTFE forms need a real bounded cleanup continuation and const proof;
the existing actual-helper placeholder denial stays intact. Owned Box ordering,
full Drop/Pin/default overrides, source-string panic forms and all other retained
PA27/PA28 gaps remain required. Ordinary upstream routines stay in Rust source.
