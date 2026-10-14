# RustGM

A cumulative Rust compiler course and its maintainer reference implementation.
The destination is Rust source to typed LowIR. CPPGM supplies the LowIR reader,
validator and native backend during frontend development.

This is the **solution repository**, following cppgm-extended's arrangement:
`dev/` owns the implementation; `paN/` owns handouts, fixtures and committed
text oracles. The student export will be a separate, deliberately incomplete
repository with pinned reference tools. It has not been built yet.

## Current state

- PA1 has a working safe Rust lexer, its draft handout and an executed oracle
  suite with original and pinned upstream fixtures.
- PA2 has typed literal decoding, target numeric conversion, a draft handout
  and text oracles independently checked against rustc values and rejections.
- PA3 has flat token trees, doc/attribute syntax, typed configuration facts,
  custom cfg options and independently checked text oracles.
- PA4 has a bounded expression/statement parser, dense typed syntax arenas,
  macro captures and independently checked text oracles.
- PA5 has crate/item/type/pattern syntax, attribute ownership and configuration
  traversal that excludes disabled nodes before literal/semantic work.
- PA6 has owned module loading, NFC name interning, lexical scopes, privacy,
  namespace-directed references and imports with independently checked text oracles.
- PA7 has ordered macro matching, cached fragment parsing, typed transcription
  and independently checked repetition/follow-set/ambiguity oracles.
- PA8 has worklist expansion, mixed-site hygiene, macro/name resolution, eager
  builtins, cached source inclusion and independently checked text oracles.
- PA9 has canonical types, signature elaboration, scalar inference/coercions,
  selected operation/control facts and independently checked type-view oracles.
- PA10 has typed function/initializer CFGs, explicit storage and evaluation order,
  text oracles and independent CFG execution checks against rustc.
- PA11 lowers checked scalar Rust procedures into typed LowIR, with committed
  text oracles and independent native execution through the pinned CPPGM backend.
- PA12 adds concrete owned aggregates, discriminants, checked indexing and
  aggregate calls, with layout vectors and independent native execution checks.
- PA13 adds selected patterns, coverage, owned decisions, conditional bindings,
  guard retries and constant patterns, with 796 text-oracle cases. Borrowed,
  text and dynamic-slice inspections retain explicit pending execution proofs.
- PA14 publishes move/initialization and Copy/Clone proofs through ownership
  views, with 124 course cases and independent native controls.
- PA15 proves lifetime contracts and exposes region identities, assumptions and
  requirements, with 191 independently checked text-oracle cases.
- PA16 checks loans, storage and non-lexical lifetimes and lowers checked
  references, with 293 cases including 113 native executions.
- PA17–PA38 have buildout briefs and Makefiles. Their suites and implementation
  are not complete; invoking an unbuilt assignment fails explicitly.

Read [ROADMAP.md](ROADMAP.md), [spec.md](spec.md),
[TESTING_AND_REFERENCES.md](TESTING_AND_REFERENCES.md) and
[the upstream test research](docs/upstream-tests.md).
The [Rust 1.98.0 Reference](doc/README.md) is vendored for offline reading.

## Build and validate

```sh
make
make test
make test-report
make -C pa1 test
make lint
make setup-audit
make audit-rust
make test-harness test-upstream
```

Rust 1.98.0 is pinned by `rust-toolchain.toml`. The course language profile is
the same release, initially edition 2024, Linux x86-64. Frontend dependencies supply Unicode XID property tables and NFC normalization;
they do not implement Rust frontend phases.
See [dependency review](docs/dependencies.md).

Run `dev/rustgm --emit-tokens source.rs`, `dev/rustgm --emit-literals source.rs`,
`dev/rustgm --emit-token-trees source.rs`, or `dev/rustgm --emit-ast expression.rs`, or `dev/rustgm --emit-crate-ast crate.rs`, or `dev/rustgm --emit-names crate.rs`,
or `dev/rustgm --emit-types crate.rs`, `--emit-layouts`, `--emit-body-ir`, `--emit-ownership`,
`--emit-regions`, `--emit-loans` or `--emit-lowir`,
or pass source on stdin. The reference
wrapper `dev/rustgm-ref` invokes the same maintained solution. Override
`RUSTGM_TEST_APP=/absolute/path/to/candidate` to test another implementation
against the committed oracles without regenerating them.

## CPPGM backend

Configure a built checkout of the commit in `backend.lock.json`:

```sh
python3 scripts/backend.py --configure /path/to/cppgm-extended
make test-backend test-native
dev/rustgm --emit-lowir -o obj/program.lowir program.rs
python3 scripts/backend.py obj/program.lowir -o obj/program
```

The initial lock records the qualified Linux binaries as well as their source
commit. The local configuration is ignored by Git; it contains machine paths.
The bridge checks binary hashes before use, writes only under this repository,
and never rebuilds or edits CPPGM. A different host build needs an explicit
reviewed lock update and backend validation, not a silent repin.

## Growing an assignment

Develop the handout, maintainer implementation and tests together. Declare
expected success/rejection independently of generated output; generate text
references with `make ref-test-paN`, review the diff, cross-check against the
pinned Rust toolchain, and preserve every earlier active suite. Only then add
the assignment to the accepted report. Do not mistake a skeleton for a pass.

`make perf` retains raw time/RSS samples and verifies deterministic token
counts on scaled inputs. Performance changes require frozen A/B binaries and
the paired measurement procedure in `spec.md`; a single timing is not a gate.
`make perf-literals` measures the literal interpreter and suffixed float
conversion with the same procedure, including full rendered-output checks.

`make perf-ast` measures expression/statement parsing with frozen inputs, full
output equality and the same paired scaling procedure.

`make perf-crate-ast` measures complete crate syntax and configuration with the
same frozen-snapshot procedure.

`make perf-names` measures source ownership, modules, interning and resolution,
including glob propagation, with deterministic full-output checks.

`make perf-types` measures the expanded/resolved scalar frontend through type
checking with immutable binaries, full-output equality, A/A calibration and
paired scaling runs. See [PA9 validation](docs/pa9-validation.md).

`make perf-layouts` measures cached layout inspection at 1k/4k/16k nominal
declarations and array aliases, with frozen inputs and full-output checks.
See [PA12 validation](docs/pa12-validation.md).

`make perf-lowir` measures demanded scalar lowering with frozen sources, full
output equality, A/A calibration and paired blocks. See
[PA11 validation](docs/pa11-validation.md) for the executable profile and evidence.
