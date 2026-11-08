# Auto-trait declaration transport packet

`auto-declaration-outcomes.json` freezes one provider and ten clients: their
hashes, edition, target, independent outcomes, and the generated and wire
controls. They were declared at 2026-11-08T12:45:17Z, before any oracle or
candidate run of these files. The two duplicate-identity clients are an
amendment declared at 12:49:06Z, also before any run of them.

The provider is `spec/lifetime-nominals/provider.rs` (lines 1-147, unchanged)
followed by a tail section. The shared lines carry the attribution inherited
from the earlier packets, and the repository NOTICE applies to them. The tail
holds:

- core 1.98.0's `Send`, `Sync`, `UnsafeUnpin` and `Unpin` declarations
  (`core/src/marker.rs`) and its `UnwindSafe` and `RefUnwindSafe` declarations
  (`core/src/panic/unwind_safe.rs`), each keeping its `lang` or
  `rustc_diagnostic_item` attribute and dropping its stability,
  `on_unimplemented` and documentation attributes;
- authored nominals `Pair<T>`, `Raw { p: *const u8 }` and `Either<A, B>`,
  whose auto traits follow their fields structurally;
- `answer`, a plain function used by the wire controls' consumer.

The core-derived declarations are copied from the Rust project under its
MIT/Apache-2.0 licence, and the repository NOTICE applies to them. The
provider declares no negative impls, so a raw pointer is structurally Send
and Sync here, unlike in core.

Seven clients prove an imported marker and return 42:

- `send-structural`: `Pair<u8>`, `&u8` and `Either<u8, Pair<u8>>` are Send;
- `sync-static`: two module statics, one of type `Pair<u8>`;
- `local-static`: a function-local static;
- `pin-markers`: `Unpin`, `UnsafeUnpin` and `Freeze` over imported nominals;
- `bounded-param`: `Pair<T>: Sync` from `T: Sync`;
- `local-holder`: a local struct holding `Pair<Raw>` is Sync;
- `answer`: calls `base::answer`.

Three clients are rejected:

- `unbounded-param`: `Pair<T>: Send` without `T: Send` (E0277);
- `duplicate-lang`: a client's own `#[lang = "sync"]` beside the imported one
  (E0152);
- `duplicate-send`: a client's own `Send` diagnostic item (rustc's duplicate
  diagnostic item error).

rustc 1.98.0 gave all ten declared outcomes, and the seven accepted clients
exit 42 natively. For `unbounded-param` it words E0277 as "the trait bound
`T: Send` is not satisfied in `Pair<T>`". That is recorded beside the
declared text, which was std's `on_unimplemented` message.
