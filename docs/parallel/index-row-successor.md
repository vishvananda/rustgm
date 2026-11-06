# Retain each source Index inference row

Full CI [42163466614](https://github.com/vishvananda/rustgm/actions/runs/42163466614)
on da066066d passed formatting, architecture, strict all-target Clippy, the
harness and upstream contracts. Its regular run passed 2726 private checks and
failed five compose-pointer checks plus one public constant-slice Index check.
All six report `source index receiver policy lost its solved borrow` at the same
provider byte. The new paid-query/radix and const-index forgery checks pass in
that run. Later course/native/debug/independent gates were not reached.

`retain_language_index_call` previously retained only the call signature when
no earlier place preparation had created `Places`. The actual method was
published, but its pending-row index was missing for the new same-Solved
receiver verifier. Retain that row in the existing map for every Index call,
as Deref already does. This preserves direct lookup and full callable, actual
borrow, expected receiver, source-node and raw-autoref checks. No scan, extra
map, changed dense record, limit increase or proof exception is introduced.
Storage is proportional to the actual selected Index nodes in the current body.

The [focused evidence](index-row-successor-evidence.json) records all six exact
public controls passing, including the formerly failing constant READ=9 and
WRITE=91, ordinary indexing, its bounded canonical-work control, actual source
Deref and legacy operators. Full formatting, architecture and strict Clippy
pass in 140.70 seconds including resource admission. Seven actual Index native
observers also pass on those same authenticated sources and artifacts. Existing
opponents and fixtures are unchanged; private compose-pointer execution and
full successor qualification remain CI work.

Original core remains at LIB-171 AsyncIterator on the current immutable ae15
probe; this small repair does not claim a new library artifact or whole-PA
release. The two existing workers continue full Field/receiver semantics and
real const cleanup qualification. Root next investigates the independent
Rust-positive alias128 canonical-selection failure using the exact receiver
prerequisites, without raising limits.
