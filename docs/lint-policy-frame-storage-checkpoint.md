# Lint policy frame storage checkpoint

LIB-046 remains open. The original core failure is storage exhaustion, and a
proved frame-capacity overcharge is now repaired without changing the10M work
or64MiB storage limits. The latest unchanged-core attempt passes the old
`num/saturating.rs` boundary and fails later in `ops/bit.rs` during
`forward_ref_binop` inside `bitand_impl`. No core artifact exists.

## Attribution and repair

The precise exhaustion diagnostic uses the existing Work counters and distinguishes
work, storage and combined exhaustion. No production debug field, global owner,
environment switch or new budget was added. CLI `d940c7f2` independently confirms
storage-only exhaustion at the original saturating boundary in8.25s, with all722
source hashes unchanged and371 files opened. Its result is under
`obj/parallel/lib046-lint-budget-v1/attribution-core` in the root-owned
`rustgm-pa25-work` checkout.

The source regression generates sibling functions through one real macro under
one actual root lint owner. Before the repair, width1 records6 frame publications,
81 work steps and2568 logical bytes. Width8 has the same owner/call count but
charges3240 bytes, failing the declared constant-depth storage invariant. The
672-byte difference is21 reused frame publications at32 bytes each. The failed
baseline is retained as failed, not counted as a successful gate.

`truncate` already retained the frame vector's capacity. `set_frame` now uses the
existing capacity-growth helper to prepay new slots and copy work only when that
capacity grows. Every publication still pays traversal work and validates its
actual tail. A failed growth payment cannot publish either the row or its frame
extent, and failed retries remain charged. Distinct owners, call sites and full
attribute witnesses retain their existing conservative accounting and provenance.

The same actual-source control now observes:

| Siblings | Owners | Frame publications | Calls | Work steps | Logical policy bytes |
| --- | --- | --- | --- | --- | --- |
| 1 | 1 | 6 | 1 | 81 | 2408 |
| 8 | 1 | 27 | 1 | 200 | 2408 |
| 32 | 1 | 99 | 1 | 608 | 2408 |
| 128 | 1 | 387 | 1 | 2240 | 2408 |
| 1024 | 1 | 3075 | 1 | 17472 | 2408 |

These are the existing lint payer's logical counts, not whole-compiler memory,
allocator telemetry or a wall-time performance comparison. Source, syntax and
name storage have separate owners. No common record gained a field.

## Qualification and current core boundary

- The failing source baseline is receipt1793116310576083780, SHA-256
  `6fbc13599122e244d8a6c982f14bd44b1ff8f6609facba18a7bfbcb45c6bc042`.
- All eight affected private controls, strict private Clippy, formatting and
  global architecture pass in123.73s: receipt1793116590159540594, SHA-256
  `ea56513edfc22b6438b07d6918ef9b71e5a93f88f3fbe2b182f6b37c970e27ad`.
  They include combined/failed budget opponents, actual changed-source/effect/
  capture rejection and ordinary attribute/permission controls.
- A source-identical existing-binary observation retains all five scaling rows
  in `obj/parallel/lib046-lint-budget-v1/frame-costs/receipt.json`, SHA-256
  `2133b59ff8015d1f1c6c460cfaace15a612834b03dfad16aa8a9c5f6c5742845`.
- Production/CLI strict Clippy, formatting/global audit, twenty exact ownership/
  lexer controls and all120 PA8 text cases pass in89.54s:
  receipt1793116797887791528, SHA-256
  `50e7503d016cecab2fc3331fd2d0670914f076f9676772ae079f7dac8190e3f6`.
  All2948 source fingerprints were rechecked against the final source.

The same-source immutable CLI has SHA-256
`9d3ea4f683a3e16b96f9d3532f1f0749ec7f73d1c555aabe506312f4d7d10c24`.
Its binding is `obj/parallel/lib046-lint-budget-v1/frame-cli/binding.json`,
SHA-256 `faed9c5f36301f51c727359b32410f78589550f4fb251b112d2dcb5d3220768f`.
The ordinary original-core observer reuses the same source tree/profile and
verifies every one of722 inputs. It opens371 files and exits after7.87s with
storage exhaustion at `ops/bit.rs`, in the actual bitand macro family. No source
was patched and this is not an infrastructure failure. Result SHA-256:
`b8c2cf82d582547f4f8cd2a60cab83726fa5f0385300a64bc093f97059691a3a`.
The different observation times are not an A/B speed claim or completion percentage.

The next step is a separately bound owning-engine counter/capacity observation
before diagnostic retention. Measure attempted cumulative charges and actual
retained collections separately; a failure snapshot excludes already-dropped
scratch. Choose any further representation change from those measurements.
Keep positive-empty nonlint records, exact source/effect/capture/parent checks
and the existing limits. Warm policy composition is a separate issue whose
special diagnostic call paths have not been shown causal here.

Full CI40533278124 exposed a missing optional assembly field in an old guard-view
test fixture. That fixture now explicitly sets `assembly: None`; all affected
ownership controls above pass. The previous CI run remains failed and needs
fresh full qualification of this corrected cut.
