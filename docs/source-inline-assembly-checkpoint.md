# Source inline assembly and combined library checkpoint

The combined compiler at `0396139b3` expands the genuine source-declared `asm`
builtin into typed syntax and retains checked assembly effects in executable
bodies. PA41 owns this contract; its required core subset is a PA32 prerequisite,
with PA31 source binding, PA27/PA28 place/safety/memory rules and PA25/PA29
constant/body transport. No new assignment is needed and PA41 remains planned.

## Implemented boundary

Templates, placeholders, options and operand expressions use the existing syntax
arena and expansion worklist. Eager `vps!`/`concat!` inputs preserve captured
expressions and source ownership without rendering or reparsing Rust. The
original `_mm_stream_ps` function, attributes and body remain in the PA41 fixture;
that expansion control does not establish the real SIMD representation or ABI.

Scalar controls reach semantic checking, ordinary body construction and original
and instance proof. Inputs and output places evaluate left to right. Inout
places evaluate once, inputs require initialization and Copy, and output stores
follow the real effect in operand order. The executable CFG control checks actual
calls before the effect, overlapping destinations and a rightmost-last store.
`noreturn` terminates its actual block without a normal return. Unsafe and const
restrictions remain enforced. Const and symbol operands retain an explicit
unsupported boundary until actual constant and relocation producers exist.

One optional boxed payload belongs to its existing Body. Cloning, instantiation,
storage accounting, initialization/loan proof and lowering consume that same
typed data. Payload and output identities are revalidated. Runtime lowering
currently reports unavailable backend capability; it cannot erase the effect or
substitute an ordinary store. No assembly metadata codec is implemented.

Actual combined widths are Kind28, Node36, Body456, Rvalue20 and Value24 bytes.
The optional owner adds8 bytes to every Body, including ordinary bodies; those
bodies allocate no assembly payload. Payload/Instruction/Operand are104/28/28.
Widths1/8/32 retain421/632/1830 logical payload bytes. These are capacity-based
accounting observations, not allocation counts or peak-memory measurements.

## Combined qualification

Evidence is under the root-owned `rustgm-pa25-work` checkout:

- `obj/qualification/fast/1793112106393563615/receipt.json` passes all33 selected
  private controls in267.67s, including the assembly controls, inert diagnostic
  names, source receiver/Neg controls and16 operator/inference controls. Private
  strict Clippy, formatting and the global architecture audit pass. SHA-256:
  `69e9f33551d0899793c1ae15ffffa58e23a32b5404c77f82145eef8635bd7d15`.
- The only subsequent source change adds filesystem output to the already-proved
  native operator producer. Its affected test and private strict/static checks
  pass in122.83s in `1793112574646911197`; the other32 successes retain identical
  tested function and production source bytes. This is not a claim of33 reruns.
- Final production/CLI strict Clippy, formatting, global audit, CLI build and ten
  lexer controls pass in212.58s in `1793113075730875585`. SHA-256:
  `fae787ae4c206156f9bcd8ec3c2e65302d7c278317afa36866faafc10aa98d4f`.
- All225 harness tests,383 maintainer-root registrations,44-stage fixture layout
  and upstream checksum verification pass. The guarded record is
  `obj/parallel/lib045-root-implementation-v1/harness/receipt.json`.

Push CI40532433959 passes. Full CI40532474766 finds an unhandled exhaustive match
in the PA10 scalar test executor before running release/native qualification.
The followup explicitly rejects both assembly value forms in that executor;
production behavior is unchanged. All nine affected typed-body controls,
production/CLI strict Clippy, formatting and global audit pass in46.55s:
`obj/qualification/fast/1793115536499896113/receipt.json`, SHA-256
`9a59277a6ac65a8e7125ffd6ca39ace62a88615e3c6124f7c48eda40f2a085f7`.
The earlier failed full run remains a failure; subsequent full qualification is
required for this cut.

Two native operator consumers now execute exact proved source/LowIR through the
locked CPPGM object backend and their independently observed C callers. They
match pinned Rust's `7 12 13` and `7 99 42`, each with status0 and empty stderr.
The earlier Rust observations disable overflow checks; the candidate uses its
checked default and CPPGM `-O0`. This establishes safe-input output parity, not
whole-profile overflow parity. The native receipt SHA-256 is
`4da3b51f3deec8cf0db18e5fd696f1819a706e794a30f827f4fe6c88e5f3a195`.
Successful native objects and executables were hash-recorded and pruned.

## Original core and remaining work

The immutable production CLI has SHA-256
`9156793b0672a90e50a727dde2b36c84c449a5476ae458c4fc820cf4413b45d5`.
Its source binding is
`obj/parallel/lib045-root-implementation-v1/cli-binding.json` (SHA-256
`81e9a91e488bf112b0628486ff7f2c3daa63b20e7ba815aa7d84616284c5dbc4`).
The unchanged-core attempt verifies all722 pinned input hashes, opens371 files
and exits after13.32s without an artifact. It passes the previous unavailable-asm
expansion boundary, then fails with source lint policy work/storage exhaustion
in `core/src/num/saturating.rs`, inside `forward_ref_op_assign`/`saturating_impl`.
This is the new LIB-046 frontier, not an infrastructure failure. Result SHA-256:
`393278d4cbe539c869d735a7ea89377161445e951675b3525ef9dab840b6c43b`.
The file count is not a percentage of semantic or compilation completion.

LIB-045 remains open. Required work includes real SIMD layout and Rust/C ABI,
target-feature call/coercion/assembly rules, demanded const/symbol operands,
authenticated body transport and actual native emission. The real non-temporal
128-bit store needs its exact memory semantics and an independently qualified
backend operation. Broader PA41 labels, ABI clobbers, register/target coverage,
global/naked assembly and unwind contracts remain required at assignment release.
There is no original core, alloc or std artifact or new assignment completion.
