# Mutable slice and array iteration checkpoint

Canonical `&mut [T]` and `&mut [T; N]` IntoIterator implementations, the slice
`iter_mut` method and `core::slice::IterMut<'a, T>` now use ordinary selected
calls and checked generic originals. Elements need neither Copy nor Clone.
The iterator itself implements neither capability. This is a PA26 functional
checkpoint; assignment activation and combined release qualification remain
separate.

## Typed identities, demand and proof

SliceIterMut, MutableSlice and the four mutable collection roles append to the
existing typed identities. IterMut has a private stored `&'a mut [T]` field and
no source constructor. Existing source, blanket and shared collection identities
remain unchanged. The optional mutable resolver and semantic catalogs keep
actual owners, inherited parameters, associated bindings and callable contracts;
spelling and nominal shape alone provide no implementation authority.

The shared collection owner reserves mutable identities after source resolution.
Semantic contracts remain cold for scalar-only programs, custom iterators,
shared-only iteration and source method winners. Typed mutable collection demand
uses the existing ImplHeader/Definition retry. A previously completed shared
group cannot suppress a later mutable request. Unknown or projected subjects
remain pending, and unresolved reference referents preserve the exact outer
reference mutability when filtering possible headers. Named types, qualified
calls and source/prelude lookup retain their existing paths. An array trait
method can win before slice autodereference; no-prelude calls still need imports.

Prepared catalog status is not executable evidence. Ordinary implementation
header and conformance completion must finish before a factory can validate its
Original. That validation checks exact owner, Self, Item, iterator, method,
signature and FunctionRegions scheme. The implementation lifetime is slot 0;
the independent next-receiver borrow is slot 1. Item is `&'a mut T`, so it is not
tied to the duration of a particular call to next. Public callable capture/late
mapping retains its normal canonical representation rather than being equated
to the flattened original proof signature.

The existing intrinsic signature builder now has a source-free facade validating
the actual registered pointer intrinsic, its parameter and owner. Internally
reserved ptr::read stays latent until mutable demand; a real earlier source use
remains eager. The factory independently validates the exact unsafe Rust
`ptr::read::<&'a mut [T]>` instance and complete registered signature. Diagnostic
origin supplies no source body or assumptions. Atomic and pointer-offset
signature builders retain their separate paths.

## Checked next kernel

The empty test uses the actual stored slice metadata before any read. Empty
returns typed None without changing the field. The nonempty path snapshots the
stored unique reference using the exact canonical ptr::read instance, then uses
the existing ConstantIndex and Subslice projections to form the disjoint first
item and remainder. It stores the remainder before exposing Some(first). No
fallible or observable runtime operation intervenes between the snapshot and
restoration, and no element value is read or moved.

Both batch and incremental collectors submit the generated generic originals to
ordinary region, loan, move, ownership and drop verification. Specialized calls
retain all ordinary formation and receiver obligations. A private altered split
with prefix zero is rejected by normal loan verification. The implementation
adds no Rvalue, Owner, BodyKey, primitive Permit, const permission or ABI rule.
Option construction uses typed variants and layout queries, with no assumed
tag, stride or pointer encoding. The subsequent Option niche ABI work is
independent of this logical loop implementation.

## Declared outcomes and qualification

`pa26/tests/mutable-iteration-outcomes.json` records 27 independently declared
sources: 19 accepted native programs and eight language rejections. The original
24 controls are retained unchanged; three additional cases cover generic const
array extents, shorter slice-rest metadata and one-time loop-expression
evaluation. The matrix also covers two simultaneously live yielded items,
items surviving iterator drop, non-Copy Drop elements, empty/ZST arrays,
source lifetime shortening and rejected widening/invariance, source writes,
double borrowing, moved mutable references, named IterMut, qualified/reified
next, source winners/lookalikes and explicit no-prelude imports. The Rust-positive
collection/range originals remain tracked; this closes their mutable subset.

Evidence is retained under worker
`obj/parallel/pa26/mutable-iteration/`. The declared-v1/v2/v3 plans precede
qualification, qualified-v1/v2/v3 retain pinned Rust 1.98 object/native results
and exact old-binary failures, and fixture hashes retain those source bytes.
The b43 baseline pin has SHA-256
`23412183ab099c33c1928347e6054ead6e41adeab90fe273cb6a39d7528eb3a1`.
The early failed probes, GDB traces and superseded validation hypotheses remain
preserved; close-v8 is the coherent functional source snapshot.

Close-v8 passed all 27 source controls and 19 native pairs, the six new private
invariants and existing pending-header invariant, shared and Clone source/native
regressions, strict all-target/all-feature Clippy and architecture audit
(1788 Rust files, zero findings). The private controls include both original
collectors, forged owner/signature/lifetime/read identities, incomplete catalogs,
overlapping split rejection, cold demand and bounded activation. Native tests
are explicitly ignored when the external CPPGM backend is unavailable.

Tail-v9 additionally passed eight shared declaration invariants, existing
volatile and pointer-memory source/native regressions, layout, suite inventory
and pinned upstream checks. Its one PA1–24 comparison preserved all 8360
outcomes. The only changed artifact is PA6 for_binding names: four names, eight
scopes, 27 definitions (76 to 103) and 27 bindings append; all existing lines and
references, stderr and status remain exact. The unchanged source independently
object-compiles under pinned Rust. Exact artifacts and the reviewed bijection
are in text-v1/names.diff and text-v1/names-review.json. No references were
generated or edited.

The immutable default-debug candidate is text-v1/candidate/rustgm, SHA-256
`dc8f0ee1754e03366bbe004a74f140c883c827062cec3f60c1042f11d87bb855`.
Its complete Rust source map and archive are beside the executable. Full release,
immutable performance comparison and publication remain integrator-owned.

## Layout, work and remaining requirements

On the combined 265c715c base, Declarations remains 8552 bytes and Resolution
2152. Type Kind and IR Value remain 24 bytes; Store and inference Context remain
448. No ordinary scalar/call/loop record widens. The demanded semantic collection
Registry is 600 bytes, its optional MutableRegistry is 556, and the optional
resolver MutableCollections record is 232. The declaration-only pending catalog
now retains six collection headers instead of three when canonical collection
identities are reserved, including custom-only iteration. Mutable semantic
ownership and read signatures remain absent on the cold controls; ordinary
scalar-only sources allocate neither catalog.

For 8/32/128 mutable loops, inference work is 3455/13655/54455 and selections
16/64/256. For one loop with 8/32/128 unrelated nominal/function pairs, inference
work is 512/608/992 and selections remain two. Activation adds exactly 16 nominal
requirement steps and 33 variance steps in every case. These are logical-work
observations, not wall-time or instruction claims.

Owned array iteration and integer/char range iteration remain required next
families. Broader executable Clone/derive implementations, structural/str/unit
comparisons, further iterator methods, question-mark and formatting also remain
required PA26 work. This checkpoint does not substitute wrappers for those
original Rust-positive requirements or declare PA26 complete.
