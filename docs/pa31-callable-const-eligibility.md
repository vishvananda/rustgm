# Actual source callable const eligibility

The reached-payment predecessor33b accepted two invalid source uses: an unused
`const FnOnce<()>` obligation on `fn()->u8`, and an actual non-const function item
passed to a helper requiring `const FnOnce<()>`. Both passed complete lowering,
so an interpreter-only check could not repair the contract.

This checkpoint separates callable signature equations from const eligibility.
The pinned implementation is Rust1.98 revision
88d9e12ae178fab0fb5cc050a94da85685d449ea,
`compiler/rustc_trait_selection/src/traits/effects.rs:541–584`: function pointers
have no const-callable implementation; function items require actual constness
and their instantiated const conditions. The original consumer remains
`library/core/src/intrinsics/mod.rs:2407–2416` (`const_eval_select`) and its
`ub_checks.rs` callers. No upstream-source acceptance is claimed here.

## Typed boundary

`Matched::permitted` now returns stack-only `Eligibility`: Allowed, Rejected,
Pending, Unpaid(full Instance), or an explicit unsupported boundary. Ordinary
applications take the existing fast path without declaration reconstruction.
Nonordinary applications require the exact validated source Fn/FnMut/FnOnce
identity. Function pointers reject; actual function items require a completed
real FunctionHeader, const declaration, prepared effect index and exact public
signature reconstructed from the complete arguments. Missing demanded generic
callable templates remain pending. No spelling supplies permission.

The reconstruction uses the existing typed dispatch instantiator. New work is
charged, including failed folds. `Catalog::prepared_call_effect` exposes only
completed declaration data; it grants no execution or caller permission. The
ordinary dispatch consumer retains its uncharged interface. No cached result,
new owner field or global permission cache is introduced.

Nonordinary reference subjects go to actual source implementation selection
before signature matching/ordinary forwarding can defer them or erase effects.
Output equations still require the complete original trait application. A
conditional function item, abstract const-trait member, const closure, imported
const item or actual source intrinsic cannot become a positive callable answer
through this first profile. Unsupported debts remain explicit errors/pending
boundaries. The predecessor's per-reached one-use payment and C's CTFE cleanup
denial remain intact.

## Qualification and retained failures

Evidence root (absolute prefix `/home/vishvananda/rustgm-pa26-work/`):
`obj/parallel/pa31/callable-const-eligibility-v1`.
The checked-in `pa31/tests/spec/source-const-payment/callable/contract.json` and
`work/contract.json` bind exact source hashes, modes and independent receipts.
They are planned PA31 development contracts, without activation or text oracles.

- `opponents-v4` retains both wrong accepts before the correction; its independent
  `independent-v3` observations settle the four corrected source hypotheses and
  two native7 positives. Earlier falsified fixture hypotheses remain separate.
- `development-v1` passed format/audit/strict and complete-lowering opponents;
  its single private setup lacked a demanded generic callable template.
- `development-v2` passed format/audit/strict, complete-lowering opponents and
  six private cases; its new work collector counted an empty-argument generic
  declaration as a ninth full Instance. Exact declared arity fixes that setup.
- `recovery-v1` stopped at a test-only concrete-default Clippy spelling.
- `recovery-v2` passed format/audit/strict and four exact private cases, including
  raw work/layout output and missing-template/pending opposition. The unchanged
  earlier cold/warm, effect-index readiness, forged signature, original actual
  invocation negative and one-use suspension tests retain their passing receipts.
- `scaling-independent-v1` independently compiled all14 frozen inputs to objects
  with pinned Rust1.98, edition2024, x86_64 and panic abort, and linked/executed
  all14 native answer7 observers. Hashes agree before/after; mismatches is empty.

Candidate all14 complete lowering uses `build_roots_prepared`, ordinary source
original/instance/ownership gates, and the canonical exported answer root. It is
separate from candidate native execution; the integrator owns candidate-native
preflight and immutable combined release/performance. This worker ran no broad
release suite or new wall-time benchmark.

## Bounded work and layouts

Raw output is `recovery-v2/focused/lib.stdout`. The full-source metric printed
as `complete_lowering_work` is specifically the existing
`facts.body_work.canonical_steps`; it is not total compiler instructions.
Reconstruction work separately measures the newly charged typed reconstruction.

| Axis | Width/calls | Body canonical steps | Reconstruction work |
| --- | --- | --- | --- |
| Same exact item | 8/32/128 | 65/209/785 | 16/64/256 |
| Distinct const-generic full Instances | 8/32/128 | 100/364/1420 | 40/160/640 |
| Shared signature depth | 8/32/128, eight calls | 1187/4739/18947 | 96/96/96 |
| Shared signature depth32 | 32/128 calls | 4883/5459 | 384/1536 |

Each reconstruction retains zero new types. Ordinary pointer8/32/128 adds zero
eligibility work and zero types even after declaration lookup is removed.
Matched stays20 bytes, selection Frame1312 and Mechanical80; Eligibility is a
24-byte stack result. Body/Call/Code/Value/Engine/Machine/State are unchanged by
this successor. No threshold or parser/proof budget changed.

Final recovery results SHA256:
20fbe2d5402fefb42a4148333cd350bf545d88a751e32787db6a6eb6a786cd37.
Raw work output SHA256:
55eba0e82367c48b4d2821972b41c00ad7bacdd693665e7b651643e9e13b7081.
Independent provenance SHA256:
4919bbd8173e64800a54d3131ed454bf9b664d202865179e55369384b809dc65.

## Required continuation

The unchanged original `const-callable-item.rs` (native29), SHA256
781192de892cac77522d5e1f70041d18ac34ff30a6103a96c0f45841c0fe519c,
stays required. The next bounded step should execute its actual function-item
FnOnce invocation with the existing selected-call/Rust-call adapter, rather than
add a library API factory. It removes the reached-payment profile's no-value-
arguments restriction for this exact closed function item plus u8 argument.
The callee's Conditional FnOnce clause must use this actual const eligibility,
and each reached use must retain its full Instance, immutable proved contract,
caller environment and consumed per-use continuation across cold/warm Code reuse.
Rich conditional child-proof transport, closures/borrowed callables, actual
const_eval_select callback execution, full Pin/Deref and CTFE source cleanup
remain separate required contracts.
