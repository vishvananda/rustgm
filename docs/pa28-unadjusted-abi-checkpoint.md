# Distinct unadjusted ABI declarations and scalar calls

The frontend now recognizes the genuine abi_unadjusted feature and retains a
distinct ABI through declarations, type interning, generics and reified pointers.
The [source packet](../pa31/tests/spec/unadjusted-request/README.md) binds thirty
independent source outcomes in each pinned edition and actual scalar execution.
Source permissions do not enable another ABI, float type, const API or unsafe
operation. Existing wire profiles reject the new convention explicitly.

The feature appends bit58 to the existing u64 word. Abi remains1 byte,
Convention2, syntax Abi8 and Node36. The ordinary packed feature/source-provider
checks remain unchanged. A header-column pass shares one paid macro allowance
session; it neither walks function bodies nor repeatedly validates each growing
header. Root crate permission skips that pass. Forwarded literal ABI syntax uses
its parsed payload plus a transparent constructor, with pretty printing preserving
valid Rust syntax. Forwarded whole items retain caller authority. Wider opaque
constructor grammar and the whole PA8/31 releases remain separate requirements.

The fixed scalar physical path uses the existing backend signatures after keeping
source convention identity distinct. Aggregate/SIMD forms stop before accidental
C classification or hidden Rust-result lowering. An independent three-word
aggregate shows actual different LLVM ABI signatures: `raw(%Triple)` returns
Triple directly, while C uses sret/byval. Its Rust native C wrapper returns17;
RustGM's required positive lowering remains unfinished. Unadjusted is non-unwinding
in the pinned fn_can_unwind policy; source nounwind checks retain that separate
rule. No backend vocabulary, code generator or metadata wire format changed.

Qualified compiler/test snapshot:

- Private7, strict Clippy, format and architecture audit:128.13s. All60 source
  outcomes, source/function-pointer ABI distinctions, unchanged layouts, source
  incarnation/captured-caller controls and shared allowance work pass. Widths
  8/32/128 and1/32 declarations retain bounded incremental query work. Receipt
  `obj/qualification/fast/1793761452105521044/receipt.json`, SHA-256
  `5bb4a49618624d74508cdafec916fa355a3562fe2ac00cc2ac57df6d4119ea11`.
- Production foreign-convention/declaration/variadic6, PA10 81/81 and PA8
  126/126, strict Clippy/format/audit:155.49s. Receipt
  `obj/qualification/fast/1793761656196318373/receipt.json`, SHA-256
  `04c23652be45b051eb5f07a2ad7b821b2035579ce3329a219980006f8b5f7bd4`.
- Independent Rust60 plus two native observers:1.89s. Receipt
  `obj/parallel/unadjusted-request-v1/reference-v3/receipt.json`, SHA-256
  `bc457633210feddaddc70fab53b658737b2b3f4de15d6f0f523d57909ab48795`.
- Joined60 production CLI outcomes, real predecessor feature rejection,
  ordinary/unused-request native7, identical ordinary LowIR against the
  predecessor, actual scalar/reified/generic execution, metadata fence,
  upstream/harness checks and original core:33.72s. Receipt
  `obj/parallel/unadjusted-request-v1/qualification-v1/receipt.json`, SHA-256
  `3249ee7eb2afa31faa54e3575f191b4e1a444e6383f63308d2090da1e000cf7c`.
  Scalar source output is
  `255 18446744073709551615 80000000000000000000000000000001 1.50 2.25 1 37 41`.
- Independent aggregate types/LLVM signatures/native17 and current explicit
  lowering boundary:0.51s. `aggregate-v1/receipt.json` in the same scratch root,
  SHA-256 `05bfbb7ca58acefb2e18c075f6dbd70387611c94f64697ee6d8feab293aa19ab`.
- Immutable A/B ordinary4096-function control, A/A calibration, three paired
  ABBA blocks and every output verified:18 retained observations,17.66s.
  Median B/A1.043; A/A ratios1.100/1.080 and paired ratios0.956/1.152/1.043
  show greater scatter. No reliable timing change is established. Receipt
  `measurement-v1/receipt.json`, SHA-256
  `bd7aeef661cabb26ed3678cb663afe87242481075a88e8b85b4a4d0bad247708`.

Immutable CLI:
`obj/parallel/unadjusted-request-v1/cli/c5bc7790eea288fd60d3210ef0d83a28005faef743c3fe95b27fc9411eb123b2/rustgm`.

Original core clears LIB-132's request boundary and reaches LIB-133:
diagnostic_on_const at original `library/core/src/lib.rs:122`. The ordinary
diagnostic is still unlocated; the bounded immutable-binary probe identifies its
exact feature. Observation18.21s/375 opens, with all722 original hashes, all six
configuration-source pins, target/options, capAllow and finite limits unchanged.
Core result SHA-256
`14193c4d4cb1f4bb2f349a2c1e0cee6edf26707ad922cb08a1de26677734f0a4`.
Probe18.57s, `feature-probe-v1/receipt.json` in the same scratch root, SHA-256
`daf72f066fbd8d5f5705ac0779923780062e895f445e1ca43a95a23784b2c39c`.
Original disassembly, stack and inventory remain alongside the receipts.

The next work belongs to PA40 diagnostic forms and PA31 source feature authority.
Inventory the actual original const-diagnostic attributes and their independent
forms; preserve const-trait/API proofs. Under the existing explicit bring-up cap,
defer nonessential diagnostic rendering rather than adding advisory lint producers
ahead of compilation. Full Drop/Pin, later inventory costs, aggregate/vector ABI,
LLVM intrinsic consumers, imported nominal/value/body-policy transport and original
core/alloc/std artifacts remain required. No whole PA is released.

Final source/fixture binding, upstream and harness review passes16.55s:
`obj/parallel/unadjusted-request-v1/final-review-v1/receipt.json`, SHA-256
`f8b0fa82c4999aa47e11d27d50248d1728f36595a0796e4864db9b906374067d`.
Full predecessor CI41759874254 onbf7e1dd3 passes every gate. Successor full CI
remains separate from these focused results. Failed setup and private/Clippy runs
remain recorded; no required ABI outcome was inverted to make the candidate pass.
