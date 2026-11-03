# Transmutability layout-language kernel

The private ABI kernel now describes possible values from completed physical
layouts and produces a Boolean/obligation DAG. This is the data foundation for
LIB-124. It does not yet authorize implicit source selection, a default-method
call, recursive trait proof or imported execution. PA31 remains incomplete.

## Structure and bounds

Hash-consed sequences, alternatives and counted repeats retain correlated enum
tags and payloads without enumerating bit patterns or array elements. The typed
adapter consumes actual field offsets, union alternatives and reference layouts;
it never reparses source or rendered output. Padding accepts the uninitialized
symbol, which is distinct from an initialized byte. Nominal safety guards survive
zero-sized fields. Unsupported raw/function-pointer and wider physical forms fail
explicitly; the focused controls do not qualify every niche/refinement profile.

Normal validity requires every source transition. Assumed validity permits a
viable transition. Source truncation and uninitialized destination extension
remain separate rules. Reference answers retain actual referent/region identities,
recursive transmutability, outlives and shared-destination Freeze obligations;
mutable destinations also retain reverse obligations. These atoms are data,
not completed caller evidence or execution permission.

Derivative and pair caches are query-local. Iterative stacks avoid recursive
graph traversal. Product/repeat factoring handles billion-element arrays without
expansion. Both roots share an interpretation memo keyed by language identity
and safety policy: the initial deep-layout failure exposed duplicate traversal
of the same graph, which was removed without raising its work bound. Growth
and work are charged to the supplied owner before allocation.

## Qualification

Fastcheck `1793742663407949535` passes ten exact private tests, strict Clippy,
formatting and architecture checks in 121.68 seconds including admission. It
checks all 512 relations between independently enumerated two-byte languages,
truncation/padding/zero-size guards, reference obligations, a 20,000-level graph,
billion-element arrays, actual ABI padding/unions/enum tags and pointer opponents.
The unchanged source-role layout bound also passes after its exact provider and
imported sizes were corrected to 1360 and 1432 bytes. CI41718752648 had failed
the stale sizes after 2591 other private tests passed; that run remains failed.

Receipt: `obj/parallel/transmutability-v1/layout-kernel-v1/receipt.json`, SHA-256
`a928ddfc75314976fb640462b0610d597bb322b2d2ff0f4226b99c174529420d`.

## Actual core frontier and next work

A separate unchanged-core observation uses the already published header compiler
`b9ce8505b`, immutable CLI `5d9a9ce5`, rather than this new kernel. It clears the
previous transmutability declaration boundary and reaches LIB-125:
`#[lang = "type_info"]` in `library/core/src/mem/type_info.rs:13`, file61 byte329.
The run takes 17.66 seconds, opens 375 sources and verifies all722 source and
six configuration hashes, with target/options/cap/finite limits unchanged.
Result: `obj/parallel/transmutability-v1/headers-core-v1/result.json`, SHA-256
`5e228ebaa519354d130ea8464569cf88d80d8cb1a6d95ba507469424dc067ed1`.

Prioritize the actual type-reflection source blocker. Retain LIB-124's remaining
scoped selection/coinduction, lifetime/Freeze evidence, real source default-method
native dispatch, complete physical profiles and PA29 nominal/constant transport.
No original core/alloc/std artifact or assignment release is established here.
