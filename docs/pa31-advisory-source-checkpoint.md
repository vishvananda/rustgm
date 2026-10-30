# Source optimization and drop-advisory declarations

Source f91fc9b24 implements the actual `rustc_no_writable` and
`rustc_insignificant_dtor` declaration contracts without changing independent
language permissions. The unchanged-core loading/expansion observation now
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
