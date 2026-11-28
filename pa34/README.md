# RustGM Programming Assignment 34 — Editions and crate builds

Run the grading suite with `make -C pa34 test` (48 edition and mixed-crate
cases covering Rust 2018/2021/2024 semantics plus the host `build.rs`
generated-source, `rustc-cfg`/`rustc-env`, mixed-edition, and replay check in
`pa34/tests/build-driver/`).

Implement reproducible crate builds for the course's Rust 1.98.0 graph.
Extend [PA32](../pa32/README.md)'s build preparation to dependency graphs.
Start with small library and client graphs using [PA29](../pa29/README.md)'s
artifact and dependency identities.

## Build graph

- Resolve the selected manifests, lockfile, dependency renames, features,
  and target configuration. Carry each crate's edition through parsing,
  expansion, and metadata.
- Support the selected 2021 and 2024 crates. The original standard-library
  graph also uses a 2015 `rustc-demangle` and a 2018 `cfg-if` package.
  Support the edition behavior they require. The follow-on track covers
  the remaining edition rules.
- Preserve the edition of a method identifier from its source or macro
  origin. Edition-sensitive dispatch for arrays and boxed slices uses
  that identifier and the adjusted receiver. Apply
  `rust_2021_incompatible_or_patterns` to pre-2021 macro definitions whose
  inferred `pat` fragment can be followed by `|`. An explicit `pat_param`
  preserves the single-alternative behavior across editions.
- Build host tools and target crates as separate graph units. Use an
  explicitly pinned host toolchain for build-time tools. Compile every
  target Rust dependency with RustGM.
- Run build scripts in their selected host and target contexts. Retain
  generated files, configuration flags, environment values, link
  directives, and dependency link metadata as explicit inputs to consumers.
  Reuse a host script build when its build inputs match. Run it separately
  for distinct contexts that affect its output.
- Retain per-crate compiler options, lint levels, crate types, panic
  settings, target features, and artifact dependencies. Keep orchestration
  outside the typed frontend phases.

## Tool surface

Accept explicit `--edition=2021` and `--edition=2024` on the compiler,
keeping earlier input and output options. Preserve explicit configuration
and runtime-check settings when selecting an edition. The driver builds
the selected graph and passes upstream metadata and generated inputs to
each target unit.

## Check your implementation

Use a small renamed-dependency graph, a package that generates source,
and mixed editions to test the driver before building the upstream
packages in [PA35](../pa35/README.md).
Freeze package bytes and versions, compilers, features, configuration,
generated files, and artifact identities. Replay the selected graph
offline after acquiring its sources. Check that the replay builds the
same target units and that their linked consumers run.
