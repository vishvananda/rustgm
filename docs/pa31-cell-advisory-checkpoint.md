# Cell declarations and conversion transport integration

Source `c6369af04` integrates the qualified conversion transport implementation
with actual Confusables and MustNotSuspend source contracts. No additional whole
assignment or original core/alloc/std artifact is complete. PA25–PA44 retain
their outstanding requirements and release gates.

## Source semantics and representation

`rustc_confusables` requires its real rustc_attrs authority, a nonempty
parenthesized string list and an actual inherent method, including associated
functions. Repeated attributes combine names; duplicate, empty and nonidentifier
strings remain accepted. These strings never create successful method aliases.
Ordinary const, unsafe, mutation and call checks remain authoritative.

`must_not_suspend` requires its own feature or authentic macro allowance, word
or string name-value form, one active attribute and an actual struct, enum,
union or trait. It grants no Send/Copy/Unpin, mutation or suspension permission
and never changes actual destruction. Suggestion and suspension-liveness lint
producers, imported intent and original library algorithms remain required.
Feature 53 preserves the conversion implementation's RustdocInternals 52.

The existing transient draft now uses one lookup per declaration and visits only
present annotations. The first annotation stays inline; only mixed declarations
allocate the cold extra vector. Actual definition, namespace, source image and
member/nominal identity authenticate completion. Constructor values, associated
Self rows, unrelated namespaces and foreign images cannot complete a type marker.
Repeated Confusables payloads are validated without a successful-name table.
Parsing, attempted growth and binding work are prepaid, including failed attempts.

With 8/32/128 unrelated functions, single-marker work is 73/97/193 and mixed-marker
work 177/201/297: the additional annotation cost stays 104. Corresponding storage
is 160 versus 244 bytes at every width. Draft 40 / Entry 32 / Annotation 12 / NodeKey 8 are
transient; common syntax Node 36 and Body 456 are unchanged. These are bounded
work/storage observations, not an output-equal wall-time speed claim.

## Joined qualification

All three compiler gates bind the same3295 source/tool fingerprints. Evidence
paths below are relative to the Root warm integration checkout's obj directory.

| Gate | Actual result | Receipt SHA-256 |
| --- | --- | --- |
| Strict private/static | 140/140, 90.65s; includes all 80 new source outcomes and ownership/payment/scaling controls | `6ab8c40285d4225aed14db3494e0259a12351d2faaa2d1b0528ffb77a19ee283` |
| Production and text | 15 exact checks plus PA9 203, PA18 502, PA19 1193, PA20 1389, PA21 758 and PA23 415; all 4,460 text cases pass, 240.29s | `f1f3af4cd626b023cdbc7447200221d5be61eb7632ec904cd654ce982496f4e6` |
| Conversion native | Both registered native tests pass, covering all 24 standalone positives and both fresh clients; 45.62s | `1f8523a95598038bccf82b847e1b50e3e91f3ba0f8e2aa0cd6120f57f100a24c` |
| Cell native | Four fresh pinned Rust/candidate pairs: Confusables mutation 16 / const 7 and suspension mutation 9 / drop 9; 3.45s | `56f8c1c3c42ff6b5c874666e5d5ac8bdd23c812a7d1168585c1933072cfff680` |
| Harness/upstream | Passed in 18.49s | `0ca6df1d60e2af3f823a024e7e1dc5034bda36523fc41282b4dc11033f1c7a9b` |

The first three receipts are qualification/fast/1793361702916463301,
1793361799599607862 and1793362261163521111, each ending in receipt.json.
Cell native and harness receipts are parallel/source-from-integration-v1/
native-v2/receipt.json and checks-v1/receipt.json. The independent pinned Rust
source batch already matched 160/160 metadata/object outcomes, receipt
`c28e07043f80db38a4d9027250b56f2931d83ac2750eb1ec50e8d135af238ed1`.

Each Cell native input preserves its complete original fixture, including public
nominals, and appends the explicit C observer entry. Candidate local library-root
lowering uses the existing API; it does not request or claim the still-unsupported
public nominal metadata artifact. The Rust reference links its real static
runtime, preserving checked arithmetic. The initial bare-object link lacked a
real panic dependency; its failed receipt is retained. No fake panic body,
overflow suppression, changed fixture or adjusted expected result was used.

Earlier failures are also retained: strict Clippy required Vec::len, and one
new rejection assertion omitted the correct missing-value-resolution category.
The latter still requires rejection at the actual consumer, without a fabricated
feature/owner failure. Final gates above execute the complete unchanged selection.

## Original core and next requirement

Immutable CLI `61f3d833ae5f72f3dffe8a5ce6ffbbf296fb0e745fb2167215d6158ef08448c2`
clears the Cell source boundary. The unchanged original core types attempt now
stops at library/core/src/ffi/va_list.rs:416, file 121, byte 16662:
`rustc_pass_indirectly_in_non_rustic_abis` on its actual local repr(C) struct.
The attempt takes 19.07s with 375 loading opens, all 722 source hashes unchanged,
explicit cap Allow and the same target/edition/finite resource profile.

Receipt parallel/source-from-integration-v1/core-v3/receipt.json has SHA
`eff395f97115562a8795fbb0800976ea837d35415dd8d127eee2e888c67755a0`.
The observer initially refused the old compiler identity. The reviewed rebind
changes only dev/bin/crates.rs's recorded hash: its source diff adds the explicit
v9 profile option and usage message. This types attempt selects no such option;
target, cfg, source hashes, caps and limits remain unchanged.

LIB-111 belongs to PA30 foreign representation/callable closure plus PA31 source
authority. It requires real argument passing, caller-copy behavior, unchanged
Rust ABI and return handling, transparent-representation conflicts and source/
metadata authority. Recognizing an inert spelling cannot qualify the requirement.
The full semantic pass, original crate artifacts, strict diagnostic qualification
and independent original-crate consumers remain outstanding.
