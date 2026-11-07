# Target unit executor

Lane D part 1 of the [core census plan](../core-census-plan.md). This checkpoint
adds orchestration only. It builds no original target crate, changes no compiler
code and does not activate PA34.

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
- **Mapped settings:** edition 2021/2024 becomes `--edition`, `lib`/`rlib`
  becomes `--crate-type=lib` with `--metadata-output`, and `bin` becomes
  `--crate-type=bin`. Features become `--cfg feature="..."`.
- **Script output:** the unit's own script `rustc-cfg` rows become `--cfg`,
  `rustc-env` rows and `OUT_DIR` become the process environment, and the Cargo
  package environment comes from the manifest.
- **Dependencies:** direct edges become `--extern alias=metadata`. The rest of
  the target closure becomes `--dependency-metadata`.
- **Sysroot modes:** the sysroot core gets `--source-core`, and every unit whose
  closure contains it gets `--source-client core@...`. Registry packages get
  `--cap-lints=allow`, as Cargo gives them.
- **Fixed profile:** settings must equal RustGM's fixed course profile: opt-level
  0, no LTO, debuginfo 0, debug assertions off, overflow checks on, panic abort.
  `--ub-checks=no` follows from debug assertions being off. Among rustflags,
  only `-Cpanic=abort`, which restates the fixed abort target, is accepted.
- **Typed gaps:** editions 2015/2018, dylib/cdylib/staticlib, other profile
  values or rustflags, `noprelude` edges, procedural macros, script link
  directives, missing script output and unreadable manifests.
- **Lint-only inputs:** cfg expectations, manifest lint tables and public or
  nounused edges are recorded per unit, not enforced; RustGM implements neither
  the `unexpected_cfgs` nor the private-dependency lint.

A unit with any gap fails before any process starts. A compile failure, a
missing or empty output, or a changed RustGM or package pin also fails. Rejected
outputs are renamed `*.rejected` and never consumed. Execution stops at the
first failure. Its transitive dependents are recorded as `blocked`, other later
units as `not_run`. The receipt records commands, environments, statuses,
output hashes and the implementation hashes. A graph with no target unit cannot
plan, and only `complete` means every unit built.

## Frozen original graphs

`pa34/tests/spec/stdlib-unit-graphs/` keeps the raw core/alloc/std unit graphs.
`scripts/stdlib_targets.py` verifies them against the frozen `graph_sha256`s
before binding packages. In all three graphs core is first, and it fails as
`unsupported` on `profile.debuginfo=2` and `profile.debug_assertions=true`: the
graphs were planned with Cargo's dev profile. Alloc blocks 2 units and std
blocks 12; no process starts. The gaps in the remaining std plan are:

- every unit: the same two profile values;
- `cfg_if`: edition 2018; `rustc_demangle`: edition 2015;
- `std`: the dylib crate type;
- `compiler_builtins`, `libc`, `std`: build-script output until the host graph
  runs.

`panic_abort`'s `-Cpanic=abort` is accepted. Supplying the frozen
compiler-builtins script output yields `--cfg intrinsics_enabled --cfg
mem_unaligned`, `OUT_DIR`, 137 recorded cfg expectations and its retained
`compiler-rt` metadata row.

## Checks

- `scripts/tests/test_target_units.py`: 11 tests. Exact frozen invocations, the
  complete std gap set, script consumption, typed gaps, empty graphs, and fake
  RustGM runs for completion, first failure, unsupported, missing output and
  changed pins.
- `make test-harness` (resource wrapper): 294 tests, 461 roots, layout pass.
  `make test-upstream` passes.
- `make test-target-units`, with dev-fast RustGM `284a7ef8…`, receipt
  `obj/qualification/target-units/1794028726686588237/result.json`. The
  closed-records library and binary build completely, with identical outputs on
  repeat. A rejected consumer stops after its library, and an edition-2018
  library fails before any process starts.
- The frozen run, `obj/parallel/target-units-stdlib-v1/report.json`, is
  described above.

## Remaining work

The executor cannot proceed until one of these is chosen: a RustGM-matching
custom profile, re-planned with `cargo_plan.py`, or debug-assertion and debuginfo
support. Wiring `host_graph.execute` outputs into a combined run, Rust 2015/2018,
the std dylib, lint enforcement, native linking of binaries and per-unit
timeouts beyond the shared 120 s command limit remain open. After the profile,
the next stop is original core itself: LIB-193.
