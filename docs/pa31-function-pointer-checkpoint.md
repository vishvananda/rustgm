# Source function-pointer proof and address checkpoint

Actual FnPtr/FnPtrAddr declarations now complete through the canonical source
header and role ledger. The required method is prepared by the existing
declaration worklist before member validation. Completion authenticates source,
parent trait, retained membership and actual callable signature. Ordinary
trait validation preserves real bounds, flags and implementation restrictions.
Reservation alone grants no trait proof, body, const or imported permission.

Builtin selection recognizes actual function-pointer types and their current
inference state, with ordinary signature formation, impl precedence and caller
obligations. Safe/unsafe and ABI distinctions survive. Function items, closures,
references and raw pointers do not gain this evidence directly; generic callers
retain their actual assumptions. No Copy requirement is invented for the
pointer's argument/result types, and an ordinary trait named FnPtr has no hook.

The selected standard address method demands a typed one-parameter body: read
the real pointer once, cast it to `*const ()`, return. It never invokes the
pointer. Higher-ranked binders remain inside the pointer type; caller-free
lifetimes abstract into an independent generated-body contract while the owner
retains its actual selected signature. Exact body/descriptor/contract validation
constructs a private permit. The shared factory permit enum preserves separate
primitive and pointer validators without widening ordinary template admission.

All factories then use existing instance memoization, recursion/storage limits,
borrow, cleanup and full original-proof checks. Source factories prepare actual
per-body Copy/cleanup requirements and charge net retained contract growth to
the existing instance budget. A factory permit does not confer physical Copy,
caller lifetime permission or a source-body exemption. Existing primitive
factories share that preparation rather than duplicating the path. There are
no dense node/body/type fields, new role ordinals, external frontend calls,
backend changes or metadata-wire relaxation.

The [packet](../pa31/tests/spec/function-pointers/README.md) independently
qualifies 38 complete Rust outcomes. Candidate tests run all through original
body proof and lowering. Its native program matches pinned Rust through the
locked CPPGM object backend: seven true address comparisons, zero callback
calls and one evaluation of the receiver producer. Generic forwarding,
reification, unsafe pointers, higher-ranked and scoped lifetimes are exercised.
Unused Rust-valid orphan/generic/borrowed/other-result declarations remain
valid; this does not claim their broader demanded shim profiles. Those runtime
profiles, const and imported execution remain required. Two private controls
cover exact permits and forged bodies, contracts, descriptors and headers;
strict all-target/all-feature Clippy compiles them, but local execution is not
claimed before full CI.

## Qualification

The final five-test public gate passes in49.54s, with full format, architecture,
strict selected Clippy and all1193 PA19 text cases. All-target strict metadata
Clippy passes in84.14s. Harness/upstream checks pass. Independent Rust confirms
38 source outcomes and the native result in1.43s. Native/ordinary-output/config/
original-core qualification passes in30.90s. Every bound compiler source hash
still matches these receipts. No cold private-harness link is repeated locally.

CLI1c5be3cac59548ec774f606e0649005259373dc24dcd32e5d8fad46be4009970
preserves all722 original input/package hashes, six source pins, fifteen
configuration values, original options, Allow cap and finite limits. Ordinary
and constant-value LowIR match predecessor8e7af054 byte-for-byte. The config
snapshot retains SHA25686c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d.

Unchanged original core clears marker.rs1136 and stops in16.3742s/375 distinct
opens at `ERROR: file 246: 1138: source compiler language item semantic contract
is not implemented`. The first-open map identifies
`library/core/src/future/async_drop.rs:29`, actual `#[lang="async_drop"]
pub trait AsyncDrop`, SHA2567a2ea6e4131f0df0c6bc6be75428b99de7b4e1b2de61b04fa1e3f562fdd0c83c.
This is LIB-154, not an infrastructure error or a core artifact. Its real async
method takes `Pin<&mut Self>`; the same original file declares compiler-replaced
`async_drop_in_place`. PA31 source identity, PA42 async signatures/futures,
PA43 pinning/destruction, PA26 hooks and PA29 transport must preserve those
contracts. The written placeholder helper body is not its implementation.

Twenty-seven raw measurements retain immutable A/A and paired ABBA with equal
ordinary output. Ordinary median B/A is0.9633. Actual distinct address-shim
signatures8/32/128 reach LowIR in candidate medians60.3/111.1/322.7ms.
The predecessor rejects the new role, so changed-path times are candidate-only.
These small observations imply no speedup, asymptotic, peak-memory or whole
library compile-time claim. Existing canonical data is borrowed and generated
bodies are demanded once per selected instance.

## Evidence and remaining work

- Public gate `obj/qualification/fast/1793831938403582610/receipt.json`:
  479fc8dfb0f1d79b295035fb1da832dd607e95f280061b693ac6842feeab35d9.
- `obj/parallel/fn-ptr-v1/private-compile-v2/receipt.json`:
  e9eaeb3dba64e796404a467bf1108829e59e1bcd1f978ea8a228c8919c0a83f9.
- `obj/parallel/fn-ptr-v1/reference-v2/receipt.json`:
  6f7e9daaf9d60f48c5da4bf6f2c5efc2a289caa8e45bbcc8599b85d41c182bd6.
- `obj/parallel/fn-ptr-v1/qualification-v1/receipt.json`:
  540829964391c935b4b89943df240d7f2338c4e9171e98a5aff41a7b0f06ac06.
- `obj/parallel/fn-ptr-v1/measurement-v1/receipt.json`:
  d0504b598726d3625bcc5e1f13dbd0fb78ea1c469e3524a1fbe2efd0a901322f.

Retained failed attempts exposed a static diagnostic-adapter mismatch, file-stem
audit collisions, oversized validation, unprepared actual method headers,
builtin proof/candidate filters, absent per-body source Copy requirements and
an older private permit argument. Each is repaired at its owning shared path.
The stronger native revision sequences counter observations and adds receiver/
lifetime checks; no positive outcome is weakened. Earlier receipts remain in
scratch, including the failed private metadata compile.

Full [CI41918191470](https://github.com/vishvananda/rustgm/actions/runs/41918191470)
onf01aa47d4 is terminal success with every gate, including private glob-payer
controls, native, debug, independent Rust and const-machine qualification.
The UseCloned push also passes. These precede this checkpoint; successor full
execution remains pending. No whole PA after24 or original core/alloc/std
artifact is complete. LIB-148 aliases, LIB-149 custom autos, LIB-152 use/capture
operations, wider pointer profiles, Field/TransmuteFrom and general transport
remain required. Root works alone in guarded warm btrfs scratch, preserving
the five-minute/two-thread/16GiB/150GiB limits and only current/predecessor pins.
