# RustGM Programming Assignment 29 — Crates and metadata

Build on [PA18](../pa18/README.md) to compile libraries and consume their
typed declarations and generic bodies in a separate client. Use Rust 1.98.0,
edition 2024, on Linux x86-64 with checked arithmetic and aborting panics.

[The PA29 manifest](tests/manifest.json) lists the graded cases. Start with
the declaration artifacts, then use the same crate boundary for executable
generic bodies. [PA32](../pa32/README.md) uses this mechanism to build and
consume the original `core` crate.

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

## Generic and provided bodies

Serialize the checked typed body templates a client may need to instantiate
or evaluate, including the declarations, types, constants and private
dependencies those bodies refer to. Use a shared, versioned representation
for these facts. Give references stable identities and remap them into the
fresh client's own tables when it reads the artifact. Preserve binders,
obligations and the defining context needed to check the bodies.

Validate an artifact before admitting it. Use the ordinary compiler pipeline
to instantiate an imported body and check obligations that depend on the
client's use. The client has the artifact and the provider's native output.
It can instantiate the body without reopening the provider's Rust source.

Start with small crate graphs that combine features already implemented in
earlier assignments. Check a generic call at multiple concrete types, a
provided trait method and an imported const call. Link and run the runtime
clients. Check reproducible artifacts and reject incompatible metadata and
invalid client arguments. As later assignments add language features, extend
the same representation for any new facts a separate client needs.

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
