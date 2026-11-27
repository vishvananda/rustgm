# RustGM Programming Assignment 30 — Foreign interfaces

Use [PA27](../pa27/README.md)'s unsafe checking and
[PA29](../pa29/README.md)'s declaration identities to call C code and make
Rust declarations available to it. Use Rust 1.98.0, edition 2024, on
Linux x86-64 with checked arithmetic and aborting panics.

[The PA30 manifest](tests/manifest.json) covers the local C ABI, foreign
imports, exact linkage, exported storage, callbacks, and calls with
variadic arguments. PA45 carries the broader foreign representation and
cross-crate metadata contracts.

## Callable signatures and safety

Retain a function's ABI, safety, and variadic status as separate typed facts.
An explicit `extern` without an ABI string selects C. Foreign items inherit
their block's ABI and default safety, with the explicit safety permitted by
their declaration. Check calls and function-pointer coercions with the selected
ABI, signature, safety, and lifetime constraints. A C method or pointer does
not gain const-call permission from its spelling.

For variadic C calls, apply normal coercions to fixed parameters and check
extra arguments independently. Require the applicable C promotions at the
source boundary. For example, pass an `f64` in place of an `f32` variadic
argument. Preserve left-to-right evaluation, borrow and move obligations,
and the argument classification for direct and indirect calls.

## Symbols and storage

Use typed symbol identities for foreign imports and Rust definitions. Apply
`link_name`, `unsafe(no_mangle)`, and `unsafe(export_name)` to select exact
linker names. Compatible declarations can refer to the same symbol. Reject
incompatible signatures, conflicting bodies, and function/global kind
collisions. An exported definition remains a lowering root even when
ordinary Rust calls do not reach it.

Retain a foreign static's type, mutability, and access safety. An imported
object is a declaration of external storage. Exported storage keeps its
initializer and the callbacks or other symbols it reaches. Forming an
address and loading a foreign value keep their distinct checks.

## Aggregate calls

For supported nonempty `repr(C)` records, classify arguments and results
using the Linux x86-64 C calling convention. Use the checked layout to
classify fields and fixed arrays, including nested fields, by their actual
eightbyte positions. Preserve INTEGER, SSE, and memory classes as
applicable. Keep the classification for imported calls, exports, and
callbacks. Check source types, initialization, Copy and Move behavior,
and lifetime constraints before using any ABI transfer.

## Build and run a C interoperation example

The [Rust fixture](tests/spec/aggregate-abi/interop.rs) calls a
[C provider](tests/spec/aggregate-abi/interop.c) and participates in a
callback through the C ABI. Build your compiler and install the pinned
backend with `make` and `make setup-backend`, then run these commands from
the student repository root.

```sh
compiler=obj/student/release/rustgm
src=pa30/tests/spec/aggregate-abi
out=obj/pa30-example
mkdir -p "$out"

"$compiler" --emit-lowir -o "$out/interop.lowir" "$src/interop.rs"
cc -std=c11 -O0 -c "$src/interop.c" -o "$out/provider.o"
python3 scripts/backend.py "$out/interop.lowir" --native-object "$out/provider.o" \
    -o "$out/interop"
"$out/interop"
```

The program checks its results and exits 0. Only LowIR and the host object
enter the backend link.

## Run the tests

Run the PA30 test target from the repository root.

```sh
make -C pa30 test
```

The manifest declares expected LowIR and diagnostics, and names any C sources
needed by executable cases in `native.providers`.
