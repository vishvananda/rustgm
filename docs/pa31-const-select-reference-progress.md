# Shared-reference selector source checkpoint

## C9 observations, native continuation and exact observer root

Frozen `02bfb4f3` ran under resource token `806eb018f368454094bb30fb6524b93b`.
All 32 independent rows and failure bindings authenticated. Every static gate
passed, followed by all 13 frontend functions and 68 of 69 private controls.
The transferred Body identity/warm reuse and complete named width/depth controls
passed. The sole failed private test was the investigation observer's missing
event assertion. The focused log SHA-256 is
`5d77005c256c43e948f1e63b3c738faaa01ba874b04f0fb1105eef991121ca1a`.
The outer guard returned 1 with no resource error. Scoped cleanup removed only
the 4096-byte backend executable in receipt `1792820219037121098.jsonl`.

The separately declared `reference-native-v1` continuation ran against that exact
unchanged clean source; it did not rerun static/focused checks or independent
oracles. Its plan SHA-256 is
`4bc46cc6baefe9f98ed8e14e0cfdcfba97ba75b28af79414da3613976fce78f8`;
runner SHA-256 is
`9e651c027c450d3c2efa4a581916ed211427995673ee6d87520d5651f03bfff0`.
Resource `84325d0aeab5416a9be006712f1b937e` finished with return code 0 and no
resource error. All five native functions covering 63 declared contexts passed,
including eleven reference positives and their runtime opposites. Native log
SHA-256 is `bb0f4c3e8fe82a4bf15aa833cebf58e7171a890d2434a220a4d7467a4a762b4f`.
Default cleanup removed 76 generated artifacts in
`obj/cleanup/1792820358717122581.jsonl`. C9's focused status remains failed.

Read-only source tracing found the observer's exact owner mismatch. Constant
Queries::prepare scans the complete scalar root and calls check_execution(root)
when any descendant needs execution. For this input, Body.source is the actual
Cast consuming the dereference. Both syntax spans start at byte 2386; their
distinct NodeIds cannot be inferred from that byte. The test-only correction
derives the Cast parent through its actual `value == dereference.node` edge,
requires that parent to be unique, and uses its exact NodeKey as the Body filter.
It separately requires an observed Read statement sourced from the original
dereference. Builder::Enter(Cast) schedules Enter(operand); projected dereference
syntax schedules ReadPlace(dereference), whose emitter calls evaluate with that
same dereference node. The later cast_value emits a distinct Cast consuming its
saved operand ValueId. Neither producer attributes that Read to a guessed node.
No broad body/offset fallback or missing-event relaxation is added. The canonical
Read type and Copy goal/decision remain unknown until this exact observer runs.

The last accepted C production was `736cb69a`, represented on main as `fc36be0e`.
Its evidence/base `f8e67119` is represented as `a67564e2`; all eight closed-selector
commit pairs have identical stable patch IDs and their accepted dev/src trees
match exactly. Only `f8e67119..02bfb4f3` is the reference successor delta, rather
than its duplicate earlier history. C9 `integration-boundary.json` binds the ten
successor commits and 76 changed paths, and its full-index net patch SHA-256 is
`3c0effa381adcb8c01e6af7c0820851ecae451ed2d23dd1e3d39c551b7bcd861`.
Later main overlaps in Copy preparation, const member entry and proof preparation
must preserve its independently integrated defining cleanup and execution gates.

## C8 test helper recovery

Frozen `95f01566` finished with status 101 under resource token
`e0abb7fe26f84728a9bdc8d4d6fcafd3`. All 32 independent rows and retained failure
bindings authenticated without an oracle invocation; formatting and architecture
audit passed. Strict checking compiled the corrected observer accesses, then
reported only the expanded early named initializer test at 103 lines against
the 100-line function limit. Its strict log SHA-256 is
`d133a0a31edf6681f030e8ca0390e6f0ccdce7e321a7e3c97392ddd3a6750dba`.
Candidate focused/native tests remained unrun. Direct scoped cleanup removed
zero generated files in receipt `obj/cleanup/1792818904124113334.jsonl`.

The test-only recovery extracts its exact transferred Body assertions into
`assert_transferred_named_body`, retaining the same Prepared declaration key,
source, result, initializer kind, const context and full-instance checks. The
owning test still checks Engine absence, anonymous-root absence, Code identity,
and every construction, proof and warm-use counter. No assertion is removed,
compressed or allowed around a lint. After formatting, the owning test has 93
physical lines and the helper has 21; the separately changed Read observer and
its test have 58 and 80, respectively, including their bodies' braces. These
conservative physical spans are below the 100-line limit. Existing API accesses
are unchanged from C8's successful type checking. Fresh strict and all previously
unrun 69 private/13 frontend/63 native contexts still require admission; no
compiler production behavior or independent source bytes change.

## C7 observer source wiring recovery

Frozen `30e45fbe` ran under resource token `f7a289e105024608b9b72f38a247ad8a`.
All 31 prior oracle rows and the exact C6 failure/layout receipts authenticated.
The additional scalar anonymous source independently passed metadata/object/native
with stdout `7 1`. Its binding SHA-256 is
`f80d135ba08de3fc1a18d9f54b482b4430394db188e65d2b1667c364608b36d2`;
results SHA-256 is
`c5ca0dd6a53bcd6d184eeb75fab44e0b920e7f6932439e242b5f0a7aced49ae4`.
These unchanged observations are retained, not rerun, in the successor.

Formatting and architecture audit passed. Strict checking stopped with E0609 in
the new test-only observer: `Value` has `ty` and `kind`, with no `source` field.
No focused or native candidate tests ran. The exact strict log SHA-256 remains
`a5a0e18a01ab4cd9c6237f49739e51fe67b81045579af9e0573b51046a64be8c`.
The outer resource returned 101 with no resource error, and direct scoped cleanup
removed zero generated files in `obj/cleanup/1792817563288166853.jsonl`.

The actual existing producer, Builder::evaluate, allocates Value and immediately
retains Statement with Action::Evaluate(ValueId) and its source site. The corrected
observer visits those actual statements, follows the retained Evaluate identity
to its Value, and obtains source through Body::syntax_site(statement.source).
It records the actual statement index and ValueId together. An invalid value ID,
missing source or missing projection cannot become an invented fallback. All
other observer fields were checked against the concrete Body, Prepared Copy,
Predicate and trait Application APIs. No solver, CFG, loans or selection changes
are introduced. The next unqualified batch retains 32 independent rows and runs
fresh static checks followed by the same 69 private, 13 frontend and 63 native
contexts; the actual reference-read classification is still pending.

## C6 observations and test-owner recovery

The frozen `4201ddce` C6 attempt finished with status 1 under resource token
`db1d8ef487414aeeb4200f295d940f3b`. It authenticated all 31 retained oracle rows
and passed formatting, architecture audit, strict all-target/all-feature Clippy,
harness, upstream and backend checks. The focused batch passed 66 of 68 private
controls and 12 of 13 frontend functions. All eleven accepted reference contexts
passed, including the original binder, complete slice provider, early and nested
named owners, and caller widths 8/32/128. Native execution remained unrun.
The focused log SHA-256 is
`460b7949d2fa22850140aa7f7102c37d2f36f407722c6d8d679269e3bc349180`.
Direct scoped cleanup removed only the 4096-byte backend smoke executable;
receipt `obj/cleanup/1792815785478434919.jsonl` preserves the deletion record.

Two failed assertions observed the wrong owner or namespace. `analyze_source`
calls `Context::transfer`, which moves demanded bodies from `Engine.concrete`
into `Prepared.concrete` before returning. The corrected control requires the
actual declaration-keyed Body in Prepared, with its exact source/result/owner,
and preserves all existing warm Code identity and construction/proof counters.
The expected 40-byte Work came from `sema::bodies::Work`; this control observes
`sema::constants::Work`, which C6 measured as 64 bytes. The separately published
pre-reference compiler SHA-256
`b0b81f5cccd3a0b96da56660c2c6709ff01a17877c8cabde4b47f97c91e9e849`
already copies four 16-byte blocks in constants Worklist::push, and allocates its
(Context,Work) entries with an 80-byte stride. Its line-table DWARF omits Work,
so no DWARF sizeof result is claimed. The read-only disassembly and command
bindings remain in C6 `baseline-layout-readonly.json`, SHA-256
`c715ecdcd6faa1b5b2d7ae9c6fccf266648781664a10235c7979b7c6abfdcadf`.

The original independently accepted `reference-anonymous-selector.rs` reaches
an earlier unfinished source-Copy/read frontier at byte 2386: `cannot move a
non-Copy value out of a reference`. It does not establish the intended later
selector owner rejection. Preserve its source, original hypothesis and mismatch;
this valid Rust path remains required unfinished work toward full PA25/std.
A bounded test-only observer at the actual Copy preparation completion records
its canonical Read type, place, registered Copy identity, retained goal/application
and selected decision before unchanged loans run. The observer neither repeats
selection nor reconstructs a CFG; absent or truncated observations cannot pass.
Those exact new observations remain pending an admitted run.

`contract-anonymous-owner.json` separately declares `anonymous-empty-selector`:
a true anonymous empty-ARG scalar call with const F=7 and runtime G=1. Metadata,
object and native success with stdout `7 1` are hypotheses declared before probes;
candidate owner rejection must match its exact diagnostic. This additional source
isolates the owner boundary and does not replace reference-read support. The next
frozen plan authenticates all 31 old oracle rows, independently observes only this
new source, then runs fresh static checks, 69 private controls, 13 frontend
functions and five native functions covering the same 63 accepted contexts.
No production selection, caller permission or loan rule changes in this recovery.

This successor is unqualified. It starts from the closed-selector evidence
checkpoint `f8e67119`, with the approved interface and initial hypotheses saved
in `ebe2de8d`. The separately qualified `736cb69a` production and all its receipts
remain unchanged. At the initial source freeze only formatting and whitespace
checks had run. The first independent observations and the separate provider
recovery below do not qualify the candidate compiler.

## Typed producer and retained owner

The common completed Code-purpose path calls `Inputs::prepare_uses` after region
and loan proof. It accumulates retention needs through every actual Call,
including after an earlier selector or a conditional owner requires retention.
Reference descriptors are collected in that existing charged Body-value pass;
there is no second Code scan, signature-opening solver or trait query. Each
descriptor retains the actual selector/F full instances and a whole caller-opened
function type made from the exact ARG tuple fields and RET already related by
the completed FnOnce proof. The first profile requires nongeneric safe Rust
source const F with an internal late binder. Imported/member/intrinsic/conditional
F and generic argument remapping retain explicit frontiers.

`CheckedInput` owns `Prepared` Code plus an optional `use_inputs::Source`
(Arc Contract and lazy descriptor table). `Catalog::stage_*` finalizes publication
mode/result transport into one unpublished Rc Code. `Inputs::plan` prepares the
Rc-bound Row and existing effect Requirements, reserves index capacity and checks
the live retained payload. `Registration::publish_source` publishes canonical
Code only after the row plan passes; its following row insertion is infallible
and uses the already reserved slot. Scope registration still precedes Code/Row
publication. Failed input index capacity remains charged to its actual owner.
Payload accounting excludes allocator metadata and shared Rc/Arc pointees, as
defined by `support::retained`; it is not a whole-heap measurement.

Rows and continuations now share Rc Row owners to match their existing
single-threaded Rc Code owner. Immutable Contract, Requirements and Environment
owners remain Arc. No new Machine/Code/Frame/Body/State fields are added. Actual
cold/hot layout sizes, descriptor capacities and retained payload remain pending
the admitted private checks; expected size is not evidence.

## Reached consumer and publication modes

Engine builds `CatalogRef::Source(&Catalog,&Inputs)` when the retained owner exists.
The reached selector inspector charges its attempt, authenticates the actual
Code object and exact Call ValueId, then validates the full selector/F and whole
opened tuple/result descriptor. A warm executable F cannot replace a missing or
foreign retained caller row. Ordinary FunctionBody demand/entry and allocation
liveness still own execution. Public and owned callback signatures are checked
separately; owned result consistency grants no caller permission.

Ordinary/lent functions, specialized functions, fresh/lent expressions, named
initializers, selected source initializers, anonymous instances and paid child
instances all use the staged Code/Row seam. Original-purpose proof creates no
Code or descriptor. Existing source cache hits keep their already published
Code and original Rc-bound Row; a new proof is not retargeted by matching headers.

Named InitializerBody demand now constructs the existing declaration-owned
initializer path before proof rather than publishing anonymous expression Code
and changing its owner later. Warm named lookup validates exact source/type/kind/
canonical index before reuse. Actual anonymous expression targets stay distinct.
The original semantic execution-target producer already uses typed initializer
ownership for a true named root. Cold/warm and forward early-name controls check
Code identity and unchanged construction/proof work on warm demand.

## Frozen hypotheses and pending guard

The original `reference-binder.rs` and its independent positive are retained by
their original hashes. `contract-reference.json` declares four new positives and
five intended language negatives. `contract-reference-accounting.json` adds a
forward early named positive and three complete frozen sources at caller widths
8/32/128. Each scaling source exercises reference, closed-selector and ordinary
callers at one versus four complete caller instances, with expected native
output `7 1`. All four added sources and clients are hashed before probes.

The source accounting control expects one warm preparation pass: V plus 14 per
reference selector, V plus 7 per closed nongeneric selector, and V for ordinary
calls. Reference descriptor filtering contributes 8 per reference selector;
closed filtering contributes 2. Generic closed filtering contributes 1. These
counts include existing relocation work; they are predeclared assertions,
not observations. No-catalog and non-const fast paths retain zero work. Closed
selectors allocate no descriptor table; ordinary callers retain no input Row.
The test observer is bounded to the six named callers and fifteen actual full
instances in each frozen source; it cannot accept an arbitrary positive CFG.

The planned focused guard retains the prior 48 private controls and adds nine
source-derived controls: exact Code/Input identity, full selector/F/correlated
signature, failed/warm row preflight accounting, paired publication, warm caller
allocation/input authority, actual owner-width scaling, early named reuse,
wrong warm named metadata and callback public/owned entry opponents. It runs
12 frontend test functions and five native functions covering 61 contexts.
Thirteen new independent inputs are declared: eight positive metadata/object/
native hypotheses and five intended metadata rejections. The original reference
positive and all earlier closed/runtime/opponent oracle receipts are authenticated
without rerunning unchanged inputs. The scope remains a shared-reference adapter;
whole-core, assignment activation and broader conditional callback authority are
not established.

## First independent attempt and provider recovery

The frozen source was `7cd74b97fb5e6271796e9fd319fb20f0d9fe7ce9` and runner
`reference-source-v1/run.py` SHA-256
`19b2d874732d991f1a64fc53e68542ce550c98264d0d08bfdb0a3eb5b3ce62f9`.
Its first resource guard, token `1f9f5000099244c6b1ddda15c2499f7c`, returned 1
before launching the runner: scoped cleanup cannot select managed `obj/rust`.
The resource receipt remains intact. The wrapper-only recovery used ordinary
default cleanup, token `47601760b60246398cea3f1a9edd09e8`, and returned 1 after
the independent stage. Prior 15 successful oracle rows and two archived Index
disagreements authenticated successfully. Seven new positives matched all
metadata/object/native hypotheses and all five intended metadata negatives matched.

The sole failed positive was the unchanged `reference-slice-identity.rs`:
pinned Rust reported E0308 at the rt return, N initializer and runtime local,
where `&[u8;1]` was expected to coerce to `&[u8]`. This provider declares the
CoerceUnsized role but includes only mutable-reference and mutable-pointer
implementations. It lacks shared-reference CoerceUnsized. Preserve that input,
its original acceptance hypothesis and both mismatching metadata/object results
as an incomplete-provider hypothesis disagreement, not RustGM slice support or
a valid-language rejection. No candidate static, frontend/private or native
stage ran. Scoped failed-artifact cleanup then completed with zero files removed;
the successful independent qualifier had already removed its disposable outputs.

The new `reference-slice-complete-provider.rs` preserves the entire original
slice identity/coercion program and adds only the shared-reference implementation
contract from pinned core `library/core/src/ops/unsize.rs:58`:
`impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a U> for &'b T {}`.
Removing that exact insertion restores the original source SHA-256. Its C client
is byte-identical to the original. Expected metadata/object/native status 0,
stdout `7 1` and empty native stderr are declared in
`contract-reference-recovery.json` before any recovery probe. Only this separately
named fixture replaces the invalid positive in `selected_references::POSITIVE`;
the test identities/counts and compiler production sources are unchanged.

The recovery contract binds all twelve unchanged successful new rows by exact
source/client/canonical-row hashes, plus the old plan/binding/results/summary
hashes and archived failed slice row. The next guard authenticates these
observations without invoking an oracle, independently qualifies only the one
complete-provider variant, then runs every previously unrun static/candidate gate.
Its candidate plan remains 57 private controls, 12 frontend functions and five
native functions covering 61 contexts. A successful oracle recovery is still
pending at that recovery freeze; the following attempt observed it independently.

## First candidate type-check attempt

The complete-provider recovery was frozen at `a021978e` with production exactly
equal to `7cd74b97`. Guard `reference-source-v2`, token
`4cacc1a5429a4011a262c83ff77b350c`, authenticated all prior 27 successful oracle
rows and the three archived provider hypotheses, then independently accepted the
one new slice fixture as metadata and object and observed native stdout `7 1`.
Its binding SHA-256 is
`6c4d432abf8f0118a9962759014fb1d84084f2b06d1f028115f43254f91100ed`;
results SHA-256 is
`71671b710911993520e231b06459ffb8a0cb28551bd987fe43fc37389b27f3a0`.
These observations are retained without another oracle invocation.

Fresh formatting passed and the architecture audit reported 2157 files with zero
findings. Strict all-target/all-feature Clippy returned 101 before any candidate
test execution. The test-only errors were missing Body/CheckedInput imports after
narrowing a parent import and attempting Debug formatting on const_eval::Error.
Production diagnostics were one wildcard import, a documentation backtick, a
default initializer followed by field assignment and an Rc argument cloned
instead of consumed. The strict log SHA-256 remains
`04159b1580a5238ee8e10e4ae1a0e039178b3e6d1a36cda41b8ea4c8f8ce7b4e`.
Scoped post-diagnosis cleanup completed with zero generated files removed.

The bounded source correction adds explicit imports, prints error.message, fixes
the documentation and Uses initializer, and moves the already supplied Rc into
Row after extracting Code metadata. The Row still retains that exact finalized
Code object; no extra allocation, contract relaxation or publication-order change
is introduced. The next guard authenticates all 28 successful prior oracle rows,
then runs fresh formatting/audit/strict before every previously unrun candidate
gate. Candidate tests, measured layouts and native support remain unqualified.

## Named query producer and restoration recovery

Guard `reference-source-v3`, token `990e21a088f1473fbb6c8b9437d0fb2b`,
authenticated the same 28 successful oracle rows and passed fresh formatting,
audit, strict all-target/all-feature Clippy, harness, provenance and backend gates.
The focused batch built successfully: 56 of 57 private controls and 11 of 12
frontend functions passed. Both failures came from the unchanged independently
accepted `reference-early-named`: its selector at byte 2412 reached anonymous
Code and retained the explicit unsupported caller-owner diagnostic. The native
phase did not run. The failed focused log remains SHA-256
`c2c2f6052e8d67e358d9947899599d182d74018454dc047748843555a5bab038`.
Cleanup removed the one generated backend smoke executable, preserving all logs
and source bytes in receipt `obj/cleanup/1792807086871340628.jsonl`.

The first five reference positives, including the complete-provider slice,
lowered before that frontend failure. Actual private caller scaling at widths
8/32/128 passed for all fifteen full instances: reference values/work/payload
were 66/178/256, 258/706/928 and 1026/2818/3616. Closed and ordinary paths retained
no descriptor tables; their predeclared work formulas held. Observed layouts
were Row96, Opening12, CheckedInput328 and Prepared312. Hot sizes stayed
Engine1696, Machine4728, State448, Body448, Code304, Value24 and Continuation80.
These are local focused observations, not combined release or native qualification.

The actual early producer is `Queries::enter(Query::Constant/Static)`: after
typing, it called `begin_pattern_value`, whose no-pattern fast path returned
without installing a named `ValueScope`. It then forwarded the child Node and
prepared execution as an anonymous expression. Changing only the later
InitializerBody requirement could not fix this earlier semantic-query path.

The source correction installs existing `ValueScope::Initializer` from the actual
dense declaration index before typing/child dispatch, independently of patterns.
One cold `Work::RestoreValueScope` event captures the caller's existing Context
before the named global query switches to the default Context. It is below all
child, Forward and PatternFinish work. A normal pop restores scope before any
fallible bookkeeping, then charges its step. Real suspension moves the current
scope and queued restoration in the existing Continuation; resume restores both.
Errors first abort active pattern frames, then discard orphan work while applying
only saved restoration scopes/contexts. Synchronous unresolved-body errors use
the same cleanup. Attempted budget work is retained. No pattern frame is allocated
just to establish ordinary named ownership, and warm ready queries add no event.

The existing root selector now checks exact dense definition index, semantic body,
root equality, completed declaration type and nongeneric/nonassociated ownership
before returning a named target. It does not retag anonymous Code or relax a
caller-owner gate. Query::Instance dispatch remains unchanged and its existing
canonical-result/publication/symbolic-owner opponents remain in the next guard.
Cold Work/Continuation layouts and actual width/depth stack work are asserted or
reported by new source-derived controls; those new observations remain pending.

`contract-named-owners.json` declares three new independent source hypotheses
before probes: actual named-to-named early dependency, true anonymous selector
CTFE that remains explicitly unsupported in the candidate, and the separate
ordinary runtime G opposite. All expect pinned Rust metadata/object/native success
with stdout `7 1`. Prior sources and all failure hypotheses remain unchanged.
The next planned guard authenticates the 28 prior successes, observes only these
three new sources, then runs fresh static gates, 68 private controls, 13 frontend
functions and five native functions covering 63 accepted contexts. It includes
the existing strict unselected projected-root control and five Query::Instance
cache/owner controls. No new conditional/member/imported callback profile is added.

## Named-owner gate wiring recovery

The first C4 admission was refused before a scope or runner launched: session
50243 reported 191.9 GiB free versus 195.2 GiB required with reservations. That
tool observation is preserved separately as `admission-refusal-v1.json`, explicitly
not a resource-generated receipt. Owner-side cleanup elsewhere restored capacity;
the same frozen `f7b3a4f4` then ran under token
`20fda23376de4e99a65a78eeac8eb417` with unchanged default cleanup and limits.

All 28 retained oracle rows authenticated. The three newly declared named/anonymous/
runtime inputs each matched pinned Rust metadata, object and native stdout `7 1`.
That proves their language hypotheses, including the true anonymous expression
which remains a candidate unsupported profile. Fresh formatting and architecture
audit passed. Strict returned 101 before any candidate tests: a test closure index
needed explicit usize, and the query work loop grew to 105 lines against Clippy's
100-line limit. Strict log SHA-256 is
`76c9bdfef263fd965933dcfbe6750209401f37d31c9f835d1dec1539896327be`.
Failed-run scoped cleanup completed with zero generated artifacts removed.

The source-only C5 correction adds the explicit test type and moves popped-scope
restoration into the coherent ScopeFrames helper. Its invocation remains the first
operation after work pop, before all fallible dispatch/bookkeeping; the helper
restores scope, records test observation and charges attempted work in that order.
No lint allowance, owner-check relaxation, new field or execution profile is added.
The next guard retains all 31 successful oracle rows and the prior three invalid
provider hypotheses without rerunning a compiler, then runs fresh static gates and
the same previously unrun 68-private/13-frontend/63-native plan. These named-owner
candidate controls are still unqualified.

## Completion dispatch recovery

C5 guard token `1eb057e17ff846aeb6eee4190b0766f3` authenticated all 31 successful
oracle rows, passed formatting and audit, then returned 101 in strict Clippy.
The only reported diagnostic was the query work loop at 101/100 lines. The index
annotation no longer produced a type error. No candidate tests or native phase
ran. Strict log SHA-256 remains
`38253a8dacc802eecc0936b77155ca4a9f4be897f721d8bdc532b7375e03cd69`.
An unnecessary outer resource admission for cleanup was refused by disk reserve;
its separate observation is retained. The documented standalone scoped storage
prune subsequently completed with zero generated files after terminal/idle and
read-only inventory checks; no reserve was lowered.

The source-only C6 correction gives the existing Forward completion its own
`finish_forward` method beside `finish_successor`. It preserves the exact old
statement order: read completed dependency, publish an Instance value when needed,
complete the query, complete referenced contents, then publish Constant/Static
values. The handler executes under the same popped Context, before enclosing
scope restoration. First-after-pop restore handling and every owner/type/root
check remain unchanged. There is no lint allowance or compressed dispatch.
The next guard retains all prior oracle/failure observations and runs the same
fresh-static and 68-private/13-frontend/63-native plan; qualification remains pending.
