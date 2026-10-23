# RustGM Programming Assignment 29: Crates and metadata

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA28. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.

## Required implementation

Separate compilation, crate identities, metadata and generic body exports.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

The current maintainer scalar checkpoint supports separate RustGM libraries:

```sh
dev/rustgm --emit-lowir --crate-type=lib --crate-name shared \
  --crate-disambiguator first --metadata-output first.rgm -o first.lowir first.rs
dev/rustgm --emit-lowir --extern one=first.rgm -o client.lowir client.rs
python3 scripts/backend.py first.lowir client.lowir -o client
```

Multi-unit native linking requires the explicitly configured reviewed ELF object
backend profile. Ordinary single-unit builds retain the existing executable
bridge. Supply metadata for every dependency. `--extern ALIAS=FILE` introduces a
direct source-visible alias; `--dependency-metadata FILE` supplies transitive
metadata without adding a name to the extern prelude. Both options share one
canonical-path decoder and combined count/byte budgets. For a client depending
only on the second library of a chain, use `--extern renamed=second.rgm
--dependency-metadata first.rgm`; the first library remains required for linkage
but its name and the second library's dependency alias remain inaccessible.
Metadata is written only after the ordinary semantic, body,
ownership and lowering gates succeed. `--crate-name` alone retains legacy names.

`tests/transitive-metadata-outcomes.json` declares the three-crate source/native
controls. `scripts/check_transitive_metadata.py` qualifies direct aliases,
inventory-only failures, duplicate/reordered inputs, missing and malformed
artifacts, and combined argument budgets. Supplying an inventory grants no extra
source-core role, generic-body permission or library implementation.
See [the development checkpoint](../docs/pa29-transitive-metadata.md) for exact
evidence and the remaining release gate.

Profiles zero and one export public nongeneric, nonconst Rust functions with scalar
parameters/results, nested public module paths and explicit unsafe permission,
and completed closed scalar named constants. Constant values retain exact integer
and floating bits, without imported initializer bodies or runtime global storage.
Profile two additionally transports scalar/reference function signatures with
explicit late-bound lifetime slots, nested shared/mutable references and static
output references. It retains ordinary unsafe permission and call/borrow checks;
it exports no function bodies or const-call permission. Private checked generic
instances remain local to each crate. Profiles zero through two reject exposed
nominals. Early lifetime bounds,
type/const-generic or const/async functions, target-feature contracts, other ABIs,
public reexports and empty public modules diagnose profile boundaries. Generic
body exports, aggregate/reference constant values and allocation graphs, and the full
assignment contract remain unfinished. See `docs/pa29-scalar-metadata.md`,
`docs/pa29-import-remapping.md`,
`docs/pa29-scalar-linking.md`, `docs/pa29-scalar-values.md` and
`docs/pa29-borrowed-signatures.md`. The assignment remains planned.

Profile three additionally transports closed nongeneric named records with the
default Rust representation. Supported fields contain scalars, other closed
records or static shared references. Public declarations and private reachable
layouts share one artifact type graph. Imports preserve nominal identity,
field visibility, ordinary sizedness, moves, patterns and layout without
inventing source nodes or constructor bodies. Private layout transport grants
no public type access. Unsupported representation, generic, derive, trait,
method and destruction contracts remain explicit profile errors. See
`docs/pa29-closed-records.md` for the independent source/native qualification
and remaining assignment work.

Profile four additionally retains fixed C callable conventions using the same
signature-local or shared nominal type graphs. Safety, late-bound lifetime slots
and exact linkage stay independent. Rust-only artifacts retain profiles zero
through three byte-for-byte. This PA30 extension grants no const or generic-body
permission and adds no static or C aggregate transport. See
`docs/pa30-foreign-metadata.md`.

Profile five adds named storage and completed scalar/record/reference values.
Hidden backing remains private; self-referential static addresses retain real
global identity across object boundaries. The source and native controls in
`tests/storage-metadata-outcomes.json` also vary shared roots at 8/32/128 sizes.
Foreign-storage extensions belong to PA30. See
`docs/pa30-storage-metadata.md` for validation and the remaining graph boundaries.

Profile six adds a public namespace independently of defining identities.
Renamed local and external reexports, module aliases, empty public modules and
resolved module-alias cycles retain one owned declaration per defining crate.
Imports reserve every target before installing bindings; hidden canonical paths
and private fields remain inaccessible. Function and static aliases keep their
original symbols and addresses. Earlier payload profiles keep their exact bytes
when an explicit namespace is unnecessary. Source and native qualification is in
`tests/public-name-outcomes.json`; `scripts/check_public_names.py` runs its original
sources and checks repeated publication and dependency order. The combined
checkpoint is qualified in `docs/iteration-storage-checkpoint.md`. Generic bodies and the other
unfinished metadata contracts remain required. See `docs/pa29-public-names.md`.

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

Profile seven retains fixed-array element types and lengths, plus the actual
default Rust or plain C representation of closed records. It wraps the earlier
payload and optional public namespace only when these facts are present.
Imports reuse ordinary type interning, moves, borrow checks, layout and ABI
classification. Array lengths never become decoder allocation counts.
`tests/array-metadata-outcomes.json` and PA30's
`tests/aggregate-metadata-outcomes.json` cover separate compilation and native
execution. The stored-array extension retains ordinary products, compressed
repeats, byte payloads and typed field/element reference paths through actual
static identities. Its 13 source cases include seven native clients, hidden
backing, nested arrays, repeated references and full-width zero-sized indices.
Generic bodies and additional representation/value contracts remain required;
see `docs/pa29-aggregate-metadata.md` and `docs/pa29-array-storage.md`.

The source-independent [original body ownership table](../docs/pa29-body-ownership.md)
is a typed prerequisite for generic transport. It validates defining artifact,
declaration, parent and ordinal relationships with bounded linear work. It does
not yet encode/import bodies, provide executable permission or activate a new
wire profile. The full generic-body and contract transport remains required.

The first executable generic checkpoint adds envelope nine, row version zero,
for defining Rust 2024 and the legacy language-role policy. An actual checked
`const fn id<T: Copy>(value: T) -> T` survives export/import and normal runtime
and constant instantiation; Copy debts are re-proved for each actual instance.
Artifacts contain typed original operations rather than proof tokens. The
narrow operation/type family rejects unsupported source-role, trait, lifetime,
call and other body contracts explicitly. Profiles zero through eight retain
their encodings. This does not activate PA29 or replace the full generic-body
requirement; see [the checkpoint](../docs/pa29-executable-generics.md) and the
unchanged complete `tests/generic-contract-outcomes.json` inventory.

## Tests to develop with the reference

multi-crate dependencies; symbol identity; visibility; deterministic exports.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.
