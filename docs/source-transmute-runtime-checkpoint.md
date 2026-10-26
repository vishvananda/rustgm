# Source transmute: checked local value transport

The local source consumer is qualified at `dff35e4a` on the `d249fd21` base.
It uses the actual attributed `pub const unsafe fn transmute<Src,Dst>(Src)->Dst`
contract, with two own implicit-Sized type parameters. Ordinary name lookalikes
remain ordinary functions. This adds no library factory, bootstrap slot, wire
profile, body operation variant or backend instruction.

The declaration view carries `DefId`, selected signature, Src and Dst. Each
reached consumer authenticates its actual source catalog, current header site,
full instance, actual Call and caller Code/body. Ordinary unsafe, const, lifetime,
ownership and body proof remain prerequisites. Pre-monomorphization size equality
uses the shared ABI storage-readiness walk and actual aggregate field producer;
generic identity and genuinely thin pointer definitions remain valid. An unused
unequal or unknown-size conversion is rejected before publication.

Runtime lowering moves the already evaluated source once into an owned ABI
snapshot and reads the destination representation. Its ordinary Call move graph
prevents a source destructor and retains the destination destructor. It adds no
runtime validity panic. CTFE validates source and destination values through the
existing byte codec: initialization masks, bool/char validity, symbolic pointer
words, actual reference duties and strict integer-value rules remain distinct
from raw byte transport. Numeric raw addresses grant no allocation, reference or
dereference authority.

Pending CTFE work occupies the existing optional Reads owner. Its Key now stores
`ReadInput::Storage(Source)` or `ReadInput::Transmute { source, value }`; the same
paid result/duty cursor resumes without encoding/decoding again. A changed input
rejects the continuation. The suspended original frame retains its source values
and allocations; return/end-local alone expires those allocations. Scratch backing
never obtains a LocationId. Decoded images own their bytes, and reconstructed
pointers/duties retain genuine original symbolic targets rather than scratch.

A same-signature function/raw/function roundtrip uses a demand-only structural
segment in the existing cold Origins roots Vec. Its old SourceSelect origin and
Instance checks remain unchanged. A temporary unique-store index refuses
borrowed/addressed/projected locals, indirect writes and multiple stores. Every
transmute edge still needs a fresh actual Reader selection and full signature.
Cold work is O(values + locals + statements + projections), with cumulative
charged query edges and a body-size cycle bound. Scratch and old/new vector
coexistence are prepaid. Ordinary lowering never prepares this segment.

## Actual qualification

Hosted run **40353475877** passed the four affected private groups, production
and private strict Clippy, global formatting/audit, native object/link/run, and
the workflow's harness/upstream/audit-cache checks. The helper used 173.02 seconds;
its immutable workflow measured 43.75 seconds already consumed before the helper,
with a 285-second overall deadline. No local compiler was run.

The other two controls are explicitly retained from **40351944062**: actual cold
and enclosing layouts, and the Origins linear/warm/alias/failed-budget group,
which also runs its existing origin adversaries. The successor changed only a
kernel attempt prepayment and invalid negative-test identity strings; no owner,
layout or Origins data representation changed. All 2,748 compiler/tool fingerprints
in each joined receipt match their actual source before and after execution.

The immutable source provider has SHA-256
`2363aff5a8c24bef2427d84853ee08851808b256cfd8de9de5a7a71510231c91`.
Its observer is
`4cfb499fa3f49f846f74b17d8c7e93cc4f061a61b83a68e97ebe052699ebc8a4`.
Pinned Rust **40339618063** already executed that exact provider and observer;
the candidate did not narrow or replace them. Both actual programs output:

```text
destination_drop
3f800000 ffffffff 3f800000 13 9 7 19 7 21
```

This covers f32/signed bits, real constant execution, numeric raw-address tags,
a source reference/raw roundtrip, same-signature function/raw roundtrip, nominal
move/drop ownership, ordinary same-name behavior and generic identity. The
candidate LowIR hash is
`95a823e17e718ef5b788a85b33dc3fd9b19f60484c3e72a0539a76c754aba289`.
The unchanged backend binary is
`ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc`.
Successful object/executable outputs were pruned after recording hashes.

The independent 28 metadata/object observations preserve their original
predictions. Acceptance agreed in every observation; the malformed declaration
predicted E0093 but actually reported E0308 in both modes. That failed diagnostic
hypothesis remains recorded. The initial candidate static failures, late invalid
test identity and missing rejected-attempt payment remain separate receipts.

## Measured storage

| Actual type | Before | After |
| --- | ---: | ---: |
| Source | 32 | 32 |
| ReadInput | absent | 48 |
| Key | 64 | 80 |
| Pending | 128 | 144 |
| Map entry | 144 | 160 |
| Reads | 40 | 40 |
| Decoder / alignment | 96 / 8 | 96 / 8 |
| Constant / alignment | 32 / 16 | 32 / 16 |
| Storage / Backing / Snapshot | 128 / 64 / 56 | 128 / 64 / 56 |
| Memory / Machine / State | 248 / 4728 / 448 | 248 / 4728 / 448 |

The actual producer also checks Catalog704, Resolution2056, Package256 and Body448.
TransmuteSelection is16 bytes. Ordinary byte storage has no Reads/word owner and
zero root capacity. There are still24 ordinary bootstrap intrinsic slots.

## Required remaining work

PA27/28 still owe genuine function ABI/signature erasure, opaque runtime callable
origins, numeric-reference/ZST validity, full zero-sized transport and genuine
MaybeUninit/union validity profiles. Rust-positive reducers remain required;
these are unsupported profiles, not language-invalid programs. Strict integer
arrays must not acquire symbolic pointer payloads by transmute. Raw transport
continues to preserve those payloads internally.

PA25 still owes the complete remaining CTFE validity family. PA29 must transport
actual source operation/body policy before imported intrinsic execution; legacy
wire/role admission remains closed. PA31's actual fmt::Arguments/Location pipeline
still needs real NonNull pattern_type, PhantomData, track_caller/caller_location,
Location/PanicInfo and real panic/format handlers. No source Assert or complete
panic/format closure follows from this value-transport milestone. Original
std-backed Clone/size_of and imported must_use debts remain unchanged.

The new PA28 fixtures and draft declarations preserve source bytes and predicted
outcomes. They do not activate a public suite or qualify a different driver/session
profile. The standard-library MaybeUninit reducer uses ordinary std, without an
internal feature exemption.

## Evidence

Own worktree evidence is under
`obj/parallel/pa31/source-transmute-runtime-v4/`.

- Qualified receipt: `observed-evidence/artifact/1793029969746755369/receipt.json`,
  SHA `828f930eab5886f27f689ff7fdcf85ffef794b8cb3e42d13c65ba16fc50f9e11`.
- Small artifact12664608084 ZIP SHA
  `1b8685a5e081cbcc2b3fc016912fff4863b1753ba451a1e5457d09fc2e6ee0ee`.
- Selected artifact12664967681 published ZIP digest
  `3af290eb6d7bbb7549b644d0432a75c98f7b897c7073179f7828f3ce7321ba1d`;
  whole ZIP/ELF stays remote. Bounded metadata ranges authenticate binding,
  source and LowIR against the actual receipt; the range proof is
  `3c9409249ef35c73469b6aacc5588108ca32e9b4273a6c2406d40dc747ab7385`.
- Actual mandatory CLI SHA
  `ee956969a4b2cb8ad05dd0ebf3159e346f0b6260ebcce40c44abd61692453522`;
  binding SHA `0d1d121894194ff3e23729b6fd0165783738749c57c9d863ccee47e6b34e16cd`.
