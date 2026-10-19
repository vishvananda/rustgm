# Repository guide

RustGM owns both the full maintainer frontend and its assignments. Read
`spec.md`, `ROADMAP.md`, the owning handout and `TESTING_AND_REFERENCES.md`.

- Compiler code belongs in `dev/src/`; CLI, environment and process boundaries
  belong in `dev/bin/` or `scripts/`. Use idiomatic Rust names and safe code.
- Put fixtures in the earliest `paN/tests/` that owns every feature they use.
  Every input must appear in the assignment's manifest. Preserve upstream
  attribution, original bytes and `third_party/sources.json` checksums.
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
- Follow the checkpoint cadence in `TESTING_AND_REFERENCES.md`. During related
  implementation edits, run focused tests and independent rustc/native probes,
  then lint/audits early enough to catch wiring errors before expensive suites.
  Keep all active PA text suites as the cheap candidate regression check.
- Batch related edits into coherent checkpoints. Before pushing compiler changes,
  run the full release Cargo suite, native maintainer suite, lint/audits/harness,
  all active PA text suites and affected independent source/native suites.
  Run `make test-upstream` after fixture changes and `make test-backend` after
  changing the backend contract or bridge. Assignment releases additionally run
  all active independent suites and their owning handout's release checks.
- Keep generated builds, logs and measurements in `obj/`, out of commits.
  Inspect Git state from the repository root. Commit only relevant work.
- Measure hot-path changes using immutable binaries, frozen inputs, A/A
  calibration and paired ABBA runs; preserve observations and output equality.
  During development, select changed-path scaling inputs and an ordinary control.
  Run the broad matrix at assignment release, for shared algorithm changes or
  when a regression needs investigation; do not repeat it for every related edit.
  Once checks pass, rerun only those invalidated by further changes or failures.

Clippy all/pedantic warnings are errors. Avoid unwrap, expect, panic, todo,
unimplemented and process exit in production. Narrow lint expectations require
reasons; blanket allowances and mechanical file splitting are prohibited.
The syntax-aware audit additionally checks file/function size, nesting, unsafe
code, duplicated blocks, phase dependencies and misplaced environment access.

## Generated storage

- Check free space before every build, validation batch or benchmark. Use the
  Makefile targets or `python3 scripts/storage.py run -- COMMAND` for direct
  Cargo/test/measurement commands; use `--no-prune` before `--` for build-only
  commands. The wrapper reserves 40 GiB locally and disables incremental caches.
  GitHub Actions defaults to an 8 GiB reserve. Do not lower the local reserve to
  work around exhaustion: clean generated output first.
- Successful wrapped runs remove generated executables, object/metadata files
  and large rendering scratch files. Source inputs, diagnostics, receipts and
  raw timing observations remain. Failed runs retain artifacts for diagnosis;
  prune them after investigating. Never accumulate a fresh binary per historical
  test case or benchmark observation indefinitely.
- When a script puts all disposable output in known directories, pass repeated
  `--prune-under obj/path` options before `--` to clean those directories without
  rescanning historical runs. Declare every output root; retain the default full
  scan for commands with scattered outputs. Scoped cleanup preserves pins,
  caches, symlink protections, the reserve and the exclusive lock.
- Use `make tidy` between work batches; `make tidy-build` also removes the
  rebuildable Cargo cache. Cleanup refuses active compiler/test executables and
  shares an exclusive lock with wrapped commands. Do not run cleanup during
  measurements, and do not modify another checkout's outputs.
- Put `.keep-artifacts` in the directory of the immutable baseline/candidate
  currently needed for comparison. Remove superseded pins after publishing a
  replacement. Keep only current baselines; historical receipts and hashes do
  not require retaining every old executable. Do not treat a pruned binary as
  available evidence for a new run: rebuild or restore it and verify its hash.
