# Inferred anonymous array outputs

The release18 baseline original-core diagnostic fails the first
Iter::next_chunk anonymous array initializer at NodeKey227:9558. All42310
declarations/25858 bodies and35120 item attempts retain zero failed functions.
Postcheck verifies722 original source hashes,375 opens, six pins, the clean
producer and unchanged120s/limits. Completion remains0; no artifact exists.

The [ordinary PA25](../pa25/tests/spec/anonymous-array-outputs/README.md) and
[source PA31](../pa31/tests/spec/anonymous-array-outputs/README.md) reductions
retain twelve accepting and four rejecting outcomes. All16 match independent
Rust. Ten of the twelve positives reject on release18; both explicitly typed
array controls accept. A redundant explicit impl lifetime bound does not repair
the inferred local, and discard/return/wrapped uses all reproduce the error.

The release-library diagnostic driver retains real inferred contracts: the
unannotated array gains an output lifetime slot, with a scalar coercion debt
and a type well-formedness debt on that slot. The existing `T: 'a` premise and
the exported `'a: 'b` debt together prove `T: 'b`. The conditional verifier
previously rejected the opaque type requirement while collecting scalar debts,
before its existing complete reproof could use those debts.

Collection now retains the scalar interface debts and leaves every other
obligation for the complete reproof under exactly those exported hypotheses.
The parent must still discharge every exported condition. A new graph regression
requires an actual scalar debt; it forbids inventing one solely to satisfy a
private type requirement. Static and local escape fixtures remain rejecting.
No interface type, proof privilege or compiler quota changes.

Receipt1794610960930869698 passes seven exact tests in158.08s: all16 source/
ordinary outcomes, previous implied-bound/batch/scaling and actual closure/capture
controls, plus strict selected/CLI Clippy, full formatting and architecture audit.
Receipt1794611404797641881 qualifies the new output-port scaling test in44.13s.
At8/32/128 owners, inference1784/7136/28544 =223n, canonical152/608/2432 =19n,
and real output ports exactlyn. The current source/harness-hash checked replay
passes all three existing anonymous native groups and those exact counters.
All19 exact packet/scaling inputs independently match Rust under panic=abort.
Fresh128 upstream and126 Reference checks pass; no backend contract changes.

fast6 and fast8 failed only formatting in the new private/scaling tests, before
compilation; their exact corrections are applied and raw gates remain. Full43530351089 later passed the new private graph unit among3074 private
tests, preserving the known local private-compilation budget.
Earlier whole-CI43517843369 passed
every stage on d1e15f2ad. The cache successor43523008382 runs exact5e9216115 and
does not qualify this newer graph change. Whole assignments and the two existing
borrowed-result native consumers remain open.


Clean release19 on65d222d27 clears this exact anonymous lifetime failure. Its
load-admitted unchanged120s original observer retains42310/25858/35120 and zero
failed/blocked functions, then reaches an IterMut live-loan conflict at227:111229.
Completion remains0; no original artifact exists. Postcheck verifies722 hashes,
375 opens, six pins, current/producer code and unchanged limits. All192 adjacent
immutable output comparisons15/16/17/18/19 pass; noisy calibration supports no
speed claim. Full43523008382 passed every stage on5e9216115. Full43530351089
on65d222d27 passed private/regular tests and stopped only at the reviewed PA24
lifetime-error location difference. See the [loop result continuation](pa16-loop-result-identities.md).
