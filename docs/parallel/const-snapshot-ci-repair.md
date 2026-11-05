# Const snapshot and instance-scope CI controls

Full CI42075839657 exposed two stale private test assumptions after the selected
future/native-context integration. Instance-scope opponents now require the exact
earliest authenticated failure for each mutation: selected public signature for
a foreign instance or absent execution signature, original identity for a foreign
source contract, and scope/environment identity for the remaining mutations.
Every opponent still requires a rejection and an actual charged check.

The ordinary const-call positive now demands `answer()` from a real source
constant and observes its actual Code-purpose bodies after region and loan proof.
It never grants const permission to a native runtime body by changing its context.
The deliberately forged conditional runtime caller remains an opponent, tested
with both cold and warm callee code. The ordinary positive still requires its
actual target code; omitting that code must fail.

Pinned independent Rust1.98 accepts both complete source fixtures, including the
new genuine constant consumer. Predictions, source bytes, diagnostics and command
results are retained under
`obj/parallel/const-snapshot-ci-repair-v1/reference/`.

Strict all-target/all-feature Clippy passes, including both private test bodies,
on the joined development source. Cargo reports2m08s. This is compilation and
lint evidence; execution of these two private controls remains for successor CI.
The original failed full log is retained under
`obj/parallel/future-const-join-v1/ci42075839657-failed.log`.
