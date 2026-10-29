# Derived field constant subtrees

Joined93e84a642 preserves selected constant-expression children when a source
derive copies a field type. This closes original stdarch's __m128i lane-count
failure without reparsing a rendered type or copying a whole source literal pool.

Each selected inline integer, boolean, character or byte gets a destination
literal-arena identity and prepaid vector growth. Deferred token literals retain
their original typed token and a new pending-materialization entry. Unary/binary
operators, casts and empty-statement plain/const block tails retain their typed
children. The output mapper retains those inline values after configuration.
Pending owner/literal slices participate in the existing O(1) payload account.
The ordinary semantic, feature and source-authority checks remain unchanged.

The fixture clusters own distinct scopes:

- PA26 ordinary literal/named/compound counts, wrong count and non-Copy negatives.
- PA31 genuine NoCore Clone/Copy/native arrays, output `5 7 11\n`.
- PA41 actual pinned stdarch types! macro/field observer and an independent full
  macro/helper reference using real Rust core providers. The finite candidate
  uses the genuine repr(simd) array header and source Clone/Copy; its C observer
  checks all128 bits through the by-value argument and result.

The SIMD consumer uses the reviewed CPPGM host-object reader, which enables and
validates SSEUP metadata before emission. The standalone course lowir validator
intentionally rejects that host-object class. The first native-v1 observation
used the wrong validator profile and remains a failed infrastructure receipt;
native-v2 passes with the existing backend.emit_object contract. No frontend,
LowIR spelling, ABI or backend pin was changed to obtain the pass.

Paid copier work for8/32/128 selected fields is173/701/2813 steps, with
15376/61328/244368 bytes and73/289/1153 nodes. Adding1024 unrelated owners leaves
those costs unchanged. A warm literal append costs2 steps with no growth.
Node36/Body456 remain unchanged. Invalid indices, stale decoded handles and
failed payment are checked. Byte-pool-backed literals, calls, statement-bearing
blocks and wider expression forms remain explicit requirements in this snapshot.

The87-case joint private gate passes168.32s, with format, architecture and strict
private Clippy. Receipt1793287589355775069 SHA-256:
`37a1aa467b835d2db598477a606ca82d4f715412ff886f0722b4ba363c4ae7c5`.
Native2 plus the two measurement observers pass, receipt SHA-256:
`18587d0aea107d6a4c2217f0a121ce47945744c8a4bd714c7bf8d7bb4b86b983`.
The independent22 Rust operations are retained separately. The authenticated
handoff is `obj/parallel/derive-array-fields-v1/handoff.json`, SHA-256
`3f3e31e3b6b306db03fb3e1db3ed95f61a6cf4c60c7413b0b813935ff6c0027b`.

Root matches all3186 compiler/test/harness entries and134 fixture hashes rather
than compiling the private binary again. Production/CLI3 passes146.36s with
production strict Clippy; receipt1793287863309023751 SHA-256:
`c8bfde2310d6a8c3c79012c1467189ce40f83094b7bb7aa8316b145072416095`.
The35.31s helper batch passes272 harness controls,385 registered roots,44-stage
layout and upstream checks; SHA-256
`f5634c8e06bc7f0a48a0ffc10a878fc02af7b65ab7a1594d85956108aaea7286`.

Current CLI813a611ccb99b529034a9203b7537a280867f8f31bb94ae50ade73915bc7beba
clears the stdarch failure and now stops at file97:24211, original any.rs730:
TypeId.data's `16 / size_of::<*const ()>()` requires a copied Call subtree.
LIB-094 owns that next actual boundary. The374 opens remain a loading count;
all722 source hashes, target settings and explicit work14,500,624/attempted1GiB/
retained64MiB are unchanged. Capped observation SHA-256 is
`d7e20470c33f392322b845cc9cd4976c6d1a1c527bbe873656d32e8e6517d5a1`.
Strict separately retains the deferred expectation at68 opens, receipt
`f8e79981dbcffa7e10a056a609b37a3619c1db6a0340757ab8dad33ce5cec895`.
The observed11.18s is not an ABBA performance claim. Full original stdarch
splat/Debug/conversion bodies, wider grammar/value transport, all original
library artifacts and assignment release gates remain required.
