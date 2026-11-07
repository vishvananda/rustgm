# Actual source pointer guaranteed comparison

PA31 owns source header and defining-context authority; PA28 owns the actual
intrinsic contract and PA25 owns reached constant pointer provenance. These are
focused source companions, not the complete standard intrinsic module or a
whole-assignment qualification.

`original-function.txt` retains the exact pinned core intrinsic function.
`predictions-v1.json` froze 34 installed-library sources before Rust 1.98
metadata/object observations. All 68 observations matched; the two real native
observers print `guaranteed 1 0 1` and `guaranteed 99 99 1`. The changed body is
intentional: native execution must execute the actual ordinary fallback body,
while reached intrinsic CTFE has its own semantics.

The `source-*-v2.rs` companions retain that original function (except explicitly
named one-fault/changed-body opponents) with separately authored actual no-core
language-trait interfaces and primitive implementations. This support cutout is
not a full core module. All 30 metadata/object observations match and both native
observers match the installed-library outputs. Version 1 remains unchanged:
three static object observations stopped at missing actual `drop_glue` setup;
these are infrastructure failures, not semantic rejections. Version 2 adds the
real source Destruct/drop-glue interface. Each prediction packet binds exact
input bytes; receipts retain compiler executable, driver, argv and version.

`predictions-source-v3.json` separately freezes alignment, offsets, zero-sized
statics and function pointer companions. Pointer displacement uses the actual
`arith_offset` intrinsic declaration instead of claiming a full pointer-method
module. Its reference results must be recorded before candidate observation.

The pinned primary execution law is in
`compiler/rustc_const_eval/src/const_eval/machine.rs:271-394` and
`interpret/memory.rs:1637`. Integer words compare exactly. Symbolic pointer vs
integer comparison tests possible nullness after wrapping displacement. Same
actual static provenance compares offsets, and distinct static provenance is
unequal only when both pointers are strictly in bounds. Alignment residues can
prove inequality. Other allocation identities, including the same promoted or
local allocation, remain unknown (`2`). Pure comparison does not dereference the
pointee; the expired-pointer/null case is a real positive.

The general marker packet keeps declaration policy separate from execution.
An unused ordinary marked function is Rust-positive. Both reached ordinary
marked cases reject E0080, including the helper-returning-99 case whose uncertain
positive prediction was disproved. The unchanged old private indirect-marker
opponent is independently positive in `reference-indirect-v1.json`. This does
not turn a marker into const-call, intrinsic, unsafe or body permission.

Actual declaration feature/signature/safety/arity checks, nonconst calls,
reification, unsafe body typing and sized target formation have independent
negative controls. No unsafe invalid-dereference construction is used as a
negative. Unsupported valid symbolic representations, imported operation proof,
broader marker contexts, wire transport and complete standard-module closure
remain required unless separately qualified.

The candidate's authored Source feature profile is separately frozen in
`predictions-source-profile-v1.json`: 35 sources / 70 matching metadata and
object observations plus both native observers. It omits both unused authored
registration declaration and feature request. All earlier original requests and
failure receipts remain unchanged PA40 requirements. The original intrinsic
function is still complete and byte-identical. The owning public Rust tests use
this declared support profile; they do not count the request-bearing cases as
candidate successes. `predictions-storage-v1.json` separately freezes one-past
nullness and a live/expired actual read pair: the pure comparison cannot grant
pointee read or liveness permission.

The separately frozen operator packet changes the actual source Eq method to
return false. Pinned Rust metadata/object/native accepts and prints
`primitive-pointer 1 0 0 0`: operator syntax remains primitive, explicit method
calls execute the changed body. This substantiates the shared primitive operator
join; it is not permission to replace the intrinsic fallback body. All ordinary
pointer promotion and const comparison fences remain independent.
