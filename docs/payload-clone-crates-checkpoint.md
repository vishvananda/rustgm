# Selected constants, enum storage, iterator cloning and public crate names

This checkpoint extends public `ce11c0b0` with selected constant patterns and
caller-owned type arguments, executable canonical slice-iterator Clone calls,
explicit-tag payload and C enum storage, unique tagless enum storage, and public
crate reexports. PA1–PA24 remain active; PA25–PA30 remain planned and incomplete.

## Qualification

The tested source commit is `b36c7e2a362ebaf2b8f8238e92af211b2d4b7e85`.
The immutable default-feature release compiler SHA-256 is
`f72651996370b67c3d2f1af68b8aa1baada6379206bc62e3128a63d563a64e56`.
Every recorded compiler, test and harness source hash remains unchanged.

The validation checkout retains `obj/parallel/payload-clone-crates-checkpoint-v3`:
2,669 regular tests, 278 native tests, strict release all-target/all-feature Clippy,
formatting, the 1,787-file architecture audit with zero findings, 90 harness tests,
registration, layout, upstream integrity and both backend capability checks pass.
All 8,362 active text cases pass with no changes to existing oracles. Two added
PA13/PA16 original source controls were independently qualified before scoped
reference generation; both use their explicitly declared LowIR modes.

The original full regular run found three stale assertions: two measured registry
sizes and one unpaid caller lifetime diagnostic. Production was unchanged while
the assertions were corrected. The entire 892-test private suite and the exact
remaining failing integration test passed afterward; all unaffected full-run
results are retained. A reviewed enum test rename preserves its original default
and explicit-payload sources, with dedicated payload validity controls added.
Original failed logs and reconciliation receipts remain intact.

All 24 affected independent course groups pass: PA6 source, PA13–PA14
source/native, PA15 source, and PA16–PA24 source/native. PA15 has no LowIR fixtures
and no empty native suite is counted. Original and new raw evidence is retained
under `obj/parallel/payload-clone-crates-independent-v1`.

## Performance

The main checkout's `obj/parallel/payload-clone-crates-performance-v1` compares
40 frozen inputs against the previous public compiler. All 16 shared inputs
preserve exact stdout and metadata bytes. The 24 newly supported inputs were
independently qualified first, including 21 native program executions; they make
no baseline speedup claim. Measurements retain A/A calibration, five ABBA blocks,
three executions per observation, eight RSS observations and Callgrind counts.

Shared instruction-count changes range from -1.21% to +1.36%, median -0.12%.
RSS changes range from -626 KiB to +418 KiB, median +134 KiB. No predeclared
instruction, RSS or scaling threshold was crossed. All twelve complete 8/32/128
series have incremental instruction ratios between 3.93 and 4.11 for fourfold
input increases; these observations do not establish every program's complexity.

Elapsed times increased in this run: shared median changes range from +0.22%
to +10.25%, with a median +3.71%. A/A spreads range from 0.70% to 28.78%.
The exclusive lease excludes registered RustGM jobs, but the retained host load
shows unrelated activity. That activity does not explain away the consistently
positive measured medians. Executable sections increased by 147,468 bytes;
this is an observed size cost, not a proven explanation for the elapsed changes.
Publication accepts these documented costs and makes no wall-time neutrality,
wall-time speedup or exact causal-attribution claim. No favorable rerun replaces
the observations.

## Remaining work

Mutable iteration and fixed-array/C-record metadata are qualified worker
handoffs for the next batch. Paid caller proofs, compact Option layout, ranges,
owned iteration, broader library hooks, generic crate bodies and the remaining
unsafe/intrinsic/foreign contracts are still underway. Upstream Serde inspection
also exposes additional library and macro-compatibility gaps; this checkpoint
makes no claim to compile Serde or a complete standard library.
