# Source tuple Default checkpoint (partial)

The source-owned Default emitter now preserves the actual struct shape. Tuple
fields become ordered `Self(Default::default(), ...)` arguments; record fields
retain their actual names and unit structs retain their existing construction.
Every field still crosses `actual_field` and its existing attempted-work payer.
The generated generic impl retains the existing actual Default bounds and calls.
There is no scalar initialization shortcut or additional execution authority.

The exact eight-member Saturating declaration from pinned core
`library/core/src/num/saturating.rs` is retained in
`pa31/tests/spec/tuple-default/saturating-original.rs`, with its original interval
and SHA-256 in `contract.json`. All eight source member identities and the
configured derive receipt are checked. This is an expansion checkpoint; full
Saturating Ord/Hash/Default instantiation and original-core acceptance are pending.

Frozen Rust 1.98.0, edition2024, Linux x86-64 observations accept the unchanged
original eight-member expansion and generic non-Copy tuple/record producer,
reject missing Default and a wrong field return type, and accept the wider enum
positive that remains explicitly unsupported here. Rust staticlib plus the
frozen C observer returns status0 and `d7;d9;p7:9;d9;d7;r9:7;7\n`.
Independent receipt `obj/parallel/tuple-default-v1/oracle-v1/receipt.json` has
SHA-256 `596788097be2f1cb9ed46121dee23a4d918efcedd5863aca8503c542925ced96`.

The candidate checkpoint is deliberately incomplete. Exact-six receipt
`obj/qualification/fast/1793238613267232145/receipt.json`, SHA-256
`4a1ef5e010d9aa2c9d9b67d414e00074e06cc8b8725bc57159b431c83b47afae`,
passes formatting, architecture, strict private Clippy and build, then passes four
controls: real field Default/type opponents, field-work scaling, original block
Span and generic fieldwise Clone. The original eight-member test fails only a
whitespace-sensitive generated-text assertion. This checkpoint replaces that
assertion with parsed constructor shape plus the actual member/expansion receipt;
that final assertion change awaits Root qualification.

The genuine generic runtime control fails at the generated record `Self { ... }`
constructor (`Record<A,B>`, authored derive ending at byte13205), with
“nominal arguments were not elaborated”. Root owns the authenticated generic Self
record constructor prerequisite and the joined rerun; C makes no semantic
fallback. Candidate LowIR/native evidence is still pending. The initial Clippy
adapter failure and unchanged-source build timeout remain preserved in receipts
`1793237998529040801` and `1793238319333511777`.

At8/32/128 real fields, charged work is7839/9495/16131, charged bytes
830383/1186255/2660847, and warm receipt checks increase work without byte growth.
Body IR remains456 bytes and syntax Node36. Wider enum, generic where/const and
metadata profiles remain required. Root must run the fresh unchanged722-source
probe after the constructor prerequisite and joined qualification.
