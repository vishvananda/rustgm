# Source target-feature contracts

Compiler `e76e27ae8d11539ff34a33b8119978decaf2ed09` replaces the two existing
target-feature presence sets with typed source/function rows. A row retains the
declared safety, complete x86 requirements, original source image and every
contributing attribute site. Each site retains its actual input-tree witness;
the generic lint owner's path witness alone does not authenticate `enable`.
Reads validate retained facts without parsing attributes again.

Direct calls keep the canonical unsafe function signature and actual selected
function identity. A safe declaration may be called without an unsafe block
only when the current function body's features cover its requirements. Closures
inherit their actual body context; nested functions and inline constants do not
acquire another owner's features. Explicitly unsafe declarations remain unsafe.
Function-pointer coercion may adjust safety in the compatible body context,
while the original item and ordinary invocation/body proofs remain required.
Global CPU baseline features do not waive language call safety.

The 77 permitted pinned x86 features use a cold `u128` set with typed implied
closures. Explicit unstable features require the actual crate request;
`allow_internal_unstable` does not authorize this attribute. The global feature
bank stays eight bytes and the existing macro-feature carrier stays one byte.
Already parsed normal/raw string roots can enter the existing attribute forest
through their genuine literal source interval. Byte/C strings and nonliteral
roots receive no ordinary-string permission.

## Evidence

Predictions and inputs were committed before each observation. The three new
pinned Rust metadata sets ran 27 + 8 + 7 commands, with zero infrastructure
failures. All five original prediction disagreements remain in their raw
results: safe extern-C acceptance; a nonfunction diagnostic without the guessed
code; an internal macro allowance that does not enable the language gate; and
the two forwarded nonliteral/concat diagnostic predictions. No AVX instructions
were executed. Primary seams are pinned rustc `target_features.rs`,
`ty/context.rs`, `coercion.rs` and `check_unsafety.rs` at commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`; preserve NOTICE.

Private six passed in 121.99 seconds, including full private strict Clippy,
formatting and the global architecture audit. The same source passed production
strict Clippy and lexer ten in 190.35 seconds. Controls cover actual direct
calls/CTFE, pointer coercion through LowIR, all owner/input/image mutations,
imported Unknown, ordinary library output, existing publication denials,
feature-index completeness and cumulative budget failure. Earlier failed
compilations and the unsorted copied-table failure remain retained.

Receipts are `obj/qualification/fast/1793123767162125000/receipt.json`
(`17960ae16a9214973e55b6c89d10064f1e10d341f1249c3290c862a5516f0ccb`)
and `1793123939270293979/receipt.json`
(`3822a10893f798d97360c8d0316c9ecf7f035a375818681869308bd99d67e561`).
The immutable handoff under
`obj/parallel/pa41/target-feature-contracts-v1/qualified-v1/` binds all 2,959
compiler/configuration fingerprints to committed Git bytes, plus 162 raw files.

## Actual retained costs

The row is 96 bytes, each site is 80, and the feature set is 16. A single-site
input witness retains 96 bytes of vector capacity. These are logical Rust
payload/capacity measurements, not allocator bucket measurements.

| Functions | Map capacity | Function map bytes | Site capacity bytes | Input capacity bytes | Cumulative ledger bytes / steps |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 3 | 336 | 320 | 96 | 1,620 / 83 |
| 8 | 14 | 1,568 | 2,560 | 768 | 17,440 / 724 |
| 32 | 56 | 6,272 | 10,240 | 3,072 | 74,240 / 2,960 |
| 128 | 224 | 25,088 | 40,960 | 12,288 | 296,064 / 11,856 |

The existing ledger/header payloads measure 96/88 bytes. Ordinary functions
without the attribute retain neither rows nor this ledger. The rare ordinary
attribute budget owner stays incomplete and has no source declaration/definition
rows; it does not activate source-library export. The outer diagnostic-owner
size and row/site alignment were not emitted by this observer and are pending
the combined observation; no padding or allocator delta is inferred.

The common active-call value measures 12 bytes and its key/value pair 16.
Compared with the old two-u32 pair's field layout, that adds eight bytes to
every retained entry, including ordinary calls. Sequential 1/8/32-call controls
reuse capacity three (48 bytes); nesting depth eight/32 retains capacity 14/56
(224/896 bytes). CallTypes is 704 bytes and its enclosing State 2,776.
Declarations is 7,048, Body 456 and Report 3,040. The post-completion inference
counter reads zero after reset and is not a cumulative-work observation.

## Remaining consumer boundaries

ASM must use baseline union actual body features through the existing assembly
owner, separately from body-only language safety. The root-owned assembly join
and actual SIMD/native consumer remain pending. This checkpoint adds no target
instruction, AVX-host, unadjusted/LLVM ABI, imported policy or wire/export
permission. Existing target-feature transport and Fn-trait fences stay closed.
