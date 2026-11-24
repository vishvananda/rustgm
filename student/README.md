# RustGM programming assignments

The assignment directories contain the handouts, fixture manifests and expected
output. Start with [PA1](pa1/README.md) and implement the compiler in `src/`.
Use Rust, Cargo, Python 3 and GNU Make. The pinned Rust toolchain is declared in
`rust-toolchain.toml`. The course target is Linux x86-64.

```sh
make
make -C pa1 test
```

Each assignment's `test` target builds your compiler and compares its output
and exit status with the expected results. The initial compiler reports an
unimplemented mode. Look in each assignment's `tests/manifest.json` for the
case list and input options. Each fixture's `.ref`, `.ref.stderr`, and
`.ref.exit_status` files give the expected output, diagnostics, and status.
Test reports are written to `obj/tests/`. Set
`RUSTGM_TEST_APP=/absolute/path/to/rustgm` to test an existing executable.


## Course order

1. Work through PA1–PA30 in order.
2. Complete PA31's source bindings and runtime integration. In PA32, prepare
   the pinned build, finish the required target and generic-body support,
   and build standalone `core` with independently compiled consumers.
3. Build `alloc` in PA33, extend the crate-build driver in PA34, then run
   PA35's core-only and allocation-dependent upstream checks. Build `std`
   in PA36.
4. Complete **self-hosting (PA45)** using that library profile. Stage 1 runs
   the earlier assignment suites and stage 2 must reproduce stage 1.

The initial track ends at PA45. Procedural macros, unwinding, the remaining
edition and target work, and async are a separate track. PA44 integrates
the broader course profile after those features.

Each exported assignment folder has a `README.md`. Available suites have
a test manifest and a `make test` target in their PA directory. A preview
may also contain handouts whose grading suites are pending.

## CPPGM backend

Starting with PA11, RustGM emits [LowIR](shared/lowir.md). The Python driver
passes LowIR to the pinned CPPGM backend and links the resulting executable.
Installation and the library example require Linux x86-64 and C/C++ build tools.

```sh
make setup-backend
python3 scripts/backend.py obj/program.lowir -o obj/program
make backend-example
```

`scripts/setup_backend.py --bundle PATH` installs from a downloaded bundle.
Either installation path verifies the bundle and the individual tool, library
and header hashes. The lock records the CPPGM release and library ABI.

[The C example](examples/lowir-library/return42.c) builds the same kind of
backend input through the library's public header. It defines a LowIR function,
validates the module, emits an ELF object and runs the linked program. Its
interface starts at LowIR. You can use it as a starting point for calling the
library from your own driver.
