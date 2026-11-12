# PA43 fixture ownership

Planned: no graded fixtures or generated reference outputs yet.

Required families: cancel at each state; drop order; self references; async closures/traits.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.

`spec/pinned-method-calls/` freezes twelve declared no_core reductions of
core's `SyncView::poll` and `resume`. Their `self.as_pin_mut()` on
`Pin<&mut SyncView<T>>` names an inherent `self: Pin<&mut Self>` method,
discovered along the LegacyReceiver/Deref chain and matched against the outer
receiver. Six positives (core's two forms, a `Sized` impl, generic and concrete
free functions, a `Pin<&Self>` receiver) and three controls (a `&self` method
through `Pin`'s `Deref`, the path call, a `&mut self` method) accept. A missing
method and an unpinned receiver reject (E0599). A same-named by-value method on
`Pin` makes the call ambiguous (E0034). A maintainer integration test checks
them through the CLI and lowers each positive. There is no graded manifest.
