# Generic const Try protocol calls

Clean original-core36 clears the borrowed Drain callable and stops in
array::try_from_fn, array/mod.rs:153/file98:5582: `const member lost its typed
Call operation`. The implicit from_output call belongs to Operation::Try and
its completed ProtocolPlanId, rather than a direct source Call operation.

Defining const validation now reuses sema::protocol_plans::checked to authenticate
the current Block/Branch plan, actual source syntax and gate, registered methods,
full callable applications/signatures and owner/effect. The actual lowered member
must match exactly one retained call, with its exact signature result. Existing
safe opened member/header checks and current scoped effect checks still apply.
Concrete dispatch, implementation constness, ordinary argument/borrow/cleanup
proof and actual Code entry remain separate. No invented Call facts, new dense
field/cache, wire/backend/storage or quota changes occur. The source path does
bounded work over one or two calls; existing protocol checks retain their own
charged query accounting.

[PA25 ordinary minima](../pa25/tests/spec/const-try-calls/manifest.json) and
[PA31 source roles](../pa31/tests/spec/const-try-calls/manifest.json) each declare19
outcomes:12 accept/7 reject. Rust1.98 independently agrees with all19.18 complete
source outcomes qualify (11 accept/7 reject), including from_output, branch and
return conversion, nested blocks, projected results, runtime and scaling. Missing
const effects, output mismatch, disabled syntax, ordinary implementations and
unpaid residual destruction/conversion remain rejecting requirements. The ordinary
disabled-gate rejection also qualifies. Other18 ordinary attempts remain UNQUALIFIED
at unresolved core::marker/core::ops imports. Accepting source CTFE remains
required at primitive/unit-result payment; no accepting prediction is inverted.

Independent Rust exposed two legitimate obligations in the initially drafted
question-mark examples: const residual destruction under this reduced profile,
and const Residual<u8,TryType=R> when converting inside a try block. The accepting
examples pay those obligations; unpaid original shapes are retained as independently
rejecting controls. Those corrections preceded further candidate execution and
production edits. The self-owned source roles omit undefined standard-library
stability feature requests; they retain the actual try_blocks language gate.
The reduction follows the original try_from_fn protocol shape and does not claim
a byte-exact original function. Original revision/hash/license remain manifest data.

Evidence under obj/parallel/const-try-calls/:

- `declaration.json`, residual-{drop,conversion}-declaration.json and baseline3/
  preserve predictions, independent obligations and immutable36 reproduction.
  Earlier baselines retain the initial independent/setup failures and input copies.
- fast4=1794701680018963910 passes four exact new/previous controls across two
  targets in153.12s: new protocol legality/scaling, borrowed callable fields and
  projected const premises. Strict selected/production/CLI Clippy, complete format
  and architecture audit pass. Earlier fast1–3 stopped before compiler tests on
  duplicated native execution and filename ownership; the shared existing observed
  program harness now owns prepared-program execution. No audit rule was weakened.
- native1794701845691912429 passes four groups/ten actual programs in47.70s.
  New actual success/branch/residual paths print41/59/23; nine prior programs pass.
  Independent Rust executes CTFE and runtime counterparts with41/59/23.
- replay2/ authenticates3966 compiler sources, all packet hashes and38 complete
  CLI attempts:19 qualify,19 unfinished attempts remain requirements. Exact
  prepared lowering/scaling pass.8/32/128 inference/canonical work is2356/2422,
  6844/5472,24796/17648. Current immutable dev-fast CLI is
  ac7bf92e45f80f3fd1e73c7270d435512631edc0dcfc15ffe7ea61c11e5c4c35.
  Initial replay stopped because the ordinary disabled gate correctly rejected
  before its imports; the qualified rejection replaces its overly broad gap label.
- 315 harness tests,497 integration roots/13 binaries,44 stages/one component,
  128 upstream fixtures and126 Reference files pass. No timing claim is made.

One private guard checks real Block/Branch plans and rejects changed target,
result/site and work exhaustion. It awaits full CI (expected3086 private); no
cold local private harness was run. Prior full43652873020/ead92a4cd passed every
stage,3084 private release5.25s/debug49.86s. One successor43656293922 on8560c4dbb
(borrowed product05d1beedd) is running release/course, expected3085. Do not duplicate
or restart it. The Try checkpoint's full successor awaits its terminal result.

Next freeze clean37 and retry the unchanged original core with722 sources/six
reviewed configuration pins/120s/unchanged quotas, then authenticate postcheck.
There is still no original metadata/LowIR/native consumer artifact. WholePA25–44,
originalv13/v14, Future/async, all older positive requirements and the separate
released113-case PA25 component retain their prior status.

Clean release37 on published694738c4e verifies the original try_from_fn/Try-plan
fence is cleared.3966 hashes/empty patch, CLI81156451…aba3781d,8m27s build.
Original v14 next rejects slice/cmp.rs:19/file225:534, the unsafe SlicePartialEq
member call under a conditional PartialEq premise.21.58684038813226s/status1/
no timeout/RSS731712KiB; no metadata/0-byte LowIR. Postcheck passes722 originals/
375 opens/six reviewed pins/all3966/clean pin/fixed120s/unchanged limits. No speed
or artifact claim. Prior full43656293922 passes release/course and is in native
maintainer qualification; new Try full successor remains pending its terminal.
