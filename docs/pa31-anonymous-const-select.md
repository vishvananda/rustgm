# Anonymous constant selector proof checkpoint

This checkpoint qualifies the actual anonymous constant caller in the unchanged
`pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs`
(SHA-256 `ed3c2609e8941711e25eeb6d0949f418ce5f11de48758653d88b4dbfa42d57b0`).
Its anonymous array-length expression selects the source const callback through
the proved reference opening; its runtime expression selects the opposite
callback. Native output is exactly `7 1\n`, with empty stderr.

The prerequisite [early Copy readiness checkpoint](pa31-early-copy-readiness.md)
is separately qualified at `96fd35a4`. Its intermediate Copy proof did not
qualify this caller. This successor includes the previously unqualified scalar
source seed `ee0a1fa8` and owner-lifetime clarification `379006ad`, together with
the selected-reference extension. Neither seed alone established the original
reference acceptance. Preimplementation contracts are
`contract-anonymous-execution.json` and
`contract-anonymous-reference-execution.json` in the owning fixture directory.
This checkpoint does not activate an assignment or qualify full PA25/std.

## Actual owner, row and call

`Target::Expression` constructs `Owner::AnonymousConstant` and proves its exact
expression contract through ordinary regions and loans. Selector discovery
retains the source input row and publishes its finalized `Rc<Code>` with the
canonical expression index. No named initializer substitutes for this owner.

`Catalog::expression` checks the live index, exact source root, result, owner,
const context, missing specialization instance and ordinary publication mode.
It compares the current semantic root result through the existing inline result
or node adjustment/type lane. `Inputs::checked` authenticates both catalog/code
and row/code pointers, supplementing the complete contract key, source, instance,
signatures and result checks. Pointer equality supplies no independent semantic
permission.

A selected reference callback requires the existing whole-call opening to
validate the actual evaluated function item and full selector signature. The
private borrowed `ReferenceOpening` holds the actual checked row, actual frame,
exact call `ValueId` and validated opening signature. The anonymous caller
checks that same row/frame/call. The capability cannot survive another invocation
or supply another caller's permission. It adds no retained proof graph or field.

The anonymous caller still requires zero contract universals, no contract
parameters or callable signature, and a result needing no region context. The
original source consumes the reference in its actual closed `usize` Cast root
(node 303, dereference node 301, offset 2386). Defining FnOnce/const obligations,
conditional callback restrictions, ordinary member/default eligibility, full
selected instances, tuple ownership, evaluation order, runtime callback semantics
and drops remain mandatory.

## Warm entries, liveness and payment

Warm expression roots and completed requests authenticate their exact canonical
code before returning an index or cached value. Missing, stale, foreign or
contextual entries reject. A completed value supplies no fresh reached callback
permission.

Expression producers publish ordinary entries without a scoped lease. A lent
expression owns its finalized code and vectors rather than borrowing the lender
body; catalog body/expression indices remain append-only. Only paid anonymous
instances use scoped publication. Parent entry moves roots, sessions and
completed values together into the active context; retirement restores that same
outer state. This extension does not revive a retired scoped entry.

Reached inspection, preparation, payment, resume and consumption retain their
existing input owner and fresh checks. Preparation releases its immutable borrow
before preparing the callee and reacquires inputs afterward. Requirements and
caller environments remain checked on every payment and consumption. No new
payment, continuation, frame or machine field is retained.

## Qualification

Evidence is retained under
`obj/parallel/pa31/anonymous-reference-v1/functional-v6/`. The continuation rebuilt
only the invalidated library test executable after correcting a stale diagnostic
assertion. Its production/frontend bytes match the preceding build; the reused
frontend executable was authenticated before execution. Earlier failed evidence
is preserved.

Formatting, architecture audit, all-target/all-feature strict Clippy, harness,
upstream verification, library test build and layout checks passed. The fresh
private inventory ran once: 1,504 unique nonignored tests passed, including all
26 required controls. All 20 frontend cases and all 78 native contexts across
six test functions passed, including the original reference `7 1\n`, the scalar
seed, callback/runtime opposites, fresh payments and 13 early-readiness contexts.

The original object, LowIR, executable, C observer, exit statuses, stdout and
stderr were hashed before successful scoped output cleanup. Retained pinned-Rust
metadata, object, link and native observations authenticate the same original
source and observer. The scalar independent receipt separately binds its actual
source, tools, edition, target and profile.

The actual-call observer captures the original row, code, contract, frame,
selector, function item and opening, and rejects foreign/missing/wrong-call
opponents. Four canonical expression controls cover both scalar and original
Cast sources, including warm/completed entries and changed source/result/context/
instance/key/const mode. Source lifetime, const eligibility, fresh payment and
lease opponents remain selected. All 18 measured layouts are unchanged,
including Code 304, Row 96, Frame 248, Machine 4728, State 448, selector Call 56
and expression Contract 184 bytes.

## Remaining requirements

Valid generic early implementation prerequisites remain unsupported. Broader
open, generic, specialized and imported anonymous callers need their own exact
source/instance proofs; this closed-root acceptance does not qualify them.
Original upstream core/alloc/std builds and consumers, plus merged release,
active-text and performance qualification, remain separate requirements.
