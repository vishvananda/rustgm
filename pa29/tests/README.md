# PA29 fixture ownership

Planned: no graded suite or generated reference outputs yet.

`scalar-outcomes.json` declares the maintainer checkpoint's multi-crate inputs,
identities, dependency aliases and independent outcomes. The explicit Cargo
`crate_linking` target checks these inputs, including backend-dependent native
controls when run with `--include-ignored`. This inventory is not an activated
course manifest: the layout check rejects `manifest.json` for planned stages.
The `spec/values/` family adds scalar named constants, exact bit preservation,
imported patterns/discriminants and consumer const-generic bounds. Its producer
identities and dependency edges are distinct from the function-only family.

Required families: multi-crate dependencies; symbol identity; visibility; deterministic exports.

`spec/conditional-native-links/contract-v2.json` inventories the conditional-link
source and real C library/observer fixtures. Private source controls and the
`ffi_dependencies` native integration consume them. This is a maintainer
checkpoint inventory; whole PA29 still requires its release manifest and checks.

Put required regressions here, in the earliest assignment owning all their
features. Register every maintainer input in the checkpoint inventory. The
reviewed assignment release must publish `manifest.json` and references together;
no undiscovered fixtures or empty-suite success is permitted.

`local-dyn-owner-outcomes.json` declares a real source provider and two fresh
clients: bare imported dyn formation succeeds, while a foreign inherent impl
rejects after actual trait resolution. It preserves the ModuleContracts-v2
provider foundation and grants no foreign permission.

`legacy-const-call-outcomes.json` retains genuine external direct-path legacy
const-argument positives. The local header phase fences their metadata until the
real index list and anonymous-constant call protocol are transported; source
expectations are preserved as required work rather than rejection cases.

`source-nominal-outcomes.json` freezes the actual source nominal constructor and
fresh generic consumer packet under `spec/source-nominals/`, including move,
visibility, generic identity and escaping-borrow opponents. The selected
`crate_linking::source_nominals` integration controls consume its serialized v10
profile. The owning checkpoint distinguishes independent, frontend, native and
remaining transport requirements.

`source-branch-outcomes.json` freezes the version-twelve source branch packet
under `spec/source-branches/`: generic two-way, else-if, loop-exit and non-Copy
move providers, fresh consumers at several types, move/type/Copy opponents and
the wire-mutation anchors. The `crate_linking::branch_transport` integration
controls consume it; the owning checkpoint records candidate results.

`source-body-ir-outcomes.json` freezes the version-thirteen generic body IR
packet under `spec/source-body-ir/`: primitive literal, or- and range-pattern
decisions in generic providers, fresh consumers at i32 and bool, scrutinee and
Copy opponents, the re-encoding equivalence of the source-branches packet and
the named-rejection wire controls. The `crate_linking::node_transport`
integration controls consume it; the owning checkpoint records candidate
results.

`language-role-outcomes.json` freezes the version-thirteen operator-role packet
under `spec/language-roles/`: AddAssign, SubAssign and MulAssign roles with
their `Rhs = Self` defaults, checked Add, Sub and Mul in generic providers,
in-range and aborting fresh consumers, a type opponent, two recorded candidate
boundaries (a consumer's own operator use and `Add` with `Output`, the latter
amended once declared results transported) and the forged role-row controls.
The `crate_linking::spelled_roles` integration controls consume it; the owning
checkpoint records candidate results.

`associated-result-outcomes.json` freezes the version-thirteen declared-result
packet under `spec/associated-results/`: `Add` with its associated `Output`, a
`Shape` trait with an associated `Item`, their i32 and u8 impl definitions,
generic providers returning `<T as Shape>::Item` and `<T as Add>::Output`,
in-range and aborting fresh consumers, a type opponent, an unimplemented
opponent and the forged result-row controls. The
`crate_linking::normalized_results` integration controls consume it; the owning
checkpoint records candidate results.

`callable-role-outcomes.json` freezes the version-thirteen callable-role packet
under `spec/callable-roles/`: the `tuple_trait` marker, `FnOnce` with its
declared `Output`, `FnMut` and `Fn`, generic `FnOnce`/`FnMut`/`Fn` appliers,
fresh consumers passing function items and their own non-capturing and
capturing closures, three opponents (a wrong result, a mutating `Fn` closure
and a consuming `FnMut` closure), a fenced provider closure and the forged
call-operator controls. The `crate_linking::fn_roles` integration
controls consume it; the owning checkpoint records candidate results.

`macro-metadata-outcomes.json` freezes the exported `macro_rules!` family under
`spec/macro-metadata/`. It covers a 2021 provider (with a 2018 copy) and a
second provider. Its clients use `use` imports, paths and
`#[macro_use] extern crate` in 2015, 2018 and 2024. Their cases cover `$crate`
binding against a local name, `pat` fragments in the provider's edition,
root-only publication, unexported macros and colliding names. All twelve rustc
predictions were declared before the run. The
`crate_linking::declarative_exports` controls check the provider envelope.
`scripts/check_macro_metadata.py` runs both macro files, with consumer import
and native execution (`docs/parallel/macro-metadata.md`).

`macro-spacing-outcomes.json` freezes definition-site `stringify!` spellings
through an exported macro (`spacing-provider.rs`). It covers a cross-crate 2024
client and a local 2021 copy of the same definition. Each client asserts all
nine spellings, and any mismatch aborts. Round one's prediction (transcription
keeps punctuation jointness) was declared first and refuted. Round two froze
rustc's observed spellings: transcribed tokens print alone, paren and brace
delimiters keep their source spacing, and a bracket group is always followed by
a space. Local and cross-crate spellings are identical.

`enum-match-outcomes.json` freezes the version-thirteen enum-and-match packet
under `spec/enum-match/`: a fieldless enum with a constant-arm match, a
`#[repr(u8)]` enum with explicit discriminants and an `as u8` cast, a generic
`Opt<T>` with `or`, an `FnOnce` `map` and a by-reference binding, fresh
consumers of each, two type opponents, fenced called-constructor and guard
controls and the forged enum-header, constructor-test and downcast controls.
The `crate_linking::match_transport` integration controls consume it; the
owning checkpoint records candidate results.

`builtin-macro-rows-outcomes.json` freezes the tag-14 version-2 family under
`spec/builtin-macro-rows/`. A no_core source-core provider (`provider.rs`)
exports five root builtin `macro_rules!` rows and four builtin `pub macro` rows
in `builtin` and `builtin::deeper`. Seven source clients reach them through
paths, `use`, a renamed `extern crate` beside a same-named local module and
`#[macro_use]`. Their rejected cases cover a placed row named at the root, a
placed row expected in the macro-use prelude, and `compile_error`. A
requirement row has rustc accept the imported `test` attribute while RustGM
rejects it by name. `path-client.rs` links with `native-driver.c`, which prints
the consumer's `line!`, `column!` and `cfg!` values. Two more providers are
requirement rows: an ordinary macros 2.0 export, rejected by name, and a
prelude-style re-export, amended to a version-2 export once re-export rows
landed. `internal-provider.rs` exports `allow_internal_unstable` lists (two
features, one, none); its three clients check that each list applies per
feature. Every rustc outcome was declared before any rustc or
RustGM run, and `scripts/check_macro_metadata.py` runs the file with the other
two macro files (`docs/parallel/macro-metadata.md`).

`enum-methods-outcomes.json` freezes the version-thirteen called-constructor
packet under `spec/enum-methods/`. The packet holds a generic `Opt<T>` and a
monomorphic `Pair` built by called tuple-variant constructors in provider
bodies, clients that name imported variants in constructor calls, paths,
patterns, a guard and `match &o`, three opponents (E0004, E0308 and E0061), the
forged constructor-application controls and a 6/24/96 scaling control. The
`crate_linking::constructor_transport` integration controls consume it.

macro files (`docs/parallel/macro-metadata.md`).

`macro-reexport-outcomes.json` freezes the re-export rows of tag-14 version 2
under `spec/macro-reexport-rows/`. A no_core source-core provider
(`exporter.rs`) defines a builtin in a private module, reached only through
re-exports, and adds root and renamed re-exports and a core-shaped prelude
glob. Eight source clients cover paths, a renamed `extern crate`, globs of one
identity, an ambiguous glob pair, a duplicate explicit import, a private path,
a placed row in the macro-use prelude and `#[macro_use]` re-exports.
`path-client.rs` links with the `builtin-macro-rows` native driver. A
source-client `bridge.rs` re-exports the provider's macro for an
inventory-only client. A facade over `internal-provider.rs` has two binary
clients, one of which keeps the gate of an unlisted row. A requirement
provider whose macro module is reached only through an alias is accepted by
rustc and rejected by name. All 15 outcomes were declared before any rustc or
RustGM run.

`macro-stability-outcomes.json` freezes tag-14 version 3 under
`spec/macro-stability-rows/`. A staged no_core provider (`provider.rs`) has an
unstable declarative row, an unstable builtin row, a builtin row with an
allowance list, listed and unlisted wrappers, and a staged re-export. Seven
source clients cover a listed `cfg(ub_checks)` expansion, direct calls with and
without the feature, the unstable builtin, listed wrappers, an unlisted wrapper
and a re-export reached through `use`. `native-client.rs` links with the
`builtin-macro-rows` native driver. A second staged provider
(`api-provider.rs`) has an unstable function called by a listed and an
unlisted row; four clients cover both rows, a direct call, and the listed row
beside `line` and `cfg`. All fourteen outcomes were declared before any rustc or
RustGM run of their files. The provider's re-export stability attribute was
added after oracle round 1, and the API client was made non-native after
round 3 (`docs/parallel/macro-metadata.md`).
