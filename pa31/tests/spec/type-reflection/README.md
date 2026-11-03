# Type-reflection source declarations

PA31 owns the actual `type_info` and `type_id` language identities. PA25 owns
reflection constant evaluation/provenance and PA29 owns genuine imported values
and nominal contracts. The assignment remains planned; these contracts do not
activate an empty grading lane.

`contract-v3.json` declares 27 source outcomes, all independently observed with
pinned Rust 1.98.0. Target, generic/field freedom, renamed/nested owners, duplicate,
cfg and feature cases use the actual source attributes. Its C observer prints
`7` from ordinary source field access. This is not intrinsic reflection.

The pinned `GenericRequirement::None` restricts neither arity nor field shape.
Consumers of `type_id`/`type_of` still need their actual result/input declarations
and checked representations. The original TypeId stores two pointer-sized words
on this target, carrying type provenance in CTFE and a type hash at runtime.
Replacing that provenance with arbitrary scalar bits or a local TypeId index
would not implement the contract. Required follow-ups include forged TypeId
rejection, actual reflected kinds/fields/offsets, unsized types, lifetimes,
canonical sharing, stable imported identity and genuine const/native consumers.

The original v1 sources and `setup-v2/` remain unchanged. The first observation
exposed missing Copy support in the authored no_core setup; the second exposed
missing usize marker impls under min_adt_const_params. `setup-v3/` adds those real
prerequisites without changing any outcome prediction. These failures are setup
evidence, not Rust language rejections. Independent diagnostics, commands and
hashes are retained under obj/parallel/type-reflection-v1/oracle-v1 through v3.

The maintainer tests consume all v3 cases, lower the actual ordinary field body,
check the real pointer-array ABI layout and preserve feature, unsupported-intrinsic
and old-metadata transport boundaries. Original core/alloc/std artifacts and the
complete reflection implementation remain required.
