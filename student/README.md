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
unimplemented mode. Test reports are written to `obj/tests/`. Set
`RUSTGM_TEST_APP=/absolute/path/to/rustgm` to test an existing executable.

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
