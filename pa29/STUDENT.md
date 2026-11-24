# RustGM Programming Assignment 29 — Crates and metadata

Build on [PA18](../pa18/README.md) to compile libraries and consume their
typed declarations in a separate client. Use Rust 1.98.0, edition 2024,
on Linux x86-64 with checked arithmetic and aborting panics.

[The PA29 manifest](tests/manifest.json) defines the declarations profile.
It covers crate identities, public namespaces, scalar signatures and values,
and the closed record, array, and reference facts the supplied graphs use.
PA44 carries broader value metadata and executable generic-body transport.
The original-library bootstrap also needs the relevant transport capabilities.

## Provider and consumer contracts

- Give each provider a crate identity from its name and disambiguator. Two
  crates with the same name can define distinct declarations and symbols.
  Imports, aliases, and re-exports preserve the original definition identity
  and symbol. An extern alias is a name in the client, not the provider's
  identity.
- Resolve visible declarations through the provider's published namespace.
  Keep public bindings independent of their targets' defining paths. A private
  layout can be required by a public signature without making the private
  name or fields accessible.
- Export typed signatures, scalar bits, and the required nominal and array
  facts after checking the provider. Reconstruct those facts in a fresh
  consumer. Preserve array lengths, field privacy, canonical type identities,
  and the obligations that apply when a declaration is used.
- Validate metadata before using it. Preserve dependency identities and fail
  clearly on missing or incompatible metadata. Use deterministic artifact
  and LowIR output. Each provider retains its private executable instances.
- A direct `--extern ALIAS=FILE` adds a visible alias and makes the artifact
  available. `--dependency-metadata FILE` supplies a transitive artifact while
  leaving its name out of the client's extern prelude. Native linking still
  includes all of the graph's object producers.

## Build and run a crate graph

The [scalar client](tests/spec/scalar/client.rs) links two providers named
`shared` with different disambiguators. The second provider calls the first.
Build your compiler and install the pinned backend with `make` and
`make setup-backend`, then run these commands from the student repository root.

```sh
compiler=obj/student/release/rustgm
src=pa29/tests/spec/scalar
out=obj/pa29-example
mkdir -p "$out"

"$compiler" --emit-lowir --crate-type=lib --crate-name shared \
    --crate-disambiguator first --metadata-output "$out/first.rgm" \
    -o "$out/first.lowir" "$src/first.rs"
"$compiler" --emit-lowir --crate-type=lib --crate-name shared \
    --crate-disambiguator second --extern "one=$out/first.rgm" \
    --metadata-output "$out/second.rgm" -o "$out/second.lowir" "$src/second.rs"
"$compiler" --emit-lowir --extern "one=$out/first.rgm" --extern "two=$out/second.rgm" \
    -o "$out/client.lowir" "$src/client.rs"
python3 scripts/backend.py "$out/first.lowir" "$out/second.lowir" "$out/client.lowir" \
    -o "$out/client"
"$out/client"
```

The client checks its results and exits 0. The provider files are inputs to
the client's source checks. Their LowIR units are inputs to the native link.

## Run the tests

Run the PA29 test target from the repository root.

```sh
make -C pa29 test
```

The runner builds the providers declared by each case's `crates` list in
dependency order. It passes the case's `externs` aliases and any transitive
metadata to the consumer, then compares the consumer's LowIR, diagnostics,
and status with the expected results.
