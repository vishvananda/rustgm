# Receiver adapter value cleanup

PA24 owns these ordinary closure, `FnMut`, move and custom `Drop` contracts.
The four original inputs preserve the bytes and expected outcomes declared in the
[pre-oracle outcome manifest](../../adapter-value-cleanup-outcomes.json).
All four independent object outcomes and three native outcomes match. The
registered source and native checks pass, as do all 748 PA24 text comparisons,
strict selected Clippy, formatting and architecture checks. The four new
reference triples are generated; all 744 older triples remain byte-identical.
These four closures are captureless, so they require no environment receiver
adapter. They remain controls for the captured successors below. Full CI found
three private fixtures made the same captureless assumption; the corrected
fixtures also select the unique generic `call` instance. Their execution remains
pending CI after the local private harness build timed out.

| Input | Required source result | Required native result |
| --- | --- | --- |
| `tiny-copy.rs` | Accept; shared source called through `FnMut`, Copy result | Exit0, empty stdout/stderr |
| `tiny-owned.rs` | Accept; owned result returned from the captureless call | Exit0, empty stdout/stderr |
| `tiny-transfer.rs` | Accept; owned input forwarded and returned | No drop before return, exactly one after; exit0, empty stdout/stderr |
| `tiny-move-twice.rs` | Reject result use after moving it to `first` | No native execution |

The positive cases require complete source-body, ownership and lifetime
proof before LowIR. The move opponent must reach its ownership rule; a missing
adapter cleanup record is an implementation failure. The transfer counter checks
that returning the result transfers it before adapter-local cleanup begins.

The main PA24 manifest declares all four inputs and the three native comparisons.
Existing text references remain unchanged. New sidecars and their provenance must
be generated only after independent outcome review; none are supplied by hand.
The existing `closures` maintainer suite owns the focused source and native checks
in `environment_execution::adapters`.

The captured successors retain the original four inputs. Their shared reads of
`n` or `marker` force a nonempty environment and the shared-to-mutable adapter.
`captured-owned` and `captured-transfer` must accept and execute with the declared
result/one-drop observations; `captured-move-twice` must reject moved-result use.
The separate [captured outcome declaration](../../captured-adapter-value-outcomes.json)
records the old compiler failure and current acceptance before reference generation.
All three independently checked statuses match. The two positive successors
failed on the prior compiler with missing destruction-scope coverage and now
produce complete LowIR and run natively. The registered source checks, all751
PA24 text comparisons and25 native programs pass. The three new reference triples
are generated separately; all748 earlier sources and2244 oracle channels are
checked byte-for-byte against the published checkpoint.
