# Pinned receiver method calls (source core)

Twelve no_core source-core libraries reduce pinned core's `SyncView::poll` 6928
(`sync/sync_view.rs:296`) and `resume` 6932, checked against rustc 1.98. Each
case's first line names its prefix, which runs through the `Deref` trait of
`pa31/tests/spec/source-div/foundation.rs`. Each case then adds a reduced `Pin`
with its `LegacyReceiver` and `Deref` impls, and core's `#[repr(transparent)]
SyncView<T: ?Sized>`.

`self.as_pin_mut()` on `Pin<&mut SyncView<T>>` names an inherent method of
`SyncView` declared with `self: Pin<&mut Self>`. Without the receiver features,
rustc assembles inherent methods from each Deref step and matches each declared
receiver against the outer one, so the call takes the `Pin` by value.

- `core-poll` and `core-resume` are core's forms, by method call and inside
  `G::resume(self.as_pin_mut(), arg)`. `sized-poll` uses a `Sized` impl;
- `free-generic`, `free-concrete` and `pin-ref` call `as_pin_mut` or a
  `Pin<&Self>` method from free functions;
- `shared-pin-named` (a `&self` method reached through `Pin`'s `Deref` beside
  the pinned one), `path-call` and `mut-receiver` are controls;
- `missing` names no method (E0599), `unpinned` calls the pinned method on
  `&mut SyncView<T>` (E0599), and `two-owners` adds a same-named by-value
  method on `Pin` itself (E0034).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run; both are recorded in `outcomes.json` (12/12 agree). One malformed
declaration is listed there and excluded. It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/smart_receivers/pinned_calls.rs` checks every case through the CLI
and lowers each positive. See `docs/pa43-pinned-method-calls.md`.
