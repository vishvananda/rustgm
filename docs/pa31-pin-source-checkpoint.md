# Actual Pin nominal and constructor source checkpoint

The real `pin` role now completes through existing nominal identity, typed field
templates and the cold role ledger. The pinned declaration profile permits
structs without an extra arity, field-count or repr restriction. Pin retains
ordinary field destruction and gets no implicit Copy, drop suppression, receiver
or projection permission. The existing wrapper lifecycle owns this work; no
new normal node/body/type field or independent contract table is added.

`new_unchecked` authenticates the actual inherent owner, source declaration,
ordinary function header, declaration lifetime domain and retained source body.
The normal body checker/instance/ownership pipeline still proves and executes
that body with its real unsafe/const effects. Rust-valid unused alternate
inherent owners/signatures stay valid. An implicit pin constructor must later
authenticate its actual Pin owner and operands; this producer grants no such
permission. Imported contracts retain the existing explicit transport fence.

The first public run exposed an overly strict assumption: a real nongeneric
constructor with an empty lifetime domain deliberately has no callable template.
The validator now consumes the actual completed empty scheme published by the
declaration transaction. It does not invent a lookup fallback or force every
ordinary function to allocate a template. Private controls verify pending versus
completed roles, ordinary field destruction, the metadata fence, changed callable
headers, removed empty schemes and distinct declaration/public lifetime frames.
Strict all-target/all-feature Clippy compiles all three controls; their execution remains pending full successor CI.

The final validator compares the actual declaration item and completed body
scheme in their own frame. Public callables relocate early/late coordinates;
unused named lifetimes may extend a body domain after its signature forms.
The initial same-signature comparison rejects the genuine impl-lifetime input;
the final immutable CLI accepts it. The separate unused-lifetime input exposes
lazy body-slot allocation and now passes unchanged. No extra callable templates
are fabricated or required.

The [source packet](../pa31/tests/spec/pin-source/README.md) freezes25 complete
Rust outcomes, all independently confirmed. Candidate tests run them through
actual source analysis, original body proof and demanded lowering. The native
companion executes its real constructor effect, unique-reference mutation and
owned field destructor. Pinned Rust and emitted LowIR through the locked CPPGM
object backend both print `record=5`, `scalar=7`, `record=5`, `borrowed=9`,
`record=5`, `record=3`, `owned=8`, each on its own line. The frontend uses no
external Rust compiler or fabricated helper body to implement these operations.

The final focused gate passes six exact tests plus1193 PA19 text cases in83.92s,
including preceding FnPtr and UseCloned controls, full formatting, architecture
and strict selected Clippy. All-target strict metadata Clippy passes in41.51s.
Native/configuration/ordinary-output/harness/upstream/core qualification passes
in33.87s. All bound compiler and packet hashes still match those receipts.
This is a development checkpoint, not an assignment release.

## Original core and performance

CLI4003bfd267e208f16816688776a42987045795a993aa7d61eb996a5743f891e5
preserves all722 original input/package hashes, six source pins, fifteen
configuration values, options, Allow cap and finite resource limits. Ordinary
and constant-value LowIR match predecessor1c5be3ca byte-for-byte. The config
snapshot retains SHA25686c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d.
Core still stops at LIB-154, actual AsyncDrop in future/async_drop.rs29:
`ERROR: file 246: 1138: source compiler language item semantic contract is not
implemented`. There is no original library artifact or additional whole PA.

The final standalone probe takes17.0272s/375 distinct opens. The interim Pin
CLI41f02b51 took23.69s once. Its subsequent immutable A/A plus ABBA block on
identical original inputs preserves exact failure-output equality: A16.928,
A16.199; A18.396, B18.238, B16.318, A17.944 seconds, paired B/A0.951. That block
belongs to the interim CLI and is not substituted for final-candidate timing.
The isolated longer observation is not reproduced as a stable regression.
Every probe retains the original profile/hashes; no successful core time exists.

Final57 raw small observations retain ordinary A/A plus paired ABBA and exact
output equality. Eight ordinary blocks have paired-median B/A1.103; individual
A/A ratios0.817–1.533 show large startup noise in these8–14ms controls. Larger
frozen ordinary sources1024/4096 have paired B/A0.986/1.066, A/A1.033/0.962 and
A/B means225.0/221.9ms and806.2/859.0ms. The larger4096 observation is retained
as6.6% development overhead, not hidden by a speedup claim. This cold source-role
change adds constant role checks, not a per-function scan. Broader qualification
retains its performance gates; these blocks prove neither global regression
absence nor general throughput improvement.

Distinct real source constructor instances8/32/128 reach LowIR in final candidate
medians47.7/98.9/296.9ms. The predecessor rejects the new role, so changed-path
scaling is candidate-only. No asymptotic, peak-memory, whole-library or speedup
claim follows. The first scale driver omitted the CLI's required main; its failed
receipt is retained and later frozen inputs include it. No prediction is weakened.

## Required next dependency

The [eleven receiver controls](../pa43/tests/spec/pin-receivers/README.md) all match
pinned Rust independently. They show that real Deref/LegacyReceiver evidence,
including ordinary similarly named wrappers, precedes receiver admission.
RustGM stops earlier at actual source Deref, so all seven Rust-positive cases
remain required and its four negative exit statuses qualify no receiver
semantics. LIB-155 owns this prerequisite; LIB-154 remains the original-core
frontier. The real constructor parent P:Deref and missing-bound opponent are
also retained; the unrestricted ordinary native constructor is not claimed as
that complete API. Implement source Deref/Target and real method/operator consumers,
then bounded receiver proof using exact Self arguments/regions and actual
selection dependencies. Reuse the typed trait/associated-result and declaration
worklists; do not unwrap Pin by name or role or scan the whole program per step.

Actual Future/signature formation follows those producers. Async result types
must retain real opaque identity, Output equality, captures and body semantics;
state machines and AsyncDrop need real pinning/destruction/cancellation behavior.
The upstream async_drop_in_place placeholder is not an implementation. PA29
transport, every unfinished parent requirement and all original core/alloc/std
artifacts remain required. Root stays alone in warm guarded btrfs scratch with
the five-minute, two-thread,16GiB/150GiB limits and current/predecessor pins.

## Evidence

- `obj/qualification/fast/1793835244177672083/receipt.json`:
  7a6035754401e19bcf735e3bf6f61d47c21279aa34b2c81654b0c282d07310cb.
- `obj/parallel/pin-source-v1/private-compile-v2/receipt.json`:
  63655def56d5f18978b1f2f9307a3ba7f13f61e07dffe7ae507025459b541927.
- `obj/parallel/pin-source-v1/reference-v2/receipt.json`:
  773cfc9a207947a3f807e51558fca95605893aae7553ffeb6eb2eba1897e70e7.
- `obj/parallel/pin-source-v1/qualification-v2/receipt.json`:
  eaec06976346fa7919f7471a21dfd2c4041d8c1602f2fe9b3c7c2de5eb761ef5.
- `obj/parallel/pin-source-v1/measurement-v4/receipt.json`:
  a472d27f7596223274f3bd833991af5b92e172ec5fc585bb231071192137b290.
- `obj/parallel/pin-source-v1/ordinary-scaling-v1/receipt.json`:
  719ff3fb5bad7134fe54fc53843680238838c79d95cd5d462af5ded4acddecb7.
- Interim `obj/parallel/pin-source-v1/core-measure-v1/receipt.json`:
  fd66247d06e2eb2ecaad2b5d13d04a37e40d20fd2862c00fbe955087e5c64579.

Full [CI41929736419](https://github.com/vishvananda/rustgm/actions/runs/41929736419)
onf94868db5 is terminal success with every gate, including private source-role/
shim controls, native, debug, independent Rust and const-machine qualification.
It precedes the Pin changes; no private execution or full Pin successor result
is inferred from local metadata compilation or an earlier green gate.
