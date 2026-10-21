# RustGM Programming Assignment 29: Crates and metadata

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA28. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

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
bridge. Supply metadata for every dependency; repeat `--extern ALIAS=FILE` for
the complete graph. Metadata is written only after the ordinary semantic, body,
ownership and lowering gates succeed. `--crate-name` alone retains legacy names.

This profile exports public nongeneric, nonconst Rust functions with scalar
parameters/results, nested public module paths and explicit unsafe permission.
Private checked generic instances remain local to each crate. Exposed nominals,
references, generic/const/async functions, target-feature contracts, other ABIs,
public reexports and empty public modules diagnose profile boundaries. Generic
body exports, constants/allocation graphs and the full assignment contract remain
unfinished. See `docs/pa29-scalar-metadata.md`, `docs/pa29-import-remapping.md` and
`docs/pa29-scalar-linking.md`. The assignment remains planned.

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

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
