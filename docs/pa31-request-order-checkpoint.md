# PA31 cumulative request-ordering checkpoint

Qualified 2026-11-04. LIB-150's original-core request-ordering frontier is
cleared. This does not complete PA31 or produce a core, alloc or std artifact.
Root works alone in the existing btrfs checkout, with two build/test threads,
five-minute feedback, a 16GiB output cap and the 150GiB free-space reserve.

## One actual payer and shared kernel

Inventory completion already used the authenticated source contract payer, but
request ordering applied metadata's fixed 1M ceiling to the accumulated source
inventory steps. The source profile still has its original finite limits:
work14500624, attempted1073741824 bytes and retained67108864 bytes.

The existing iterative, fallible index-sort kernel now accepts its caller's
charge callback. Its metadata wrapper still uses the original Work and limits.
Source ordering borrows the existing authenticated payer and inventory history;
it creates no independent budget or permission. Comparisons pay visited bytes,
and index/swap work and attempted scratch are paid before mutation/allocation.
Scratch is checked in addition to the retained inventory, without releasing or
permanently increasing that inventory reservation. Every failed attempt remains
paid and sticky. Labels move only after all fallible index work completes.
Ordinary request lookup still charges its actual ordinary caller's cumulative
finite budget. No dense source/body/type records grow, and no source is reparsed.

The [ordering packet](../pa31/tests/spec/inventory-budget/ordering.json) freezes
the provider seed, generator, five source hashes and independent outcomes.
Its131072-function library fails in predecessor4a3107d3 at the metadata ceiling
and completes in candidate21c9b6ca. The public regression checks the actual
published inventory exceeds1M and contains all131073 functions; the same input
still fails with an exhausted1M source budget. Private checks interrupt every
paid ordering step and test unchanged unpublished labels, sticky retries,
attempted scratch, retained inventory and default metadata/lookup limits.
Those new private tests await successor full CI; the local gate does not run them.

## Previous full-CI regressions

[Full CI41900207733](https://github.com/vishvananda/rustgm/actions/runs/41900207733)
on75033d317 is terminal failure. All ten fundamental/forced-dyn private policy
checks pass, but79 other private tests and one linking test fail through one
overly narrow transport fence: it forgot Destruct's already encoded role flags.
The guard now consumes the actual typed catalog restriction; the existing
wire/profile validators still check those flags. General unencoded policy stays
fenced. The affected source-callable CLI export/consumer now passes locally.
Successor full CI must confirm the complete set; predecessor success is not
substitute evidence.

One additional frontend failure was a stale expected unsupported Tuple-header
boundary. The unchanged full-staged callback provider is independently valid
Rust and now completes checked lowering using its actual policies. Its test now
requires that success. No fixture source or independent outcome was weakened.

## Focused and independent evidence

The final gate passes13 exact public tests across frontend/linking in140.76s,
including format, architecture and strict selected Clippy. It covers actual
large inventory, exhausted source work, ordinary unknown requests, all five
feature-request tests, forty forced-dyn outcomes, typed cached diamond rejection,
ordinary feature authority, generic/native inputs, the full-staged callback
provider and the source-callable export/consumer regression. Source helpers are
registered once per integration binary, preserving the strict duplicate-module
lint. The initial duplicate helper failure and selector mistake remain recorded.

All five generated libraries independently pass pinned rustc1.98.0 (88d9e12ae).
The smaller four match predecessor names output byte-for-byte. Ordinary/value
LowIR, all fifteen frozen configuration values, six configuration source pins,
upstream checks and harness checks pass. The configuration output hash remains
86c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d.

The89 retained measurements include immutable A/A calibration, paired ABBA
blocks and output equality. Median B/A is1.004 ordinary,0.940 at1024 functions,
1.006 at32768 and0.981 at65536; A/A is0.941/1.137/1.005/1.002. This noisy sample
does not establish a speedup. Candidate source names-view medians are0.631s,
1.208s and2.460s at32768/65536/131072 functions. This finite linear scaling
sample is not whole-library compile time or RSS evidence. The newly accepted
largest input has no predecessor equal-output timing comparison.

## Original core and next requirement

CLI21c9b6ca8ffe074ff91da396bfbf1d06e65067e877a80e59cb3b36dad0813cfa
preserves all722 original source/package hashes, original options, Allow cap,
six configuration pins and finite limits. In18.616s/375 distinct opens it clears
request ordering and reaches `ERROR: file 0: 0: source lint policy storage budget
exceeded`. The observation reports no infrastructure failure and unchanged
inputs.375 opens does not establish a compiled artifact.

LIB-151 owns the actual source lint-policy storage frontier under PA40/31.
Identify the failing charge and actual capacity/payload axes before changing
accounting. Preserve allow/deny/expect, source/attribute identity and every
failed payment; use shared typed facts and real collection growth rather than
raising a guard or dropping source witnesses. Diagnostic-parity work unrelated
to compilation stays deferred. LIB-148 unused alias demand, LIB-149 custom auto
traits, Field/TransmuteFrom semantics, general imports and all remaining
assignment and original library requirements remain open.

An unchanged-binary GDB probe identifies the actual failing handoff:
`Report::complete_hidden_globs` passes the entire incoming import lineage
budget into `Producer::diagnostic_payload`. The transfer adds4069745 steps and
56637440 bytes to the lint producer, whose earlier charges are5094035 steps
and44446065 bytes. The resulting9163780 work is below10M, but101083505 bytes
exceed64MiB. This is not evidence that the retained diagnostic causes need that
much storage. Preserve the import payer and all failed attempts, then account
the actual new diagnostic work/witness storage under its proper owner. PA12
resolution provenance joins PA40/31 ownership. An immutable lint cap may suppress
unneeded diagnostics, but source/cap identity, deferred expectations and genuine
hard failures must not be bypassed. Broad warning-parity work stays deferred.

Both probes use this exact immutable CLI and original command/options, verifying
all722 inputs before and after; no rebuild, debug profile or limit change is
involved. Raw counters and call stacks are in `lint-trace-v1` and
`lint-transfer-v1`. The first probe's printed file/node labels are physical ABI
argument slots, not validated NodeKey field labels; its separate interpretation
records that limitation. The work/storage offsets and second probe's actual
transfer arguments are checked against the executable's disassembly.

## Retained receipts

- `obj/qualification/fast/1793823229178289040/receipt.json`:
  e0e76282af202bf3a7279f0c04a20f32711bbc67c148ccef4a8bfb2f4b639fd4.
- `obj/parallel/request-order-v1/reference-v1/receipt.json`:
  e1ed74e6148689d29788b253b1c18d48f149f3659b87875e6198ebb3b1d57574.
- `obj/parallel/request-order-v1/qualification-v1/receipt.json`:
  f501b4018f3ed8b4e1641684adb6d78dcf43b2a54593a09bb3601d7c36d96231.
- `obj/parallel/request-order-v1/measurement-v1/receipt.json`:
  1de21f1417f7b0b0eaa9dcb2b8d648a3da5ac6a785df07420fbe1e9e56dcb8fc.
- `obj/parallel/request-order-v1/lint-trace-v1/receipt.json`:
  91320aecf1fd29203b425d7998964a5ab542f2c14a8ae6e5dfe886f113ad29a8.
- `obj/parallel/request-order-v1/lint-transfer-v1/receipt.json`:
  3e34df4e83f20da2905fa43180b7e723019aa8bf4aa020dbe26a2871db08a00f.

Failed predecessor discovery, CI logs and all raw observations stay in the same
scratch root. Current/predecessor immutable CLIs are21c9b6ca/4a3107d3; superseded
pins are retired with receipts after publication. No root build output or new
worktree is created.
