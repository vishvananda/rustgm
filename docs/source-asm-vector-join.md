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
original/instance proof. Demanded lowering still rejects the unavailable locked
backend capability. Existing source/target/SIMD publication fences stay closed;
no assembly body codec, imported policy, target instruction or external backend
change is added.

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

Remaining LIB-045/PA41 work includes actual full-width non-temporal native
emission and its Rust/C ABI/memory/sentinel controls, authenticated assembly body
transport, demanded const/symbol operands and broader target/register/option/
clobber/global/naked/unwind closure. Backend work remains separately coordinated.
