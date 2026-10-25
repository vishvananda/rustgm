# Repository guide

RustGM owns both the full maintainer frontend and its assignments. Read
`spec.md`, `ROADMAP.md`, the owning handout and `TESTING_AND_REFERENCES.md`.

- Compiler code belongs in `dev/src/`; CLI, environment and process boundaries
  belong in `dev/bin/` or `scripts/`. Use idiomatic Rust names and safe code.
- Put fixtures in the earliest `paN/tests/` that owns every feature they use.
  Every input must appear in the assignment's manifest. Preserve upstream
  attribution, original bytes and `third_party/sources.json` checksums.
- Register each maintainer integration root in its owning `dev/tests/suites/`
  group. Use `scripts/maintainer_suites.py run SUITE` for exact focused selection;
  `make test-harness` rejects missing or duplicate registrations.
- Never hand-edit generated references or bless outputs just to hide a failure.
  Declare expected outcomes before generating text oracles. Reference updates
  are authorized when developing a new contract, but must be independently
  checked and reviewed with implementation and fixtures.
- Planned assignments must fail, not return empty-suite success. Exit 86,
  signals, timeouts and panics are infrastructure/implementation failures.
- Reuse typed phase results. Do not call rustc, CPPGM or another frontend to
  implement Rust lexing, parsing, semantics or Rust-to-LowIR lowering.
  CPPGM is expressly allowed as the external LowIR-to-native backend and in
  test execution. Keep that bridge outside the Rust frontend library.
- Read CPPGM's specification and implementation for performance guidance;
  `docs/cppgm-spec.md` and `shared/CPPGM-NOTICE` record the initial baseline.
  Do not modify the active C++ or v5rust checkouts while working here.
- The development feedback budget is five minutes, including resource admission.
  Use `make fastcheck FAST_ARGS='SUITE --lib-case full::test::path'`: cached
  unoptimized builds without debug symbols, exact affected regressions, full
  formatting, content-cached architecture audit and strict Clippy for selected
  targets (including the CLI for integration/text selections). Unknown/empty
  selections and timeouts fail. Do not run a
  release build, all-target compilation, full private inventory, broad native
  sweep or performance matrix for each edit/new binary. Do not duplicate Clippy
  with `cargo check`. Settle related edits and choose controls before running.
- Batch related edits into coherent checkpoints. Focused passing evidence is
  sufficient to integrate/push development work; retain honest pending checks.
  Full release regular/native tests, uncached audits, all-target strict Clippy,
  harness, course text and independent suites run in scheduled/manual CI.
  Run `make test-upstream` after fixture changes and `make test-backend` after
  changing the backend contract or bridge. Assignment releases additionally run
  all active independent suites, full debug tests and their owning handout's
  release checks. `make test-maintainer` combines full release regular/native
  execution; do not repeat its library tests in a separate full private pass.
- Keep generated builds, logs and measurements in `obj/`, out of commits.
  Inspect Git state from the repository root. Commit only relevant work.
- Measure hot-path changes using immutable binaries, frozen inputs, A/A
  calibration and paired ABBA runs; preserve observations and output equality.
  During development, select changed-path scaling inputs and an ordinary control.
  Run measurements at meaningful algorithm checkpoints and the broad matrix at
  assignment release or when a regression needs investigation. During ordinary
  edits prefer bounded-work/scaling invariants over repeated profiling builds.
  Once checks pass, rerun only those invalidated by further changes or failures.

Clippy all/pedantic warnings are errors. Avoid unwrap, expect, panic, todo,
unimplemented and process exit in production. Narrow lint expectations require
reasons; blanket allowances and mechanical file splitting are prohibited.
The syntax-aware audit additionally checks file/function size, nesting, unsafe
code, duplicated blocks, phase dependencies and misplaced environment access.

## Generated storage

- Check free space before every build, validation batch or benchmark. Use the
  Makefile targets or `python3 scripts/storage.py run -- COMMAND` for direct
  Cargo/test/measurement commands; `make fastcheck` supplies its own guards.
  Use `--no-prune` before `--` for other build-only commands. The wrapper reserves
  40 GiB locally and disables incremental caches outside `dev-fast`. Fastcheck
  keeps one bounded incremental cache under `obj/rust/dev-fast`; shared output
  caps include that cache. Retire it with `make tidy-build` when obsolete.
  GitHub Actions defaults to an 8 GiB reserve. Do not lower the local reserve to
  work around exhaustion: clean generated output first.
- Successful wrapped runs remove generated executables, object/metadata files
  and large rendering scratch files. Source inputs, diagnostics, receipts and
  raw timing observations remain. Failed runs retain artifacts for diagnosis;
  prune them after investigating. Never accumulate a fresh binary per historical
  test case or benchmark observation indefinitely.
- Cleanup losslessly compresses large rustc `*.long-type-N.txt` diagnostics to
  `.txt.gz`, verifies the restored bytes before removing the original, and records
  their SHA-256s. Other diagnostic text stays in place; use gzip to read an archive.
- When a script puts all disposable output in known directories, pass repeated
  `--prune-under obj/path` options before `--` to clean those directories without
  rescanning historical runs. Declare every output root; retain the default full
  scan for commands with scattered outputs. Scoped cleanup preserves pins,
  caches, symlink protections, the reserve and the exclusive lock.
- Use `make tidy` between work batches; `make tidy-build` also removes the
  rebuildable Cargo cache. Cleanup refuses active compiler/test executables and
  shares an exclusive lock with wrapped commands. Do not run cleanup during
  measurements, and do not modify another checkout's outputs.
- Use `make tidy-archive` periodically to compact inactive PID-named scratch
  directories losslessly. It verifies restored file hashes before removing the
  originals, preserves pins/managed caches and writes a restore index under
  `obj/archives/`. Thousands of retained input directories make accounting scans
  slow even when generated binaries have already been pruned. With the shared
  controller, pass a justified cap via `ARCHIVE_RESOURCE_ARGS` as for other work.
- Put `.keep-artifacts` in the directory of the immutable baseline/candidate
  currently needed for comparison. Remove superseded pins after publishing a
  replacement. Keep only current baselines; historical receipts and hashes do
  not require retaining every old executable. Do not treat a pruned binary as
  available evidence for a new run: rebuild or restore it and verify its hash.
  After retiring a pin, explicitly prune its directory with
  `python3 scripts/storage.py prune --apply --prune-under obj/path`.
  Automatic post-run cleanup only considers files written during that run;
  removing a pin does not make an older executable eligible for that cleanup.

## Parallel implementation

When the local shared resource controller is configured, follow
[docs/parallel-work.md](docs/parallel-work.md). Every build, test, probe and
benchmark must use the storage/resource wrapper. Worktree isolation does not
provide a separate CPU, RAM or filesystem budget. Worker ownership, model
settings, queued assignments and integration requirements are in that document.
