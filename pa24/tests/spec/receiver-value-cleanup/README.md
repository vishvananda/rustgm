# Receiver adapter value cleanup

PA24 owns these ordinary closure, `FnMut`, move and custom `Drop` contracts.
All four inputs preserve the bytes and expected outcomes declared in the
[pre-oracle outcome manifest](../../adapter-value-cleanup-outcomes.json).
All four independent object outcomes and three native outcomes match. The
registered source and native checks pass, as do all 748 PA24 text comparisons,
strict selected Clippy, formatting and architecture checks. The four new
reference triples are generated; all 744 older triples remain byte-identical.
Private controls and full CI remain pending after the local private build timeout.

| Input | Required source result | Required native result |
| --- | --- | --- |
| `tiny-copy.rs` | Accept; shared source called through `FnMut`, Copy result | Exit0, empty stdout/stderr |
| `tiny-owned.rs` | Accept; owned result returned from the receiver adapter | Exit0, empty stdout/stderr |
| `tiny-transfer.rs` | Accept; owned input forwarded and returned | No drop before return, exactly one after; exit0, empty stdout/stderr |
| `tiny-move-twice.rs` | Reject result use after moving it to `first` | No native execution |

The positive cases require complete source-body, adapter, ownership and lifetime
proof before LowIR. The move opponent must reach its ownership rule; a missing
adapter cleanup record is an implementation failure. The transfer counter checks
that returning the result transfers it before adapter-local cleanup begins.

The main PA24 manifest declares all four inputs and the three native comparisons.
Existing text references remain unchanged. New sidecars and their provenance must
be generated only after independent outcome review; none are supplied by hand.
The existing `closures` maintainer suite owns the focused source and native checks
in `environment_execution::adapters`.
