# Documentation and optional provenance requests

The [source packet](../pa40/tests/spec/advisory-requests/README.md) recognizes
intra_doc_pointers and strict_provenance_lints. Pinned Rust uses the former for
Rustdoc link resolution and the latter for an optional default-Allow cast lint.
Neither changes program pointer operations, unsafe requirements, primitive type
permissions, ABI, target configuration or original source selection.

Both requests use the existing no-capability classification. No feature bit,
source binding context, dense fact table or ordinary record is added. The existing
lint-policy parser admits standard levels under an authenticated Allow cap and
retains unsupported expectations as deferred debt. Strict explicit provenance
producers stop honestly; required warn/expect successes remain unfinished positive
cases. Rustdoc output, full producer events/rendering/fulfillment, metadata
transport, whole PA40 and original core/alloc/std artifacts remain required.

Qualified compiler/test snapshot:

- Private4, strict Clippy/format/architecture:211.29s. All72 source edition outcomes,
  current strict boundaries, unchanged8-byte feature word/binding mode, prior
  inactive diagnostic handling and documentation gate controls pass. Receipt
  `obj/qualification/fast/1793769124248291659/receipt.json`, SHA-256
  `d75474b4bf55ba2ebb0156016b62861380433f0037ac40313eadc8322cc2e15f`.
- Production10, strict Clippy/format/architecture and PA8 126/126:105.79s. Receipt
  `obj/qualification/fast/1793769371241354540/receipt.json`, SHA-256
  `6e0c28f45d497fc930c5a2b992cec4c7590764f04d5f82c7761a4fc4fb7a243d`.
- Independent Rust72 outcomes and native7 9:3.63s. Receipt
  `obj/parallel/advisory-requests-v1/reference-v2/receipt.json`, SHA-256
  `a8d14320b7f752a34c48945b324440d175030640189d1de2aed3f20b43b1a91e`.
  Version1's metadata predictions passed; its checked native object needed the
  Rust panic runtime. Version2 explicitly matches the existing native profile,
  panic=abort/debug_assertions=false, without changing fixture bytes or predictions.
- Joined qualification:40.12s. All72 CLI outcomes, source-provider compilation,
  strict boundary messages, predecessor request failures, actual pointer round-trip
  and ordinary native7 9, upstream hashes and harness checks pass. Annotated/control
  LowIR is identical; ordinary bytes match the predecessor. Receipt
  `obj/parallel/advisory-requests-v1/qualification-v1/receipt.json`, SHA-256
  `25b1486049d718e43883cfe1d94bb9a368ce340438aac979bbcb391096d4fb71`.

Immutable CLI SHA-256
`871c6abdf10323112b1f301dce02d5f8caef7ac1293904dca95e497562553f64`
clears LIB-135's request and reaches LIB-136, link_cfg at original lib.rs138,
in20.69s/375 opened sources. All722 original hashes, six configuration-source
pins, options, explicit Allow cap and finite limits remain unchanged. Receipt
`obj/parallel/advisory-requests-v1/qualification-v1/core/result.json`, SHA-256
`fde4fc85ad37bbf945954eab3ff231e92f96772bd4bf393a37b1111ec7ddedfa`.
A bounded immutable-binary debugger observation identifies the exact request after
checking the saved disassembly. Receipt
`obj/parallel/advisory-requests-v1/feature-probe-v1/receipt.json`, SHA-256
`98f34f9fbc3bb993ed0292933fc027d90931900dcd7c16ea3e2f8afe9a8a7113`.
This is not completed later-phase inventory work or an original crate artifact.

Next implement the actual link_cfg directive contract, including its independent
source gate, predicate validation and library selection. Pinned link_attrs.rs
parse_link_cfg explicitly checks the crate feature and reuses cfg syntax. Keep
link_llvm_intrinsics separate: its declarations and lowering require actual LLVM
intrinsic proof. All earlier Drop/Pin, ABI/SIMD/intrinsic execution, source macro
publication, imported nominal/value/body-policy transport, full source checking
and original linked consumers remain required. Root stays alone in the existing
warm btrfs tree under five-minute/two-thread/16GiB/150GiB guards with scoped cleanup.

Ordinary4096-function dev-fast ABBA retains18 observations with immutable binaries,
frozen equal-output input and A/A calibration. A/A ratios0.902/1.021 and paired
ratios0.972/1.077/1.089 do not establish a reliable timing effect or speedup.
No allocation, ordinary binding mode or feature-word growth is introduced.
Measurement receipt `obj/parallel/advisory-requests-v1/measurement-v1/receipt.json`,
SHA-256 `850411e66395f667d609069f18ba9d7dfdadf8beaa518aedd09a38941aad1260`.

CI cadence: full predecessor41776106178 was canceled during independent Rust
qualification after release/course, native and debug checks completed; no full
predecessor pass is inferred. Current manual full run41779219834 on54ae77027
must reach a terminal result before the next manual full dispatch. Focused source
checkpoints and ordinary fast CI can continue without replacing it.
