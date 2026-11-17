# Explicit nominal coercion casts

These16 complete sources preserve the predeclared Box-coercion packet bytes.
The [outcome manifest](outcomes.json) records independent pinned Rust1.98 metadata
acceptance/rejection and diagnostic codes. The candidate matches all16 complete
source outcomes and19 existing Box controls. All15 candidate native programs
pass, including the two new cases with independently checked allocation, drop,
free and result traces. Five private controls remain pending CI after the linked
harness build timed out before execution. All751 PA24 text comparisons, strict
selected Clippy, formatting, architecture, harness and upstream checks pass.

The source cases require the existing nominal `CoerceUnsized` goal and final-field
recipe. They cover generic arrays, a method result, caller premises, a once-only
slice conversion and dynamic dispatch. Different owners, changed non-tail allocator
arguments, wrong elements, reverse conversion, missing implementations, stronger
mutability, missing object implementations, moved sources and moves from Drop
content remain rejected. `ordinary-reference.rs` and `ordinary-nominal.rs` preserve
the declared frozen bootstrap controls byte for byte.

The complete source prefix is copied from the already-qualified `owned-boxes`
fixture family. `prefix.rs` records that shared foundation; it is not a standalone
case. The unchanged C observer records allocation, content drops, deallocation and
`run`'s result. The two declared native traces require one source evaluation and
ordinary move/drop behavior. No standard-library semantics are synthesized here.

The actual source caller is alloc's `boxed.rs:10093`
`box_assume_init_into_vec_unsafe`: its explicit `Box<[T;N]> as Box<[T]>` conversion
must consume the selected adjustment. The ordinary PA24 bootstrap, backend ABI,
source trait authority and remaining core/alloc/std requirements stay unchanged.

The new semantic operation consumes the operand's exact unadjusted type and
selected nominal conversion. Body construction forwards its existing adjusted
`Unsize` value and authenticated `NominalUse`; it creates no aggregate pointer
cast. Source-negative checks cannot pass through an internal endpoint or
provenance failure. Existing sources, independent outcomes and backend forms
remain unchanged.

The clean compiler preserves both ordinary controls byte-for-byte. Original
Vec now compiles, links and matches all 1,494 golden bytes in three fresh native
runs with an empty environment. The original 20-row skip file is preserved.
A derived 19-row list executes the repaired Box body but still aborts; removing
only the additional obsolete `str.rs:5886` skip produces the passing 18-row list.
All 801 library inputs and the original source/golden bytes remain unchanged.
No app blockers, unproven bodies or skips satisfy this result. The final live
allocation is the declared `Vec::leak`, exactly 1 block/4 bytes. Other consumers,
full library artifacts and strict qualification remain required.
