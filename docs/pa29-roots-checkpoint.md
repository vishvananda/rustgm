# Shared executable/library roots

`lower::Roots { entry: Option<DefId>, exports: &[DefId] }` selects an executable
adapter or entry-free library exports. `lower::instances::build_roots_prepared`
uses the existing checked template, ownership, region, destruction, promotion,
constant and instance gates. Existing executable APIs delegate to the same path.
Registry exports are merged with explicit roots once, sorted and deduplicated;
the 100000-root occurrence budget is checked before publication. No export vector
is allocated for the ordinary empty-export/no-registry path.

`Registry::exports()` returns canonical Export function IDs in deterministic
order. Imported functions never require invented local CFGs. Private generic
dependencies still create distinct checked instances; the two-export control
demands `id<u8>` and `id<u32>` separately. Empty library roots emit no adapter,
while unused invalid bodies still fail their ordinary proof obligations.

Packed pointer fields currently require runtime stores emitted in the executable
initialization adapter. Entry-free lowering rejects pending packed fixups with
`entry-free library requires packed address initialization support`. Aligned
backend relocations remain accepted. Library startup-initializer transport is
subsequent work; no zero placeholder pointers are accepted as final library data.

Independent qualification also established that aliasing a source foreign-block
function as `main` is Rust-invalid. Entry selection now rejects that typed
`foreign_owners` case. This is distinct from an ordinary Rust function imported
through future crate metadata; that path will receive separate qualification.

Validation passed: check, six new root tests, four symbol tests, three initializer
tests (including four existing native value cases), strict all-target/all-feature
Clippy, formatting and the architecture/file audit. Eight independent pinned
Rust object outcomes cover six accepted sources and two rejected sources. The
packed source is independently valid but remains an explicit entry-free boundary.
Four frozen ordinary/repeated-root executable outputs match the immutable
`cf06d8fd` binary byte for byte. Initial function-size and test-style lint findings
were refactored without weakening the checks.

Evidence: `obj/parallel/pa29-roots/` contains individual gate logs, qualification
sources/diagnostics, `qualification.json`, `default-output-equality.json` and the
source manifest. Manifest SHA-256:
`3be4098e8155c18c6dc90cec43dcd1afd1a9a3c828c5614bc8c7bc7b8f5fb039`.
Final resource receipt: `f2d501b903d34cbd8a41ee972fa31c9c`, exit zero.

This API checkpoint depends on the qualified Naming/Registry checkpoints
`d20f136b` and `8de31d05`. It does not implement crate identity transport,
metadata codecs/import remapping, public export collection or library CLI modes;
those remain the next PA29 slice. It does not activate either assignment.
