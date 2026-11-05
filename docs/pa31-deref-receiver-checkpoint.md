# Actual source dereference and receiver checkpoint

CLI0cc88c10 implements local source Deref/Target contracts, actual dereference
method calls and genuine smart-pointer receiver proof. It qualifies31 complete
Deref outcomes and12 receiver outcomes against pinned Rust, plus two exact
native observers. This removes the demonstrated LIB-155 source prerequisite.
Wider demanded profiles, const, metadata and whole parent requirements remain
required. No whole PA after24 or original core/alloc/std artifact is complete.

## Implementation

Actual source role completion preserves canonical trait/member/owner/schema
identity. Unused alternate declarations stay Rust-valid. A demanded dereference
checks the real safe Rust method, exact borrowed Self and Target/lifetime profile.
Selection uses ordinary trait obligations, actual method instantiation, existing
late-binder opening and receiver loans. Mutable place consumers refine the same
selected-call record to DerefMut; lowering evaluates the real receiver once and
uses the returned reference as the place. No source method is substituted.

Receiver declaration validation waits for actual impl headers. A bounded,
cycle-checked chain requires real LegacyReceiver and Deref evidence, normalizes
the actual associated Target and retains lifetime debts in the existing proof.
Full bidirectional Self constraints check final generic arguments and regions;
matching an item or recognizing Pin's spelling supplies no permission. The
ordinary wrapper, nested-reference, named-lifetime and wrong-Self controls make
that distinction observable. There is no second receiver/role registry.

Qualification exposed a shared substitution error: a selected impl's anonymous
lifetime captures prefix its named arguments, but the non-GAT shortcut indexed
them using named parameter slots. Captured owners now use the existing cached
identity-based family mapping. The positional shortcut remains for uncaptured
owners. This fixes actual `impl<T> Deref for Pin<&T>` normalization without
rewriting source, losing binder obligations or adding a parallel substitution
engine. Ordinary source bodies, Copy/Clone, borrow/move and field drops remain
subject to their original checks.

## Verification

`make fastcheck FAST_ARGS='deref_contracts smart_receivers pinning_contracts --pa pa19'`
passes in95.26s: six selected public tests, all31/12/25 source outcomes, both new
native LowIR inputs,1193 PA19 text cases, formatting, architecture and strict
selected Clippy. Failed intermediate gates are retained: unopened late binders,
a seven-root filename audit, module formatting, mutable-place publication and
finally anonymous-capture normalization. The final gate preserves all expected
outcomes; no rejected positive was removed.

The exact original native sources compile both with pinned Rust to objects and
with RustGM to LowIR and the immutable locked CPPGM backend to objects. The same
C observers confirm shared/unique method effects, one-time place evaluation and
Pin receiver read/write order. The31-case independent packet and12-case receiver
packet were frozen and validated before candidate outcomes. Ordinary/value
LowIR and all15 configuration values are byte-identical to published CLI4003bfd2.
Upstream/harness checks pass. No C++ compiler checkout was modified or rebuilt.

The unchanged722-input original-core manifest, six pins, options, capAllow and
finite limits still reach AsyncDrop, file246 byte1138, after19.6120s/375 source
opens. This is admission/frontier evidence, not722 compiled files or an artifact.
Actual Future/Output/async signatures and genuine state/destruction/cancellation
consumers remain required; the written async-drop helper placeholder is not its
compiler implementation. The two isolated workers continue async signatures and
PA29 fresh source transport under the two-job guarded policy.

## Performance and evidence

Exclusive immutable A/B ordinary1024/4096-function blocks retain A/A calibration,
paired ABBA and byte-equal outputs. Candidate/baseline ratios are1.027/0.955,
with A/A ratios0.921/0.960. These small noisy development blocks establish no
general speedup. Frozen, independently accepted changed-path inputs with8/32/128
distinct source receiver consumers take median0.100/0.267/0.901s. The predecessor
rejects the new source role, so those are candidate-only scaling observations,
not a claimed A/B improvement or proof of global complexity.

Receipts remain under the warm btrfs root:

- `obj/qualification/fast/1793838198480667859/receipt.json`:
  038d2118e1c48fe3fa5c0dfe8948a56f9fb0652348701dff80ed0773342f0674.
- `obj/parallel/deref-source-v1/reference-v1/receipt.json`:
  849d83e3db28ce6d1ab31d425d8105ce1ac677c64d21a0e14c62a148ed6416f2.
- `obj/parallel/deref-source-v1/receiver-reference-v1/receipt.json`:
  e0f87effe25db79b2d7a014cfb3b779c9ab1ab41bd585c1baefe45a752cde3b3.
- `obj/parallel/deref-source-v1/qualification-v1/receipt.json`:
  06cf0fc165870dd7869990418f94d4af73b2235605f3f5508d9e17837d22b49f.
- `obj/parallel/deref-source-v1/ordinary-scaling-v1/receipt.json`:
  5a4ec3bf88ed780bd28db588a5e39d066b55d0b946a00f275d9b2b97b479ff28.
- `obj/parallel/deref-source-v1/measurement-v1/receipt.json`:
  dff9d3e57adfc0294c3c6e4fffd4c91f6c0e4d174a3631968eeacb20db44b21b.

Full CI41929736419 is green for the preceding FnPtr checkpoint. Pin full
41936708120 failed one stale diagnostic assertion (2702 private tests and all
integration suites passed); published4af6e5f10 repairs that exact expectation.
Its successor41939550904 is pending. These earlier runs do not qualify new
Deref/receiver private execution or a full successor; that remains CI work.
