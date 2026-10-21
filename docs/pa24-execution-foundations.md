# PA24 checked execution foundations

The public closure callable signature remains separate from the optional checked
execution signature retained by `Owner::Closure`. Existing noncapturing bodies
publish no execution prefix and retain their existing rendering and lowering.
This checkpoint supplies the neutral vocabulary; it does not yet implement
captured environment construction or invocation.

`ProjectionKind::CaptureBoundary { mutable }` is inserted at a logical captured
binding after any compiler-inserted dereference of the stored capture reference.
It preserves the physical place and field path. Immutable binding authority
survives owned field/index descendants, and only a subsequent source reference
or raw dereference clears it. A source binding can therefore remain immutable
while the referent of its contained mutable reference stays writable. Native
projection carries the same address and any wide metadata; loan/move paths do
not gain a synthetic owned root. Reached constant execution rejects the capture
boundary explicitly for the current runtime-only profile.

`BorrowKind::UniqueImmutable` provides immediate exclusive capture formation
without making the source binding replaceable. It still requires initialized,
nonshared storage and a mutable internal reference type. Ordinary mutable borrow
formation keeps its binding-mutability requirement. Guard ownership and its
strict immutable views are unchanged; owned captures use physical field paths.

Four new invariant groups cover owned field/index restrictions, stored versus
source dereferences, unique formation versus ordinary mutable/shared formation,
and shared physical move state across a capture boundary. Together with guard
view lowering and existing source/generic/lifetime closure controls, all 37
focused groups pass, including their native controls. Strict all-target/all-
feature Clippy passes; architecture audit covers 1,299 files with zero findings;
layout covers 38 stages. Evidence is retained in
`obj/parallel/pa24/capture-execution/foundations-second.json` and its logs.
The initial exhaustive projection match and lint failures remain recorded.

Next consumers are capture place/environment construction, distinct execution
contracts and ordinary checked call entries. Stronger Fn/FnMut/FnOnce adapters,
nested formation transport, generic nominal formation qualification and full
release/performance checks are not claimed here.
