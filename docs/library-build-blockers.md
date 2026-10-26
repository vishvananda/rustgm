# Library build blocker tracker

Updated 2026-10-26. This is the work queue for compiling the original pinned
`core`, `alloc` and `std`, not a count of completed internal checkpoints.
PA1–PA24 remain implemented drafts; PA25–PA44 remain planned. No additional
assignment or whole upstream library has been completed.

Use the [current execution plan](library-bootstrap-acceleration.md) to choose work:
real artifact/source consumers, demanded imported-body proof and consolidated
qualification. The existing complete assignment and upstream gates remain required.

The source pin is Rust `88d9e12ae178fab0fb5cc050a94da85685d449ea`, edition 2024,
x86-64 Linux, panic abort. Use the existing immutable shared source tree and
record target configuration, compiler identity and opened-file hashes on each
whole-crate attempt. Never patch upstream source to move a blocker.

## Observable crate milestones

| Milestone | Current result | Next acceptance evidence |
| --- | --- | --- |
| Original core loading and expansion | Qualified CLI `af93787c` passes Thin syntax and reaches `ptr/metadata.rs:170`, byte6689, at foreign `type VTable;`. All722 pinned inputs verify;65 original files opened; no artifact. | Implement LIB-032 foreign opaque types while closing LIB-031 alias and Pointee consumers, imported bodies and actual assertion/formatting. |
| Original core semantic checking and body export | Not reached end to end. | Check every required declaration/body and produce the real crate artifact. |
| Original core object and consumer | No successful build. | Link and execute an independent consumer using its actual generic implementations. |
| Original alloc and consumer | Not reached; depends on core and cross-crate transport. | Build unchanged alloc and execute allocation, reallocation and destruction. |
| Original std and consumer | Not reached; depends on the preceding crate graph and platform contracts. | Build unchanged std and execute a default-std consumer. |

## Current execution queue

The [execution plan](library-bootstrap-acceleration.md#scheduling-decisions)
supersedes historical immediate tasks in checkpoint records. Keep these exits
concrete; a private Ready state is not an executable result.

| Owner | Issues and owning assignments | Next observable exit |
| --- | --- | --- |
| A | LIB-007/030, PA29/31 | Complete encoded provider to fresh named Holder consumer, original/instance proof and native output 7. Names/receiver shapes are merged locally; the public Provided codec and body joins remain. |
| B | LIB-029/021, PA25–28/31 | Actual NonNull source, then Location/PanicInfo/handler and false assertion reaching Abort; follow with dynamic formatting. |
| C | LIB-032, PA22/30/31 | Real Foreign VTable identity and metadata, then an unchanged-core attempt past the current declaration. Share Pattern/Pointee representation with B. |
| Integrator | LIB-031/008, PA19–21/31/32 | Complete actual Thin declaration/use with Pointee, integrate qualified consumers, and advance original core through checking/export. |
| Integrator, queued independent preparation | LIB-009/010/011, PA30/33/34/36 | Selected alloc/std dependency/build/runtime inventory and small real ABI/build consumers; do not wait until core works to discover graph prerequisites. |

Track assignment closure separately: implemented behavior, owning text fixtures,
independent/native evidence and publication/release checks. Broad remaining
features stay required but only enter this immediate queue when a real library
consumer needs them or a PA is otherwise ready for release. No whole core build
or additional PA release is implied by these targets.

The next scheduling boundary is the Foreign handoff and a fresh original-core
observation. Integrate that independently useful change without waiting for
all alias/panic work. If the next blocker is already assigned, C can take A's
separate Provided namespace/advisory/defaultness codec while A finishes genuine
member/body execution. A defines the wire rows first in the existing
[Provided milestone](parallel/provided-method-execution.md); no new worker or
parallel edit of A's semantic tables is needed. B stays on the actual
PhantomData/Location/PanicInfo path. The integrator finishes its current Thin
join and prioritizes queued integrations over another implementation workstream.

The latest full CI run
[40382875468](https://github.com/vishvananda/rustgm/actions/runs/40382875468)
failed one old Pointee diagnostic assertion; 1,787 ordinary private tests and
all thirteen ordinary integration binaries passed. C's 27b1712dd Foreign
successor qualifies the exact correction and its remaining ordinary-mode group
in 112.86 seconds; root integration/CLI/core observation are pending.
Native/debug/independent checks were skipped, so they
remain pending. The current push check still selects lexer tests; the shared
source/const/import/LowIR smoke is an integrator task, not an implemented gate.

## Assignment closure queue

All rows below remain open. This queue identifies the next reconciliation, not
an exhaustive fresh audit of every historical unsupported case. PA25–PA31 have
no published text-test manifest. Preserve complete handout requirements and
independent outcomes; missing release packaging and missing semantics are
different debts. Reconcile existing fixtures at consumer exits instead of
creating another parallel test corpus or repeatedly running every PA.

| Owning PA | Library-critical work first | Closure review at the next consumer exit |
| --- | --- | --- |
| PA25 | Source const operations and imported const/generic calls required by the actual core path | Reconcile constants-view, provenance/promotion and sequence outcomes; identify remaining valid unsupported inputs, then review the text vocabulary and references. |
| PA26 | Source-owned operators/desugaring and actual panic/format constructors | Reconcile iteration, question-mark, derive and formatting contracts against source-library execution. Do not count a bootstrap factory as compiling its upstream implementation. |
| PA27 | Actual core pointer/storage validity and layout consumers | Reconcile existing raw/reference/union/byte-storage outcomes and live unsupported backing families. Keep runtime unsafe permission distinct from CTFE validity. |
| PA28 | Source-used memory/numeric/atomic intrinsics and metadata operations | Match active source declarations to implemented operations and backend evidence; retain other required intrinsic/atomic families in the release inventory. |
| PA29 | Encoded provided/generic bodies, fresh consumers and downstream republication | Audit generic/lifetime/trait/default/const transport and deterministic public metadata, then generate the complete text/export reference set. The first native Holder call alone does not close this PA. |
| PA30 | Foreign VTable now; genuine allocator/platform ABI and linkage consumers next | Reconcile remaining representations, callable/static/linkage contracts and native interoperability evidence with the handout. |
| PA31/32 | Actual bindings, then unchanged whole core through checking/export/object/consumer | Reconcile active source features and bindings against the complete configured crate. Reducer/header successes remain prerequisites, not whole-core acceptance. |

The integrator owns publication reconciliation; each worker updates the existing
owning handout and declared outcomes as its consumer passes. Reuse independent
evidence whose inputs remain identical. Full cumulative release qualification
starts once semantic and reference debts are closed, not at every helper commit.

## Historical checkpoint evidence

The [private environment/namespace integration](private-environment-namespace-integration.md)
passes eleven focused controls plus production/private Clippy and global audits
in a 117.86-second helper, alongside a separate 30.44-second production CLI
build. It closes two transport prerequisites while leaving the current original
core frontier and public Provided/restricted-policy boundaries explicit.

The [empty Contract successor](private-empty-contracts-checkpoint.md) now passes
six focused groups, exact first-encoding old v0/v1 bytes and fresh plain-function
consumers. Its 132.99-second focused helper plus 34.79-second production CLI build
retain the five-minute development budget. Public Provided execution and full
original-crate builds remain required.

The [numbered trait-input checkpoint](private-numbered-trait-inputs.md) now passes
six focused controls, the complete architecture audit and strict production and
private Clippy in a133.06-second helper. Actual Holder::carry prepares inherited
Self/V/U; Holder::borrow retains its nonzero boundary. Selected numbering records
authenticate actual body ownership without granting defining-environment proof.
Seven original v0/v1 first-encoding files remain byte-identical. Nonzero binding,
original/instance proof and public Provided execution remain required.

The [neutral numbered-environment successor](private-neutral-numbered-environments.md)
qualifies actual nonzero Declaration signatures with empty regional inputs and
positively absent builtin/region payloads. Six controls and strict static checks
pass in a153.01-second helper. Warm work is constant across1/8/32 actual
universals. Genuine Holder::borrow has no alias ledger; the separate real-ledger
cost utility does not qualify a present-ledger body. Actual body/availability
ports, regional proof, substituted instances and public execution remain open.

The [provided-method execution milestone](parallel/provided-method-execution.md)
keeps the next acceptance target concrete: the complete source family plus a
real non-role provided method, actual encoded artifact, fresh imported original
and instance proof, and a linked consumer printing `7\n`. Private producer and
Environment checkpoints are prerequisites to that target, not its replacement.

The [fresh consumer authority checkpoint](private-consumer-authority-checkpoint.md)
now constructs a distinct consumer Report, Store and one shared imported Catalog
from actual source/configuration/context. Six focused groups pass, with all seven
old v0/v1 raw captures unchanged. The owner remains reserved: selected header
append, complete role/member validation, original/instance proofs and the native
consumer remain required. Unpaid header columns cannot certify absent attributes
or defaultness, and public Provided admission remains closed.

The [Assert condition checkpoint](source-assert-implementation.md) qualifies
single parsing and owned typed transfer, including real opaque-fragment
provenance, failed-attempt accounting and an existing configuration sentinel.
Its three focused controls pass; Assert execution stays closed while the exact
expression writer and source Not/panic/abort/format/constant paths are built.
Integration retains the component evidence after checking its ten compiler/test
files are disjoint from the fifteen neutral-environment files. This is not a
fresh combined execution or completion of LIB-029.

The [integrated writer/verifier checkpoint](frontend-components-integration.md)
now freshly passes six cross-component groups and all strict static checks in
108.91 seconds. It adds the real private expression writer and specialization
forest/Min/finality verifier; their execution fences remain closed. Alias clause
expansion, imported body proof and actual source panic/runtime paths still block
the original libraries.

The [canonical alias/selected-consumer integration](trait-alias-clauses-checkpoint.md)
now passes14 focused groups and both strict static modes in174.59seconds. It
adds actual local alias schemas/clauses and shared application expansion, selected
callable registration in the genuine fresh consumer, and validated source
panic/abort declaration claims. Complete alias consumers, imported body proof,
source Not/panic execution and the original library artifacts remain open.

The [source-contract integration](library-contracts-integration-checkpoint.md)
passes 33 private and two frontend controls in a 103.33-second helper. It combines
nested alias binders, actual collected-header admission/shared imported-input
kernels, source Not, source layout/nominal Clone bodies, local specialization
method dispatch and the bounded source-edition option. Strict static checks are
retained from successful steps of the unchanged-source predecessor; its failed
selector adapter is preserved. Actual Not and storage provider source/LowIR match
their native-qualified bytes exactly. Full alias consumers, imported provided
bodies and actual panic/abort/format execution remain required; no new PA or
original library artifact is complete.

The [alias naming/source-sizing successor](trait-alias-naming-checkpoint.md)
passes20 exact controls in0.05seconds with a133.09-second helper. Shorthand names
now come from real leaf applications through a declaration-Self expansion view;
full proof clauses retain other-formal requirements. Eight pinned Rust naming
observations preserve the original wrong prediction and all raw diagnostics.
Source Self-sizing uses actual direct-RHS role identities. Header-time aliases,
original/normalized formation, region/object/
default/const and imported consumers still block full alias execution/export.

The [runtime/header integration](runtime-header-integration-checkpoint.md) passes
32 exact controls and both strict static modes in a150.13-second helper. It
integrates genuine source Abort, one shared imported-signature mapping and the
sealed Pointee declaration producer. Alias family inputs now retain every
expanded clause, while associated-result aliases remain separate promises;
actual projection use rejects missing Copy. Three pinned Rust observations
agree and are retained without repeating execution. All three native-qualified
Abort/Not/storage source and LowIR outputs match exactly. Metadata normalization,
full aliases, imported provided-body proof and panic/formatting remain required.

The [scheduled-regression successor](runtime-header-integration-checkpoint.md#scheduled-regression-successor)
now passes14 exact controls, strict static checks with explicit retained provenance,
all215 harness checks and upstream verification. Ordinary bootstrap diagnostic
ownership and failed closed-projection provenance are repaired; imported Unknown
and actual execution proof remain separate. The subsequent full run40363637715
passes all13 integration binaries; its sole private failure is a stale Catalog
size assertion, corrected by the formatter integration below. Native/debug/independent
steps did not run after that failure.

The [formatter/transmute integration](formatter-pointer-erasure-checkpoint.md#combined-integration-and-core-frontier)
qualifies actual const pointer formation and runtime calls, all26 selected controls
and217 harness checks. A fresh unchanged-core observation passes Thin syntax and
reaches the foreign `type VTable;` declaration at ptr/metadata.rs170 (LIB-032).
All722 pinned inputs verify,65 files are opened and no artifact is produced.

The preceding published probe uses immutable compiler `9793c30f2d29a99196687032f88960c317c28c47a3f4f6973ba681bb09698ac4`
from integration `1df52349` (production `91113764`). It exits 1 with
`ERROR: file 3: 50: source builtin macro expansion is not implemented`.
The declaration is `core_panic`; no invocation has yet been expanded. The
four opened original files are core's root, `prelude/mod.rs`, `prelude/v1.rs`
and `macros/mod.rs`. All match the pinned source inventory. The retained
`rustgm-validation-work/obj/parallel/whole-core-blockers-v3/` record binds the
compiler, target/edition configuration, source inventory and actual open trace.
Its result SHA-256 is `b2f6bb683c0c6800b0dc2e89fa46d616bbdc925e66a444829c02cb7f7b2529dc`.

The preceding probe's `expected path segment` error at byte 6527 is retained in
`library-stability-whole-core-frontier-v2/`; it no longer describes the fresh
frontier. Parsing that file now succeeds, but neither probe completes core.
The preceding integration passed 3,605 release/native tests, 8,382 active text
tests and all 60 functional qualification stages. The newer
[lookup/input/panic checkpoint](library-lookup-panic-checkpoint.md) passes
3,644 release/native tests (1,721 fresh plus 1,923 authenticated retained),
8,382 text cases and 65 functional stages. Neither count releases a PA.

An earlier published unchanged-core observation used the qualified integrated compiler
`20eb54f33961841b6e496bec52dae3a41f51e10ee97dbdb4b03de68d8e728134`,
source `6bba6589`, in the validation worktree's
`obj/parallel/whole-core-blockers-v5/`. It exits 1 at file 3, byte 1250,
with `internal macro allowance feature is not implemented`. This is the
`panic_internals` allowance on `assert_eq`, line 41. The same four original files
were opened; all hashes and configuration remain unchanged. Its result SHA-256
is `b333b6b448ed489df47726a86aac71040c8025d3ad32e664b2ce0dcd9a8c940a`.
The preceding worker-only result at `2c31704b` is retained separately.
Passing the prior declaration advances this specific crate boundary; it does not
complete expansion, semantic checking, an artifact, or an assignment.

The allowance worker subsequently qualified source `222b637a` (production
`45c9a757`) with 1,393 nonignored private tests, 43 frontend contexts and 34
native programs. Its immutable CLI
`2a8bf052b0d0d77d5d19ba740c7d8e1c105544f3a64efe45d741e50a6f48554f`
advances the same unchanged core root to line 170, byte 6523:
`scoped declarative macro hygiene is not implemented` on `pub macro assert_matches`.
The four opened files and configuration are unchanged. Evidence is in the
worker's `obj/parallel/pa31/complete-allowances-whole-core-v1/`, result SHA-256
`fb12fac6b2860c8572417c4e3df13919e4d54ac7ef335ee8d7441571a137c502`.
The combined candidate now passes functional qualification and confirms this
frontier independently; one projection-opening growth failure remains under repair. The [allowance handoff](pa31-internal-allowances-handoff.md)
records source/native evidence, retained failures, measured costs and the
foreign API-use boundary.

## Current implementation and qualification

The [combined library frontier](library-frontier-checkpoint.md) qualifies actual
CLI attribute-contract consumers, anonymous Function recipes and local trait
implementation domains. Its final integration check passes all 8485 text cases,
sixteen native outputs, three independent lint rejections, the fresh global
architecture audit, formatting, metadata, harness and upstream checks in 126.82
seconds without a local compiler build. Selected strict-Clippy/private checks
remain authenticated from their focused gates. The subsequent
[unnamed-constant checkpoint](unnamed-free-constants.md) passes six focused
controls and all 8,493 text cases, with eight reviewed scoped references and
215 fresh harness tests. Its unchanged-core observer advances to LIB-028 with
58 original files opened; no original crate artifact exists.

The [source-artifact/readiness checkpoint](source-artifact-readiness-checkpoint.md)
combines A15 imported-body/full-instance proof, LIB-004 early implementation
readiness and LIB-005 actual anonymous-reference callers at `fa0f4e08`.
All fifteen actual Cargo inventories execute once, including ignored native
tests: 3,852 passes. All 8,385 active text cases, static/strict/harness checks,
affected independent source/native suites, transitive metadata and adapters pass.
The original selected-reference program now produces `7 1`; both API and CLI
artifact-to-fresh-consumer native programs produce `7`. The 130-input preflight,
37 focused measurements and eighteen growth controls complete; twenty RSS costs
retain bounded attribution and the separate A14→A15 timing costs remain visible.
There is no new assignment or original crate artifact.

The bounded development workflow is public at `1ac88c11`: its local warm
format/audit/strict-Clippy/selected-test/text-oracle check passed in 57.35 seconds;
the new push CI job passed in 2 minutes 22 seconds. Full release/native and
independent qualification runs in nightly/manual CI. The defining-environment
producer and raw original numbering are public at `f8cff8ba`, with 11 exact
merged controls passing in 54.15 seconds. These smaller checkpoints do not
replace assignment-release acceptance or claim a new core artifact.

Periodic [full qualification of `66bd3750`](https://github.com/vishvananda/rustgm/actions/runs/40170960696)
passed in 20 minutes 49 seconds in hosted CI: 3,538 regular release tests,
343 native tests, all 8,385 active text cases, 1,581 separate debug invariants,
strict lint/audits, harness, host-build and independent Rust checks. This is the
public UB-policy integration before the newer Provided codec/ordinal changes;
the latter have their separate fourteen-control focused gate. Full qualification
does not block each development edit or replace the five-minute iteration budget.

The preceding [scoped/byte checkpoint](scoped-bytes-checkpoint.md) is the
performance baseline: 125 inputs, 116 equal-output pairs, 100 native
preflights and 68 growth controls. Seven measured RSS costs remain recorded.
Its unchanged-core observer reaches `autodiff_forward` at `macros/mod.rs:1553`,
four files unchanged. Stricter attribute checking separately exposes the earlier
`rustc_coherence_is_core` requirement at `lib.rs:65`; implement its real semantics.

The [new original-core observation](original-core-lib020-checkpoint.md) uses
the qualified primitive compiler `50dac790` and reaches `offload_kernel`,
line 1668, byte 62644. All 722 selected source files and the four actually
opened files match the original pinned inventory. Download, verification and
the traced attempt take 5.729 seconds. This advances the declaration boundary
without producing a crate artifact. The next related family is LIB-022.

Periodic full CI on `e081f8ab` found one regular integration regression:
the stricter classifier rejects the known `non_exhaustive` attribute before
its existing unsupported artifact-profile boundary. The
[LIB-023 repair](non-exhaustive-checkpoint.md) now passes that exact regression
and three further integration controls in a 129.31-second focused gate.
Its nineteen sources have 38 actual independent observations. The genuine local
consumer preserves the closed export contract; imported openness remains required.

The next periodic run on `62757320` passes that repaired record test and all
1,634 private release controls, then finds an older ordinary-macro assertion
that still assumes every internal allowance requires provider mode. The revised
test checks both ordinary and provider declarations, explicitly proves ordinary
source retains no provider/client roles, and preserves the real unsupported
scoped-stability boundary. Its two exact integration controls pass with strict
Clippy/audit/build in 208.57 seconds locally. The remaining full CI stages are
still pending; a failed periodic run is not recorded as full qualification.

Periodic [full CI on `ebc918c9`](https://github.com/vishvananda/rustgm/actions/runs/40194216972)
finds two stale assertions that unused known assembly declarations must fail.
Its regular lane records 1,646 private and 343 frontend passes, with one failure
in each target; later native/text/debug stages did not run. The corrected tests
accept genuine unused declarations in both pipelines and explicitly reject
their invocation, including placeholder-body execution. Three exact controls
pass metadata, formatting, architecture, strict Clippy and execution locally in
223.57 seconds, receipt `obj/qualification/fast/1792933285445625938`.
This is a test-only repair; compiler behavior and references are unchanged.
The failed periodic receipt remains evidence, with broader qualification pending.

The [complete builtin declaration checkpoint](builtin-declarations-checkpoint.md)
passes seven isolated controls in 165.33 seconds. Its exact CLI advances the
unchanged root to `num/f128.rs:27`, with nine opened original files and no input
changes. The 6.731-second observer verifies all 722 source files. LIB-024 owns
the newly observed valid suffix failure; metadata/object Rust observations alone
do not establish exact literal bits or native wide-float support.

The [wide-literal successor](wide-literals-checkpoint.md) passes eight merged
controls in 170.27 seconds and advances the unchanged root from nine to 42
opened files. Its next failure is actual `impl_restriction` syntax at
`num/nonzero.rs:34` (LIB-026). All 4902 LowIR outputs and every active text
case's exit status remain exact in a 51.79-second read-only effect run. Checked
reference generation and a complete 8478-case text pass subsequently qualify
the reviewed effects in 73.99 seconds, including metadata/harness/upstream checks.
Six new public PA2/PA9 regressions and pinned-rustc observations pass in a
19.31-second follow-up. The validated text inventory is now 8484; 8139 unchanged
cases are retained from the preceding pass. No assignment or core artifact is
complete.

Periodic [full CI on `fb6cc056`](https://github.com/vishvananda/rustgm/actions/runs/40204216976)
finds five stale expectations for the new cold feature-word width and later
source TypeIds. A test-only repair passes the three exact private controls and
two relationship controls. Static checks pass in the initial bounded cold run;
a source-authenticated continuation completes the missing build and execution
without repeating them. The [wide checkpoint](wide-literals-checkpoint.md)
records both receipts and the retained timeout. This does not turn the failed
periodic run into full qualification; its later stages remain pending.

The combined anonymous-function gate passes twelve of thirteen exact private
controls and exposes a local crate-root compatibility regression in the new
header adapters. Its [five-control repair](source-header-contract-transport.md)
passes in 131.15 seconds, preserving genuine imported Unknown and missing-producer
errors. The anonymous-function source itself and its three owning PA18 controls
pass. The [argument-index checkpoint](parallel/pa29-function-argument-indices.md)
now qualifies all 8485 paired text cases and the 122 reviewed stdout changes.
Checked scoped generation, metadata, harness and upstream checks take 18.62
seconds; no old status, diagnostic or LowIR output changes. Fresh callback-bearing
executable consumers and broader Provided import remain required.

- Integrator: the [structural owner checkpoint](primitive-inherent-checkpoint.md)
  qualifies local primitive policy and full-Self lookup with 66 source cases,
  nineteen focused private controls and eight actual native programs. Ordinary
  local allowance headers retain prelude behavior without provider roles. Next
  priorities are a fresh unchanged-core attempt and imported/default transport;
  original crate artifacts remain absent.
- A: the shared defining-environment producer and raw numbering data are
  integrated at `f8cff8ba`. The unpaid Provided codec and charged inverse-
  allocation helper are integrated at source `03190e3c`; fourteen merged private
  controls, formatting, architecture audit and private strict Clippy pass in
  157.01 seconds in CI. All 198 harness tests also pass. All canonical Provided
  import guards remain closed. The distinct nonpublishing reservation/typed-input
  session at source `3229ee3e` also passes nine focused controls with formatting,
  audit and strict Clippy in 166.66 seconds. It owns actual reservations and a
  private catalog without interning or semantic publication. Named lifetimes,
  demanded recipes and fresh body/instance proof remain required.
  The [shared signature cursor](parallel/pa29-signature-cursor.md), merged at
  `584cf944`, now passes eleven focused controls in 163.43 seconds including
  strict Clippy/audit/build. Its actual flat-reference producer exposed and
  repaired missing raw CFG borrow obligations by sharing the ordinary checker
  kernel. Deep ready signatures, genuine empty schemes, named environments and
  executable Provided import remain unimplemented; the public guards stay closed.
- B: the bounded LIB-021 component at `ca2f0a80` passes 1,519 private tests,
  24 frontend cases, 84 native contexts, 26 layouts and seven CLI controls.
  Its merge preserving current macro permissions and A's source data is public
  at `66bd3750`, including focused integration fixes. Sixteen private controls
  and four frontend functions plus PA7/PA8 text suites pass; periodic merged
  native checks run separately in CI. Imported runtime-policy
  transport and the combined Test-body/configuration demand seam remain open.
- C: the [attribute/receipt checkpoint](attribute-consumers-checkpoint.md) now
  has ten private controls (115.70 seconds), nine frontend controls (62.40
  seconds), 93 independently checked new text oracles and fifteen actual native
  comparisons. All 212 integration harness tests pass. Configuration no-demand
  receipts and original owner-flag restoration are implemented; procedural
  execution, test-harness generation and imported contracts remain required.
  The early metadata gate catches undeclared modes and missing native outcomes
  before compiler builds.

Use three workers under the unchanged three-job/150 GiB reserve policy. Group
related changes, reuse authenticated unaffected evidence and clean known
successful output roots. Broad measurements remain exclusive.

## Prioritized issues and assignment ownership

"Owner PA" identifies the compiler/course responsibility. "Test PA" is the
earliest assignment owning **all** features in the complete reproduction. A
source-library reproduction using `no_core`, lang items or compiler-private
attributes belongs in PA31 even when its underlying algorithm has an earlier
owner. Add an earlier-PA regression too when the failure can be reproduced
without those later features. Do not move fixtures mechanically by subsystem.

| ID | Priority and evidence | Blocking issue | Owner PA / test PA | Worker | State / dependencies |
| --- | --- | --- | --- | --- | --- |
| LIB-001 | P0; preceding whole-core parse blocker removed, scoped semantics still required | Scoped declarative macro publication, item/path/import/glob lookup and definition-site visibility | PA31 internal feature; PA8 shared machinery / PA31, plus applicable PA7–PA8 regressions | C + integrator | Local scoped invocation, actual owner/scope authentication and physical privacy pass combined ca54609a functional/performance qualification. Foreign macro transport remains required; the full issue is not closed. |
| LIB-002 | P0; prerequisite of LIB-001 | Hygienic field, method and associated-member identity; captured names and container-relative privacy | PA31; PA12/PA19 shared consumers / PA31, with earlier ordinary controls | C + integrator | Contextual field/member identity, canonical conformance and physical type privacy pass combined ca54609a qualification. PA12 ordinary controls and PA31 source contracts are independently checked. Foreign transported member identities remain required. |
| LIB-003 | P1; actual reduced source failure | Projected Copy proof sees one lifetime premise in inconsistent declaration/body namespaces | PA21 scoped proof + PA31 source const-trait defaults / PA31; add PA21 control where expressible | B | Declaration/body alias scope and fresh Copy reproof pass combined functional qualification. The related LIB-016 projection-growth failure is repaired and all combined growth controls pass; do not conflate this with LIB-004 early readiness. |
| LIB-004 | P1; actual reduced source failure, readiness cause observed directly | Early const read treats source `u8: Copy` as absent although its impl is present | PA20 implementation readiness + PA25 early CTFE / PA31 source-binding reproduction | B, coordinated with C | Bounded scalar readiness fix is integrated and functionally qualified at `fa0f4e08`, including declaration order, invalid unused body, cycles and repeated-use controls. The original selected-reference reproduction also passes through LIB-005. Generic prerequisite and imported early-readiness profiles remain required. |
| LIB-005 | P1; independently qualified reduced program | Anonymous constant-expression `const_eval_select` ownership and cache validation | PA25 execution + PA28 intrinsic semantics + PA31 source identity / PA31 | B, coordinated with C | The original reproduction now produces native `7 1` in the qualified B checkpoint and integrated full functional gate. Actual caller/operand/instance and cached Code/body controls pass; the old stale diagnostic assertion failure remains preserved. Wider generic/imported anonymous profiles remain explicitly unsupported and required. |
| LIB-006 | P1; core-to-consumer prerequisite | Complete substituted signatures, operation types and required inputs for real exported generic originals | PA29 / PA29 data invariants; PA31 source-provider fixtures | A | Complete operation-derived input/signature validation passes the combined checkpoint, including all 11 private controls and actual complete-source A/B output equality. Executable artifact/import proof is still required. |
| LIB-007 | P1; core-to-consumer prerequisite | Deterministic generic/source metadata, authenticated remapping and fresh imported body proof | PA29 + PA31 source-role admission / PA29 ordinary multi-crate tests and PA31 source-provider tests | A + integrator | API and CLI tag10 artifact→fresh consumer→native result 7 and stronger full-instance Scope/foreign-owner guards pass A15 and the combined `fa0f4e08` functional gate. Private callable-template, zero-domain Declaration and actual [empty Contract](private-empty-contracts-checkpoint.md) preparation now pass focused controls with bounded warm work. The [shared signature memo](private-signature-mapping-checkpoint.md) and real named-formal certificate now pass combined qualification; checked trait/impl headers and defining/body proof remain required. Exact old first-encoded v0/v1 bytes and fresh plain consumers pass. Inherited/nonrole, default/nonzero environments, original/instance proof and public Provided execution remain required, alongside const/auto roles and nonempty lifetime/body environments. Ordinary Functions origins and v0/v1 bytes must not change. |
| LIB-008 | P1; next whole-crate acceptance gate | Close remaining active core source requirements and emit a real core artifact | PA32, with each failure assigned back to its language owner / PA32 original crate plus reduced owning-PA cases | Integrator | No artifact. Current loading frontier is LIB-032 after alias syntax advances within ptr/metadata.rs to its foreign VTable declaration. Specialization execution, wider macro execution, imported policy and metadata remain required. |
| LIB-009 | P2; crate-graph prerequisite | Source dependency identity, edition/build configuration and reproducible build orchestration | PA34 / PA34 | Integrator | Partial interfaces exist; complete original graph qualification remains required. |
| LIB-010 | P2; not yet reached as a whole crate | Original alloc, allocator/runtime contracts and collection consumers | PA33; PA27/PA28 operation prerequisites / PA33 plus reduced owning-PA cases | Unassigned until core advances | Blocked on LIB-007/008/009; split observed alloc failures into concrete issues when reached. |
| LIB-011 | P2; not yet reached as a whole crate | Original std, platform linkage and abort-profile consumers | PA36; PA30/PA34 platform/build prerequisites / PA36 plus reduced owning-PA cases | Unassigned until prerequisites advance | Blocked on the original core/alloc graph; no unsupported percentage-complete estimate. |
| LIB-012 | P2; externally useful acceptance | Original no-default-features serde_core, then alloc/std serde_json graphs | PA35 / PA35 | Integrator | Run the core-only graph as soon as LIB-007/008/009 permit it; do not wait for std. |
| LIB-013 | P0; prior unchanged-core blocker advanced | Source-declared `core_panic` builtin and edition-dependent delegation with authentic defining-crate/token provenance | PA31 source builtin contract; PA8 expansion machinery / PA31, plus applicable PA8 ordinary controls | C + integrator | Local 2021/2024 source delegation and exact warm spelling guards pass combined qualification. The qualified unchanged-core probe advances to LIB-014. Foreign macro transport and Rust 2018 remain required; full LIB-013 is not closed. |
| LIB-014 | P0; previous unchanged-core failure at `assert_eq`, line 41 | Macro-specific internal unstable feature allowances, beginning with actual `panic_internals` | PA31 source/compiler policy; PA8 token provenance / PA31 with applicable PA8 ordinary controls | C + integrator | Complete local allowance lists and provenance pass combined functional qualification and move original core to LIB-001. The allowance growth controls pass; foreign API/const consumers remain required. |
| LIB-015 | Component fixed; unchanged-core boundary advanced | Authenticated source-declared builtin `cfg_select`, complete predicate/branch parsing and selected syntax | PA31 source/compiler role; PA7/PA8 shared cfg/expansion machinery / PA31 plus earlier ordinary controls where expressible | C + integrator | Local cfg_select behavior passes combined functional qualification at `65f20e8b`, including width/depth growth controls. Foreign macro transport remains open; that older snapshot reaches LIB-018. |
| LIB-016 | Algorithmic repair qualified; one reviewed RSS cost | Repeated full caller-environment opening and premise-index construction across independent projections | PA21 projection/formation machinery; PA31 complete source regression / PA31 plus earlier ordinary controls | B + integrator | Shared mapped-view opening is integrated as `5a5e9d51` and passes combined functional/performance checks at `65f20e8b`. The 32→128 subject ratio is 3.732; 128 subjects use 103,395,319 instructions versus 550,412,747 previously. All 58 combined growth controls pass; one reviewed RSS cost remains. |
| LIB-017 | P0; earlier unchanged-core failure at `format_args`, line 994 | Source formatting builtins and actual constructor calls, including full argument parsing/evaluation | PA26 formatting + PA31 compiler/source bindings / PA31 complete providers, plus earlier ordinary controls | C | Literal source formatting and its real constructor privacy boundary pass combined functional qualification at `65f20e8b`. Nonempty arguments, obligations, temporary lifetimes and foreign transport remain open. |
| LIB-018 | P0; preceding unchanged-core failure at `concat_bytes`, line 1126 | Source-declared byte concatenation builtin, typed byte/string/array operands and exact rejection rules | PA8 builtin expansion + PA31 source identity / PA31 complete providers, plus PA8 where independently expressible | C | Local typed operand/source identity contracts pass combined ca54609a release/native and performance qualification. The same CLI original core reaches LIB-019. Foreign macro transport and wider source contracts remain required. |
| LIB-019 | P0; current unchanged-core failure at `autodiff_forward`, line 1553 | Source attribute macro identity, declaration/import kinds and genuine invocation semantics | PA31 source compiler bindings; PA38 actual procedural/attribute execution / PA31 declaration/import controls, PA38 execution | C | Typed Bang/Attribute lookup, must_use/inline/Test, trait messages, persistent mixed warning/error transport and authenticated removed-body cfg receipts pass the focused attribute checkpoint. All 93 added active source oracles and fifteen native comparisons are qualified. Procedural execution, test-harness generation and imported macro/attribute contracts remain open; no original crate artifact. |
| LIB-020 | P0; stricter original-root failure at `lib.rs:65` | Actual rustc_coherence_is_core and primitive inherent implementation policy, lookup and coherence | PA31 binding + PA19/20 language/index; PA29 transport / PA31 source reproductions plus applicable earlier controls | Integrator | Local structural ownership, contextual full-Self lookup, genuine receiver-step assembly and ordinary local feature-header authority pass nineteen focused private controls and eight native programs. All 66 independently observed source outcomes pass the candidate. No hot package/report fields were added. Imported policy and wider duplicate/unsizing profiles remain required; no original core artifact. |
| LIB-021 | P0; active original crate policy at `lib.rs:66` | rustc_preserve_ub_checks and actual UB-check intrinsic/runtime/CTFE behavior | PA28 operations + PA25 CTFE + PA31 policy + PA29 transport / PA31 complete source providers | B, coordinated with C/A | Typed UB/session/config/owned source identity is integrated at `66bd3750` and passes focused and periodic full CI qualification. Actual CTFE, producer code and deferred consumer specialization remain distinct. Imported policy transport and the combined Test-body/configuration receipt are still required; CPPGM remains unchanged. |
| LIB-022 | Local declarations qualified; original-root boundary advanced | Complete authenticated builtin declaration kinds independently of invocation availability | PA31 source binding + PA8 expansion; execution stays with owning feature / PA31 source providers plus earlier applicable controls | B + integrator | All 61 primary registry roles have actual kinds/availability; seven isolated controls pass in 165.33 seconds. Original 24 slots/ordinals and hot layouts are preserved. All 37 new executions stay closed; imported macro contracts remain required. |
| LIB-023 | Local regression repaired; imported openness still required | Actual local `non_exhaustive` attribute consumption and retained unsupported cross-crate contract | PA29 declaration/export contract / PA29 local and artifact opponents | C | Four merged controls pass with strict Clippy/audit/build in 129.31 seconds; nineteen sources bind 38 independent observations. Same-crate behavior and actual field/duplicate lint policy are implemented through the existing attribute traversal. Typed export still rejects unsupported cross-crate contracts. |
| LIB-024 | Local literal/gate/type slice qualified; wider operations still required | Exact f16/f128 literal values, active feature authority and real primitive/type consumers | PA2 literals + PA9 types; PA25 operations and later ABI/metadata own their effects / PA2 ordinary controls plus PA31 actual source | B | Eight merged controls pass in 170.27 seconds; 40 independent language observations and 64 actual exported-static bit comparisons pass. Reviewed references and 8484 text cases are qualified, including six new public PA2/PA9 cases. The unchanged root reaches LIB-026. Wide arithmetic, casts, pattern/wire/ABI/runtime support remain explicit requirements; no native-wide or core-artifact claim. |
| LIB-025 | Library transport and CLI consumers qualified | Source Functions publication must retain actual must-use/inline contracts and positively checked absence | PA29 publication + PA31 actual headers / PA31 selected-header source variants | C | Nine library controls and the five-control root-owner repair pass. CLI frontend/native gates pass in 112.60/90.60 seconds, including seventeen exact native outputs and three independent lint rejections. Legacy bytes and imported Unknown remain exact. See the [CLI checkpoint](source-core-cli-contracts.md); wider profiles remain required. |
| LIB-026 | Local domains and explicit v2 transport qualified; wider source contracts open | Source trait implementation restrictions with genuine feature, module-scope and imported identity checks | PA5 grammar + PA6 paths/privacy + PA19 traits; PA31 internal feature and PA29 transport / earliest expressible controls plus complete PA31 providers | B | The [v2 checkpoint](pa29-module-contracts-v2.md) carries complete namespace/domain/advisory rows, preserves old v0/v1 bytes, denies fresh foreign restricted impls and selects a valid provider-owned impl through actual LowIR. Final A+B source has a fresh actual CLI consumer pass and strict production/integration/private Clippy; earlier six component controls are retained separately. Wider trait/associated/unsafe/stability contracts, mixed-provider re-exports and Provided/default body execution remain required. Local domains retain74 independent observations and the prior56-file frontier as historical evidence. |
| LIB-027 | Qualified; original root advances | Repeated unnamed free constants retain distinct owners and compile-time checks without colliding lexical names | PA5 syntax + PA6 names + PA25 initializer evaluation / PA6 ordinary names, PA8 macro repetitions, PA25 unused failures and PA31 original-source reducer | C | Six focused controls, 76 independent metadata observations and all 8,493 text cases pass; eight new references are independently checked and reviewed. Actual visibility lint policy, warning provenance/error retention and unused initializer obligations are preserved. The unchanged root advances to 58 files. The alignment reducer retains its separate LIB-029 assertion prerequisite; see the [checkpoint](unnamed-free-constants.md). |
| LIB-028 | P0; local function-member execution qualified; wider specialization open | Specialization defaultness on implementation members, with actual feature authority, overlap and method selection | PA5 grammar + PA19 default members + PA20 specialization + PA31 internal features / applicable grammar and complete source providers | C | The defaultness and sealed-forest verifier checkpoints now have actual SelectedMethod dispatch with complete defining-owner arguments. Both RPIT consumers retain real parent lifetime debt; scalar inheritance imposes no invented static bound. Six merged private controls pass; component native/negative evidence remains in [the design](specialization-graph-design.md). Original std Clone/size_of and printing fixtures remain required, alongside default impl/value and imported graph/defaultness transport. The last whole-core observer remains40239001850,65 opened, no artifact. |
| LIB-029 | P1; source Not and layout/Clone prerequisites qualified; assertion execution open | Real assertion expansion with source-owned panic, runtime and constant semantics | PA8 expansion + PA25 constants + PA26 hooks + PA27/28 memory/intrinsics + PA31 source identity / actual providers and earlier controls | B; C coordinates source contracts | Typed condition transfer and the AST writer remain qualified. [Source Not](source-not-checkpoint.md) now uses actual trait/member/caller proof and has native evidence; [source layout and nominal Clone](source-storage-contracts-checkpoint.md) retain genuine associated constants and provided bodies. Fresh combined source/LowIR equality preserves those native observations. [Source Transmute and const formatter erasure](formatter-pointer-erasure-checkpoint.md) now have actual runtime/CTFE and native evidence. Assert execution still requires real pinned Arguments/NonNull/Location/PanicInfo/handler behavior and full formatting. Genuine [source Abort](source-abort-runtime-checkpoint.md) is now runtime/native-qualified and retains exact LowIR in the combined integration; source-role claims alone do not grant execution. Imported contracts and original factory/advisory debts remain required. |
| LIB-030 | P1; observed SourceClient library-export boundary | Republish genuine imported source trait/generic contracts from a downstream library | PA29 cross-crate generic export plus PA31 authentic source headers / complete source-provider/client cases, with ordinary PA29 controls where expressible | A + B + integrator | Run 40235770528 accepts the public sealed-bound consumer semantically, then its library helper reaches the existing legacy generic re-export rejection. Preserve that rejection until a complete source contract can be transported; a types-only control is semantic evidence only. The module/domain checkpoint still requires actual LowIR provider-impl selection. Mixed-provider generic re-exports and downstream library publication remain required for real crate graphs; no foreign owner may be retagged local. |
| LIB-031 | P0; clauses, binders, shorthand naming and source sizing qualified; complete consumers open | Genuine trait aliases with full declared contracts and actual use-site proof | PA5/6 syntax and identity + PA19/21 bounds/projections + PA31 source features + PA29 transport / owning controls and unchanged core | Integrator; C source Pointee | Canonical typed clauses/edges and nested binders now supply actual shorthand leaf applications, equalities and preserved original inputs. [Naming](trait-alias-naming-checkpoint.md) distinguishes declaration Self from an unrelated formal even when actual substitution makes them equal; complete proof clauses stay. Source sizing distinguishes direct RHS from where/alias mediation. The [family correction](runtime-header-integration-checkpoint.md) now retains all expanded input clauses and separate projected-result aliases, including real missing-Copy rejection. Header-time names, full original/normalized formation, scoped regions, object principal multiplicity, default WF/const and imported contracts remain required. Execution/export stays fenced. The new unchanged-core observer passes Thin syntax and reaches foreign VTable at ptr/metadata.rs170 (LIB-032); this does not establish complete alias semantics. |
| LIB-032 | P0; current unchanged-core loading blocker | Genuine foreign opaque type declarations, identity, sizing and pointer metadata | PA30 foreign interfaces + PA22 unsized types + PA31 source features / PA30 draft foreign-type controls and unchanged PA31 source | C after Pointee consumers | Qualified CLI af93787c reaches `unsafe extern "C" { type VTable; }` in original ptr/metadata.rs170, byte6689, with item-owner rejection. Require actual feature/owner/identity validation, PointeeSized versus MetaSized/Sized behavior, unknown layout and genuine pointer metadata. Never model VTable as an empty ADT. Actual reducers and independent outcomes remain to be developed. |

The combined functional recovery is recorded in
[the alias/allowance checkpoint](library-alias-allowances-checkpoint.md):
2,084 fresh release/native passes, 1,609 authenticated retained passes and all
8,382 active text cases. Its unchanged-core probe at `5602dc27` confirms the
line-170 boundary with the same source/configuration and four opened files.
Those performance measurements recorded one growth failure (LIB-016). Its
qualified repair now passes the combined prerequisite checkpoint above. The
following older measurements remain historical evidence, not the current state.

The focused LIB-016 successor at `f96ec23a` passes 1,439 nonignored private
tests, thirteen frontend cases and 31 native programs. Its fixed optimized
comparison preserves all nine equal-output pairs and sixteen native executions,
but still fails subject growth: 48,267,926 to 441,032,521 instructions for
32 to 128 subjects (9.137×). The previous compiler takes 550,420,452 instructions
at 128 subjects, so this is about 20% less work without resolving the scaling
problem. Evidence is in B's
`obj/parallel/pa31/projection-opening-optimized-v2/`; analysis SHA-256 is
`49c92d4bd53c839c192a08970cd6d9b8f50af4e4bed1f831ad18d3dc556aa78f`.
The bounded observation on the same source's qualified debug test records
1,046 successful exact-owner comparisons and no foreign-owner cache misses;
867 queries use genuinely captured caller regions. Further work targets that
structural remapping, preserving fresh proof for every caller. This performance
result does not advance the unchanged-core source boundary.

The [mapped-view successor](pa31-projection-opening-sharing.md) fixes that measured
failure without changing its inputs or limit. Complete-source subject counts
8/32/128 use 9,086,952 / 27,625,883 / 103,096,127 instructions. The largest case
previously used 550,420,452 instructions: 81.27% of that work was removed, and paired
wall time fell 75.56%. Subject growth is 3.040× / 3.732×; site growth is
1.879× / 2.868×, all below 6×. Ordinary Copy and macro instruction controls
change by -0.163% and -0.018%. All nine output pairs and sixteen native contexts
pass, with no instruction, RSS or growth triggers. The actual source consumers
entered the mapped path 867 times, all successfully; that bounded observation
is separate from the optimized measurements. Hot layouts remain unchanged and
cold allocation costs are documented. B's `captured-opening-optimized-v1`
completion SHA-256 is
`5aa3265a232ed91a7f550a9bc77c19c9128ad35ac3803587baa1faf5c4ca64d2`.
The exact reviewed implementation is integrated locally as `5a5e9d51`; the
preceding failed measurements remain retained. This fixes a prerequisite and
does not produce a core artifact or advance its latest line-994 source boundary.

The separately qualified scoped-declaration component uses worker CLI
`1c2298f1ac0ac64e30e3ae6db4ecf5432e86219a69adb64c0ede50747ad2a29f`.
It reaches file 3, byte 8704, the `rustc_builtin_macro` attribute at line 234
on `cfg_select` (declaration line 235). The compiler exits 1 without an
infrastructure failure; the four original source hashes remain unchanged.
Evidence is in the C worktree's
`obj/parallel/pa31/scoped-declarations-whole-core-v3/`, result SHA-256
`69f63ebdab2a9b0a7a5d244fca0b3f413c468f54836a9a6e62af3acf20a21d8c`.
The final component gate passed twelve fresh frontend tests and 46 fresh native
programs, retaining 1,437 private passes by exact unchanged source and receipt
hashes. Formatting, architecture/file audit, strict Clippy, harness and upstream
checks passed. The preceding failed test-contract checks and provisional probe
remain preserved in the [component handoff](pa31-scoped-declarations-handoff.md).
The exact reviewed source is integrated locally as `68a30f1b`; combined release
and performance qualification remain pending. No core artifact has been built.

The [qualified cfg_select checkpoint](pa31-cfg-select-progress.md) includes
canonical source ownership, complete parsing of every branch in its actual
fragment category, and configuration/installation of the selected syntax.
Its 61-source corpus and original scoped provider retain 206 independent
observations, including 41 native executions. The final candidate gate passes
1,449 private cases, twelve frontend cases and 51 native programs. Its exact
compiler files are integrated locally as `4ad4db87`. Combined release and paired
performance remain pending; general scoped invocation and foreign transport
remain open.

The original provisional observation (`ac5aaebb`, CLI `687cf42c` prefix) at
`obj/parallel/pa31/cfg-select-provisional-core-v1/` is retained with its failed
component gate: 1,446 private passes, ten of twelve frontend passes, no native
phase. The diagnostic assertion and overly narrow ordinary-Clone profile were
fixed without changing the required fixtures. The provisional result is now
superseded by the successfully qualified component's unchanged-core probe:

- Source `d8ed72868bb73ccd8fd481720ed0f1f5cdb64144`, CLI SHA-256
  `0836f6af2d4ad8dfb58d88475d486bca99d223ba614e22627587ee7cbf31f33d`.
- C worktree `obj/parallel/pa31/cfg-select-whole-core-v2/`, result SHA-256
  `05b59bbd32e145fb92d534f21eaa705a6975fd3cceeccb87e0b30238c3e0d6b5`.
- Compiler exit 1, no infrastructure failure, `format_args` builtin attribute
  at line 994, byte 35014. All four opened original files still match their pin.

LIB-002's [inherent-method component](pa31-inherent-member-contexts.md) now
passes 1,443 private cases, 28 affected integration cases, seven native test
functions and all 8,382 active text cases. The final `8e798fcd` recovery preserves
both the declaration-record layout and the committed private-receiver diagnostic.
It is integrated locally through `abae2094`. Seven scoped trait-method cases in
`pa31/tests/spec/declarative-macros/trait-members/` have 24 independent observations,
including five native programs. They establish discovery and identifier-context
rules, not candidate support. The subsequent trait-discovery component below
implements the scope walk; associated-member conformance, scoped invocation and
the complete source artifact transaction remain open.
No core artifact or additional assignment has been completed.

The [trait-discovery component](pa31-trait-discovery.md) at `956c9904`, integrated
locally as `708ee191`, passes format, the 2,327-file audit and strict Clippy,
1,460 private cases, 202 affected integration cases, twenty native test functions
and all 8,382 active text cases. Eleven independent scope programs establish
actual block-boundary and method-context rules; their 34 observations preserve
two rejected initial hypotheses. These component checks do not enable general
scoped invocation or advance the latest observed core boundary beyond line 994.
The next conformance corpus independently passes sixteen metadata/object checks
and four native programs across eight sources. It requires impl identifiers to
map to actual trait members after hygiene adjustment and rejects duplicate
canonical-member mappings; candidate conformance changes remain pending.

The integrated source `df2c48cf` passes fresh format, the 2,325-file audit,
all-target/all-feature strict Clippy, all 183 harness tests, 377 registered
maintainer roots and upstream provenance verification (24 fixtures and 126
Reference files). Evidence is in
`obj/parallel/cfg-members-integrated-static-v1/`; completion SHA-256 is
`2ac0af9765916861d0da067926fc46605939f4875805486cab07ae7a71a090ce`.
This validates the combined wiring and retains each component's functional
evidence; it is not a combined release or performance pass. Guard
`4cebb57ecc814be3ae0d3988a9ca0258` finished with status 0/error null and cleanup
`1792863734641562721` completed without new disposable outputs. The two diagnosed
failed inherent-method runs were separately pruned after recording all 334
native observations: cleanup `1792863758154341210` removed 668 generated files,
1.40 GiB of unshared blocks, while preserving sources, diagnostics and hashes.

The first LIB-007 executable transaction uses a complete smaller provider with
ordinary sizing/Clone/Copy/receiver declarations and one generic free function.
Thirteen independent rustc observations establish source acceptance, native
consumer answer `7`, and the expected double-move errors without Copy impls.
The source profile is defined by supported semantics, never the fixture's name,
arity or body count. Renamed, reordered and multiple-function controls are
required. Every actual provided body must cross the boundary and prove again.
The larger three-body const-trait/default provider remains a required successor;
this first transaction does not claim to build core or finish PA29.

The contextual lookup, input-plan and local panic checkpoints are integrated
and qualified together; their exact fresh/retained counts and measured costs
are in the linked checkpoint. Worker A's intermediate column codec passes
component checks; its actual artifact/import execution remains disabled and
unqualified. B's actual alias-scope recovery, C's complete allowances and the
enum-alias correction pass combined functional checks; the measured projection
opening growth failure remains under repair. Public and worker-only core observations remain separate above.

## Reproductions, tests and closure criteria

Paths below include unpublished worker fixtures. Their presence in a working
branch is not a passing test or a released assignment contract.

- **LIB-001:** `pa31/tests/spec/declarative-macros/` contains predeclared source
  contracts and independent observations for concise/braced declarations,
  forward/imported aliases, repeated introduced items, mixed transparency,
  captured path segments and privacy. Maintainer suites `declarative_macros`,
  `hygiene`, `textual_scopes` and the PA7/PA8 suites retain legacy behavior.
  Close only after the real positive/negative source corpus and native answers
  pass, including exact-context import wakeups, then rerun original core.
- **LIB-002:** extend that PA31 fixture cluster with predeclared captured/literal
  field names, repeated opaque containers, method/associated-name collisions
  and private-member opponents. These new cases are **required, not yet
  qualified**. Reuse PA12 aggregate and PA19 method controls. Close only when
  actual member selection and privacy consume the shared typed context rule;
  a context-aware storage table alone is insufficient.
- **LIB-003:** retain
  `pa31/tests/spec/trait-default-effects/projections/provider-moved.rs` and
  `provider-partial-container.rs`, their public-context counterparts and
  independent contracts. Add actual body-alias/foreign-environment tests,
  independent `'a`/`'b`, complete trait/family arguments, nested binders,
  unresolved inference and failed-budget controls. The private
  `body::defining_cleanups` and trait-selection tests must pass alongside the
  retained native contexts. Do not fix ambiguity by merging arbitrary matches.
- **LIB-004:** keep the unchanged
  `pa31/tests/spec/const-eval-select-ctfe/reference-anonymous-selector.rs` and
  its independent Rust acceptance. The bounded `read_frontier` invariant now
  observes a canonical `u8` read and false Copy decision despite the actual impl.
  The retained 54-event trace resolves all ten required breakpoints: both exact
  queries use the same environment, no reuse cache, and an actual positive header;
  no validation completion occurs before the error. Pending header completion is
  already finished, so waiting on that dependency alone cannot fix the bug. Add
  validated-implementation dependencies and cold/warm/declaration-order controls. Close
  only with correct dependency resumption and a successful complete program;
  returning Deferred forever is not a fix.
- **LIB-005:** preserve `reference-anonymous-selector.rs` and its exact native
  `7 1` result. The complete owner/cached-result/foreign-Code controls now pass
  combined functional qualification. General generic/imported anonymous callers
  and wider reference/conditional callback profiles remain required.
- **LIB-006:** use the actual
  `pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs`
  collector and private `qualified_source_rows`, `transport_domains` and
  `regional_forest` controls. Add same-arity wrong signatures, changed reference
  types, wrong binder relocation, omitted inputs and unused-body failures.
  Require complete operation-derived coverage, atomic failure and independent
  width/depth/shared-DAG work and storage bounds.
- **LIB-007:** extend PA29's multi-crate contracts and the PA31 source-header
  transport family. Require complete provider → artifact → fresh consumer
  checking → native execution, input reordering, duplicate/missing dependencies,
  tampered owners/roles, const permissions, lifetime/loan/cleanup proof and
  deterministic text views. Support indexes or serialized bytes alone do not
  close this issue. Exported macro provenance is a separately required subtask.
- **LIB-008–012:** each actual build attempt supplies a versioned source/config/
  dependency lock, exact diagnostic, opened-source inventory and artifact/native
  result. Add a reduced owning-PA fixture for every newly reached failure while
  retaining the unchanged whole-crate acceptance test. An earlier parser error
  must not make a later semantic negative appear to pass.
- **LIB-013:** `pa31/tests/spec/source-panic-macros/contract.json` (original source
  checkpoint `af49643a`) declares ten single-source cases in three
  editions and nine imported-wrapper contexts. These source-builtin controls cover real
  declaration identity, aliases, call-site edition, unchanged argument tokens,
  definition-crate delegation, and forged/missing/duplicate claims. Independently
  qualify source and native outcomes before generating text. Keep the original
  core root as acceptance evidence; a declaration-only reducer cannot qualify
  actual panic delegation or a whole-core build.

- **LIB-014:** retain the unchanged core `assert_eq` declaration and add a
  reduced PA31 source-macro allowance family. Independently declare and qualify
  actual `panic_internals` and `core_intrinsics` use, caller versus definition
  literals, captured-token opponents, missing/foreign/disabled owners, repeated
  names and bounded work. A parsed allow-list alone cannot qualify feature use;
  real stability/feature consumers must check the authenticated expansion grant.

Every completed fix includes its owning handout update, independently declared
positive/negative outcomes, relevant maintainer tests, and manifest-discovered
text-oracle tests when that assignment's contract is published. Generate and
review oracle text; never hand-edit it or accept an empty/planned suite as a pass.
Text fixtures belong in the earliest PA owning all features, and private test
roots must remain registered in the existing maintainer suite groups.


### LIB-019 diagnostic follow-up

Mixed error-plus-warning transport remains incomplete. The frozen
`diagnostic-duplicate.rs` control in
`obj/parallel/pa31/remaining-attribute-headers-independent-v1` independently
reports a hard duplicate internal `on_unimplemented` error together with a
`malformed_diagnostic_attributes` warning in both editions and emissions
(results SHA-256
`a688e91fbe17c8ac7b9a42c4c1b62e77e7e4eb2e5ee3a47becce8ae68a422e3a`).
The current semantic error result carries only `Error`, so declaration warnings
cannot survive when its facts are dropped. Preserve the exact-output control;
retain this as a separate requirement for a typed cold error carrier and driver
rendering. Successful default warnings and primary selected-trait messages must
still work in the current consumer slice. Neither success nor complete
`on_unimplemented` diagnostic parity is claimed before qualification.

## Work and qualification rules

Keep the three existing GPT-6.1-Sol/high workers. A worker can carry multiple
independent issues, but keeps coherent commits and an explicit handoff for each.
Agree shared APIs before concurrent edits; use separate worktrees. Current
primary lanes and next executable exits are in the current execution queue above.
These are independent blockers; keep three implementation workers while giving
integration/qualification priority for heavy slots, without increasing concurrency.

Source work proceeds concurrently. Heavy jobs use the existing 150-GiB reserve,
50-GiB/worktree ceiling and CPU/RAM limits. After the qualified schema-2 tooling
is installed at an idle boundary, admit up to three jobs with explicit smaller
absolute caps and measured headroom. Keep measurements exclusive.
Use focused batched checks during a fix and combined focused checks at integration.
Long release qualification runs in scheduled/manual CI and at actual assignment
release, not after each integration. Reuse unchanged qualification and independent
observations with exact source/tool bindings; retain failures and clean outputs.

Update this tracker at each handoff/integration and each original crate attempt.
Report three separate outcomes: **issue fixed**, **crate boundary advanced**,
and **assignment released**. Closing an issue requires its complete reproduction
and guards to pass. Advancing a crate requires a fresh unchanged-crate result.
Releasing a PA requires its entire cumulative contract; no partial activation.
