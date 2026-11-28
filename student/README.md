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

1. Work through PA1–PA30 in order. PA29 establishes separate compilation,
   including the shared metadata for generic bodies used by fresh clients.
2. Complete PA31's source bindings and runtime integration (`make -C pa31 test`).
   Before running PA32–PA37, install the pinned Rust 1.98.0 library sources
   and locked PA35 crates once with `make setup-rust-src` (or set
   `RUSTGM_RUST_SRC` to an existing pinned checkout).
3. In PA32 (`make -C pa32 test`), build standalone `core` and run all 18
   independently compiled `no_std` consumers.
4. Build `alloc` in PA33 (`make -C pa33 test`), complete editions and crate
   builds in PA34 (`make -C pa34 test`), run the upstream crate consumers
   (`unicode-ident`, `tinyvec`, `unicode-normalization`) in PA35
   (`make -C pa35 test`), and build `std` in PA36 (`make -C pa36 test`).
5. Complete **self-hosting (PA37)** with `make inception` (or `make -C pa37 test`).
   Stage 1 runs the PA1–PA24 suites (`make -C pa37 test-through-pa24`) and
   stage 2 must reproduce stage 1 byte for byte (`MATCH rustgm`).

The initial track ends at PA37. Procedural macros, unwinding, the remaining
edition and target work, and async are a separate follow-on track (PA38–PA44).
PA45 integrates the broader course profile after those features.

Every exported assignment folder (PA1–PA37) has a `README.md`, a grading suite
manifest, and a runnable `make -C paN test` target.

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
