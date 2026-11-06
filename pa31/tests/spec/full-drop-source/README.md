# Complete source Drop and pinned cleanup

Every provider embeds the unchanged pinned `library/core/src/ops/drop.rs` after
an authored no_core foundation. The original remains in
`../source-const-drop/original/drop.rs`, with its upstream checksum and license
record in `third_party/sources.json`. The foundation's genuine `drop_glue` helper
comes from `library/core/src/ptr/mod.rs:822–830`; its ordinary source placeholder
is replaced only by the existing checked compiler operation. Initial independent
native attempts without that helper remain retained as setup failures.

`predictions.json` freezes twelve source outcomes and four C-observer traces.
Pinned Rust 1.98 independently confirms every outcome and native trace. The
candidate checks complete source, actual ordinary and pin-only implementations,
generic applications, missing/conflicting overrides, wrong signatures and
explicit destructor calls. A subtrait default bridge cannot call destructor
methods merely because it has a Drop supertrait. Existing independent unused
by-value and non-unit Drop declarations remain legal; exact executable signature
requirements apply when actual implementations are registered.

The provided `Drop::drop` body constructs the actual Pin and calls the actual
selected pin_drop override through ordinary checked dispatch. Both non-generic
and generic pin-only cleanup must use that provided callable, preserving its
source identity, signature, substitution and normal body proof. No copied body or
ABI coincidence supplies authority. The native traces are `23 7`, `23 7`, `17 7`
and `19 7`, each followed by LF. A persistent ignored maintainer test executes all
four through the locked CPPGM object backend.

This finite packet does not release PA31/26/29 or produce an original library
artifact. Wider pin_v2/projection, async destruction, CTFE and authenticated
destructor/body metadata remain required. See the
[checkpoint](../../../../docs/parallel/full-drop-source-checkpoint.md).
