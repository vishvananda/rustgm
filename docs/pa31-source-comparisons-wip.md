# LIB056 source comparison WIP

## Joined executable checkpoint, 2026-10-28

The actual comparison sources now pass analysis, checked LowIR generation and
native execution through the frozen CPPGM backend. Both the full actual profile
and the authored-override profile match their independent Rust traces. The
closed E/N/G member constants and actual Ordering tag O also execute as `-1 1`
under both independent Rust and the candidate's CPPGM-backed consumer.

Root private receipt1793195049033919221 passes all18 selected controls and strict
checks in38.47s on the warm unchanged source. Its preceding cold check passed
strict checks/build but exhausted the five-minute window before execution; it
is retained as an infrastructure failure, not a semantic pass. The joined controls
include genuine Option matched-payload cleanup with no invented T:Destruct,
custom Drop/other-variant/loop/cached-boolean opponents, unchanged read/move
permissions, sparse8/32/128+noise work, and constant cleanup selection work for
untouched wide aggregates. Both corrected original const-function carrier tests,
concrete CTFE and instance denials, wrong-Self/wrong-Rhs applications, reached
argument work and payment replay controls pass.

Native receipts under `obj/parallel/source-comparison-integration-v1/`:

- `native-observed-v1/receipt.json`: SHA256
  `2f973af01454d379b6b1f88de407c4b6ca850019fa1685e7ce531b3ac90684b6`.
- `const-native-observed-v1/receipt.json`: SHA256
  `c05e5a271e548e16d21bc3fabfc57773e1562bdb96f7fed9bb87cb93603c4402`.
- Independent const observer `const-oracle-v1/observed/receipt.json`: SHA256
  `1f63de2463f998456c14daba1074603e76cd3b054bb133d8dae6424309d53adc`.

C's qualified20e9d8c1c and originally pending fast-path repair1bff494d6 are now
normally merged and jointly qualified by root. The latter restores the original
fully initialized aggregate fast path by requiring both must-initialized and
may-live interval coverage for Always; it does not widen read permission.
Production/CLI strict checks,12 integration controls and full PA6 text pass in
157.36s at1793195495531636536. A precise lowering control now rejects all three
predeclared opponents at their actual missing-impl/Ordering/Copy contracts;
its first run passed strict/build/tests but failed final16GiB resource accounting
by49MB. After idle retirement of superseded caches, the unchanged-source warm
replay1793196733699384870 passes all3 selected controls and the complete resource
guard in35.92s. All260 harness tests pass, including active-work, symlink, pin,
hardlink and pre-admission cleanup opponents. The generic `--emit-body-ir`
view stops earlier at its existing unparameterized generic-body fence and is not
evidence for those three individual comparison guards.

A separate real CLI library export attempt retains the existing rejection
`source stability contracts are outside this metadata profile` (LIB-059,
PA29/PA31). Neither this failure nor the earlier incorrect library-mode prediction
has been erased. Typed source CLI views pass; complete library export remains
required. Root adds idle preflight retirement of superseded completed incremental
sessions to avoid cache accumulation while preserving each newest warm session,
in-progress directories and explicit pins. The aggregate closing receipt is
`closing-observed-v1/receipt.json`, SHA256
`90f913c8d6577cbeeefdd0814032c71c8adb88902f7416cff8262e2c0b439af4`;
maintainer registration,44-stage fixture placement and upstream checks pass.
No new whole PA release or original core/alloc/std artifact is claimed. Generic/borrowed
callable CTFE, conditional callback payment, actual provided-default Code
execution and wider original crate/metadata closure remain required.

The earlier investigation below is historical and retains failures at their
actual source snapshots; its pending comparison/native statements are superseded
by this checkpoint only within the exact observed profile.

## Direct continuation checkpoint, 2026-10-28

This remains unfinished development work, with no native/CTFE comparison or
assignment release claim. Public main4ecc9c41b has green full CI; this branch
adds the saved comparison candidate and the following direct prerequisites.
Monomorphic inferred function-item representations agree, primitive/nonowning
const Destruct evidence shares its real formation producer, generic defining
calls consume exact actual parent/method arguments, and full callable const
substitution includes inherited parameters using the existing dispatch mapping.
Owned FnOnce checks now consume complete typed arguments/results, with one
transient formal index per body. Concrete own const overrides can satisfy a
member whose declaration has a default; actual default Code execution stays
rejected independently. The original signatures and bodies are preserved.

Receipt1793189982223550005 passed9 controls, with the source comparison exits
still failing. Receipt1793191266449361411 passed strict checks and6 controls;
new fixture setup and original generic cleanup remained failing. Diagnostic
receipt1793191749186672517 passed strict checks but all3 selected controls failed:
new consume<T> had no carrier because its callable registration is lazy, while
is_some_and had an active composite cleanup, TypeId120/Adt ItemId61, rather than
only the unused F's direct conditional Destruct claim. The test-only diagnostic
has been removed. The cold gate now uses a completed registration when available
and otherwise reads actual non-lifetime declaration parameters; that final
fallback now reaches both genuine const-function carriers. Its first two manual
controls still failed later fixture setup and remain pending correction. Never
invent T:Destruct, suppress accesses, or
classify a conditional cleanup as dropless to satisfy this consumer.

The new original const-function carrier reuses existing defining-cleanup
preparation, seals actual source/owner/signatures/environment/const-context,
and clears at specialization. Scalar and lifetime-only functions stay cold.
A tagged transient schedule distinguishes symbolic from concrete even when
empty; only a real original symbolic schedule with an authenticated prepared
carrier may consume conditional cleanup legality. Concrete CTFE destructor
execution and symbolic instance reuse retain their independent rejections.
Effective runtime const panic bodies retain ordinary cleanup after exact carrier
authentication; their Code flag still rejects CTFE entry.

At the user's renewed two-agent authorization, the second worker takes precise
symbolic cleanup in the existing pa27 worktree. Root takes the accepted reached
arity-accounting review finding; the first worker independently implements real
deprecation/expectation semantics in pa26. No new worktrees or increased budgets.
The scalar/owned-item const profile now precharges attempted work across all four
inspect phases against the configured cumulative budget. Receipt1793193560251533885
passed strict formatting/audit/private Clippy and7 of9 controls, including genuine
8/32/128-operand cold/warm, first/last malformed-value, exhausted resume and
one-use payment opponents. Its two new manual cleanup controls failed: the
ordinary-bound negative had not demanded its generic proof, and the positive
manual loan query lacked the actual parameter environment. Those setups are now
corrected but not yet rerun; no cleanup permission was weakened. The same-Self,
changed-Rhs generic-premise opponent is also added and awaiting the next check.
Independent Rust1.98 metadata/object/link/run observations passed all24 operations
for the exact8/32/128 sources in editions2021/2024, each native result7; receipt
`arity-oracle-v1/observed/receipt.json` has SHA256
`c1e0e808a178966c1427c9b79bd06c3ee53b80053d6c939084d59c473ab18eff`.
This does not qualify the still-pending comparison/native consumer.
Native observer is prepared under obj/parallel/source-comparison-integration-v1
but has not run; do not claim its producer outputs or stale artifacts as passes.

Historical stopped-worker notes below retain their evidence and narrower scope.


Root continuation: the worker is stopped and has not been resumed. Root
authenticated all45074 tracked files,400 raw comparison streams and184 fast
streams, then normally merged the saved branch. The test now selects v3 and the
exact independently qualified native-v4 inputs/bridge are committed under
`pa31/tests/spec/source-comparisons/native`. A full source/lowering producer and
an exported const-member control are added; neither currently passes.

Root receipt1793165606061771964 passed strict checks but exposed a declaration
validator reading elaborated supertrait consequences. The validator now borrows
the existing canonical header inputs. Follow-up1793165895436732855 passes strict
checks and source equality, advances ordering to offset15662, then fails
`invalid inference identity` in all three ordering controls. These are retained
failures, not qualified native/const support. No oracles were changed.

Full public CI40637336885 meanwhile exposed ordinary reference regressions and
structural test assumptions in the preceding reborrow change. Root saved this
comparison checkpoint to repair those independently on the public base first.
Resume the comparison tests after integrating that repair; preserve the genuine
source algorithms and all existing inherited/default const execution fences.

Stopped at the user's subagent shutdown request. This is an unfinished local
checkpoint, not a qualified compiler handoff. Base is a1da9ba298dde9f33961e1462d3d581a02dd5d27;
branch is pa31-source-comparisons-v1. No active C guard remains.

The compiler WIP authenticates actual PartialEq/PartialOrd declarations, uses the
existing independent operand-borrow/call proof, classifies real selected source
members without the bootstrap comparison registry, and erases primitive operators
after proof. UFCS retains actual implementation bodies. ThreeWayCompare is appended
to SourceIntrinsic only; its transient descriptor validates the own Copy/declared
sizing prefix and actual Ordering identity/variants, with existing direct-call,
CTFE enum-value and native enum-storage consumers. LangItem remains 65. The
const_precise_live_drops request is recognized without a capability grant.

The isolated regular enum variant availability change is cee808e83: actual parsed
and resolved source variants consume their existing regular policy; const-variant,
wrong-kind, detached and stale-policy opponents remain rejected. Its three private
controls passed in source5 receipt1793161590020661470 (overall gate failed only the
two comparison controls). Independent variants4 metadata commands matched. No
standalone production claim is made.

Actual equality passed private source1 in receipt1793161986028222025. Later changes
only address the intrinsic declaration. Full ordering's latest private receipt is
1793162949070653146: strict format/audit/Clippy/build pass, test fails because the
explicitly authored minimal Option support lacks existing Some/None language claims.
The v3 fixtures add those claims, preserving all pinned comparison/default/helper
bodies; the private test still points to v2. No candidate rerun was started after
that fixture correction. Original Option declaration including all attributes is
retained separately as source data; this does not claim full Option/derive parity.

Independent Rust1.98 observations are under
obj/parallel/pa31/source-comparisons-v1. Native-v4 has sixteen matching
metadata/object/link/run commands across both editions for genuine algorithms and
an explicitly authored eq/gt override control. It covers six operators and UFCS,
actual integer partial_cmp/three_way_compare, signed extrema, char, const values,
threshold comparison and exactly-once left-to-right operands. This is oracle
qualification, not candidate native qualification. Original Ordering-tag and missing
intrinsic Copy rejection predictions disagreed with Rust metadata; those profiles
remain explicit Rust-positive unsupported source contracts. Failed fixture setup,
format/audit/compile/lint receipts and authored override rewrite disagreements remain.

Root alone can resume by selecting the v3 ordering fixture in
comparison_authority.rs, then the bounded exact private command below. The v3
standalone metadata observer is frozen but has not run; native-v4 metadata already
covers its actual algorithm/support composition with additional scalar controls.

```
make fastcheck FAST_ARGS='--max-output-gib 16 --lib-case sema::compiler_items::comparison_contract::tests::actual_integer_partial_ord_retains_full_source_body_and_intrinsic'
```

After that: qualify candidate lowering/native from native-frozen-v4 inputs, add the
already independently positive const fixture through candidate CTFE, authenticate
any actual provided-member/late-signature failure before implementing a selected
own-override proof join. No body/constants/provided_calls edits were made. Preserve
inherited/default const execution fences and existing defining/forgery/runtime
controls. B's real full Count/panic/Formatter helper closure is pending; the threshold
probe does not substitute for it. B's handler Work::Enter join is disjoint from C's
new intrinsic branch in lower/instances/demand.rs. Root's ordered capture reset
change is disjoint from C's comparison classification hunk in closures/uses.rs.

Keep the existing C worktree and both dev-fast caches. C16 GiB, shared3 jobs,
150 GiB reserve, two Cargo/test threads and five-minute guarded checks remain.
No float/SIMD, metadata/wire or external backend capability was added. No references
were blessed or regenerated, no release/full inventory was run, and no root-owned
reference/capture or B formatting work was edited.
