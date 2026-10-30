# Source optimization and advisory declarations

## Current local legacy and method-advisory checkpoint

Source `a9cc9cf53` qualifies actual local legacy const-generic headers and
`rustc_should_not_be_called_on_const_items` method declarations. Legacy indices
join real owned const formals and the published function signature; modern local
calls retain their normal syntax. External legacy indices/call rewriting and
metadata transport remain required. The advisory accepts inherent and trait-impl
methods, including associated functions, and changes no const, unsafe, mutation,
ownership or call permission. Broader advisory lint production remains deferred.

Pinned Rust gates unstable builtin attributes on the full retained `#` through
`]` envelope. The existing paid macro session now uses that span for these
consumers: a generated `#[$meta]` can have macro authority while its captured name
retains caller provenance. A forwarded whole item and a nonallowing macro remain
rejections. No general origin mutation, new provenance cache or captured-name
privilege is introduced.

- Exact40 private controls pass124.05s, including strict Clippy/link92.69s,
  formatting and architecture. Receipt B
  `obj/qualification/fast/1793356451939646414/receipt.json`, SHA
  `1e8ad0983bb5f1d8a463db33f6a00a9023e89b5911cbd7b4c81d4a54b4366fb1`.
- Production CLI/lexer10 and all2605 affected text cases pass: PA3 90/90,
  PA4 200/200, PA17 205/205, PA18 502/502, PA19 1193/1193, PA23 415/415.
  The initial production gate takes134.74s; the final matching-source gate
  takes47.14s with cached Clippy/build and identical actual CLI bytes. Final
  receipt B `obj/qualification/fast/1793356590580700587/receipt.json`, SHA
  `4d191f23005717d9ee891d4397b31de66693cae07893fa310a29cdcd55cba432`.
- All3258 source/tool fingerprints match. Fresh candidate LowIR through locked
  CPPGM and fresh pinned Rust objects match all seven native results:
  legacy runtime/const/unsafe/meta7, advisory mutation9/const7/unsafe7.
  Advisory programs use the existing local library-root API; the CLI's public
  nominal/member metadata restriction remains an explicit separate gap. Receipt B
  `obj/parallel/legacy-advisory-join-v1/native-v2/receipt.json`, SHA
  `932510ef8f8ed123277e3441f46f4ecd70ade09c22a37e3e56e1857608ba0cad`.
  Successful cleanup removes28 generated objects/executables.
- Original independent Legacy91/91 and advisory76/76 outcomes are preserved;
  two additional Legacy macro-envelope cases match4/4. Header80/Node36/Body456
  are unchanged. Legacy width8/32/128 work216/864/3456 and bytes12320/49184/196640
  are linear; cold Marker136/Promises232. Advisory work328/1026/3794 and
  bytes2080/9120/36320 are linear; transient Draft32/Entry24/key12.
- Full harness/upstream passes15.55s. The final native/core guarded batch takes
  43.74s including cleanup. Every development gate stays under300s, with two
  threads,16GiB absolute output caps and150GiB free reserve.

Current immutable CLI
`f154c53ef2cb40e45c1a859a71db8fc0dcff3cd4fe0f19c0fb975e8c7db985a4`
recognizes the previous LIB109 marker. The unchanged-core types probe now stops
at `file3:0: source lint policy work budget exceeded` in21.07s, with375 loading
opens and all722 original hashes unchanged. Source-contract limits stay at
14500624 work/1GiB attempted/64MiB retained; the separate lint-policy limit stays
10M work/64MiB. This is a LIB046 work-budget recurrence requiring attribution,
not evidence of full semantic completion or an output-equal speedup. Receipt B
`obj/parallel/legacy-advisory-join-v1/core-v2/receipt.json`, SHA
`350dc7d15488dd477002d3f67e18e55e56dcfc2abfac087aeefb2d01ac73d15b`.
No core/alloc/std artifact or additional whole PA is complete.

The previous public5beb full
[CI41038124565](https://github.com/vishvananda/rustgm/actions/runs/41038124565)
is terminal SUCCESS. It does not qualify this new compiler's full release gates.

## Earlier optimization and drop declaration checkpoint

Source f91fc9b24 implements the actual `rustc_no_writable` and
`rustc_insignificant_dtor` declaration contracts without changing independent
language permissions. That unchanged-core loading/expansion observation
clears these markers and reaches LIB-109 at `cell/lazy.rs:137`:
`rustc_should_not_be_called_on_const_items`. No whole assignment or original
library artifact is complete.

NoWritable shares the existing transient MIR-boundary draft. A typed restriction
key preserves independent duplicate checks for the two markers. Actual configured
owners, source images, bodies/closures, features and named definitions are checked
once through the existing declaration walks. Ordinary empty drafts return early.
The present RustGM/CPPGM backend emits neither MIR inlining nor LLVM Writable
assumptions. A future optimizer must first authenticate imported intent or remain
conservative; no current absence query or new wire authority is introduced.

Closure annotations also need their actual `stmt_expr_attributes` request or
macro allowance. The existing root feature owner now retains this already parsed
bit for ordinary crates. Provider/client identity and macro provenance checks
remain shared; this does not grant compiler language items. A genuine imported
u8 function keeps its actual u8 client contract.

InsignificantDtor validates word form, rustc_attrs/actual macro allowance,
struct/enum/foreign-type target and hard duplicates. Per-owner state lasts only
for the existing configured attribute walk. No ordinary nominal permission rows
are retained. The annotation affects advisory significant-drop analyses, not
actual destruction, move/borrow legality or capture semantics. Broader advisory
lint producers stay deferred under the compilation-first plan; metadata does not
claim their checked absence. Real own and field destruction still execute.

## Qualification

The unchanged development policy is300s including admission, two threads,
16GiB absolute output cap and150GiB free reserve. No local release or C-tree
compiler build ran.

- Current exact18 passes89.62s, including strict private Clippy/link57.37s,
  full formatting and architecture. The selection covers seven new Writable,
  six existing NoMir, two insignificant-drop, two Has and the coverage control.
  B receipt `obj/qualification/fast/1793353410607545172/receipt.json`, SHA
  `aa79b9ec2e4fe39d048ab32d351f9135d2b0b79f7d357f9ce40cb27c8b3de150`.
- Production Clippy, CLI and lexer10 pass202.07s; affected text suites pass
  PA3 90/90, PA4 200/200, PA17 205/205, PA19 1193/1193 and PA23 415/415.
  B receipt `obj/qualification/fast/1793353545695013853/receipt.json`, SHA
  `db21168a8c87f6ee17990d57baaecf22db74ef8e961427771c9124d6a2ba9c35`.
- Independently frozen Rust41 Writable and31 insignificant-drop inputs retain
  every metadata/object outcome. Fresh candidate LowIR→locked CPPGM→C observers
  and fresh Rust objects→the same observers match7 for mutation and37 for actual
  own/field destruction. Existing Has7/7,9 native evidence is reused only after
  freshly emitted source/LowIR, observer and tool bytes match exactly.
  B native receipt `obj/parallel/advisory-source-checkpoint-v1/native-v1/receipt.json`,
  SHA `349284bad25b6b7c483c85536e4bdc970d631165aea15ea1079d2e7e102fd6fc`.
  It retains negative diagnostics and raw scaling output. Successful scoped
  cleanup removes all eight generated objects/executables.
- Restriction width8/32/128 pays313/1077/4109 work and4480/18240/72320 bytes.
  Draft32/Entry24/key12 are transient cold storage. Existing Node36/Body456
  widths and ordinary no-permission-row checks pass. These are bounded operation
  counts, not an output-equal wall-time performance claim.
- Full harness and pinned upstream checks pass19.96s after the fixture and
  contract updates, without a compiler rebuild. B receipt
  `obj/parallel/advisory-source-checkpoint-v1/checks-v1/receipt.json` retains
  the actual command, output hashes and terminal status.

All failed intermediate observations remain retained. Real checking exposed and
fixed the missing ordinary expression-attribute feature and a draft imported
client's incorrect u32 return type. No frozen fixture, Rust-positive outcome,
unsafe/const/body permission or failed payment was waived. Initial source typing,
format/name audit and production import-style issues are also retained; current
private and production Clippy both pass without new lint allowances.

## Original core

Immutable CLI SHA
`e464a0b2948abeb6fa39472686002dc18812619748a85849dfe5e5439eee1bed`
uses the existing edition2024/x86-64 Linux/panic-abort/allow-capped profile,
14500624 work/1GiB attempted/64MiB retained limits and all722 unchanged source
hashes. It reaches exact byte4274 at first-open113, `library/core/src/cell/lazy.rs`
line137, in20.10s with375 loading opens. This is a new source-expansion frontier,
not semantic completion or a speed comparison. Strict lint debt remains separate.

B receipt `obj/parallel/advisory-source-checkpoint-v1/core-v1/receipt.json`,
SHA `caccb10e4e0162619a2fb7631923e7575c489bce0790b8f5f750ca8672e1d5c8`.
Original IntoIter, slice IndexMut and Cell body/helper graphs, all broader parent
requirements and original core/alloc/std artifacts remain required.
