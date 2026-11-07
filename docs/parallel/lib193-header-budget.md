# LIB-193 header budget: attribution, literal reuse and input-graph bound

LIB-193 was the unlocated `source header contract query work exceeds profile
limit` failure in `target_features::collect`, reached from
`Checker::from_facts` before `Checker::run`. The sema header ledger was capped by
`MAX_COMPONENTS`/`MAX_BYTES` (1,000,000 work, 16,000,000 bytes), constants that
began as metadata decoding bounds.

## Attribution

One analysis-only diagnostic of main `beabc1161` recorded every ledger charge by
`#[track_caller]` call site (CLI `8a8a7bba...`, never qualified). At the crossing
(1,000,002 work, 14,686,030 bytes) the totals were:

| Payer | Work |
| --- | ---: |
| Intrinsic-annotation owner-effect scans (`tracking.rs`) | 209,626 |
| Target-feature collection, 1,446 of 6,292 sites | 161,621 |
| Header definition rows (`bind_definition`) | 139,216 |
| MIR-boundary binding prepayment (definitions + 16 x annotations) | 125,566 |
| Header owner rows (`observe_selected_owner`) | 123,615 |
| Definition-binding prepayment (definitions + pointer markers) | 118,345 |
| Non-admitted owner checks | 77,268 |
| Legacy const-generic indices | 26,614 |
| Everything else | 18,131 |

838,381 units (84%) were spent before target-feature collection began. Those
payers are linear and visited once per owner, definition or effect. The repeated
work was in target-feature collection. 6,292 sites use only 85 distinct `enable`
values, and every site decoded, searched and implication-closed its value again.
Replaying the exact charges over the compiler's own census reproduces all 32
observed per-literal costs exactly. The full walk would cost 928,255 units.

## Repair: validate each spelling once

`collect` keeps a walk-local map from each plain string-literal `enable`
spelling, borrowed from the unit source, to its closed `FeatureSet` and its
explicitly named gated features. The first site decodes, validates and closes
the value as before, with the same diagnostics at its own entry. It then pays
the memo's growth before inserting. Each later site with that spelling still
does all of the following:

- pays two units per spelled byte to hash and compare its own literal;
- obtains its own crate gate result for each gated feature;
- keeps its own owner, function and safety checks;
- keeps its row, site and witness retention.

Raw, byte and nonliteral tokens keep the uncached path. Memoization removes
748,665 of the 928,255 parse units. Lookups cost 170,920 and the 85 first
validations cost 8,670.

The memo boundary is one collection walk; nothing is retained afterward. Work
can still multiply by the number of sites times literal length (linear), by
distinct spellings times names times the 77-entry search, and by closure size,
which is bounded by the 128-bit feature domain. Expected throughput relies on
the existing fast hash, which gives no worst-case bound against colliding
spellings.

## Input-graph bound

The local header ledger now snapshots an `InputBound` once in
`Declarations::initialize`, before any charge. The bound covers:

- definitions;
- every unit's owners, attribute effects, AST nodes and edges;
- retained attribute token trees and the bytes they spell.

Work is limited to 16 units and attempted storage to 128 bytes per element.
Failure stays sticky because charges are added before they are checked. The
configured `SourceContractLimits` and the observer profile are unchanged.
Errors now say `exceeds its input-graph bound`.

The factors were sized from one analysis-only measurement of the candidate,
which observed the bound without enforcing it (CLI `7fbc8cdc...`). Original
core has 3,389,691 graph elements before attribute bytes. Its header work through
collection and into `Checker::run` was 1,118,860 units and 20,507,722 bytes:
0.33 units and 6 bytes per element. Collection itself took 276,052 units.

The densest genuinely linear payer is a first-site literal validation, at about
13 units per spelled byte. Dense minimal target-feature functions retain about
49 bytes per element, including publication. A factor of 16 is the
declaration-checking precedent. A factor of 128 bytes leaves about 2.6x
headroom for those functions. Neither factor may be raised
to absorb repeated or superlinear work.

`Declarations` grows by 16 bytes, to 7,136, and `Headers` grows to 136 bytes.

## Controls

- Shared-spelling scaling: 1, 8, 32 and 128 sites of a 26-name value differ
  from a cheap value only by each site's own spelling read, with equal bytes.
  Reused rows are closed.
- Gate checks: 16 shared gated sites pay one gate result each.
- Duplicate attributes keep two sites.
- Distinct permutations: each pays at least full name and implication work.
- A later unknown feature still rejects at its own entry.
- An exact budget boundary admits or rejects the final charge.
- Paired linear inputs: 64 and 256 callees stay within half of their bound,
  and the bound scales with the graph. At 64 callees they use 4,486 of 48,304
  work and 148,184 of 386,432 bytes. Dense 7,000- and 28,000-item
  source headers stay within half of theirs.
- Adversarial superlinear control: one function with 512 sites called 512 times
  revalidates its whole row at each call. It fails with the bound error, while
  the 32 x 32 case passes.

## Re-probe

The qualified CLI (`ce4c689d...`, base `46b3e3bb0` plus this checkpoint) probed
unchanged original core once. The manifest, the six configuration-source pins,
cap-lints `allow` and the source-contract limits (14,500,624 work, 1 GiB
attempted, 64 MiB retained) were unchanged. LIB-193 is cleared. The first error
is now `file 97: 6032: object requires exactly one principal trait`, in 20.09 s
with 375 opened sources. File 97 is original `core/src/any.rs`, and byte 6032 is
the `Send` bound in `impl fmt::Debug for dyn Any + Send` at line 162. No
original artifact or whole assignment is complete.
