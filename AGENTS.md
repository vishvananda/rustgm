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
- After changes run affected course tests, `make lint audit-rust test-harness`
  and all active earlier stages. Run `make test-upstream` after fixture changes,
  and `make test-backend` after changing the backend contract or bridge.
- Keep generated builds, logs and measurements in `obj/`, out of commits.
  Inspect Git state from the repository root. Commit only relevant work.
- Measure hot-path changes using immutable binaries, frozen inputs, A/A
  calibration and paired ABBA runs; preserve observations and output equality.

Clippy all/pedantic warnings are errors. Avoid unwrap, expect, panic, todo,
unimplemented and process exit in production. Narrow lint expectations require
reasons; blanket allowances and mechanical file splitting are prohibited.
The syntax-aware audit additionally checks file/function size, nesting, unsafe
code, duplicated blocks, phase dependencies and misplaced environment access.
