# Source assembly vector join

The required `_mm_stream_ps` path now consumes genuine SIMD fields and actual
function target-feature facts through the existing assembly owner. PA41 remains
planned and LIB-045 remains open; there is no new core, alloc or std artifact.

## Checked boundary

`Config`'s existing fixed target rows remain the sole CPU baseline owner.
A borrowed resolution query crosses the phase boundary without retaining another
policy map or package/body field. One transient `FeatureSet` per actual assembly
invocation joins that baseline with `target_body_features`. Construction pays the
existing source-macro work ledger; body/source authority keeps its existing
inference/header payers. Compiler-controlled `target_*` cfg cannot be forged.
Language call/coercion safety continues to use body features alone.

Existing XMM classes require SSE and consume `Definitions::simd_shape`. Supported
integer lanes of 8/16/32/64 bits and f32/f64 lanes must total exactly 128 bits.
A padded 12-byte vector, scalar wrapper, pointer vector or wider/shorter vector
receives no XMM permission. SIMD values cannot enter a general register merely
because their storage has a scalar-sized component. Ymm/Zmm remain explicitly
unsupported, including when independent Rust accepts the body-feature opponent.
The pinned primary policy is
[rustc_target's x86 assembly table](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_target/src/asm/x86.rs);
preserve NOTICE.

Every typed input, output-only and tied operand must implement actual Copy.
The existing source Copy collector adds operand types to its same deduplicated
set, charges their count and proves each distinct type in the caller scope.
The existing ownership boundary consumes its completed CopyView after payload
validation. Untyped discarded clobbers need no invented type. No additional proof
owner/cache/fields are introduced. Ordinary transfers, initialization, output
places, output order, lifetime debts and source declaration identities remain
required; body features provide none of those permissions.

The original function bytes, `vps!`/`concat!`, `movntps [{p}],{a}`, two real
operand types and `nostack`/`preserves_flags` survive through the typed body and
original/instance proof. Demanded lowering now emits the pinned backend's
optional assembly template for this form ([below](#initial-native-assembly-profile)).
Existing source/target/SIMD publication fences stay closed; no assembly body
codec or imported policy is added.

## Qualification and retained failures

All 32 committed metadata-only pinned Rust predictions match: 27 vector/type/
ordinary-obligation opponents, 2 original stream/safety controls, and 3 explicit
output/inout Copy opponents. Rust inputs and the compiler executable are hashed
before and after each guarded observation. None executes AVX or candidate native
assembly. The source fixture inventories are `pa41/tests/spec/asm-vectors/`'s
three prediction files. These are private planned-stage controls, not a released
PA41 text manifest.

Private 14 pass in 130.33s at
`obj/qualification/fast/1793128381886913838/receipt.json` (SHA256 `3fdc890b91d3e8854610fcee6a0b2e4ec37903c75f4d0f0a581d98b6ee6159b7`), including source SIMD/layout/copy, assembly
safety/CFG/backend denial and body-only CPU policy. A final exact 6 pass in 211.95s
at `obj/qualification/fast/1793129293766221185/receipt.json` (SHA256 `78000a2a50d0389bf25c0015161881366db646cf1c177c0d6642b33b736a4762`), after adding the exact template and
zero charged-byte assertions. Strict private Clippy, formatting and the global
architecture audit pass on those final test bytes. The final authenticated
same-binary 5 observation also passes the source Copy carrier/wrong-owner checks,
record sizes and width counters.

Production/CLI strict Clippy, CLI build and lexer 10 pass in 210.21s at
`obj/qualification/fast/1793128636327555680/receipt.json` (SHA256 `eea9c2d45b55691b0822ac4a15a6480e870016fcd9eab8de1c571667eaad7bbe`). Production compiler/configuration bytes are
identical to the final source; only two private test files changed afterward.
`make test-upstream` verifies 24 upstream fixtures and 126 Reference files.

Earlier architecture routing, unused receiver and test-foundation failures are
retained. The final assertion-only cold private run timed out during its build
in `fast/1793128993498648005`; it executed no tests and remains an infrastructure
failure. The identical selection subsequently passed with warm caches under the
same five-minute deadline. No limit was raised or selection omitted. Disposable
C build caches were retired through the guard; their former executable hashes
and raw observations remain, and pruned binaries are not reusable evidence.

## Work and storage

The actual baseline/body query adds 158 source-macro work and 0 cumulative charged
bytes (5568 before/after). That ledger is not a live-retention or allocator
measurement; inference/header query work is accounted separately. An earlier raw
observer mislabeled those charged bytes as retained bytes; the final observer
corrects the label and asserts the zero charge delta. Body remains 456 bytes and
FeatureSet 16. Existing BorrowEnvironment/Contract/BodyBuiltins/SourceRequirements/
Prepared record widths remain 40/184/56/32/104.

| Operand width | Logical assembly payload bytes |
| ---: | ---: |
| 1 | 421 |
| 8 | 632 |
| 32 | 1830 |

These are capacity-based payload observations, not allocation buckets. The
assembly payload/instruction/operand remain 104/28/28 bytes. Ordinary bodies retain
no new owner or record field. ASM operand discovery adds one bounded linear scan
to the existing typed collector; distinct source Copy goals still share one set,
sort and caller proof. No wall-time speedup or whole-compiler performance claim
is inferred from the counters.

Remaining LIB-045/PA41 work includes the rest of assembly body transport (B2:
`xmm_reg` inputs need S and T's vector type rows; A transports the pointer
forms), the other library forms in the [inventory](#pinned-library-asm-inventory),
demanded const/symbol operands and broader target/register/option/clobber/
global/naked/unwind closure. Backend work remains separately coordinated.

## Initial native assembly profile

RustGM now lowers the checked asm body nodes of the original `_mm_stream_ps`
to the pinned backend's optional LowIR `assembly` templates and `asm`
invocations. It reads only the retained body payload, through the existing
`instruction()` check. Every other checked form fails in lowering with its own
text, and module-body transport still rejects asm bodies (B2).

Each invocation in each emitted body gets its own template, `@asmN`, numbered
in emission order and written after the globals and before the functions. A
template declares:

| Field | Value |
|---|---|
| Dialect, memory | `intel readwrite`. AT&T, `nomem` and `readonly` are fenced. |
| Properties | 1 (side effect), plus 4\|8 (stack, red zone) unless `nostack` |
| State reads | 24: x87 control and MXCSR control |
| State writes | 0 with `preserves_flags`, otherwise 7: EFLAGS status, x87 status and MXCSR status |
| `in(reg)` raw pointer | `operand ptr i -1 1 0 0`, one group each of rax, rcx, rdx, rsi, rdi and r8–r15. rbx, rbp and rsp are excluded, as in rustc's x86-64 `reg` class. |
| `in(xmm_reg)` 128-bit vector | `operand v128 i -1 1 1 0`, groups xmm0–xmm15. The argument is the address of the operand's snapshot, and the backend loads it into the register. |
| `{name}` | `use OP 0 0 0`: the operand's own view, with no modifier |

Template strings are joined with `\n`, `{{` and `}}` are unescaped, and the
text uses the backend reader's escapes. The snapshot is the object copy that an
object-typed read already makes, so a later assignment to the local, including
one inside a later operand's expression, cannot reach the asm.

State reads are 8|16, not the conservative 63. The Reference says that "Any
registers not specified as inputs will contain an undefined value on entry to
the assembly code" (`asm.rules.reg-not-input`) and that "the direction flag
(DF in `EFLAGS`) is clear on entry to the assembly code and must be clear on
exit" (`asm.rules.x86-df`). The status flags therefore carry nothing into the
asm, and DF needs no read. The control fields stay live because a template may
depend on rounding or exception masks. Declaring 63 would also forbid the
backend from moving status-flag producers across the asm.

### Fences

Options are checked first in bit order, then operands in source order, then
template modifiers, then (since B2 A's comment fence) comments other than
`#`. Each text is `inline assembly <form> is outside the initial assembly
profile`, with these forms:

- `pure option`, `nomem option`, `readonly option`, `noreturn option`,
  `att_syntax option`, `raw option`
- `output operand`: `out`, `lateout`, `inout`, `inlateout` and discarded
  clobbers
- `reg input other than a raw pointer`: integers, floats and function pointers
- `reg_abcd operand`, `reg_byte operand`, `explicit register operand`
- `xmm_reg input other than a 128-bit vector`: the 4- and 8-byte scalars that
  the frontend accepts in `xmm_reg`
- `template modifier`
- `comments other than #`: `/* */` and `//` outside a quoted string, added
  with B2 A's comment fence

`const`, `sym` and `label` operands, ymm and the other unimplemented classes
stay rejected earlier, at body construction or in the frontend.

### Module-body transport

B1 adds no transport. A pointer-only `#[inline]` asm body on `native.rs`'s
prelude lowers in its own crate. The prelude alone publishes in version 13;
with the body, the packet fails with "source rvalue requires a complete wider
transport row".

The crate holding the real stream body (`native.rs` before its authored
observers) was observed after the first run, not predicted. It stops at
"source macro/semantic-name provenance metadata is not implemented", which
the test now pins. That fence is crate-wide: `vps!` and `concat!` expand in
the body, so the crate has more than one syntax context. One exploratory run
(`obj/qualification/fast/1794180468170512714`, not pinned by a test) shows
what lies behind it:

| Source | First error |
|---|---|
| prelude and `__m128`, const-generic `[T; N]` impls | source const parameters are outside the initial source profile |
| prelude and `__m128`, concrete `[f32; 4]` impls | source nominal representation requires its wider complete contract |
| the full prefix, concrete `[f32; 4]` impls | source macro/semantic-name provenance metadata is not implemented |

So for this body at least three fences stand before the asm value row:
macro provenance, the `repr(simd)` record contract and the const-generic
array impls. B2's design notice starts there.

Amendment (B2 M, [macro contexts](parallel/macro-contexts.md)): version 13
now admits the semi-opaque marks `vps!` and `concat!` leave, so this crate
stops at its first const parameter, "source const parameters are outside the
initial source profile". That outcome was declared before the run, and the
test pins it in place of the B1 observation.

Amendment (B2 K, [const parameters](parallel/const-parameters.md)): version
13 now exports const parameters, so the const-parameter fence no longer stops
this crate. It stops at `__m128`'s `repr(simd)`, "source nominal
representation requires its wider complete contract", which this run reaches
whether or not the array impls behind it would transport. That outcome was
declared before the K
candidate run in `pa41/tests/spec/asm-native/contract.json`, and the test pins
it in place of the M observation.

Amendment (B2 A, [asm transport](parallel/asm-transport.md)): version 13 now
exports the initial assembly profile's forms as value code 0 and a cold
`Original.assembly` column. The pointer-only transport control above no
longer fails: its packet carries one assembly value and the template
`prefetchnta [{p}]`, and `asm_lowering.rs` pins that in place of "source
rvalue requires a complete wider transport row". Every form outside the
profile keeps its lowering text at export, through the same table. The real
stream body's crate still stops at `__m128`'s `repr(simd)`, as declared in
`pa41/tests/asm-transport-outcomes.json`.

Amendment (B2 A's comment fence, [asm transport](parallel/asm-transport.md)):
the pinned backend's assembler reads only `#` comments, so `/* */` and `//`
templates now fail with "inline assembly comments other than # are outside
the initial assembly profile", after the modifiers. `opponents/block-comment.rs`
and `opponents/line-comment.rs` join the 16 opponents. `joined.rs` now uses
`# {{ {p} }}` in place of a block comment. The new positive control
`quoted.rs` keeps `//` and `/*` inside a quoted `.ascii` string and lowers.
`contract.json` records each change with its before and after values.

Amendment (B2 T, [array type rows](parallel/array-type-rows.md)): version 13
now exports whole array types as type recipe tag 14, with impl headers over
`[T; N]` like the Clone and Copy impls after `__m128` here. The stream body's
crate still stops at `__m128`'s `repr(simd)`, before those impls, as declared
before and after T in `pa29/tests/array-type-outcomes.json`. S's
representation byte comes next.

### Qualification

All outcomes except the stream body's publication error were declared in
`pa41/tests/spec/asm-native/` before any observation.

- Private 7 pass in 131.70s at
  `obj/qualification/fast/1794181994637258348/receipt.json` (SHA256
  `d4a0fd71247f8a5786d674417ee3e6c630cd3efc019bac9821d8b1435334d634`): the
  16 declared fences, joined templates, the transport control and stream
  observation, the 1/8/32 scaling control, the declared stream template, the
  scalar effect contract and the LowIR record sizes. Strict private Clippy,
  formatting and the architecture audit pass on those bytes.
- The native root passes 2 in 132.15s at
  `obj/qualification/fast/1794182411807900717/receipt.json` (SHA256
  `c9b175166fc0e8e92e40c810578380e600dc269885cafe1ea4e299fb7c5d5474`), with
  CLI strict Clippy. Both executables print `expected.stdout`. RustGM's
  object has six `movntps XMMWORD PTR [r10],xmm14`, and pinned Rust's has
  six `movntps` with `rdi` or `rax` and `xmm0`. Pinned Rust accepts all 16
  opponents as metadata.
- PA11, PA14, PA16, PA18, PA19, PA20, PA21 and PA24 pass 5130/5130 text
  cases, LowIR oracles included, at
  `obj/qualification/fast/1794182577425192096/receipt.json` (SHA256
  `2becdb3d27b20f50b68dd0f6e3ac9ae2c2f1248287354b404e7fad31839d9deb`).
- `make test-upstream` verifies 128 upstream fixtures and 126 Reference
  files. `make test-harness` passes 307 script tests, 465 maintainer roots in
  13 binaries and the layout check.

Retained failures, all fixed before the passing runs:

- The audit rejected ten `assembly` and seven `inline` file stems. The LowIR builder
  and writer code moved into `builder.rs` and `writer.rs`, and the new
  modules are `lowir/asm.rs`, `lower/function/asm_invocations.rs` and
  `tests/stream_assembly.rs`.
- Strict Clippy rejected unquoted `LowIR` in four test doc comments and a
  101-line native test.
- The first transport control used the private test foundation. Its
  const-generic `Index`, `Clone` and `Copy` array impls stopped publication
  with "source const parameters are outside the initial source profile"
  before the asm body was reached. The control now uses `native.rs`'s
  prelude, which publishes on its own.
- An exploratory run showed that `__m128`'s `Copy` needs `[T; N]` impls,
  which pinned core provides in `array/mod.rs`. `native.rs` gained two of
  the same form before its first native run.

### Template storage

`Operation` stays 32 bytes and `Instruction` 40. `Program` gains one
optional boxed template store, allocated by the first template; asm-free
programs allocate none. A generic asm body emitted for 1, 8 and 32 types
gives exactly 1, 8 and 32 templates, and the same asm-free counts allocate
no store. Templates hold ranges into one operand-kind vector, one piece
vector and one text buffer.

## Pinned library asm inventory

Every `asm!` in the pinned (88d9) core, alloc and std sources that is active
for x86_64-unknown-linux-gnu, by form. Alloc, std's Linux paths and
portable-simd contain none. Core's `dec2flt/fpu.rs` (x86 without SSE2), the
x86 `__cpuid_count` branch and the x86 `eflags.rs` variants are inactive on
this target. Paths are under `library/stdarch/crates/core_arch/src/`.

| Form | Sites | Needs |
|---|---|---|
| `in(reg)` raw pointer, `in(xmm_reg)` 128-bit vector; `nostack, preserves_flags` | 5: `x86/sse.rs:2071` `_mm_stream_ps`, `x86/sse2.rs:1440` `_mm_stream_si128`, `x86/sse2.rs:2800` `_mm_stream_pd`, `x86/sse4a.rs:114` `_mm_stream_sd`, `x86/sse4a.rs:140` `_mm_stream_ss` | nothing more: this profile |
| `in(reg)` raw pointer, `in(reg)` integer; `nostack, preserves_flags` | 2: `x86/sse2.rs:1468` `_mm_stream_si32` (`{a:e}`), `x86_64/sse2.rs:82` `_mm_stream_si64` | integer `reg` inputs; the `e` modifier for si32 |
| `out(xmm_reg)`, `in(reg)` raw pointer; `pure, readonly, nostack, preserves_flags` | 1: `x86/sse41.rs:1174` `_mm_stream_load_si128` | outputs, `pure`, `readonly` |
| `in(reg)` raw pointer, `in(reg)` integer, `out(reg_byte)`; `nostack, att_syntax`, plus `pure, readonly` for `_bittest` and `_bittest64` | 8: `x86/bt.rs:29,48,67,86` (32-bit `{b:e}`), `x86_64/bt.rs:29,48,67,86` | outputs, `reg_byte`, AT&T, the `e` modifier for the 32-bit forms |
| `out(reg)` (`{0:r}`), `inout("eax")`, `inout("ecx")`, `out("edx")`; `nostack, preserves_flags` | 1: `x86/cpuid.rs:88` `__cpuid_count` | outputs, explicit registers, the `r` modifier |
| `out(reg)`; `nomem, att_syntax`, uses the stack | 1: `x86/eflags.rs:35` `__readeflags` | outputs, `nomem`, AT&T |
| `in(reg)` integer; `nomem, att_syntax`, uses the stack | 1: `x86/eflags.rs:66` `__writeeflags` | integer `reg` inputs, `nomem`, AT&T |
| ymm, zmm and kreg operands (AVX, AVX2, AVX-512F, AVX-512FP16 stream stores, stream loads and masked scalar loads and stores) | 17: `x86/avx.rs` 3, `x86/avx2.rs` 1, `x86/avx512f.rs` 10, `x86/avx512fp16.rs` 3 | wider vector and mask classes |

That is 36 sites. The order for stage C follows from it: integer `reg`
inputs with the `e` modifier (2 sites), then outputs with `pure`/`readonly`
(1), then the `bt` family and the remaining scalar forms (11), then wider
classes (17).
