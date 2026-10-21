# PA24 borrowed closure receiver adapters

This checkpoint extends the owned adapter implementation on integrated source
`4a8f6bce`. It supports exactly one shared or mutable reference to a captured
closure whose owner has no type or const parameters. PA24 remains planned.

The adapter identity now retains the exact selected receiver Self as well as
the complete leaf closure instance and requested call trait. Owned and borrowed
receivers cannot share a wrapper ABI or cache entry. Selection retains the
existing canonical blanket-reference child proof; a shared permission helper
does not add selection or proof authority. The typed wrapper owns only its
reference, dereferences it and reborrows the closure before calling the original
strongest checked entry. Ordinary cleanup leaves the borrowed environment alive.
The exact original source proof remains required in final assembly.

Sparse source-slot matching preserves all source universals, including unused
slots. A reference-prefix lifetime reuses an exactly matching source region or
gets an appended slot; the hidden receiver lifetime follows it. Repeated-slot
equalities remain caller obligations. Nested callable binders remain rigid, and
a callable-local lifetime cannot be exported through this mapping. There are no
new source assumptions or factory permissions. Body views and stable owner
ordering include the complete receiver identity.

The independently declared matrix now has 34 unchanged, hash-checked fixtures:
20 Rust acceptances and 14 rejections. Nineteen accepted programs execute through
the external CPPGM backend, including shared/mutable forwarding, external-reference
results and environment drop controls. The remaining accepted nested-reference
captured case is an explicit implementation boundary. The three former reference
boundaries retain their superseded evidence in the outcome metadata. Generic-owner
and dynamic callable adapters remain separate work.

Evidence is retained under `obj/parallel/pa24/receiver-adapters/`:

- `borrowed-qualification.json` qualifies all 34 fixtures before implementation.
- `borrowed-first.json` records the complete source/native matrix passing.
- `borrowed-invariants-second.json` records seven mapping and nine adapter
  proof/cache invariants passing. An earlier assertion incorrectly assumed the
  orientation of a returned equality; that failed observation is retained.
- `borrowed-static-second.json` records strict release all-target/all-feature
  Clippy, file audit 1379/0, layout38 and pinned upstream24+126 agreement.
- `borrowed-final.json` records 83 affected proof/source/native groups passing,
  then the retained command error selecting nonexistent target `nested_outputs`.
  `borrowed-finish.json` records the corrected `nested_evaluation` target's 26
  passing groups and final identity-view strict Clippy, audit1379/0, layout38 and
  upstream24+126 checks. The affected functional total is 109 passing groups.

Widths 8/32/128 retain exactly two borrowed wrappers, with total checked bodies
13/37/133 and CFG bytes 17,960/49,608/181,320. This is a bounded linear envelope
with a capacity transition, rather than an exact increment claim. Corresponding
owned controls retain two wrappers and CFG bytes 16,440/49,368/181,080. No wall-time
performance claim is made. The root's newer captured-output producer checkpoint
`96c37682` awaits combined integration; these observations describe this worker's
frozen source. No assignment activation or reference regeneration is included.

The later [nested receiver checkpoint](pa24-nested-receivers.md) extends
ordinary captured invocation through reference chains and supersedes the
remaining nested-reference boundary, retaining the earlier evidence.
