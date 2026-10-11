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
- PA2–PA38 have buildout briefs and Makefiles. Their suites and implementation
  are not complete; invoking an unbuilt assignment fails explicitly.
- The backend bridge has an executable LowIR smoke test. Rust source lowering
  begins at PA11; the smoke test is not evidence of Rust source compilation.

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
the same release, initially edition 2024, Linux x86-64. The only frontend
dependency initially supplies Unicode XID property tables; it is not a lexer.
See [dependency review](docs/dependencies.md).

Run `dev/rustgm --emit-tokens source.rs`, or pass source on stdin. The reference
wrapper `dev/rustgm-ref` invokes the same maintained solution. Override
`RUSTGM_TEST_APP=/absolute/path/to/candidate` to test another implementation
against the committed oracles without regenerating them.

## CPPGM backend

Configure a built checkout of the commit in `backend.lock.json`:

```sh
python3 scripts/backend.py --configure /path/to/cppgm-extended
make test-backend
python3 scripts/backend.py program.lowir -o obj/program
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
