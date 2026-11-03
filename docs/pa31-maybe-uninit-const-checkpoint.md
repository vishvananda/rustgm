# Source MaybeUninit constant-execution checkpoint

This finite PA31 source/constant profile clears the original `maybe_uninit`
language-item error. PA31 and its runtime unit remain planned. Candidate native
execution, general PA29 nominal transport, original core/alloc/std artifacts and
independent imported consumers remain required.

The frozen [packet](../pa31/tests/spec/source-maybe-uninit/README.md) contains
40 source cases and three C observers, independently checked with pinned Rust
revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`. Candidate full CFG/proof/lowering
checks cover 37 constant cases, including ownership opponents. The CLI constant
profile covers 35 cases, both uncapped and with cap-lints=allow. Its stage does
not replace the two full-pipeline move/borrow checks. None of the three required
native observers is claimed as candidate-qualified.

## Implementation

The actual language item requires a union and has no role-imposed arity, field,
representation, Copy, constructor or method requirements. Its identity is appended
at language-item ordinal 81; earlier ordinals remain intact. Ordinary source
traits, field rules and cleanup policies still supply their own evidence.

Union byte capability is an opaque leaf after completed layout. It does not read
inactive fields; construction and extraction validate the selected typed field.
Initialization masks, pointer provenance and strict image/backing identities stay
in force. Zero-sized constant transmute uses the existing codecs and validity
rules. Assert-inhabited returns actual unit or faults for an uninhabited type;
every reached call still needs its own const authority. The real source
panic_display hook validates its declaration policies, actual Code/call and
evaluated argument before producing a constant-execution fault. Runtime panic
execution is still required. Transparent unions require the actual crate feature,
including inside a macro allowance envelope, matching the pinned semantic gate.

The reference cases exposed a producer mismatch: solved static equalities selected
canonical executable targets while Code storage roots retained original lifetimes.
The proved publication path now reuses that same substitution object for storage,
call roots, intrinsic operation data, unsizing and inline results. It leaves the
original proved Body and source callable contexts intact. The existing compact
four-byte type facts identify graphs containing function items without walking
shared descendants at each root. No Code, Frame, Body field or persistent lookup
map was added. Unchanged call parameter lists allocate no replacement vector.

Reference-selector descriptors must consume the same finalized Code tuple/result
roots. Their existing sparse rows now finalize before publication, retaining raw
selector/callback identity and the original caller contract. Reached consumers
validate the exact Code, source owner, current operands and solved target mapping.
The transmute-specific caller workaround was removed; image identity checks and
callback provenance checks were retained.

## Qualification

Raw local receipts are under `obj/parallel/maybe-uninit-v2/`; they are generated
evidence rather than repository artifacts. Important immutable receipt hashes:

| Evidence | SHA-256 |
| --- | --- |
| Pinned Rust, 40 source outcomes and three native observers | `2c9311175a142cdf7f73bf06110fcfcc80d372f34b230cd29068bd8c3e5de84a` |
| Joined CLI, bounded union work, upstream/harness and original-core observation | `4891cd69e6aa2e26f52e83f21fdc070b553c095a1b6721152f4a8b92e51da07d` |
| Immutable A/A and ABBA measurements, v2 | `bb44dd6a197a4e71038232bb55811fba4779da0b7d24605c30918f782158e9f4` |

The focused combined gate passed three production tests covering 105 source
outcomes and five provenance/owner/current-call controls in 218.95 seconds. A
second gate passed ten affected cold-storage guards, five layout/union controls
and PA12 125/125 in 93.54 seconds. The only test-source change between these gates
was the imported binding guard, updated from 1368 to its measured 1384 bytes;
joined qualification checks that exact difference and unchanged compiler sources.
After removing temporary diagnostic tracing, the exact final source passed a
consolidated 24-test gate (three production tests and 21 private controls), PA12,
formatting, architecture audits and strict selected-target Clippy in 135.23 seconds.
The final joined qualification uses that exact source fingerprint and the same
immutable CLI; its repeated original-core observation took 16.12 seconds.

Measured Code=304, Prepared=312, CheckedInput=328, Machine=4728 and State=448 bytes
are unchanged. SourceCatalog=1840, Reservations=2544, Declarations=7088,
Resolution=2032 and Package=256. Provider/imported binding arrays are 1640/1384;
the 256-byte difference bound and hot Node=36/Body=456 checks are unchanged.
For union widths 8, 32 and 128, cold work is 2, 128 warm queries cost 128, and
retained cells stay at 4. Inactive uninhabited fields add no child cache entries.

Performance uses immutable baseline `a716191e` and candidate `8d2641b0`, frozen
inputs, four A/A pairs and two ABBA blocks with four processes per sample. All
three final inputs have byte-identical output. The 32-reference-selector input
measured candidate/baseline ratios 0.913 and 0.938, against A/A ratios 0.984–1.023.
Ordinary and opaque-union controls remain within their wider calibration noise;
no improvement is claimed for them. An earlier early-array input changed two
later diagnostic type ordinals when the new canonical signature was interned.
That failed equality check and all observations are retained. The final reference
input uses the existing binder fixture without that early array dependency;
outputs were not normalized or blessed.

## Original-core observation and remaining work

CLI SHA-256 is `8d2641b020ef21c252b3a41351c89401754154f92213a671d28ac9c12e539489`.
The unchanged 722-file input manifest, six configuration bindings, cap-lints=allow
and finite limits remain: work 14,500,624, attempted bytes 1 GiB and retained
bytes 64 MiB. The observation took 17.66 seconds and opened 375 sources. It now
fails at `mem/transmutability.rs:87`, file59 byte3207, on `transmute_trait`;
this is LIB-124. Opening a source is not compiling that file or building a crate.

LIB-123 still requires the candidate native value/destructor observations and
assert-inhabited failure through the real source panic_nounwind function, with
its actual type-specific message. General source struct/enum/union metadata and
fresh imported consumers remain PA29 requirements. LIB-124 must retain the real
TransmuteFrom signature, structured Assume const arguments, feature/attribute
claims, coinductive proof and layout/validity/lifetime semantics. Header admission
alone does not complete that protocol. All original library artifacts and the
assignment release ladder remain required.
