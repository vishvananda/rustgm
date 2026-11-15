# PA29 declarations component release

This releases the finite **declarations** work unit. PA29 itself remains
planned. The graded contract of this unit is its published manifest; it has no
deferred fixture, and one table row that a course case cannot represent.

## Released contract

The canonical manifest is [declarations.json](../pa29/tests/units/declarations.json):
30 declared outcomes, 17 accept/13 reject, all `--emit-lowir` with file input,
under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting
panics. Nine accepted programs execute and pass. Eight accepted library crates
are compared as objects; one of them is a declared empty unit. 23 cases are
compiled against declared provider crates.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `scalar` | 4 | 4 | 0 | 2 |
| `values` | 3 | 3 | 0 | 1 |
| `transitive-metadata` | 4 | 2 | 2 | 2 |
| `public-names` | 12 | 5 | 7 | 2 |
| `array-metadata` | 7 | 3 | 4 | 2 |

Outcomes come from four committed draft tables for requirements 29.1 and 29.2:
`scalar-outcomes.json`, `transitive-metadata-outcomes.json`,
`public-name-outcomes.json` and `array-metadata-outcomes.json`. No fixture was
authored or edited. Each contract outcome is the table's pinned-Rust outcome.
Every rejection carries the diagnostic the tables and their maintainer checkers
name (unresolved, private, ambiguous, incompatible types, moved, lifetime).

### Crate-graph cases

Each table is a dependency graph. A course case now declares it:

```json
"crates": [
  {"name": "first", "path": "tests/spec/scalar/first.rs",
   "crate_name": "shared", "disambiguator": "first", "externs": {}},
  {"name": "second", "path": "tests/spec/scalar/second.rs",
   "crate_name": "shared", "disambiguator": "second", "externs": {"one": "first"}}],
"externs": {"renamed": "second"}
```

`crates` lists the provider crates in build order; `externs` binds the
consumer's aliases. The runner compiles each provider with the candidate
(`--crate-type=lib`, its name and disambiguator, `--metadata-output`), then the
consumer with `--extern ALIAS=ARTIFACT` for each binding and
`--dependency-metadata ARTIFACT` for each provider reached only through
another. Pinned rustc builds the same graph as rlibs. The native check emits
every LowIR unit as an object and links them with the host `cc`. A provider is
an input: one that fails to build is a harness error, not a case outcome.

- A row's providers are the earlier rows its dependencies reach, in table
  order. A library row is published with its own identity and an object
  comparison; a `native` client is an executed program.
- `scalar/first.rs` and `scalar/second.rs` appear in the scalar and transitive
  tables with the same identity and are one case each.
- The reference text of a consumer is its own LowIR and diagnostics. Provider
  sources are hashed in the receipt; metadata artifacts are not references.

## Deferred list

No candidate is deferred: all 30 pass pinned rustc, the candidate and the
native lane.

One table row is not representable and is not published:
`transitive-metadata-outcomes.json` row `inventory-only` compiles
`transitive-metadata/client.rs` a second time with no alias bound and expects
Rust E0433. A course case is keyed by its source path, so one source has one
graph. The row stays checked by `scripts/check_transitive_metadata.py`.

Not owned by this unit and unchanged: requirement 29.4's fixtures live under
`pa31/tests/spec/source-header-transport`; the value, generic and macro
metadata tables belong to `value-metadata` and `generic-transport`; the
maintainer checkers' artifact-level controls (repeat determinism, malformed and
missing inventory, the C caller) stay in `scripts/check_public_names.py`,
`check_transitive_metadata.py` and `check_aggregate_metadata.py`.

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. Checks ran on 2026-11-15 through the resource and
storage guards, using the [release recipe](../TESTING_AND_REFERENCES.md#component-release-recipe):

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit declarations` | 30 cases valid |
| `check_syntax.py pa29 --unit declarations` | 30 independent rustc checks agree (1.3 s) |
| `course.py pa29 --unit declarations --bless` | 30/30 declared outcomes, then references written (1.5 s) |
| `check_layout.py --unit declarations` | complete oracle provenance |
| `course.py pa29 --unit declarations` | 30/30 against the committed text (0.3 s) |
| `check_native.py pa29 --unit declarations` | 30 checks; 9 executable and 8 object comparisons, 23 cases against provider crates (2.9 s) |

The probe ran all 30 candidates before any reference was written and all
passed. The text check rebuilds every provider in a fresh temporary directory,
so the references do not depend on artifact paths. A review of the rejection
diagnostics found no implementation-gap wording. Receipts are under
`obj/publication/declarations/`.

## Requirements

Requirements 29.1 and 29.2 are implemented components covered by this release.
Requirement 29.4 stays partial and unpublished. Requirements 29.3, 29.5 and
29.6 are unchanged. No performance claim is made. Scheduled CI runs the unit
through the `--released-units` text, independent and native schedulers.
