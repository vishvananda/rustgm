# RustGM Programming Assignment 37 — Reproducible self-hosting

Build your completed RustGM compiler with RustGM, verify it on the PA1–PA24
oracle ladder, then rebuild RustGM using that self-hosted compiler and verify
that stage 1 and stage 2 match byte for byte.

Use the Rust 1.98.0 Linux x86-64 panic-abort profile with the original `core`,
`alloc`, and `std` sources and your compiler's real locked dependencies. This
completes the initial course track. Procedural macros, unwinding, remaining
edition/target closure, async, and final frontend acceptance follow in
PA38–PA45.

## Completion criterion

From the repository root (after `make setup-backend` and `make setup-rust-src`):

```sh
make inception
```

or equivalently:

```sh
make -C pa37 test
```

The build performs three generations:

| Stage | Built by | Output |
| --- | --- | --- |
| 0 (seed) | Pinned Rust 1.98.0 toolchain (`cargo build --release --locked`) | `obj/student/release/rustgm` (maintainer: `obj/rust/release/rustgm`) |
| 1 (self) | Stage 0 compiling the frozen whole-source graph through CPPGM | `obj/pa37/stage1/native/rustgm` |
| 2 (inception) | Stage 1 compiling the exact same frozen graph and linker pins | `obj/pa37/stage2/native/rustgm` |

Each self-hosted stage is one strict whole-graph build with **zero bring-up
records**, zero unproven obligations, zero waivers, and no trapping stubs.
Stage 1 and stage 2 must match byte for byte on:

- `program.lowir`
- `program.native-links.json`
- the linked `rustgm` executable

A passing run also verifies negative controls (a perturbed stage 2 output and
a nonzero bring-up record both fail) and prints `MATCH rustgm`.

## Work through the ladder

Modeled on CPPGM's PA35 self-hosting ladder, you can build stage 1 once and
advance through the oracle suites incrementally:

```sh
make -C pa37 stage0
make -C pa37 stage1
make -C pa37 test-through-pa4
make -C pa37 test-through-pa11
make -C pa37 test-through-pa24
make -C pa37 stage2
make -C pa37 compare
make -C pa37 negative-controls
```

You can run any single assignment against stage 1 with `make -C pa37 test-pa12`,
or extend the ladder all the way through PA30 with `make -C pa37 test-through-pa30`.

## Configuration and reproducibility

- In a student repository, `scripts/pa37_self_host.py` automatically discovers
  your crate graph from `Cargo.toml` and `Cargo.lock` (resolving your locked
  dependencies plus `src/lib.rs` if present and `src/main.rs` as `app`). If you
  need custom crate settings, supply `pa37/student-configuration.json`.
- Freeze the source inventory, configuration, compiler and linker identities,
  and backend lock before both stages. Prevent timestamps, machine paths, or
  unordered traversal from changing emitted LowIR or linked binaries.
- Pass `PA37_OPT_LEVEL=0` or `PA37_OPT_LEVEL=1` (default `1`) to select the
  CPPGM object optimization level used identically for both stages.
- Pass `PA37_WORK=../obj/pa37-experiment` to keep experimental builds separate.
