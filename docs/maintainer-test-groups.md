# Grouped maintainer integration tests

The 333 integration roots now compile as modules in 13 semantic test binaries.
Shared test helpers compile once per group. Test bodies, names within each
original suite, feature gates and ignore markers are preserved. Production
compiler sources and course fixtures are unchanged.

`scripts/maintainer_suites.py check` verifies a complete one-to-one mapping from
every root `dev/tests/*.rs` to an unconditional Cargo target. Missing, duplicate,
conditional and orphan registrations fail the harness. New roots must be added
to their owning group. Focused selection enumerates the actual binary, chooses
the exact outer module and passes complete names to libtest with `--exact`.
An unknown case or empty selection fails before test execution; failed inventory
commands and test failures retain their nonzero status. This avoids substring
collisions such as `loans` and `template_loans`.

The independent Rust qualification helper adds a per-test-thread directory below
its existing process directory. Suites sharing a process can otherwise reuse a
case name. An AST comparison of 1,059 functions across 226 existing files found
this to be the only changed function body; other changes register modules and
import shared helpers. Four obsolete dead-code expectations and two unused
imports were removed. No lint allowance or ignore marker was added.

## Qualification

The public PA24 compiler and tests at `66455db6` form the fixed baseline.
`obj/parallel/test-groups-v6/` in the validation checkout retains source hashes,
commands, timings and full logs. Its inventory comparison matches every test
name and result/ignore status in every one of the previous 333 integration roots.
All 2,329 regular functions and all 222 native functions pass. Formatting,
strict release all-target/all-feature Clippy and the 1,516-file audit pass.

The subsequent `test-groups-v7` gate confirms unchanged Rust/TOML hashes and
qualifies the final selector implementation. All 89 Python harness tests,
registration/layout checks and strict default-feature release Clippy pass.
Real binary controls select exactly six `loans` tests, one exact lexer test and
one ignored backend test. A deliberately unknown lexer case exits one without
running tests. Eight harness tests cover registration and selector failure
boundaries. Earlier failed preflights remain in the retained evidence.

The ordinary and all-feature checks use the same resource/storage controller:
three shared heavy jobs, two Cargo/test threads per job, 12 GiB memory per job,
50 GiB generated storage per checkout and a 150 GiB filesystem reserve. The full
successful run removed 6,633 disposable artifacts (12.38 GiB unshared blocks)
and losslessly archived one large diagnostic. Inputs, diagnostic bytes, receipts
and observations remain available.

## Observed development time

| Measurement | Previous separate binaries | Grouped binaries |
| --- | ---: | ---: |
| Integration binaries | 333 | 13 |
| Cargo-reported release build | 29m23s | 12m56s |
| Complete regular test command | 35m50s | 17m29s |
| Complete ignored native command | 7m48s | 5m34s |
| Passing regular functions | 2,329 | 2,329 |
| Passing native functions | 222 | 222 |

These are two recorded development runs on the shared machine, not an isolated
ABBA benchmark or a compiler-throughput claim. The implementation removes
repeated integration-crate/helper compilation while preserving coverage. Exact
elapsed seconds and log hashes are retained in the respective receipts;
`obj/parallel/test-build-investigation-v1/baseline.json` in the main checkout
binds the previous inventory and timing to the immutable PA24 release log.
