# Cold no-body declarations

Twenty independently frozen inputs exercise real trait prototypes and foreign
functions, with normal declaration, call, lint and ABI checks. All forty Rust
1.98 metadata/object outcomes match the predictions. The sole native program
links actual unsafe and explicitly safe C functions and prints
`cold-foreign 28 36 2`; the final field counts real foreign calls.

Pinned ColdParser permits foreign functions. Required trait methods receive an
unused_attributes diagnostic: default/Allow accepts, Deny rejects. Duplicate
handling stays independent. Cold never provides a required implementation or
caller bound, and grants no unsafe or const call permission. Free functions still
need a definition. Invalid ABI, result and foreign type parameters remain errors.
The native support is a separately authored no_core companion reusing the earlier
source-cold callable foundation, not a claim that the whole standard module graph
or arbitrary foreign profiles are complete.

reference-v2.json binds the exact source hashes, resolved rustc executable/driver,
argv, forty compiler outcomes and the actual native observer. The earlier runner
incorrectly linked that observer to unrelated header-only objects; seven setup
link failures are preserved under obj/parallel/source-transport/cold-prototypes/
reference-v1, and are not semantic rejections. V2 changes only runner selection,
with all original input and prediction bytes intact.

The earlier two accepted inputs prototype-required.rs and foreign-required.rs in
source-cold are preserved unchanged. Their candidate no-body fences are closed
by actual bound owner checks in the existing header producer. Advisory metadata
export, generated body/const execution and wider source-parent profiles retain
their separate existing contracts and explicit requirements.
