# Legacy editions

Lane B of the [core census plan](../core-census-plan.md). The original std graph
has two target dependencies on older editions: cfg-if 1.0.4 (Rust 2018, unit 1)
and rustc-demangle 0.1.27 (Rust 2015, unit 11). This checkpoint adds Rust 2015
and 2018 to the shared typed `Edition`. Each rule those crates use is
implemented. Every other pre-2021 difference fails with a typed
"... is not implemented for edition 2015|2018" diagnostic. No legacy rule
silently gets 2021 behavior.

## Inventory

**cfg-if** (lib, non-test cfg). It uses `#![no_std]` and
`deny(missing_docs)`, plus one `#[macro_export] macro_rules! cfg_if`. That macro
uses only `tt` fragments, `$crate` and `$(...)?`. None of its rules depend on
the edition after token origin. It does need macro-definition transport to a
2024 consumer, which RustGM metadata does not carry yet.

**rustc-demangle** (features `core`, `rustc-dep-of-std`). It uses:

- `use core::...` in submodules and function bodies;
- an explicit `use core::convert::TryFrom`;
- `unreachable!()`, with and without format arguments;
- local `ident` and `expr` macro fragments;
- closures capturing `*f` (a `&mut Formatter` reborrow) and `(*self).field`;
- ordinary derives.

It uses no `dyn`, `async` or `try` keyword, no anonymous parameter, no bare
trait object and no `...` pattern.

## Defining contexts

Each rule names where its edition comes from:

| Rule | 2015 | 2018 | Context |
|---|---|---|---|
| `expr` fragment | `Expression2021` | `Expression2021` | macro definition |
| `pat` fragment | no top-level `\|`, `\|` may follow | same | macro definition |
| `gen` keyword | identifier | identifier | token origin |
| `use a::b` with plain first segment, `::a` | crate-root relative | extern/lexical | first-segment token origin |
| injected `core`/`std` nameable at root | yes | no | crate edition, as rustc's inject |
| closure capture | whole variable | whole variable | closure introducer token |
| wildcard `let _ = x` in a closure | captures `x` | captures `x` | closure introducer token |
| RPIT captures, unsafe attributes, never fallback, extern blocks, temporaries, pattern modifiers | 2021 rule | 2021 rule | existing contexts |

Closure captures use the existing capture graph. Each root variable whose
propagated mode is set becomes one capture. A `&mut` place keeps its existing
unique-immutable promotion, so `*f` and `(*self).field` capture `f` and `self`
as rustc 2018 does.

## Fences

These rules are typed `LegacyRule` diagnostics:

- 2015 `async`, `await`, `dyn` and `try` used as identifiers, plus their 2018
  keyword forms reached from 2015 tokens;
- anonymous trait method parameters (2015);
- unreserved `ident#`, `ident"` and `ident'` prefixes in physical legacy source;
- bare trait objects;
- `...` range patterns;
- array `into_iter` method dispatch;
- pre-2021 `panic!` and `assert!` delegation (`panic_2015`). Pre-2021
  `unreachable!` now delegates to `unreachable_2015`, and 2015/2018 source
  clients are admitted ([definition-lineage](definition-lineage.md#5-rust-20152018-as-cfg-if-and-rustc-demangle-need-them)).

## Shared builtin model

RustGM's bootstrap library model has no 2021 prelude additions (`TryFrom`,
`TryInto`, `FromIterator`) and no `Future`/`IntoFuture` in any edition. It also
has no `panic!`, `assert!` or `unreachable!` outside source core. These cases
therefore give the Rust 2021 outcome in every edition. That outcome is a
missing name, which is the 2018 answer as well.

## Evidence

`pa34/tests/legacy-edition-outcomes.json` freezes 100 pinned rustc 1.98.0
outcomes, all declared before execution. 96 come from plan v1 and 4 from the
v2 addendum, which covers 2015 `use core::...` in a module and in a body. The
plan, oracle and receipt hashes are under `obj/parallel/legacy-editions-v{1,2}/`.
The file declares 82 RustGM expectations: accepted, rejected, fenced or
same-as-2021. Six mixed-edition macro cases are rustc-only requirements.
`dev/tests/legacy_editions.rs` runs every declared case through the CLI.

## Results

- **cfg-if 1.0.4, original source, `--edition=2018`:** the names, expansion,
  types, semantics and loans phases all succeed. Library metadata
  (`--crate-type=lib --metadata-output`) stops on "exposed declaration is outside
  the scalar metadata profile". That is the exported `macro_rules!`. A plain
  2018 library publishes, and a one-macro library fails the same way in 2018,
  2021 and 2024.
- **rustc-demangle 0.1.27, `--edition=2015`:** parsing reaches name resolution,
  which stops on `use core::char` (`legacy.rs`). Expansion stops on
  `unreachable!` (`v0.rs`). Both belong to the bootstrap library model, and 2018
  stops at the same points.
- **Std target graph** (combined run with the candidate CLI): cfg_if (unit 1)
  and rustc_demangle (unit 11) plan without gaps. The only remaining planning
  gap is std's `crate_types=dylib` (unit 14). Execution still stops first at
  core (unit 5), on `strict provenance lint producer is not implemented`.
