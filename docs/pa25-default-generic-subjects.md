# Generic subjects in const defaults

Clean candidate32 stopped in original `RangeBounds::contains`,
ops/range.rs:872/file94:27710. Its default compares references to generic T/U
under actual conditional PartialOrd bounds. The readonly validator restricted
requests to defining Self, then lacked the source-request path for reference
implementations derived from those bounds.

A direct parameter request now consumes its exact current environment premise
unless both the subject and complete canonical application match the defining
Self fast path. Matching the trait application alone cannot authorize a foreign
subject. The one local Self preparation scan remains shared by every call.

An actual original const default can also consume the existing normalized source
request validator after its local Self context has been prepared. The context is
bound to the real defining body; parameter subjects still pay exact premises.
Composite requests retain provider/report, source owner, current typed operation,
full callable/signature/result, selected method/UFCS and caller-effect checks.
This authenticates defining legality only. Original default bodies are not marked
executable; actual implementation dispatch, const entry, borrow/lifetime proof,
paired materialization and Code remain independently required. Unsafe/projected
abstract requests keep their prior path. No new cache, persistent AST/body/value
field, wire, backend or storage-policy change.

[PA25 ordinary minima](../pa25/tests/spec/default-generic-subjects/manifest.json)
and [PA31 actual source counterparts](../pa31/tests/spec/default-generic-subjects/manifest.json)
freeze17 outcomes each:11 accept/6 reject. Pinned Rust1.98 independently agrees.
The source packet qualifies16 complete outcomes (10 accept/6 reject), including
the exact original contains item, complete source reference impls, direct and
inherited bounds, the same application on another subject, wrong/missing/ordinary
bounds and non-const reference/concrete implementations. Fourteen ordinary
outcomes qualify (8 accept/6 reject). The three importing core::ops::Bound remain
required/unqualified at bootstrap; the source CTFE range consumer remains a
required positive at `source const payment currently requires a closed callable
signature`. Its runtime-only counterpart qualifies. Nothing is inverted to a
negative contract.

Raw evidence under `obj/parallel/default-generic-subjects/`:

- `declaration.json`/`baseline/` bind initial12 predictions and immutable clean32;
  `types/` records the actual reference-valued callee applications. Additional
  opposing/native predictions and hashes precede execution. Original contains,
  reference impls and the earlier comparison foundation have separate hashes.
- fast5,1794686954186365728, passes three exact tests in91.85s, with strict
  selected/production/CLI Clippy, full formatting and architecture checks.
  Post-lowering inference/canonical work at8/32/128 is2890/1611,11506/5287,
 45970/19943. No timing/speed claim. Two new private scope/body and foreign-subject
  guards await full CI; no cold private harness.
- native1794687165779066549 passes five real programs in62.73s: three earlier
  default controls, generic-subject CTFE/runtime `41 59`, and unchanged contains
  runtime `1 0 1 0`. Independent Rust executes those new observations and the
  still-required CTFE contains `1 0 1 0` in `reference-native/`.
- `replay/` authenticates3964 compiler sources, all packet/basis hashes,30 complete
  CLI outcomes and the four explicitly unqualified cases, then replays lowering/
  scaling. All315 harness/495 roots/13 binaries/44 stages/one component,
 128 upstream and126 Reference checks pass.

Fast1 confirms that direct premises alone leave reference requests blocked.
Fast2 caught the source validator's function-size limit; selected-target checks
now form their own authentication helper. Fast3 exposed a missing fixture const
stability annotation, corrected to match its real trait and explicitly request
const_range. Fast4 then reached the separate closed-signature CTFE payment gate;
that accepting input is retained and runtime execution is qualified separately.
Initial input extraction selected an earlier inherent contains item; it was
corrected to the actual RangeBounds method before any declaration or execution.

Full43627935665 on ca20dc143 passed3081 private release tests (6.56s), static and
other regular/course checks, then failed one closure test's old gap diagnostic.
Four already-accepting requirements now clear the prior Fn check and reach the
same existing closure-owner fence as two other positives. The test records that
actual fence; all six remain required accepting outcomes. The exact13-case
closure control passes locally. No reference bytes or independent expectations
changed. The new full successor remains pending.

Next freeze one clean release and retry original core with unchanged inputs,
profile, deadline and quotas. Original v13/v14 metadata/LowIR/native consumers,
all three callable-signature/implied-borrow native requirements, closure owners,
ordinary bootstrap, futures and every earlier valid unsupported contract remain
open. PA25–PA44 remain planned; the113-case const-machine component is unchanged.

Clean release33 verifies RangeBounds::contains cleared in the genuine unchanged
original v14 library command. It now rejects residual_into_try_type in
ops/try_trait.rs:383/file95:15668 with the separate projected conditional-premise
fence. Source236732077/3964 clean hashes, CLIc638c2da…004c1af,25.41980179119855s,
status1/no timeout/no core.rgm/0-byte core.lowir. Postcheck33 passes722 originals/
375 opens/six reviewed pins/fixed120s and quotas. No original artifact or timing
claim. Full43636803381 remains pending; do not restart for this documentation.
