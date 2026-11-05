# Authored future runtime contract

`predictions.json` was frozen before running pinned Rust1.98.0. It binds thirteen
compiler outcomes and eight native traces. Six traces exercise the first no-await
runtime milestone; the two Pending/await traces retain required suspension and
cancellation behavior for subsequent work. This packet does not activate PA42/43.

The native source uses real standard Future, Pin, Poll, Context and Waker values.
Every future stays in its original storage from unchecked pin construction until
implicit in-place destruction. The output can move independently after Ready.
The traces distinguish argument evaluation from deferred body/pattern execution,
unpolled capture order, completed destruction, returned captures and retained
borrows. Static opponents preserve !Unpin/!Copy/!Clone, conditional Send/Sync,
constructor moves, loan lifetime, actual Output and unused body checking.

Independent receipts are retained under `obj/parallel/future-runtime-v1/`.
The initial twelve statuses and all eight native traces match the original predictions.
The original E0277 prediction for captured !Send differed: Rust emitted its
dedicated unnumbered async diagnostic with the predicted raw-pointer capture
cause. `predictions.initial.json` and the original failed receipt are preserved;
`review.json` records only that diagnostic-code correction, without changing any
source, status or native trace. An additional independently frozen positive covers a Send+Sync future returning a
!Send raw-pointer output; its receipt is `returned-nonsend.json`. Compilation used edition2024/Linux x86-64,
panic=abort, overflow checks, two compiler threads and two codegen units.

There is no candidate runtime/native evidence yet. Compiler-owned
async_drop_in_place, arbitrary authored async metadata, full await/cancellation,
self references and recursive async destruction remain separate requirements.
