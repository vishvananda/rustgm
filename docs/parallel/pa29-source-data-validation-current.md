# Source data validator development handoff

Branch `pa29-source-client-data-validation-20261024`, HEAD
`39d2652fd9ad1cb41513cdec788ea2c6f6e8bfd8`, retains the qualified raw producer.
The dirty continuation below is UNCOMPILED and UNTESTED. No A guard is queued.
Root's public main fe0eeffe is the authoritative integrated B46+A39 reference;
validation's current macro branch is not that compiler identity. Do not rebase
this dirty tree or copy peer work while a guard is active.

## Production interface and scope

`support::crates::generics::source` re-exports `HeaderIndex`, `HeaderRef`,
`DomainIndex` and `DomainWork` from authored_domains.rs and source_validation/.

- `HeaderIndex::new(&SourceRows, base_bytes, &mut Work)` preflights combined
  raw Vec/name capacities, then constructs one borrowed owner/header ledger and
  the existing ScopeIndex. `header`, `scopes`, `retained_bytes`, `peak_bytes`
  expose data only. `domains(&mut Work)` constructs a separate borrowed domain
  completion view.
- `DomainIndex::{work,retained_bytes,peak_bytes,binder_contains}` expose counted
  completion observations and bounded ancestry queries. DomainWork records
  attempted successful-construction steps/root queries, binder frames/queries/
  maximum depth, memoized type nodes/hits and value/statement operation ports.
  A failure retains consumed caller Work but returns no partially built index.
- ScopeIndex adds charged `parent` and `type_argument_position` queries without
  new fields or flattened inherited formal vectors. Its prior 80-byte contract
  remains unmeasured for this draft; do not claim a passing size assertion.

The index checks complete genuine declaration/header/member/canonical-prefix
ownership; required-body absence versus supplied BodyRef/Original/BodyInputs;
actual header formals and supplied input/local binding identities; actual local
body/scope associations; original/public/quantified/call binder roles; complete
late-slot permutations and full unused type-formal arity; exact caller parameter,
body allocation and current binder domains on typed roots; Call value/site/
request/effect/opening/method identity; CFG/storage/owned-value/normalization and
raw input coordinates. Named parameters on required bodyless methods retain real
formal declarations without inventing body-local coordinates.

The binder relation is a single-parent forest: CSR child and bound-query buckets
plus one active path resolve every labelled depth query once. Preorder/exit
intervals answer later ancestry questions in O(1), with no per-query chain walk.
Type/application summaries are computed once across their recipe DAG, retaining
actual parameter-owner, free-body and current binder dependence. Every consuming
root checks that summary; closed roots can share structure. Callable target
recursion is not treated as a recipe cycle. Application rows require explicit
owning direct header/body requests in this first form; broader nested-only uses
reject. The raw rows remain borrowed; no duplicate BodyCatalog or original body.

All owned arrays/stacks are preflighted with the same caller base and existing
aggregate byte/component limits. Retained capacities and conservative transient
peak envelopes are separate. Work charges rejected queries, complete unused
formals, input/operation ports and allocation/growth traversals. The current
binder scratch envelope explicitly includes both CSR sets/cursors, active path,
two events per frame and late-slot coverage/stamps. Measurements are pending.

## Proof boundary remains closed

This is data/index completion, not `ValidatedRows`, a descriptor, an authenticated
frontend remap or an execution witness. There is no new constructor/conversion,
codec/profile tag, Interface publication, Package field or dispatch enablement.
Hot Body/Owner/Value/Contract records are unchanged by this continuation.

Full actual target-signature substitution/equality, operation typing and complete
operation-derived obligations remain mandatory before acceptance. Unsafe Calls,
early/captured regional applications/schemes, contextual application recipes and
wider external-artifact forms remain explicitly unsupported here. Type roots or
same-shaped headers cannot pay source Copy/Sized/qualified trait debts. Import
must freshly promote and prove originals, then instances, loans and cleanup.
B46 imported defaults still need authenticated original_defining_context,
Required reconstruction, the same defining Environment Arc through Copy
settlement, actual role remapping and fresh active-cleanup preparation/reproof.
Prepared carriers/claims will never be serialized. Declaration, body-only
conditional Self and contract-derived Copy input domains remain distinct.

The local source base includes B12c2 call-context work, not B46's full defining
cleanup closure. Root separately qualified actual unchanged Clone on integrated
B46+A39 and the carrier observer (3538 maintained +8382 text plus performance).
That retained root proof does not qualify this new support index. Whole-provider
contextual item/field/member provenance rejection and stability-bearing source
rejection remain required before descriptor publication; root owns those queries
and Package/library policy.

## Frozen first development gate — not submitted

Runner: obj/parallel/pa29-source-client-staging/source-domains-first.py.
Frozen manifest: source-domains-first-frozen.json in the same directory.
It authenticates current A9 pin and exact production/test/fixture/harness sources,
then format-check -> architecture audit -> all-target/all-feature strict Clippy
-> one debug batch with seven exact cases:

1. Existing unchanged complete-provider observer, enhanced with HeaderIndex and
   DomainIndex retained/peak/layout/work output through actual staged proof.
2. Missing headers/body inputs, forged canonical membership and combined budget.
3. Sibling width 8/32/128, exact O(1) ancestry query work and opposing intervals.
4. Independent depth/query 8/32/128 grid with separated finite differences,
   attempted work, retained bytes and scratch envelope.
5. Escaping depth/slot, cycle and same-artifact foreign binder owner rejection.
6. Foreign caller type formal, equal-arity wrong binder, foreign free allocation,
   wrong Call site, unsafe Call, duplicate evaluation, orphan application and
   wrong-kind late-slot opponents on the actual unchanged provider rows.
7. Attempted work exhaustion returns no domain index/descriptor.

No tests, build, rustc probes, native/text sweeps or resource jobs have run for
this continuation. Source-only cargo fmt and git diff --check completed. Earlier
formatter module-path failure and all A6/A7/A8 producer failures remain retained.
First gate must honestly report actual errors/layouts; no predeclared pass.
Existing A9 group/collector rollback and legacy zero-state cases are retained
unchanged, not described as freshly rerun. The new API has no publication path.

## Current storage and next admission

A9 current pin remains protected:
obj/parallel/pa29-source-client-staging/source-producer-ninth/pin/lib-tests,
SHA256 70eaa1eee10da37beb86a9faeadf08003266a7ab6a0a3b9db9e273b008eb9d91.
A6/A8 obsolete ELFs were explicitly retired after unpinning; their hashes,
receipts, sources and diagnostics remain. Scoped prune receipt:
obj/cleanup/1792813665584703315.jsonl (735219712 allocated bytes, two ELFs).
Do not run the historical A9 launcher: its old executable authentication paths
are intentionally retired. The new launcher requires only the current A9 pin.

The obsolete A9 Cargo cache was retired once with authorized
`storage.py prune --apply --build-cache`; receipt
obj/cleanup/1792814558216477569.jsonl, SHA256
4ca8745d27f3866e53b7bbf87af441b6fc430d046e794fcd630ac5928bc6ba74.
obj/rust is gone; the next build is cold. No repeated historical scan is needed.
No active A storage/resource scope remains. Root C5/C6 and Bv3/macro queue retain
admission priority. Do not launch the frozen runner until root explicitly admits
it under the unchanged resource/storage wrappers (2 threads,150 GiB reserve,
50 GiB quota), and do not modify source during that gate.

## First gate and rename-only recovery

source-domains-first exec53605 was actually admitted token
e75c8614be4f4c2d9746f28edd2703c4, slot0, and is terminal1/released.
Format passed (8.63s). Audit failed (35.60s) solely because four newly added
filenames reached existing repeated-stem count7: storage, input, member, call.
Strict/codegen/tests did not run: zero of seven cases executed, no layouts.
Resource receipt obj/parallel/resources/e75c8614be4f4c2d9746f28edd2703c4.json
records state=finished,returncode=1,error=null,used_bytes=7269924864.
Original runner abb72a62… and frozen749ae396… plus checks/output remain intact.
Diagnosed scoped cleanup terminal0, removed0 files, receipt
obj/cleanup/1792817391627752747.jsonl; controller was active[].

Only four new own filenames and corresponding path literals changed:
storage_ports→cfg_storage, input_ports→authored_inputs,
member_rows→membership_rows, call_ports→request_ports. Module aliases, test names,
function bodies and the seven-case plan are unchanged. Recovery record
obj/parallel/pa29-source-client-staging/source-domains-rename-recovery.json
(SHA256 8c982508147f6d19a00a3a89fa492cdf3222a2d975f883baf31a90fd2c0dabed)
verifies an exact inverse against all2217 original bound paths. Post-rename
cargo fmt completed0 and left every successor-bound source hash unchanged.

Successor remains UNQUEUED pending root admission after C/B rotation:
source-domains-second.py SHA256
b59767a17f4b8464c7f2848ccf3005d992fa09b6963a0472607b22109891c07f;
source-domains-second-frozen.json SHA256
478c8b945271e590ea4854adb28a6de02352bd6c9eccebb494de77d020a7c199.
Both are in obj/parallel/pa29-source-client-staging. It repeats fresh format,
audit, strict and all seven actual tests; the failed audit is not called a pass.
Authorized next command only after explicit admission:

```
CARGO_BUILD_JOBS=2 RAYON_NUM_THREADS=2 python3 scripts/storage.py run \
  --minimum-gib 150 \
  --prune-under obj/parallel/pa29-source-client-staging/source-domains-second \
  -- python3 obj/parallel/pa29-source-client-staging/source-domains-second.py
```

## Second gate and scoped Clippy recovery

A2 exec3177 admitted token27d44b2c497b48b1ae5d009f1320645a/slot0 and is
terminal101/released. Fresh format0 (8.80s), audit0 (2214files,0findings,35.53s),
strict101 (58.87s). Strict reported only ten Clippy findings: unused-self4,
collapsible-if, private struct-field naming, unit semicolon, test let-else2 and
corrupt test helper106/100 lines. No Rust type errors were reported; no codegen
or seven-case execution occurred and no layouts were observed. Scoped cleanup
terminal0 removed0 artifacts: obj/cleanup/1792818715923234731.jsonl. Controller
active[] at release, C8 assigned next; no automatic A retry.

Source-only correction makes stateless coordinate and summary checks associated
functions, including downstream region/scopes/temporary callers; exact caller
review preserved work charges and source/body/binder identity checks. The private
member coverage Vec field is renamed, equivalent nested-if/semicolon/let-else
styles applied, and actual foreign-formal construction extracted into one
cohesive test helper. No blanket allowance, new test, fixture or semantic policy.
Cargo fmt completed0; all changed callers were manually reviewed. The unchanged
source-client root integration policy was also reviewed read-only and reported
without an authority regression (root integrated450a5033 separately).

Recovery record source-domains-style-recovery.json SHA256
b981b3048b8a9c22554662901221b12a82314902ea3c5cd6d90be0b397d836be
binds eleven changed source files against A2's frozen map. Fresh audit/strict and
all seven tests remain required; the prior strict failure is retained.
Successor source-domains-third.py SHA256
9436e0e5dfb2b9f52879f4dbe5c63dccfd3544d82c20297fd54b2ce98be57c4a;
source-domains-third-frozen.json SHA256
57b08a6dda908cb15384d3a0b52f33eb6af5136970fc6396feb74f29c20454e6.
These files remain in obj/parallel/pa29-source-client-staging. A3 is frozen and
UNQUEUED until explicit root admission after C/root/B rotation; use the same
normal wrapper command above substituting source-domains-third in both paths.

## A3 actually passed; two-commit handoff requested

A3 exec17299/token38bb3eea668d42d79adfe145ebda6d28 terminal0: fresh format0,
audit0, all-target/all-feature strict0, actual7/7 debug tests0. One lib build
142.10s and selected execution0.0529s; no full/native/text claim. Default scoped
cleanup1792819791070154706 terminal0 removed0, controlleractive[] at release.
The actual complete provider reports HeaderIndex128B/retained21286/peak23307;
DomainIndex328B/retained24081/peak24403 with steps2615,roots64,frames21,queries13,
maxdepth2,typeNodes66,hits54,ports25. Dense Owner32/Kind20/Value24 hold. Exact
raw observations/counters, finite differences and failed opponents are recorded
in docs/pa29-source-domain-index-checkpoint.md and the immutable A3 evidence.

Root requests the exact green A3 slice be committed first, then a separate
source-only canonical late-index correction: two distinct SlotRefs must not repeat
canonical slot.index within the same actual function. Reuse the existing Vec
stamp array keyed by checked canonical index, add one ninth mutation to the same
selected domain-opponent test, and freeze for root combined qualification. No A
heavy recovery will run. Keep old A3 source/counters explicitly pre-correction.
No unpinned domain executable is needed for a comparison; A9 pin remains current.

Passing data slice committed251e829e1ae0b0f9ed155861b8a31e6e144af624, parentA39.
The separate source-only correction is ready: canonical index keys the same
checked stamp Vec; ninth actual clone_from mutation is inside the same seven-case
selection and requires the exact canonical-index rejection. No arrays/fields or
work scans added, no local tests/builds/probes after A3. Root combined gate owns
qualification; keep A3 evidence explicitly pre-change. No unpinned current domain
executable is needed for comparison, so Cargo cache can retire at root's requested
resource boundary while A9 pin and receipts remain protected.
