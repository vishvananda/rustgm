# PA30 c-abi component release

This releases the finite **c-abi** work unit. PA30 itself remains planned. The
graded contract of this unit is its published manifest plus the deferred list
below.

## Released contract

The canonical manifest is [c-abi.json](../pa30/tests/units/c-abi.json): 143
declared outcomes, 84 accept/59 reject, all `--emit-lowir` with file input,
under Rust 1.98.0, edition 2024, x86-64 Linux, checked overflow and aborting
panics. 40 accepted programs execute and pass, and 44 are
code-generation-only. 60 of those 84 programs link declared host C providers.

| Family (`tests/spec/`) | Cases | Accept | Reject | Executed |
| --- | --- | --- | --- | --- |
| `aggregate-abi` | 8 | 5 | 3 | 5 |
| `aggregate-abi/composites` | 15 | 12 | 3 | 12 |
| `conventions` | 13 | 6 | 7 | 0 |
| `imports` | 21 | 11 | 10 | 0 |
| `linkage` | 16 | 8 | 8 | 0 |
| `statics` | 28 | 18 | 10 | 0 |
| `variadic-tails` | 11 | 7 | 4 | 7 |
| `variadics` | 31 | 17 | 14 | 16 |

Outcomes come from the eight committed draft tables for requirements 30.1 to
30.4 and 30.7: `convention`, `import`, `linkage`, `static`, `aggregate`,
`composite`, `variadic` and `variadic-tail`. No Rust fixture was authored or
edited. Each contract outcome is the table's pinned-Rust outcome. Five cases
whose rejection is a lint level declare `default_lints`.

### Host C providers

A program that imports C symbols declares its providers in
`native.providers`. The native checker compiles each one with
`cc -std=c11 -O0 -c` and links the same objects into both executables: pinned
rustc through `-Clink-arg`, the candidate by emitting the program's LowIR as an
object and linking with the host `cc`. Providers are hashed in the reference
receipt like auxiliary sources.

- 22 providers are the committed ones the draft tables name: a row's `.c`
  companion or `host_provider`, the `variadic` and `variadic-tail` tables'
  shared provider, and the three `aggregate` providers.
- Three are new link stubs, because no committed provider defines the symbol:
  `imports/imported.c` (`imported`, six programs), `linkage/alias.c` (`ffi`)
  and `statics/storage.c` (`X`, two programs). These nine programs are
  code-generation-only, so a stub's behaviour is never observed.
- Two cases released on 2026-11-15 change declaration and keep their text
  references: `statics/alias.rs` now names `statics/imports.c` as a provider
  instead of an auxiliary, and `variadics/bare-signature.rs` gains its table's
  provider.

## Deferred list

All 15 deferred fixtures are Rust-valid programs. None is published as an
expected rejection.

| Fixtures (`tests/spec/`) | Observed with the candidate |
| --- | --- |
| `aggregate-abi/`: rust-repr, packed, union, enum, empty (5) | `ERROR: C aggregate ABI profile requires a nonempty aligned repr(C) struct` |
| `aggregate-abi/composites/`: packed-inner, rust-layout-inner, union-inner (3) | `ERROR: C aggregate ABI profile requires nonempty aligned repr(C) fields` |
| `linkage/`: unicode, unsupported-byte (2) | `ERROR: exact symbol spelling is outside the pinned LowIR vocabulary` |
| `linkage/`: helper-first, export-first (2) | `ERROR: multiple function definitions for one LowIR symbol` |
| `statics/`: generated-collision, generated-import-collision (2) | `ERROR: LowIR symbol kind conflicts with prior declaration` |
| `statics/interior-storage.rs` (1) | Passes text, rustc and native, but declares `pa28.atomic-storage`. c-abi has no atomic-u32 prerequisite edge, so the unit validator rejects it as a future-stage feature |

Not owned by this unit and unchanged: the four `*-metadata` tables
(value-metadata, 30.5), `option-niche` (const-memory), `call-argument-context`
(a PA19 prerequisite control) and the remaining PA30 packets without a draft
outcome table.

## Qualification

The candidate is the release CLI built from clean `80229d28d`, SHA-256
`9717800a61fce3779a9ca7b0715fdc2e2594672365191bb778477bbbb140a2b9`, with the
locked CPPGM v3 backend. Checks ran on 2026-11-15 through the resource and
storage guards:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit c-abi` | 143 cases valid |
| `check_syntax.py pa30 --unit c-abi` | 143 independent rustc checks agree (2.9 s) |
| `course.py pa30 --unit c-abi --bless` | 143/143 declared outcomes, then references written (1.5 s) |
| `check_layout.py --unit c-abi` | complete oracle provenance |
| `course.py pa30 --unit c-abi` | 143/143 against the committed text (0.6 s) |
| `check_native.py pa30 --unit c-abi` | 143 checks; 40 executable and 44 code-generation comparisons, 60 programs linked with host providers (7.9 s) |

The first release published 85 cases and deferred 73, of which 58 needed a
host C provider at link. The provider lane was added to the native checker the
same day and the unit was probed, declared and checked again: all 158
candidates ran, the 58 passed, and no sidecar of the 85 earlier cases changed.
Two flagged rejection diagnostics, `statics/const-read.rs` (Rust E0080) and
`variadics/variadic-body.rs` (Rust E0658), name the Rust rule and are recorded
as accepted in `scripts/unit_tables.json`. Receipts are under
`obj/publication/c-abi/`.

## Requirements

Requirements 30.1 to 30.4 and 30.7 stay implemented components. This release
publishes their text contract, standalone programs and C-provider native
matrix. Requirements 30.5 and 30.6 are unchanged.
