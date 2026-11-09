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

`inherent-methods-outcomes.json` freezes the version-thirteen inherent-method
packet under `spec/inherent-methods/`. The packet holds:

- inherent impls on a provider enum and struct, including one on a single
  concrete instance;
- clients that use by-value, auto-ref and path-call receivers;
- the free late-region function `present`;
- four opponents (E0599, E0308, E0382 and E0599);
- generated controls: an older-profile gate, late-region re-proof, forged
  inherent-header and region controls, and a 6/24/96 scaling control.

The `crate_linking::inherent_transport` integration controls consume it.

`imported-panic-outcomes.json` freezes tag-14 version 4 under
`spec/imported-panic-rows/`. A staged no_core provider (`provider.rs`) has
core's builtin `panic`, `assert` and `unreachable` shapes and declarative
wrappers whose allowance lists carry `edition_panic`, one wrapper without it,
and four semiopaque `pub macro` rows in its `panic` module. `native-client.rs`
runs the imported roles in a fresh 2024 consumer and links with
`native-driver.c`. `legacy-native-client.rs` with `legacy-driver.c` is a 2018
client whose wrapper call selects `panic_2021`; `legacy-client.rs` reaches a
2018 `panic_2015` call site, which stays rejected by name.
`legacy-unreachable-client.rs` with `legacy-unreachable-driver.c` runs in 2015
and 2018 and selects `unreachable_2015`, as rustc-demangle calls core's;
`legacy-panic-client.rs` is its 2015 `panic_2015` opponent. Further clients
cover the second `panic_2021` rule (`format-client.rs`), `assert` and
`debug_assert` reaching the provider's missing `Not` item, a direct call of
the unstable row, an uncalled `use` of it with and without its feature and a
glob import of its module. Three requirement providers cover the single-rule
`pub macro` form, a transparent `pub macro` and the operator items an
executable `assert` needs. Eleven outcomes were declared in plan 1, two in
plan 2 (the uncalled `use`), one in plan 3 (the glob) and three in plan 4 (the
2015/2018 source clients), each before any rustc or RustGM run of its files.
Three frontend expectations were amended after candidate rounds 1 and 2
(`docs/parallel/macro-metadata.md`). Plan 4 restored the two 2018 rows'
originally declared outcomes (`docs/parallel/definition-lineage.md`).

`no-std-client-outcomes.json` freezes rustc's `no_std` injection for source
clients, under `spec/no-std-clients/`. The provider is a no_core source-core
`core` with core's per-edition prelude modules. Native object clients in
2015, 2018, 2021 and 2024 reach it through the injected `core` and their own
edition's prelude glob. The opponents are a 2024-only prelude item named in
2021, `crate::core` after 2015, a path through `std`, and a `core` without
prelude modules. All thirteen outcomes were declared before any rustc or
RustGM run of these files. A `compiler_builtins` stand-in serves rustc's
injection in an empty sysroot. The amendment of 2026-11-08 compiles it with
RustGM too, adds it to each `no_std` client's inventory, and adds four cases:
a client with no such crate (E0463), a crate named `compiler_builtins`
without the attribute and its client, and the attribute without its feature
(E0658). It was declared before any rustc or RustGM run of the amended cases.
The variant re-export, `--extern core=` and shim glob clients are amended
the same way.

`variant-reexport-outcomes.json` freezes variant re-exports from a source
core under `spec/variant-reexports/`. The provider's `prelude::v1` re-exports
`Option` with `None` and `Some`, as core's does, and a second module renames
them. Native 2021 and 2024 object clients construct and match the variants
through the prelude and the renamed paths. The opponent names a variant as a
type. All five outcomes were declared before any rustc or RustGM run of these
files.

`extern-core-outcomes.json` freezes `--extern core=` naming a crate's direct
`core` dependency, under `spec/extern-core/`. The `no-std-clients` 2024 and
2015 clients run natively with the alias, a `no_core` client reaches
`core::base` through it, and the opponent aliases a crate without prelude
modules. All four outcomes were declared before any rustc or RustGM run of
these cases; the opponent was redesigned once before any run.

`shim-glob-outcomes.json` freezes glob re-exports of a dependency, the
`rustc_std_workspace_core` shape, under `spec/shim-globs/`. A shim publishes
`pub use core::*;` as one version-five GLOB row, and the `no-std-clients`
clients run natively with that shim as their `core`. The controls cover
`core`'s exported macro, its `pub(crate)` and private names, an explicit
shim module that shadows the glob, and one name with two glob targets, used
and unused. All fifteen outcomes were declared before the rustc oracle and
before any RustGM candidate run of these cases; exploratory probes of
similar shapes ran first. Under the oracle's `--cap-lints=allow`, rustc
accepts the used ambiguity, which RustGM rejects by name. The
`compiler_builtins` amendment adds the exact upstream
`rustc_std_workspace_core` and a native 2024 client through it.

`provider-guard-outcomes.json` freezes the version-thirteen provider-guard
packet under `spec/provider-guards/`. The packet holds:

- match guards in provider bodies: by-value and borrowed scrutinees, retried
  arms, a generic predicate and a guard in an inherent method;
- single-variant enum matches and an irrefutable `let`, with no constructor
  test;
- eight accepted clients and three opponents (E0382, E0271 and E0308);
- generated controls: an older-profile guard gate, the amended enum-match
  `guarded` control, forged guard rows and a 6/24/96 scaling control.

The `crate_linking::guard_transport` integration controls consume it.

`static-str-never-outcomes.json` freezes the version-thirteen `&'static str`
and `!` packet under `spec/static-str-never/`. The packet holds:

- the shapes of core's `panicking::panic(&'static str) -> !` and generic
  `assert_failed`, called from an `if` branch, a match arm and with
  reborrowed arguments;
- static literals returned, forwarded and merged, and a `!` signature;
- eight accepted clients and three opponents (E0308, E0061 and E0599);
- generated controls: older-profile `str` and `!` gates, forged text and
  reborrow rows and a 6/24/96 scaling control.

The `crate_linking::str_never_transport` integration controls consume it.

`panic-attribute-outcomes.json` freezes the version-thirteen panic-attributes
packet under `spec/panic-attributes/`. The packet holds:

- core 1.98's `panicking::panic` attributes and `#[lang = "panic"]` role;
- `#[cold]` advisories, alone and with an inline hint, and `#[track_caller]`
  on a free function and an inherent method;
- three accepted clients, two clients that rustc accepts and RustGM fences
  by name, and two opponents (E0308 and E0061);
- generated controls: older-profile cold, caller-tracking and role gates, a
  pending nonconforming role, forged flag and role rows and a 6/24/96
  scaling control.

The `crate_linking::cold_transport` integration controls consume it.

`lifetime-nominal-outcomes.json` freezes the version-thirteen lifetime-nominal
packet under `spec/lifetime-nominals/`. The packet holds:

- covariant and invariant lifetime structs, a struct whose field implies
  `T: 'a`, and a lifetime enum with a tuple variant;
- free functions over them, a method of a `'static` instance and a provider
  constructor call;
- nine accepted clients and five opponents (three variance errors, E0515 and
  E0309);
- generated controls: older-profile lifetime gates, fenced lifetime bounds and
  captured impl lifetimes, forged slot, region, argument and field rows and a
  6/24/96 scaling control.

The `crate_linking::lifetime_transport` integration controls consume it.

`impl-region-method-outcomes.json` freezes the version-thirteen captured-region
method packet under `spec/impl-region-methods/`. The packet holds:

- methods of `impl<'a> X<'a>` over covariant and invariant lifetime structs,
  an impl with two lifetimes and a type, an impl on `Slot<&'a u8>` and a
  struct that borrows another lifetime struct;
- constructors, by-value and by-reference receivers, a method with its own
  late lifetime, a path call and provider bodies that call the methods;
- thirteen accepted clients and four opponents (two lifetime errors, E0515
  and E0597);
- generated controls: an older-profile method gate, fenced trait-impl,
  early-lifetime and const methods, forged result, region-scheme, impl-header
  and slot rows and a 6/24/96 scaling control.

The `crate_linking::lifetime_methods` integration controls consume it.

`unique-write-outcomes.json` freezes the version-thirteen unique-path write
packet under `spec/unique-writes/`. The packet holds:

- writes and a mutable reborrow through a `&mut` field of an immutable
  by-value binding, a method with an immutable by-value receiver and a path
  through two unique references;
- six accepted clients and two opponents (both E0594);
- generated controls: an older-profile write gate, forged Store and
  dereference rows and a 6/24/96 scaling control.

The `crate_linking::unique_writes` integration controls consume it.

`struct-constructor-outcomes.json` freezes the version-thirteen tuple and unit
struct constructor packet under `spec/struct-constructors/`. The packet holds:

- a monomorphic and a generic tuple struct, a unit struct, a lifetime tuple
  struct whose impl captures its lifetime, and two structs whose private or
  crate-visible field narrows their constructors;
- provider calls, field reads, destructuring and `Self(..)`, and client
  calls, paths, renamed imports, function values, braced literals and
  patterns, constants and unit values;
- eleven accepted clients and nine opponents (three E0603, E0616, E0451,
  E0061, E0308, E0618 and E0597);
- generated controls: older-profile constructor gates, fenced empty tuple,
  non_exhaustive and PhantomData forms, forged parent, name, visibility and
  field rows and a 6/24/96 scaling control.

The `crate_linking::struct_constructors` integration controls consume it.

`const-method-outcomes.json` freezes the version-thirteen const inherent-method
packet under `spec/const-methods/`. The packet holds:

- const methods of a lifetime-capturing inherent impl and of an impl that
  captures none, and a const free function that calls a const method;
- runtime calls, `const` items and a client const function that evaluate
  them, including a field read of a call's temporary;
- ten accepted clients and four opponents (E0015, E0435, E0308 and a lifetime
  error), with one declared static client withdrawn;
- generated controls: older-profile gates, fenced `const unsafe` and early
  lifetime methods, temporary-field reads and writes, forged constness and
  context rows and a 6/24/96 scaling control;
- rustc controls for a temporary-field read and write.

The `crate_linking::evaluated_methods` integration controls consume it.

`pointer-slice-outcomes.json` freezes the version-thirteen raw-pointer and
slice packet under `spec/pointer-slices/`. The packet holds:

- raw pointers of each mutability, to a sized, an unsized and a generic
  pointee, and a record with a raw-pointer field;
- shared slices passed through, held by a lifetime-generic record and of a
  generic element;
- seven accepted clients and four opponents (E0308 three times and E0133);
- generated controls: older-profile gates, forged mutability, pointee,
  element and version rows, a forged slice of a `?Sized` parameter that only
  the consumer's own facts reject, and a 6/24/96 scaling control.

The `crate_linking::addressed_views` integration controls consume it.

`static-sync-outcomes.json` freezes the shared-static Sync packet under
`spec/static-sync/`. The packet holds:

- a provider of the shared lines, which declare no `sync` lang item;
- two accepted clients (a `static mut` read and a provider call) and two
  opponents, a crate-level and a function-local shared static, both rejected
  with rustc's "requires `sync` lang_item";
- generated single-crate source cores: no `sync` item, `u8`, a raw-pointer
  record with no impl, with `unsafe impl Sync` and as `static mut`, and an
  explicit `impl !Sync` (E0277) with its `static mut` form.

The `crate_linking::synced_statics` integration controls consume it.

`transparent-record-outcomes.json` freezes the version-thirteen transparent
record packet under `spec/transparent-records/`. The packet holds:

- `#[repr(transparent)]` tuple, record, generic and raw-pointer structs, the
  record with a trivial `()` second field;
- seven accepted clients, one of which declares its own transparent struct
  over an imported one, and two opponents (E0308 and E0690);
- generated controls: an older-profile gate, a transparent struct with two
  nontrivial fields, forged representation, enum and version rows, and a
  6/24/96 scaling control.

The `crate_linking::newtype_records` integration controls consume it.

`phantom-role-outcomes.json` freezes the version-thirteen `PhantomData` role
packet under `spec/phantom-roles/`. The packet holds:

- core's `PhantomData<T: PointeeSized>`, a function returning one and a
  record holding one beside a value;
- six accepted clients, one of which shortens a `PhantomData<&'b u8>` and one
  of which declares its own struct over an imported `PhantomData`, and three
  opponents: two lengthen a lifetime through an imported formal, and one is
  E0308;
- generated controls: an older-profile gate, the role row retargeted to a
  unit struct with no formal, a record and an enum, respelled and unknown
  roles, the role row deleted, a record whose only use of its formal was
  removed, a version row and a 6/24/96 scaling control;
- the found false rejection's controls: the gate client's `PhantomData`
  unit value and an imported generic enum's unit variant, both without a
  lifetime-generic import, with their rustc outcomes.

The `crate_linking::phantom_roles` integration controls consume it.

`nonnull-pattern-outcomes.json` freezes the version-thirteen `NonNull` and
pattern-type packet under `spec/nonnull-patterns/`. The packet holds:

- core's `pattern_type` macro declaration, core's `NonNull`, core's closed
  niche types `Nanoseconds`, `NonZeroCharInner` and `CodePointInner`, and two
  authored closed ranges;
- eight clients that rustc accepts, each binding `Opt::None` of one imported
  type, and two opponents (E0616 and E0308). RustGM accepts four. The other
  four are pending its scalar-range niche layout and are rejected by name, in
  a fresh consumer and in one source-core crate alike;
- generated controls: older-profile and diagnostic-identity gates, the
  parameter-base fence, a layout witness for each range type and a 6/24/96
  scaling control;
- wire controls: a rewritten constraint kind, rewritten bases and endpoints, a
  well-formed rewritten wrapped row, a respelled, moved, duplicated and deleted
  identity row, and version-12 packets with tags 16 and 254.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::nonnull_patterns` integration controls
consume it.

`auto-declaration-outcomes.json` freezes the version-thirteen auto-trait
declaration packet under `spec/auto-declarations/`. The packet holds:

- core's `Send`, `Sync`, `UnsafeUnpin`, `Unpin`, `UnwindSafe` and
  `RefUnwindSafe` declarations, and three nominals whose auto traits follow
  their fields;
- seven clients that rustc accepts, proving imported markers structurally,
  from a caller bound and for statics, and three opponents: an unbounded
  parameter (E0277) and a client's own `sync` lang item or `Send` diagnostic
  item (E0152 and the duplicate diagnostic item error);
- generated controls: six older-profile declaration gates, and the
  static-sync client without a `sync` item;
- wire controls: `AUTO` set on a trait with members, each role-bound marker's
  unsafety flipped, `AUTO` cleared on `Send`, and the 254 `Send` row moved to a
  struct, duplicated, respelled and deleted.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::marker_transport` integration controls and
the `freezing::payment_tests` private controls consume it.

`auto-impl-outcomes.json` freezes the version-thirteen auto-trait impl packet
under `spec/auto-impls/`. The packet holds:

- the auto-declarations provider, followed by positive and negative auto
  impls shaped as core's and alloc's: raw pointers, references, `Arguments`,
  `Cell`, `Unique`, `Location`, `PhantomPinned` and `Rc`, plus a specialized
  `Foo<u8>` and a self-referential `List`;
- seven clients that rustc accepts, through explicit impls, statics, a
  coinductive cycle and a client's own impls, and nineteen opponents: thirteen
  E0277 cases, an orphan impl (E0117), a foreign auto impl on a reference
  (E0321), the three safety errors (E0200, E0198, E0199) and an explicit
  Freeze impl (E0658);
- generated controls: a strengthened, a specialized and a `?Sized` blanket
  provider impl (E0367, E0366, E0321), a Sized blanket impl that a client
  proves, an older-profile negative impl gate, a client library that would
  export an auto impl, and a linear-scaling chain;
- wire controls: flipped safety and polarity bytes, a generic Self, an `Rc`
  bound retargeted, a widened member range, a version relabel and a foreign
  trait application.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::polarity_transport` integration controls
and the `portable_impls::scaling_tests` private control consume it.

`unsafe-negative-impl-outcomes.json` freezes the version-thirteen unsafe trait
and non-auto negative impl packet under `spec/unsafe-negative-impls/`. The
packet holds:

- the lifetime-nominals provider, followed by authored impls shaped as
  core's: negative impls of traits with members (`!Clone for &mut T`,
  `!DerefMutLike for &T`, `!Iterator for [T]`), an `Error` family, the unsafe
  `TrustedFused` and `Searcher` with conditional, scalar and nominal unsafe
  impls whose methods have and lack receivers, and a safe negative impl of an
  unsafe trait;
- six clients that rustc accepts, among them `negative-coherence`, a
  recorded negative-coherence gap until Lane B's MetaSized batch amended it
  to accepted, and ten opponents: six E0277 cases, the three safety errors
  (E0200, E0199, E0198) and a positive impl beside a provider negative
  (E0751);
- generated controls: core's same-head `Error` pair (C3b's gap), and
  older-profile gates for an unsafe trait and for a negative impl of a trait
  with members;
- wire controls: flipped impl safety and polarity bytes, `UNSAFE` cleared on
  both unsafe traits, and a version relabel.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::modifier_transport` integration controls
consume it.

`knowable-absence-outcomes.json` freezes the coherence packet under
`spec/knowable-absence/`, which re-proves imported and client pairs from the
crate whose coherence each proves. The packet holds:

- the unsafe-negative-impls provider plus core's `impl !Error for &str {}`,
  the same-head pair beside `Error for &T`;
- six clients that rustc accepts, whose absence goals are knowable through
  the packet's local trait, a client-local Self, `&mut Local`, or a local
  trait argument;
- seven opponents, each rejected at its own impl: five E0119 cases, whose
  missing impl another crate could add, and two orphan-rule cases (E0210,
  E0117);
- a generated scaling control of N same-head pairs;
- generated controls that an imported impl's where-clauses are read in
  coherence: one that holds and keeps its pair overlapping, and `?Sized`
  conditions under negative coherence, among them `negative-metasized-param`,
  a recorded gap until Lane B's MetaSized batch amended it to accepted.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::absence_transport` integration controls
and the `overlap::disjointness` scaling control consume it.

`imported-formation-outcomes.json` freezes the formation packet under
`spec/imported-formation/`, in which a client forms each imported nominal
under its transported where-clauses. The packet holds:

- the knowable-absence provider plus authored, alloc-shaped lines: `Allocator`,
  `Global`, `Rc<T: ?Sized, A: Allocator>`, `Bounded<T> where T: Copy` and
  `Nested<A: Allocator>`;
- six clients that rustc accepts: a satisfying argument, a caller bound, a
  client impl of the imported trait, a `?Sized` argument, a holding
  where-clause and a source-local positive;
- seven imported opponents, each E0277 in rustc and each rejected at the same
  kind of site as its local twin: a signature, a generic signature, a field, a
  reference, an impl header, a where-clause and a nested nominal;
- `slot-unsized`, whose imported Sized edge was already enforced, and four
  source-local twins, all unchanged.

The `crate_linking::where_clause_transport` integration controls and the
`nominals::requirements` payment and scaling invariants consume it.

`macro-context-outcomes.json` freezes the version-thirteen macro-context
packet under `spec/macro-contexts/`, B2's macro-context slice of LIB-059. The
packet holds:

- private `macro_rules!` expansions, nested and with a root captured operand,
  in exported bodies, and an exported `$crate` macro the provider also invokes;
- four accepted clients, one of which expands the imported macro, and one
  opponent (E0308);
- generated controls: older-profile gates, a local, label and parameter bound
  by marked tokens, a macro 2.0 expansion and item, a macro-generated
  exported macro and an unused exported `$crate` macro.

The `crate_linking::hygienic_exports` integration controls consume it.

`const-parameter-outcomes.json` freezes the version-thirteen const-parameter
packet under `spec/const-parameters/`, B2's const-parameter slice (K). The
packet holds:

- const-generic nominals, impls and functions over usize, i8, bool and char,
  with the const slot before and after a type slot, and body applications
  with evaluated and rigid const arguments, own and inherited;
- four accepted clients and four opponents (E0308 three times, E0747);
- generated controls: older-profile gates, a body reading `N` as a value and
  a const default, beside forged terms, values, slots and arguments.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::valued_generics` integration controls
consume it, and the `valued_slots` validation invariant reads its provider.

`array-type-outcomes.json` freezes the version-thirteen array-type packet
under `spec/array-types/`, B2's array type row (T). The packet holds:

- whole `[T; N]` values in signatures, record and tuple-struct fields, nested
  arrays, a const-generic record, the `[T; N]` Clone and Copy impl headers,
  and bodies that copy and store whole arrays;
- five accepted clients and four opponents (E0308 three times, E0382);
- generated controls: list, repeat, element-pattern, unsizing and indexing
  fences, the older-profile gate and scaling, beside forged terms, slots,
  elements and codec rows.

The indexing control's text is frozen as observed after one approved probe.
The `crate_linking::whole_arrays` integration controls consume it, and the
`dense_lengths` validation invariant reads its provider.

`reified-item-outcomes.json` freezes the reification packet under
`spec/reified-items/`. Its provider bodies return fn items reified to closed
Rust fn pointers, and its clients call those pointers. The packet holds:

- the callable-roles provider plus authored lines;
- nine clients that rustc accepts:
  - a safe item;
  - a generic instance;
  - a formation-requesting generic;
  - an inherent method;
  - a trait method;
  - a rigid-`Self` method in a const fn, at a client and at a provider type;
  - a safe item reified to `unsafe fn`;
  - an unsafe item;
- six generated provider opponents that keep their export fences by name, four
  former opponents that export since R2 (amendments 1 and 2), and the
  older-profile, old-reader, forged-row, scaling and flip-survey controls.

The `crate_linking::reified_items` integration controls and the
`input_plan::signatures` row and scaling invariants consume it. The
nonnull-patterns layout witness's single-crate control also depends on it.

`pointer-call-outcomes.json` freezes the pointer-call packet under
`spec/pointer-calls/`, Lane C's R2. Its provider bodies call closed Rust fn
pointers and return reified constructors. The packet holds:

- the reified-items provider plus authored lines;
- 13 clients that rustc accepts:
  - a pointer parameter, two nested calls and a generic `fn(T) -> T`;
  - a body that reifies an item and calls it;
  - unsafe pointers in unsafe blocks, and one called bare in an unsafe fn;
  - a safe pointer in an unnecessary unsafe block;
  - the fmt-shaped formatter twin, and a call in an inherent associated fn;
  - struct, variant and generic constructors;
- six generated provider opponents that keep their export fences by name,
  and the older-profile, old-reader, wire, forged-row, scaling and
  flip-survey controls.

The `crate_linking::called_pointers` integration controls, the
`input_plan::values` row and scaling invariants and the
`ordinary_columns::ir_nodes` wire invariant consume it.

`named-constant-outcomes.json` freezes the named-constant packet under
`spec/named-constants/`, Lane C's item 5b-1. Its providers export named
constants of primitive and unit types under version 13, and their bodies read
them through value code 17. The packet holds:

- four providers, each the branch provider's first 57 lines plus authored
  lines: the module-constant provider, the unnamed-constant pair and the
  generic item-local provider;
- thirteen clients that rustc accepts: runtime reads, a client const, an array
  length, a pattern, imported bodies reading public, private and item-local
  constants, f32, f64, u128 and unit values, an imported const fn as an array
  length, three same-named item-local constants, the constant after an
  unnamed one and a generic body's item-local constant;
- three opponents (E0308 twice, E0603), each rejected as its local twin is;
- nine generated provider opponents that keep their export fences by name, and
  the older-profile, old-reader, forged-row, scaling and flip-survey controls.

Amendments are recorded beside the declarations they amend, each before the
run it governs. The `crate_linking::named_constants` integration controls and
the `constant_tests` codec and validation invariants consume it.

`scheduling-context-outcomes.json` freezes the scheduling-context packet under
`spec/scheduling-context/`, Lane C's fence item after 5b-1. Its clients
const-evaluate imported and local const fns while declarations are prepared,
before the eager implementation walk. The packet holds:

- eight providers, each the branch provider's first 57 lines plus authored
  lines: no Copy impl, one primitive's, a unit struct's, fourteen
  primitives', a blanket marker impl and a Copy-bounded generic const fn;
- twelve clients in 35 cases. 14 cases flip from the base pin's "implementation
  validation has no source scheduling context" to accepted, and 17 accepted
  cases are unchanged;
- four rejected cases: two overlap clients that rustc rejects with E0119, the
  client-local Copy rejects-valid and one failed prediction at an existing
  cleanup fence;
- the specialization-gate, selected-mode, scaling and flip-survey controls.

Its amendments are recorded beside the declarations they amend, each before
the run it governs. The `crate_linking::scheduling_context` integration controls
and the `executable_demands` and `unordered_imports` invariants consume it.
