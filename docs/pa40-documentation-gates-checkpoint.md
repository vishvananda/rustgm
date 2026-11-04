# Documentation gates and inactive diagnostic metadata

The [documentation packet](../pa40/tests/spec/documentation-gates/README.md)
qualifies actual doc_cfg/doc_notable_trait requests, attribute-constructor macro
permissions and independent unstable cfg-query permissions. Documentation cannot
filter program owners or grant const, unsafe, float, trait or ABI operations.
Uncapped documentation option diagnostics/output and imported metadata remain
required. No whole PA or original core/alloc/std artifact is established.

The shared cfg grammar now belongs to syntax. Expansion supplies its real lookup
callback; documentation validation has no callback and returns only gate facts.
Both consume retained token forests and preserve exact opaque-fragment relocation
and token editions. One flat vector records all demanded gate occurrences in
capture order, including operands after an already-false condition. This removes
per-leaf allocations and repeated subtree-vector merging through nested operands.
The existing nesting bound and finite use limit remain enforced. No semantic-to-
expansion dependency exception, source reparse or new dense AST table is added.

Feature bits59/60 retain the two documentation permissions. Bits61/62 retain
whether each diagnostic handler is actually enabled; they grant no operation.
The feature word stays8 bytes and ordinary declaration/layout records stay fixed.
Pass-local caches query crate permissions once per feature, while every actual
macro occurrence keeps its own source authority in the existing shared session.

Full CI41770919155 on predecessor c152ce4a8 exposed nine original offset_of
regressions. Its new uncapped guard incorrectly rejected diagnostic metadata even
without a requested feature. The pinned handlers ignore that metadata; an internal
macro allowance does not enable them. Restore strict inactive-handler compilation,
retain the explicit enabled-handler rendering boundary and preserve hard errors.
The original fixtures and their field/privacy/intrinsic/const expectations remain
unchanged; all nine exact regressions pass in the successor snapshot. Optional
unknown-diagnostic warning parity remains required.

Qualified snapshot:

- Private21, strict Clippy, formatting and architecture audit:121.96s. This covers
  all146 documentation and118 diagnostic edition outcomes, actual demanded const
  rejection, no false-predicate pruning, flat gate occurrence order, exact opaque
  relocation, shared allowance work and all nine predecessor CI regressions.
  Receipt `obj/qualification/fast/1793767176074196716/receipt.json`, SHA-256
  `e30dfff76fb907b7d0cba1ebb1f0f7b590550fad174539bea6b486e5541deaa6`.
- Production19, strict Clippy/format/audit, PA2 142/142, PA5 231/231 and PA8
  126/126:191.34s. Receipt
  `obj/qualification/fast/1793767316377038114/receipt.json`, SHA-256
  `89d298fae09c10b584254ebd1bd77270cad09a505b2b4bbc5e744cee098f4e97`.
- Independent documentation146 and native7 9:3.76s. Receipt
  `obj/parallel/documentation-gates-v1/reference-v2/receipt.json`, SHA-256
  `86ae3daa10d39d3643d8d0e64eda18b17db888381e89b418ebb7059e3a91e070`.
  Independent diagnostic118, four strict inactive successes and native7 9:3.37s.
  Receipt `obj/parallel/diagnostic-directives-v1/reference-v4/receipt.json`, SHA-256
  `ca87f77722bd8c8da2ba44741fcf5057ce9a08d4edc242f8dde242596eaac8fb`.
- Joined qualification:37.14s. All264 candidate CLI outcomes, four strict inactive
  successes versus predecessor failures, six independent strict documentation
  successes versus explicit current boundaries, native trait/macro consumers,
  upstream hashes and harness checks pass. Annotated/control LowIR is byte-identical,
  and ordinary control bytes equal the predecessor. Receipt
  `obj/parallel/documentation-gates-v1/qualification-v1/receipt.json`, SHA-256
  `b541053adbbeab000d215435e6d4034bcd60634d1070860aeb27393536d600ea`.

Extracting shared grammar/types and the literal helper changes config.rs's source
identity, explicitly requalified from a93164ee to9147ac7c. Its fixed target/custom
values and five other configuration-source pins stay unchanged. All fifteen actual
configuration constants match the predecessor byte for byte, SHA-256
`86c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d`.
No target option, cfg override, lint cap or finite resource limit changes.

Immutable CLI SHA-256
`e6f104b119d2d242c0d42f517131abd9b4dbcd57032b8d7c9edf75e9afa5a1a7`
clears the documentation requests and reaches LIB-135, intra_doc_pointers at
original lib.rs135, in19.31s/375 opened sources. All722 original source hashes
remain unchanged. Result receipt
`obj/parallel/documentation-gates-v1/qualification-v1/core/result.json`, SHA-256
`d993127e42b3044f620eaee869488fafd46c209de8a03bedc2d8eb4bba7260c3`.
A bounded immutable-binary debugger observation, with its breakpoint checked
against saved disassembly, identifies the exact request. Receipt
`obj/parallel/documentation-gates-v1/feature-probe-v1/receipt.json`, SHA-256
`9a780779523d6b41eafa0a6b9197081608466f0293bbbe4fa3404847c3fdf171`.
Later inventory work and the original artifact gates remain unqualified.

Next inventory the actual intra_doc_pointers compiler/Rustdoc contract before
admitting its root request. Preserve all previous Drop/Pin, aggregate/SIMD ABI,
source macro publication, nominal/value/body-policy transport, full original
semantic inventory and linked original library consumers. Root stays alone in
the existing warm btrfs tree under five-minute, two-thread,16GiB/150GiB guards.

Dev-fast wall-time measurements retain all72 observations: immutable A/B binaries,
frozen equal-output inputs, A/A calibration and three paired ABBA blocks per input.
Ordinary4096-function ratios span1.011–1.068 against A/A0.906–1.062. With128
configured functions at predicate depths1/32/96, candidate medians are
0.01354/0.02913/0.06249s. Small depth1 ratios are higher,1.098–1.136; the deeper
inputs have substantial scatter. These observations establish neither a speedup
nor absence of every timing regression. Release qualification remains required
for broader performance claims. The structural change removes repeated subtree
allocation/copying while retaining exact ordered gate facts. Measurement receipt
`obj/parallel/documentation-gates-v1/measurement-v1/receipt.json`, SHA-256
`fe71e4db149776055a85622e284ed487e979000573068e12e809eb4bd00f6c30`.

The diagnostic request-state change also retains actual annotated/control native
execution7 9 and identical LowIR after introducing the metadata-enable bits.
Supplemental receipt
`obj/parallel/documentation-gates-v1/diagnostic-native-v1/receipt.json`, SHA-256
`dce86b40f79f765507ed072c126da8e0ea115807bc46dd996ed4c6bb42784993`.
