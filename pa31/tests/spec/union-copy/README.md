# Actual source union Copy

`contract.json` freezes source hashes, Rust1.98/2024 Linux target settings and
independent outcomes before observation. The original MaybeUninit union and its
following authored Clone implementation are byte-identical pinned intervals from
`library/core/src/mem/maybe_uninit.rs`. See NOTICE and third_party/sources.json.
That producer is expansion/receipt evidence. Its real language role, ManuallyDrop
and transparent-union semantics and full runtime remain separate requirements.

`source-runtime.rs` reuses the qualified SourceCore foundation, declares a real
source-derived Copy union with type/const/where parameters, and implements Clone
as a whole-value `*self` copy. The observer checks initialized nonzero bits and
completion of copies whose payload bytes were never initialized; those copies
never read a union field. Expected output is
`11223344:11223344:11223344;u;7\n` with status0.

Missing Clone, a different trait aliased as Copy, a non-Copy reference field,
private access and a field read outside unsafe retain real failures. Source union
Clone was explicitly Required in this historical contract; the successor
`../union-clone/` qualifies its actual recipe and ordinary Copy obligation.
The unchanged `clone-required.rs` now remains a successful expansion control;
its missing actual Copy helper prevents a full compilation claim. The original
contract and independent receipts remain unchanged. Its incomplete NoCore helper reducer failed
the first Rust observation; that mismatch is retained. The separately labelled
ordinary Rust Clone positive establishes the valid language boundary without
substituting a std implementation into the SourceCore runtime producer. Debug is
a real Rust negative and cannot enter a struct/enum generated body recipe.

Maintainer controls are registered through derived_locations.rs and
copied_union_derives.rs in the existing source derive invariant suite.
