# Source auto traits in object bounds

Fourteen inputs check how source-declared auto traits classify `dyn` bounds.
The prefix declares pinned core's auto traits: Send with only a diagnostic item,
Sync with a lang item and a diagnostic item, and Unpin with a lang item. Pinned
Rust 1.98 checked all fourteen in metadata and object modes (28 checks), and
every outcome matched the frozen predictions.

- Accepted (8): `dyn Any + Send` and `dyn Any + Send + Sync` beside `dyn Any`
  as distinct impl self types (the `core/src/any.rs:162`/`:169` shape);
  an auto trait before the principal; auto-only objects; Unpin; Send/Sync
  demands discharged by the object's own bounds; a Send supertrait; inherent
  impls on local principal objects; and a custom `auto trait` beside a principal.
- Rejected (6): two ordinary principals (E0225); a non-auto trait spelled `Send`
  (E0225, so the spelling grants nothing); `Send<u8>` (E0107); a missing Send
  demand (E0277); `dyn Any + Send` and `dyn Send + Any` being one type (E0119);
  and an inherent impl on an auto-only object (E0785).

`predictions-v1.json` was frozen before any observation. Its run produced six
no_core setup failures (missing `copy`/`legacy_receiver` lang items); those
inputs stay byte-for-byte in this directory, and their failures are recorded in
`reference-v1.json` as setup failures, not as semantic outcomes.
`predictions-v2.json` was frozen before its observation. The `v2/` inputs add
only the receiver, Clone and Copy foundation lines; each case line and
expectation is unchanged. `manifest.json` lists every input and binds the
reference.

The RustGM candidate matches 13 of the 14 cases. `v2/custom-auto.rs` is a
recorded gap: no object marker exists for a custom auto trait, so it is refused
with "object auto traits without a represented marker are not implemented"
instead of being accepted.
