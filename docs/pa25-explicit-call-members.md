# Explicit const call-trait members

Clean original-core candidate34 clears the residual helper and stops at
array/drain.rs:62/file100:2808: `self.call_mut(args)` in the const FnOnce impl
for &mut Drain, under F:[const]FnMut(T)->U. The source request accepted only the
safe Rust convention, while the actual validated call-trait methods retain
extern "rust-call" signatures.

The existing completed callable-role ledger now classifies the exact declared
method and rechecks its actual trait owner against the current callable registry.
The readonly signature validator admits safe rust-call for those actual Fn,
FnMut and FnOnce members. Ordinary Rust calls retain their prior boundary;
opened signatures still require zero unopened binders. Source/report ownership,
current full callable/signature/result, selected method or UFCS, caller effect,
constness and all ordinary dispatch/borrow/lifetime/Code proofs remain required.
The tuple ABI is preserved. No signature retagging, dense field, persistent cache,
wire, backend, storage policy or quota change occurs.

[PA25 ordinary counterparts](../pa25/tests/spec/explicit-call-members/manifest.json)
and [PA31 actual source cases](../pa31/tests/spec/explicit-call-members/manifest.json)
declare15 outcomes each:10 accept/5 reject, independently confirmed by Rust1.98.
Fourteen complete source outcomes qualify (9 accept/5 reject): explicit shared,
mutable and consuming calls, peer-impl delegation, original adapter, runtime
execution, scaling and missing/ordinary/immutable/wrong-tuple/unsafe opponents.
All15 ordinary attempts remain UNQUALIFIED at unsupported fn_traits or the
original counterpart's PhantomData import, including independently rejecting
cases. Their predictions remain unchanged. The accepting source CTFE consumer
remains required at the existing open/lifetime/argument const-payment boundary.

The original FnOnce implementation is preserved byte-exact, including attributes,
comment and body, SHA256434b0dad787cfb5b5fb10ad7f50394d54f3376fbea0289699fb87dd8633caef6.
The full upstream array/drain.rs hash, revision and MIT OR Apache-2.0 attribution
are separate manifest data. Its reduced Drain carrier uses a diverging support
FnMut body; that support body is never executed or claimed as full Drain behavior.
The missing support impl stability attributes were corrected to match the actual
upstream attributes before reexecution. The exact original item never changes.
The executable Relay counterpart runs its real FnMut body and actual function
item through a mutable receiver.

Raw evidence under obj/parallel/explicit-call-members/:

- `declaration.json`, `baseline/`, `original-corrected/` freeze predictions and
  reproduce the genuine signature/premise failure with immutable clean34.
- fast1 clears defining/scaling checks and exposes the separate CTFE payment
  boundary. The accepting CTFE fixture is retained; runtime-native is predicted
  before execution. fast2 (1794696879652162802) passes new semantics/scaling and
  the prior projected-const control. fast3 (1794697385218943481) passes the final
  two exact controls in67.28s, with strict selected/production/CLI Clippy, complete
  formatting and architecture audit.
- native1794696997540382381 passes eight actual programs in76.21s: new runtime
  delegation prints41/59 and seven old native controls still pass. Independent
  Rust executes both the runtime and still-required CTFE examples with41/59.
- `replay/` authenticates3965 compiler sources, all packet/basis hashes and all30
  complete CLI attempts:14 source outcomes qualify; one source CTFE and15 ordinary
  attempts remain explicitly unqualified. Exact prepared lowering and scaling
  replay pass. Work at8/32/128 calls is10749/2610,37965/7916,146829/29116 for
  inference/canonical steps. No timing/speed claim is made.
- 315 harness tests,496 integration roots/13 binaries,44 stages/one component,
 128 pinned upstream fixtures and126 Reference files pass. The113-case released
  const-machine component and planned whole assignments remain unchanged.

A new private guard checks all three exact role members, rejects FnOnce::Output
as a call member, and rejects swapped role slots. It awaits full CI (expected
private inventory3084); no cold private harness is repeated locally.

The preceding accounting repair7a4e9547f has passed full43649121503 release/course
and is running debug invariants at last observation. Its final stages and
this new checkpoint's full successor remain pending. After publication, freeze
one clean candidate35 release and retry genuine original core with unchanged
722-source manifest, configuration pins,120s and quotas. No original metadata,
LowIR or native consumer artifact exists. All earlier valid unsupported contracts,
full Drain behavior and original v13/v14 artifacts remain required.


Clean release35 verifies the genuine original adapter is cleared. Product/head
ead92a4cd,3965 hashes/empty patch, CLI5c416841…bba994a,7m14s build. Original v14
library35 now rejects the following array/drain FnMut body at file100:3330/line76,
`const invocation differs from its defining call-trait or effect fact`, involving
the borrowed callable field.26.301281564868987s/status1/no timeout/RSS725944KiB;
no core.rgm/0-byte core.lowir. Postcheck35 passes722 originals/375 opens/six
reviewed pins/all3965/clean pin/fixed120s and unchanged quotas. No artifact or
speed claim. Full43649121503/7a4 is entirely green3083 release6.82s/debug64.37s;
one successor43652873020/ead92 is running. Do not restart for documentation.
