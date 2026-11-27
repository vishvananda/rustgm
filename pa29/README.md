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
  consumer. Carry evaluated array lengths and enum discriminants with their
  types and variant declarations. A fresh client can cast an eligible
  imported enum directly to an integer. Preserve checked cast eligibility,
  field privacy, canonical type identities, and use-site obligations.
- Validate artifact headers, versions, dependency identities, and table bounds
  before using metadata. Preserve dependency identities and fail clearly on
  missing, truncated, or incompatible metadata. Use deterministic artifact and
  LowIR output. Each provider retains its private executable instances.
- A direct `--extern ALIAS=FILE` adds a visible alias and makes the artifact
  available. `--dependency-metadata FILE` supplies a transitive artifact while
  leaving its name out of the client's extern prelude. Native linking still
  includes all of the graph's object producers.

## Generic and provided bodies

Serialize the checked typed body templates a client may need to instantiate
or evaluate, including the declarations, types, constants and private
dependencies those bodies refer to. Serialize the compiler's own typed IR
through one shared, versioned representation rather than maintaining a parallel
metadata-only IR. Give every declaration, closure, anonymous or inline `const`
block, and body a stable `(crate, ordinal)` identity and remap those identities
into the fresh client's tables. Nested items keep their lexical owners even
when the provider has already evaluated an enclosing constant or static
initializer, while their names stay outside the crate's published namespace.

Design the artifact and reader so separate compilation scales to `core`:

- **Decode lazily on demand.** Separate lightweight export and header tables
  from indexed body, closure, inline-constant, and impl payloads so a consumer
  only decodes and remaps items demanded by name resolution, trait selection,
  constant evaluation, or monomorphization.
- **Trust producer-proved templates on read.** Validate wire framing, version
  and target compatibility, dependency crate identities, and index bounds when
  admitting an artifact. Do not re-run the provider's borrow, region, loan,
  exhaustiveness, coherence, or template body proofs on every import; put deep
  artifact re-verification behind an opt-in self-check mode for testing. Run
  only client-dependent checks (use-site nominal where-clauses, visibility and
  stability, concrete dispatch, and drop elaboration) when instantiating a
  demanded body.
- **Keep post-resolution logic source-independent.** Key `TypeId` identities,
  constant evaluation, intrinsic recognition, compiler shims, and lowering off
  uniform local-or-imported item identities rather than local AST nodes. The
  client has the artifact and the provider's native output and instantiates
  bodies without reopening the provider's Rust source.
- **Carry the full generic and callable frame.** Record early-bound versus
  late-bound lifetimes and distinguish impl- or trait-captured lifetimes from a
  method's own lifetimes so concrete dispatch reconstructs method arguments
  accurately. Preserve const parameter types and defaults, calling conventions
  (`Rust`, `rust-call`, `C`, `C-unwind`, `unadjusted`), `#[track_caller]`
  policies, and field templates needed to normalize associated-type projections
  on concrete aggregates without forcing uninstantiated types behind raw
  pointers or references.

When an imported function declares legacy const argument positions, a direct
call without explicit generic arguments evaluates the marked operands as
const arguments. Runtime operands evaluate in source order.

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
