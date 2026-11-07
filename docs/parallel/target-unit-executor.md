# Target unit executor

Lane D of the [core census plan](../core-census-plan.md). This checkpoint adds
orchestration only. It builds no original target crate, changes no compiler code
and does not activate PA34. Part 1 added the executor; part 2 re-planned the
frozen graphs under RustGM's profile and wired in the host build scripts.

## Interface

`scripts/target_units.py` takes a decoded frozen `build_units.Plan`, each
package bound to its authenticated identity and current root, the optional
sysroot core package, and completed host-graph script outputs. `plan_units`
returns one step per target compile unit, in dependency order. Each step holds
its exact RustGM arguments, environment, planned outputs and typed gaps.
`execute` runs the steps under a declared RustGM SHA-256 and writes a receipt.

Setting rules:

- **Identity:** the crate name comes from the target. The disambiguator `h` plus
  20 hex digits hashes the package identity and inventory, target, edition,
  features, profile, dependency identities and script output key. The unit's own
  paths and row indices do not enter it; a script key comes from the host graph.
- **Mapped settings:** edition 2015/2018/2021/2024 becomes `--edition`, `lib`/`rlib`
  becomes `--crate-type=lib` with `--metadata-output`, and `bin` becomes
  `--crate-type=bin`. Features become `--cfg feature="..."`.
- **Script output:** the unit's own script `rustc-cfg` rows become `--cfg`,
  `rustc-env` rows and `OUT_DIR` become the process environment, and the Cargo
  package environment comes from the manifest.
- **Dependencies:** direct edges become `--extern alias=metadata`. The rest of
  the target closure becomes `--dependency-metadata`.
- **Sysroot modes:** the sysroot core gets `--source-core`, and every unit whose
  closure contains it gets `--source-client core@...`.
- **Lint caps:** as Cargo without `-vv` and bootstrap pass them, registry
  packages get `--cap-lints=allow` and local path packages are uncapped. Each
  step records its cap. `lint_cap='allow'` (`--analysis-cap-lints allow`) caps
  local packages too, for analysis only, and is recorded in the receipt.
- **Fixed profile:** settings must equal RustGM's fixed course profile: opt-level
  0, no LTO, debuginfo 0, debug assertions off, overflow checks on, panic abort.
  `--ub-checks=no` follows from debug assertions being off. Among rustflags,
  only `-Cpanic=abort`, which restates the fixed abort target, is accepted.
- **Typed gaps:** editions 2015/2018, dylib/cdylib/staticlib, other profile
  values or rustflags, `noprelude` edges, procedural macros, script link
  directives, missing script output and unreadable manifests.
- **Lint-only inputs (interim gap):** cfg expectations, manifest lint tables and
  public or nounused edges are recorded per unit, not enforced. RustGM implements
  neither `unexpected_cfgs` nor the private-dependency lints, and has no switch
  for bootstrap's deny-warnings. Each receipt states this in `lint_policy`.
- **Timeouts:** every compile keeps the shared 120 s limit unless the caller
  names that crate with an explicit longer one (`timeouts`, `--unit-timeout
  core=SECONDS`). Every command row records its limit; nothing extends silently.

A unit with any gap fails before any process starts. RustGM exit 1 fails as
`compile` and exit 2 as `usage`, each with RustGM's first `ERROR:`/`USAGE:` line.
Any other status or signal is `abnormal`, and an expired limit is `timeout`. A
missing or empty output, or a changed RustGM or package pin, also fails. Rejected
outputs are renamed `*.rejected` and never consumed. Execution stops at the
first failure. Its transitive dependents are recorded as `blocked`, other later
units as `not_run`. The receipt records commands, environments, statuses,
output hashes and the implementation hashes. A graph with no target unit cannot
plan, and only `complete` means every unit built.

## Course-profile graphs

`scripts/stdlib_plan.py` re-plans core, alloc and std through `cargo_plan.py`.
`Job.profile` passes explicit, closed `CARGO_PROFILE_DEV_*` settings: opt-level
0, debug 0, debug assertions off, overflow checks on, panic abort. Upstream
bootstrap ships std without debug assertions or debuginfo, so those two match
it. Bootstrap also turns overflow checks off and optimizes, but RustGM fixes
overflow checks on and has no optimizer; those two settings are RustGM's. The
release profile is not used, because the workspace manifest overrides
`compiler_builtins` (10,000 codegen units) and `rustc-demangle` (opt-level "s")
there.

The declaration preceded planning. Inputs are authenticated in a fresh tree:
the 30 registry archives by their frozen hashes, the 19 local inventories and
the workspace manifest and lockfile. Each new graph matches its declaration and
equals its historical graph in every field except the profile.
[The observation](../../pa34/tests/stdlib-course-profile-outcomes.json) pins the
original observations, tools, declaration and implementation. The raw graphs
are in `pa34/tests/spec/stdlib-unit-graphs/course-profile/`. The dev-profile
graphs and hashes stay in `dev-profile/` as history.

With `--archives`, `scripts/stdlib_targets.py` does one combined run. It
authenticates the closure, relocates each graph to the current roots, records
two pinned rustc `--print=cfg` contexts and runs every host unit through
`host_graph.execute`. The target context is panic abort with debug assertions
off, and registry build scripts are capped like Cargo caps them. The outputs must
reproduce the frozen observations: compiler-builtins byte for byte against its
template, and libc and std by directive count (141, 27 and 8). Target units then
consume them: libc gets `--cfg freebsd12`, std `--cfg backtrace_in_libstd` and
`STD_ENV_ARCH`, and compiler-builtins its two cfgs and `OUT_DIR`.

## Current results

The run uses main `219c13b80` CLI `7051deee…`, with no profile gap left.

- **Core, Cargo lint policy:** core is uncapped and fails as `compile` after
  0.02 s: `strict provenance lint producer is not implemented` at file 0 byte
  3519, which is `#![deny(implicit_provenance_casts)]` in `core/src/lib.rs`. The
  core census reaches its declarations only because it passes `--cap-lints=allow`.
- **Core, `--analysis-cap-lints allow`:** after 7.6 s, `source macro contracts
  exceed their work budget` at file 179 byte 9035, which is `net/ip_addr.rs` in
  the census open order. The executor passes RustGM's default source-contract
  limits; the census passes larger explicit ones.
- **Alloc:** with real compiler-builtins output, core, compiler_builtins and
  alloc have no gaps.
- **Std:** the remaining gaps are cfg_if's edition 2018, rustc_demangle's edition
  2015 and std's dylib crate type. The first, in dependency order, is cfg_if
  (unit 1).

Evidence is in `obj/parallel/target-unit-replan-v1/`: `course-plan-v3/`,
`core-run/`, `core-run-capped/` and `combined-plan/`.

## Checks

- `scripts/tests/test_target_units.py`: 17 tests. They cover:
  - both graph sets;
  - the profile-only difference from history;
  - exact core and cfg_if invocations;
  - the course-profile std gaps;
  - relocation;
  - checked and consumed host outputs;
  - caps and timeouts;
  - fake-RustGM completion, first failure, timeout, usage, unsupported units,
    missing outputs and changed pins.
- `scripts/tests/test_build_units.py` covers the explicit, closed and recorded
  dev-profile settings.
- `make test-harness` (resource wrapper): 301 tests, 461 roots and the layout
  check pass. `make test-upstream` passes.
- `make test-target-units` with CLI `7051deee…` passes and is reproducible.
  Receipt: `obj/qualification/target-units/1794030643608179103/result.json`.

## Remaining work

- **Lints:** the strict-provenance lint producer, or a reviewed decision to cap
  original core, stands in front of core under Cargo's policy. Lint enforcement
  remains the recorded interim gap.
- **Source-contract limits:** resolved by the
  [input-proportional budget](macro-budget.md). Default limits now admit core
  past `ip_addr.rs`; its first gap is `flatten.rs` 8286.
- **Compiler support:** Rust 2015/2018, the std dylib and native linking of
  binaries.
- **Out of scope here:** the full source closure outside package roots (stdarch,
  backtrace) is pinned only by the original source inventory.
