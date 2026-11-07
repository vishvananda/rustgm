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
Consumer import and native execution remain required work
(`docs/parallel/macro-metadata.md`).

`macro-spacing-outcomes.json` freezes definition-site `stringify!` spellings
through an exported macro (`spacing-provider.rs`). It covers a cross-crate 2024
client and a local 2021 copy of the same definition. Each client asserts all
nine spellings, and any mismatch aborts. Round one's prediction (transcription
keeps punctuation jointness) was declared first and refuted. Round two froze
rustc's observed spellings: transcribed tokens print alone, paren and brace
delimiters keep their source spacing, and a bracket group is always followed by
a space. Local and cross-crate spellings are identical.
