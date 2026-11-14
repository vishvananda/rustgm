# Projected conditional const promises

Clean original-core candidate33 stopped at `residual_into_try_type`,
ops/try_trait.rs:383/file95:15668. Its `R: [const] Residual<O>` promises a
`TryType: [const] Try<Output = O, Residual = Self>`, whose supertrait supplies
FromResidual. The readonly const-call validator could not consume that projected
promise. Removing its projection filters alone exposed a genuine typing error:
an ordinary parent bound incorrectly paid a conditional associated result.

The existing result selector now retains an additional scoped child proof when
using a conditional associated promise at a nonordinary effect. A conditional
request requires a conditional parent; an always-const request requires an
always-const parent. Ordinary uses retain ordinary parent proof, and an explicitly
always-const result promise needs no stronger parent. The actual canonical parent
application, complete arguments, selected declaration clause and existential
intervals stay attached to the original proof stack. A lazy Result continuation
reuses the existing scoped clause cursor and partial-child handling; common
Frame/Node/Source layouts, persistent caches and phase interfaces do not grow.

Final generic-use validation rechecks the selected result subject/application at
the call's demanded effect, preserving quantified declaration provenance. It must
not replace an ordinary use with a declaration's stronger conditional qualifier.
The existing quantified caller-premise check is factored without changing its
contract. The readonly source request then admits projected safe Rust signatures
through its existing provider/report, source owner, current operation, complete
call/signature/result, selected method/UFCS and effect checks. Actual const impl,
borrow/lifetime proof, materialization and Code entry remain independently owed.
No wire, backend, storage policy, source-context permission or quota changes.

[PA25 ordinary cases](../pa25/tests/spec/projected-const-members/manifest.json)
and [PA31 source cases](../pa31/tests/spec/projected-const-members/manifest.json)
declare23 outcomes each:16 accept/7 reject. Pinned Rust1.98 agrees with every
prediction. Each packet qualifies22 complete outcomes,15 accept/7 reject,
including direct/always bounds, supertraits, nested projections, higher-ranked
lifetimes, ordinary runtime calls and opposing parent/result/argument effects.
An applicable blanket const impl cannot bypass a declared conditional promise
through an ordinary parent; an unbounded projected type can use that impl.
The initial new blanket-const acceptance prediction was corrected to rejection
from independent E0277 evidence before candidate execution. No previously valid
accepting requirement was inverted.

The source packet retains the byte-exact original helper, including its stability
attributes, lint expectation and body (SHA256 aa7409a687b4b7c60c8c603d43f43a5fb80124265215cb286a1b172354527866).
The owning original try_trait.rs and attribution remain bound separately.
Extracted support declarations needed staged_api, actual trait stability names
and const stability on their support traits/glue. Their setup corrections and
empty executable main wrappers are recorded before reexecution; helper bytes
and semantic predictions never change. The test uses the original command's
explicit lint cap rather than deleting its expectation.

Two accepting requirements remain unqualified: the source constructor's aggregate
CTFE return reaches `source const payment currently requires a primitive or unit
result`; the ordinary original helper reaches unresolved core::ops imports.
Both retain accepting manifest outcomes. The ordinary constructor and source
scalar receiver each execute CTFE/runtime observations `41 59`; the still-required
source constructor has the same independently executed Rust observation.

Raw evidence is under `obj/parallel/projected-const-members/`:

- `baseline/` binds immutable clean33 and the initial11 predictions; subsequent
  effect, scope, blanket and setup records preserve independent observations.
  Fast1 caught the plain-parent misacceptance before the semantic repair.
- fast7 (1794691669999805863) passes four exact controls in44.95s; fast8
  (1794691978469603016) passes the final source wrappers/scaling in41.90s.
  Strict production/selected/CLI Clippy, full format and architecture checks pass.
- native4 (1794692601948384080) passes seven actual programs in47.66s: the five
  prior default controls plus source projected reading and ordinary projected
  construction. The ordinary observer is owned and hashed in PA25.
- `reference/` qualifies23 independent predictions; `reference-native/` executes
  both new standard-Rust observers. `replay4/` authenticates3964 compiler
  sources and every packet/basis hash, all46 complete CLI attempts (44 qualified,
  two explicit accepting gaps), prepared lowering and scaling.
- At8/32/128 calls, post-lowering inference/canonical work is6758/8534,
 26966/32944,107798/130560 for source, and3880/2641,15496/10561,61960/42241
  for ordinary input. Bounded work qualifies; no timing or speed claim is made.
- 315 harness checks,495 integration roots/13 binaries,44 stages/one component,
 128 upstream fixtures and126 Reference files pass. The113-case released
  const-machine component is unchanged.

Previous full CI43636803381 on236732077 passed every stage, including3083 private
release tests in8.77s and debug in92.68s. Full successor43645030432 on published0c4d3d721 is running; clean release34
original-core verification follows. No cold private build is repeated locally. No original library metadata/LowIR/native consumer artifact
exists. Whole PA25–PA44 and all earlier valid unsupported contracts remain open.


Clean candidate34 verifies that the genuine original residual helper is cleared.
Source product0c4d3d721/docs-only producercc1bbf993,3964 hashes/empty patch,
CLI fdb81f6e5f53d924a40ca847678c4e6f0e25a47c054ae5ad08861c4574f64d1a,
7m02s build. Original v14 library34 stops later at array/drain.rs:62/file100:2808,
`self.call_mut(args)` in its const FnOnce implementation for &mut Drain.
22.45777091383934s/status1/no timeout/RSS727408KiB; no core.rgm/0-byte core.lowir.
Postcheck34 passes722 originals/375 opens/six reviewed pins/all3964/clean pin/
fixed120s and unchanged quotas. No original artifact or speed claim. Raw
`core-artifact34.log`, `artifact-postcheck34.log`, `original-library-candidate34/`.
Full43645030432 has passed static/all-target checks and is running release/course;
its complete successor evidence remains pending.
