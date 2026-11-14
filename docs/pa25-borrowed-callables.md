# Borrowed generic const callables

Clean original-core candidate35 rejects array/drain.rs:76/file100:3330 when
its const FnMut body invokes `(self.f)`, with `f: &'f mut F`. Semantic selection
already strips references and retains the final call-trait borrow. The defining
const validator formerly required zero dereferences, rejecting that valid path.

The validator now authenticates a nonempty retained adjustment against the
actual source Call and its typed callee. It walks exactly the recorded reference
count, charges every attempted step, requires mutable references for FnMut, and
compares the final subject with the retained receiver borrow. Existing exact
kind/capability, source owner/effect/signature/result, complete inherited/own
formal binding, actual operand and opened signature checks remain. Typed place
and loan checking still rejects inaccessible fields and moves from borrowed
storage. Concrete execution still needs the real const implementation and Code.
No new dense field, persistent cache, wire, backend, storage or quota changes.

[PA25 ordinary minima](../pa25/tests/spec/borrowed-callables/manifest.json) and
[PA31 source roles](../pa31/tests/spec/borrowed-callables/manifest.json) each
predict18 outcomes:13 accept/5 reject. Rust1.98 independently agrees with all18.
Seventeen source outcomes qualify:12 accept/5 reject, covering direct/field,
shared/mutable/nested, shared calls through mutable fields, actual borrowed
Relay FnOnce/FnMut implementation receivers, runtime execution and scaling.
Missing const effect, shared mutation, inaccessible field mutation, consuming
moves and argument mismatch stay rejected. The ordinary missing-const-bound
rejection also qualifies. Other17 ordinary attempts remain UNQUALIFIED at
const-trait declaration preparation or fn_traits bootstrap; their independent
predictions remain requirements. The accepting source CTFE consumer still
requires closed-signature payment and is retained as an accepting requirement.
The reductions preserve the original borrowed-field shape; they do not claim a
byte-exact copy or complete execution of the upstream Drain implementation.
Role declarations reuse the attributed explicit-call-members source prefix;
upstream originals/checksums and shared/CPPGM-NOTICE remain unchanged.

Evidence under obj/parallel/borrowed-callables/:

- `declaration.json` and `baseline/` freeze independent outcomes and reproduce
  the zero-dereference fence with immutable candidate35 (CLI5c416841…bba994a).
  The borrowed Relay addition was declared before its execution.
- fast2=1794699715593282460 passes four exact new/old semantic and scaling
  controls in109.58s. fast3=1794699862141919037 passes the final two new controls.
  Strict selected/production/CLI Clippy, full format and architecture audit pass.
  fast1 stopped at unformatted new code; it executed no compiler tests.
- native1794699922304522985 passes three groups/nine actual programs in44.60s.
  The new actual mutable-field reborrow prints41/59; eight old programs pass.
  Independent Rust native CTFE and runtime consumers both print41/59.
- `replay/` authenticates3965 compiler sources, all packet hashes and36 complete
  CLI attempts:18 qualify (17 source and the ordinary missing-bound rejection),
  while18 explicit unfinished attempts remain required. Its immutable dev-fast
  CLI is4391e4f03da251e1c4a2bfa015a8871191e81a17836fb4c4a2cd7da9fd27faa9.
  Exact lowering/scaling replay passes;8/32/128 inference/canonical work is
  1146/1909,4530/6447,18066/24575. No speed claim is made.
- 315 harness tests,496 integration roots/13 binaries,44 stages/one component,
  128 pinned upstream fixtures and126 Reference files pass.

One new private guard asserts actual one/two-reference facts and rejects altered
depth, missing source syntax, changed final borrow and work exhaustion. It awaits
full CI (expected3085 private); no cold local private harness was run.
Previous full43652873020/ead92a4cd is still active; no duplicate successor is
launched. A clean optimized36 build and unchanged original-core artifact retry
are next. Original v13/v14 artifacts/native consumers, every earlier positive,
whole PA25–44 and the separate113-case PA25 component retain their prior status.
