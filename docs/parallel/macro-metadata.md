# Exported macro metadata

Lane B, task 2 (PA29). Library metadata carried no `macro_rules!` definitions,
so a crate that exports one (cfg-if is the first target unit) could not publish
metadata, and no consumer could invoke `krate::m!` or `#[macro_use]` macros. This
checkpoint fixes the design, freezes independent outcomes and implements the
provider half. The consumer half is listed under "Remaining work".

## Which metadata path

`--crate-type=lib --metadata-output` without a source-core profile goes through
`lower::instances::build_library_prepared` and
`sema::crates::export_with_originals`. Before this change `public_names::collect`
saw the root binding of each `#[macro_export]` macro and failed with "exposed
declaration is outside the scalar metadata profile". Original cfg-if 1.0.4
failed this way. The macro table is a separate outer envelope. Lane C's version
thirteen columns are untouched.

## Wire profile (envelope tag 14, version 1)

Canonical nesting is: native links (tag 12), then macros (tag 14), then the core
profile. A crate without exported macros writes no envelope, so its bytes are
unchanged. The decoder rejects trailing bytes and a macro or link envelope nested
inside a macro envelope.

Each row, sorted by unique name, holds:

- the crate-root name (ASCII identifier, at most 256 bytes);
- the definition edition (0–3 for 2015–2024);
- flags: bit 0 is `local_inner_macros`; any other bit is rejected;
- the rule count, which must be at least 1;
- the canonical image: the spelling of the single delimited body. Two pieces
  (tokens, open and close delimiters) are adjacent exactly when they were
  adjacent in the definition source. One space separates them exactly when
  whitespace or comments did. See
  [Jointness-preserving image](#jointness-preserving-image);
- typed preorder rows. A token row is `(class, byte length)`, with 15 closed
  classes; punctuation identity is its spelling. A group row is
  `(0x80 | delimiter, descendant rows)`.

`support::crates::exported_macros` checks, without a lexer, that the rows lay
out the image exactly. This covers the one root group, every optional
single-space separator, every delimiter byte, group containment and token
boundaries. Unknown edition, flag,
class and delimiter codes are rejected by name. Budgets: 4,096 macros, 1 MiB of
image and 2^18 rows per macro, plus the shared component and 16 MB artifact
limits.

## Provider (implemented)

`sema::crates::export::declarative_rows` selects every direct public
`Namespace::Macro` binding in the crate root whose definition is `DefKind::Macro`.
These are the `#[macro_export]` definitions, which resolve already places at the
root whatever their defining module. `public_names` skips exactly those bindings.
Re-exports and other macro bindings still reach the old by-name profile
rejection.

For each binding, the provider takes the expansion definition at the same
`NodeKey` and walks its retained body capture once with
`resolve::crates::token_images::canonical`. Consumers will use the same walk.
The following are rejected by name rather than dropped:

- **Macro-generated definitions** (introduction ≠ 0). Their tokens may carry
  several editions and hygiene marks, which this profile does not transport.
- **Doc comments in the body.** Doc trees are typed desugarings, not tokens.
- **Builtin or scoped (`macro`) bodies.**
- **Checked definition-site permissions:** source macro contracts and edition
  panic.

The edition is the definition's edition (`editions::at` of the item). One edition
per macro is exact because generated definitions are excluded. Every token of an
ordinary definition comes from one file.

## Consumer (design)

Implemented as designed; see [Consumer import](#consumer-import) for the decisions.

- **Authentication.** Never trust provider rows or proofs. Lex the transported
  image once with `Forest::parse`. Rebuild `(image, rows)` with `canonical` and
  require them to equal the transported pair; any difference is rejected by name.
  Then compile with `compile::in_edition(image, forest, names, edition)` and
  require the transported rule count. Matching and transcription later reuse that
  compiled forest. Generated spellings are never re-lexed.
- **`$crate`.** Transcription already writes `Kind::CrateRoot(environment.crate_id)`,
  with id 0 for the local crate, and `Package::append_path` already rejects any
  other id ("external defining crate requires metadata"). The consumer gives each
  imported artifact the id 1 + its dependency index. It maps that id to the
  artifact's root `DefId` via `ExternalDef(identity, [], Module)`, which is the
  crate identity (name and disambiguator), never the alias or name.
- **Hygiene and spans.** External definition tokens get the root raw context of
  their image and the normal semi-transparent mark of each invocation, exactly
  like local `macro_rules!`. Items named by definition tokens resolve at the call
  site, and locals and labels stay hygienic. Diagnostics inside a definition point
  into the image. Three lookups need a package `Unit` for the image:
  `unit::contexts::at` (it returns ROOT inside the physical source),
  `unit::editions::at` (the unit's own edition) and transcription, which reads
  `Definition::node.file` for the definition text. One candidate is an image unit
  parsed once as a provenance-only include template
  (`Unit::parse_include` with an items context, as cached `include!` templates
  are kept today). It must be held out of module traversal and the
  unvisited-syntax checks. Its `Config` edition is the transported edition.
  Settle this before writing the consumer.
- **Binding forms.**
  - `use krate::m;` and `krate::m!`: a `DefKind::Macro` definition bound
    publicly in the imported root scope (`Namespace::Macro`), classified as
    `MacroTarget::Declarative` in `Paths::macro_targets`.
  - `#[macro_use] extern crate krate;`: every row of that artifact enters the
    macro-use prelude in source order, and a later crate shadows an earlier one
    (frozen `shadow-client`).
  - Two `use` imports of one name collide (`duplicate-use-client`).
- **Editions.** Fragment grammar follows the transported definition edition,
  whatever the consumer's edition (`use-client-2018`, `pat-client`,
  `pat-client-provider-2018`).

## Frozen independent outcomes

`pa29/tests/macro-metadata-outcomes.json` was frozen with pinned rustc 1.98.0
(`3690cc57…`). The 12 predictions were declared first, in plan `47fda844…`
(`obj/parallel/macro-metadata-v1/plan.json`, oracle `251df089…`). All 12 rustc
outcomes matched them (receipt `075f03c2…`):

| Case | Editions (consumer / provider) | rustc | Native |
|---|---|---|---|
| provider, other, provider-2018 (rlib) | 2021, 2021, 2018 | 0 | |
| use-client: `use` import, `$crate::f` beats local `f`, 2021 `pat` or-pattern, inner `#[macro_export]` at root | 2024 / 2021 | 0 | exit 0 |
| use-client-2018 | 2018 / 2021 | 0 | exit 0 |
| macro-use-client (`#[macro_use] extern crate`) | 2015 / 2021 | 0 | exit 0 |
| pat-client | 2015 / 2021 | 0 | exit 0 |
| pat-client-provider-2018 | 2024 / 2018 | 1 (no rules expected `\|`) | |
| private-client (unexported) | 2024 | 1 E0433 | |
| inner-path-client (`provider::inner::m!`) | 2024 | 1 E0433 | |
| duplicate-use-client | 2024 | 1 E0252 | |
| shadow-client (later `#[macro_use]` shadows; path form unaffected) | 2015 | 0 | exit 0 |

The PA34 `mixed` rows in `legacy-edition-outcomes.json` (2018/2021 `pat`
providers and original cfg-if consumers) remain the edition-mixing requirements
for library consumers.

## Qualification (provider slice)

`make fastcheck` passed 3 exact `crate_linking::declarative_exports` tests, plus
strict Clippy, format and architecture
(`obj/qualification/fast/1794044105558983582`):

- **Publication.** The `provider.rs` fixture publishes `call_f`, `nested_export`
  and `select`, but not `private_only`. The image of `call_f` was
  `{ () => { $ crate :: f () } ; }`, with its typed rows. The jointness
  checkpoint amends it to `{ () => { $crate::f() }; }`; the rows are unchanged. Decode, re-encode and
  equality round-trip byte-identically.
- **Old bytes.** A crate with only an unexported `macro_rules!` encodes to the
  same bytes as the same crate without it, and never uses tag 14.
- **Forgery.** Each of these is rejected by name: version, edition, flag,
  delimiter and class codes, a relabelled group row, a respelled image delimiter,
  a zero rule count, trailing bytes, a nested envelope and truncation.
- **Unsupported definitions.** Macro-generated and documented exported
  definitions fail by name.

The pinned dev-fast CLI (`obj/parallel/macro-metadata-v1/slice1/rustgm`,
`0946e5aa…`) exports:

- `provider.rs` with edition 2021, as `provider.rgm` (`5bcfa0eb…`);
- original cfg-if 1.0.4 with edition 2018, as `cfg_if.rgm` (`e69107ca…`): one
  `cfg_if` row, edition 2018, 4 rules and a 920-byte image.

## Remaining work

1. **Consumer import.** Done: see [Consumer import](#consumer-import).
2. **Native qualification.** Done: `scripts/check_macro_metadata.py`.
3. **cfg-if.** The PA34 consumers are rerun under [Consumer import](#consumer-import).
   The target-unit executor's cfg_if unit stays blocked behind core (unit 5,
   the `flatten.rs` gap), since its `rustc-dep-of-std` build depends on core.
4. **Spacing.** Done: see [Jointness-preserving image](#jointness-preserving-image).
5. **Core's exported macros.** See [Core census](#core-census-of-exported-macros).

## Jointness-preserving image

Follow-up (Lane B, task 2). The image no longer puts one space between every
pair of pieces. It stays envelope tag 14, version 1; no published artifact needs
the old layout.

**Rule.** `token_images::canonical` walks the preorder forest once. Each piece
is a token's span, a group's open byte or its close byte (a group span runs from
its open to its close delimiter). A piece is *separated* exactly when the
previous piece ends before it starts. `ImageBuilder` then writes one space before
a separated piece, and nothing before an adjacent one. The first piece is never
separated, and overlapping pieces are rejected by name. `validate_layout`
accepts one optional space before every piece but the first, including a close.
Since no token begins or ends with a space, the layout of rows over an image is
unambiguous.

**Uniqueness and re-lex.** The forest comes from the lexer, so every gap between
two pieces is whitespace or comments. Hence:

1. Each maximal run of adjacent pieces is the exact source substring.
2. Every boundary between runs was whitespace or a comment in the source, and is
   one space in the image.
3. A Rust token never spans whitespace or a comment. A token's end never depends
   on whether the next byte is whitespace, `/` (a comment start) or the end of
   input: no token ends in `/` or extends over a following `/`, a float such as
   `1.` and a lifetime such as `'a` decide the same way, and the image starts
   with a delimiter, so there is no shebang.
4. The lexer is edition-independent.

So lexing the image yields the same tokens in the same order, with a gap exactly
where the source had one. `canonical` over that forest rebuilds the same image
and rows. The image is a function of the definition's tokens and their
adjacency, and it is the unique fixed point of this re-lex. This is the
consumer's authentication. Each token's jointness (the next byte is not
whitespace and does not start a comment, as `TokenSources::joint` reads it)
therefore equals its source jointness.

**Edge cases** (private tests in `dev/tests/invariants/joint_images.rs`; each
case lexes the body, builds the image, lexes the image again and requires the
same image and rows, then passes `Macro::new`):

- `$crate::f()`, `$x+1` and `$($e:expr),*` stay adjacent, while `$ x : ident`
  keeps its spaces.
- Block, nested and line comments, tabs and newlines each become one space,
  including between punctuation that would otherwise be joint
  (`+/**/=` becomes `+ =`).
- Delimiters next to tokens (`f(a)[b]{c}`), empty groups with and without a
  space (`( )[]{ }`) and a leading or trailing comment inside a group
  (`{ a }`).
- Lifetimes against characters next to delimiters (`'a 'b' ('c) ('d')
  'static&'e`).
- Raw strings that contain double spaces, comment text and quotes, and raw
  identifiers, raw lifetimes, suffixed numbers, `1.` and byte, C and raw C
  strings.
- Doc comments are still rejected by name.
- The layout check accepts only adjacency or a single space: two spaces, a tab,
  a leading or trailing space and a doubled space before a close are rejected.
  `{ab}` passes the layout check for two identifier rows, but its re-lex is one
  identifier, so a consumer rejects it.

**Independent control.** `pa29/tests/macro-spacing-outcomes.json` freezes
pinned rustc 1.98.0 (`3690cc57…`) on `spacing-provider.rs`. Its `spelled!(0..=8)`
each expand to a definition-site `stringify!`, with `spacing-client.rs` (2024,
cross-crate) and `spacing-local.rs` (2021, the same definition locally).

- **Round one** (plan `3fe406ac…`, receipt `062ee0e8…`) predicted that
  transcription keeps punctuation jointness. That was refuted: 7 of the 9
  spellings differed.
- **What rustc does.** It prints every transcribed definition token as alone
  (`a+=b` gives `a += b`), but keeps paren and brace delimiter spacing
  (`f(a)`, `{c}{ d }`). It always spaces after a bracket group (`[b] {c}`); this
  is rustc's `hex-literal` transcription workaround. Local and cross-crate
  spellings are identical.
- **Round two** (plan `3677b36b…`, receipt `2720c250…`) froze those spellings
  as the clients' assertions, and all three cases matched (native exit 0).

Delimiter adjacency is therefore what a cross-crate `stringify!` can observe.
Under RustGM's printer rules the one-space image would have printed `(a)[b]`
and `{c}{ d }` as `(a) [b]` and `{ c } { d }`.

The control also exposed a local RustGM divergence: a transcribed `]` kept its
source jointness. `TokenSources::joint` now treats a definition-introduced close
bracket as alone, like the other transcribed tokens. A new
`crate_expansion::stringify_uses_template_spacing…` row,
`f(a)[b] {c}{ d } [e].f [g] + h`, was checked against pinned rustc. With that
fix the local spellings match rustc. The cross-crate client needs the consumer
(task 3).

### Qualification (image)

These controls were declared before the runs.

- **Private tests.** `make fastcheck` passed both `token_images::tests`, plus
  strict private Clippy, format and architecture
  (`obj/qualification/fast/1794048163032447092`).
- **Integration tests.** `make fastcheck` passed 7 exact tests, plus CLI Clippy
  and the PA8 text suite (`obj/qualification/fast/1794048295573044366`):
  - the 3 `crate_linking::declarative_exports` tests. The `call_f` image
    assertion is amended to `{ () => { $crate::f() }; }`; the rows and every
    forgery rejection are unchanged;
  - 4 `crate_expansion::stringify_*` tests, including the new bracket row.
- **Native local control.** The dev-fast CLI compiled `spacing-local.rs`, the
  backend linked it, and it exits 0 (`obj/parallel/macro-spacing-v1/rustgm-local/`).
- **Oracle evidence.** Plans, the oracle and both receipts are under
  `obj/parallel/macro-spacing-v1/` (`plan.json`, `rustc/` and `round2/`).

## Consumer import

Follow-up (Lane B, task 3). A client now imports, authenticates and invokes
exported `macro_rules!` rows.

**Decisions.**

- **Binding.** `resolve::crates::bound_macro_rows` binds every row of an
  artifact publicly in that artifact's imported root scope, as a
  `DefKind::Macro` definition in `Namespace::Macro` with no source node. So
  `use krate::m;` and `krate::m!` take the ordinary import and path routes, and
  two `use` imports of one name collide. Nothing is bound by name outside the
  provider's root.
- **Authentication, once.** Before the first call is attempted,
  `expansion::transported_macros` lexes each image with `Forest::parse`. It
  rebuilds `(image, rows)` with `canonical` and requires equality
  ("does not rebuild its canonical rows"). Then it compiles that same forest with
  `compile::in_edition` in the transported edition and requires the rule count
  ("rule count changed"). Matching and transcription reuse the compiled forest.
  Generated spellings are never re-lexed.
- **Image unit.** `Unit::image` gives each image a package unit whose source text
  is the image and whose edition is the transported edition. It holds an empty
  items fragment, so module traversal and the unvisited-syntax checks never see
  it. Its label names the provider identity and the macro
  (`<macro name@disambiguator::m>`). Definition tokens therefore get the image's
  root context and the normal semi-transparent mark of each invocation, exactly
  like a local `macro_rules!`. A failed match is reported as
  `defined at file N: 0`, inside the image.
- **Definition record.** `expansion::Definition` gains
  `imported: Option<ImportedDefinition>`, holding the image file, the crate id
  and the provider root scope. It keeps `node: None`, so the `(module, node)`
  keys of hygiene and node scopes cannot collide. One `site_scope` helper serves
  invocation scopes and permission-frame checks. One `spelling` helper gives
  transcription the image file and the crate id.
- **`$crate`.** Transcription writes `Kind::CrateRoot(1 + dependency index)`.
  `append_path` turns a nonzero id into the unspellable root `$crate:<id>`.
  `lookup_path` resolves that root through `Imported::macro_root`, which was
  registered from the artifact's `ExternalDef(identity, [], Module)`. It is never
  resolved through a name or alias, so a renamed `--extern` still reaches the
  provider. `local_inner_macros` prepends the same root for an imported
  definition, and `crate` for a local one.
- **`#[macro_use] extern crate`.** When the extern crate item is declared, every
  row of that artifact enters the macro-use prelude (`macro_prelude`), in source
  order, so a later crate replaces an earlier one. Waiting calls wake through the
  ordinary `declared` journal, as for the test prelude. Path forms are
  unaffected. Behind textual and path-based scopes, the macro-use prelude can
  also replace a bootstrap builtin of the same name. rustc likewise ranks the
  macro-use prelude above the standard prelude, but no frozen case covers this.
- **Editions.** The fragment grammar is the definition edition's, whatever the
  consumer's edition.

**Frozen outcomes.** `scripts/check_macro_metadata.py`, modelled on
`check_public_names.py`, adds per-case editions and runs both frozen files
(15 cases). For each case it checks:

- the RustGM frontend exit, with no diagnostic on success and no metadata on
  failure;
- LowIR that does not change when the dependency order is reversed;
- byte-identical repeated publication;
- the pinned rustc exit and error codes.

With `--native`, every passing binary is linked by the backend with only its
own dependencies' LowIR, run beside the rustc binary, and must match its exit
status and output.

**Rejections** all fail for rustc's reason:

| Case | RustGM | rustc |
|---|---|---|
| `pat-client-provider-2018` | no macro rule matches this invocation | no rules expected `\|` |
| `private-client` | unresolved macro is not in textual scope or path scope | E0433 |
| `inner-path-client` | unresolved macro is not in textual scope or path scope | E0433 |
| `duplicate-use-client` | duplicate imported macro name `call_f` | E0252 |

**PA34 mixed rows and cfg-if.** All six `mixed` rows of
`legacy-edition-outcomes.json` now match rustc through metadata
(`obj/parallel/macro-consumer-v1/mixed/`):

- the 2018 `pat` provider fails with both a 2024 and a 2018 consumer;
- the 2021 provider passes with both consumers;
- original cfg-if 1.0.4 (edition 2018) publishes its metadata, and
  `cfg-if-consumer.rs` passes through LowIR as a 2024 and as a 2015 library.

An unfrozen native probe (2015, an `if`/`else if`/`else` chain selecting `unix`)
links the cfg-if LowIR and exits 0. The PA34 consumer rows' `rustgm` requirement
("imports carry no macro_rules definitions") is now met. The frozen PA34 file
is left unchanged; its rows record rustc outcomes.

### Qualification (consumer)

These outcomes were declared before the runs. The frozen PA29 files and the
PA34 file are unchanged.

- **Integration tests.** `make fastcheck` passed 12 exact tests, plus CLI Clippy
  and the PA8 text suite (`obj/qualification/fast/1794049916652985002`):
  - the 3 `crate_linking::transported_rows` tests, whose runner test checked all
    15 cases with the current CLI;
  - both `declarative_exports` tests, `imports`, `reexports`, the 3
    `crate_expansion::stringify_*` tests, `census_mode` and `inventory_ordering`.
- **PA34 edition selections.** The 2 `legacy_editions` and 4
  `edition_fragments` tests passed (`obj/qualification/fast/1794050802031934213`).
- **Private tests.** Strict private Clippy passed, with the 2 `prelude_scope`,
  2 `token_images` and 2 `choice_dispatch` tests
  (`obj/qualification/fast/1794050707776525625`).
- **Harness.** `make test-harness` accepts the new root and runner
  (`obj/parallel/macro-consumer-v1/harness.log`).
- **Runner.** Both runs passed 15 of 15 cases:
  - non-native (`obj/parallel/macro-consumer-v1/run1/complete.json`);
  - native (`native1/complete.json`): all 7 binaries exit 0 and match rustc's
    stdout and stderr.

  An earlier CLI produced these runs; only the image-frame diagnostics changed
  after them. Every LowIR and metadata file in `native1/` is byte-identical to
  the current CLI's run
  (`obj/parallel/pa29-macro-metadata/maintainer-2009733-false-1794050005222854758`).
- **Amended control.** `choice_dispatch`'s sticky-failure check charged
  `1_000_001` units, the former flat limit plus one. That no longer exhausts a
  ledger whose allowance has grown with admitted source bytes since the
  source-proportional allowance landed. It now charges `usize::MAX`; the asserted
  failure is unchanged. Its fixture also gains `imported: None`.

## Core census of exported macros

Analysis only (Lane B, task 4). This is a source census of the pinned core
inputs (`original_core_manifest.json`, 722 files): `library/core/src` and the
`core_simd` sources that core includes. The scanner and its rows are
`obj/parallel/macro-consumer-v1/census-macros.py` and `census-rows.json`. Each
hit was cross-checked by hand against the `#[macro_export]` lines; the scanner
missed `r#try` and wrongly flagged a doc-comment example (`make_error` in
`hint.rs`), and both are corrected here. The portable-simd tests and test
helpers are not compiled into core and are excluded.

| Kind | Exported | Builtin | Ordinary | Ordinary with definition permissions |
|---|---|---|---|---|
| `#[macro_export] macro_rules!` | 38 | 23 | 15 | 7 (`allow_internal_unstable`) |
| `pub macro` (macros 2.0) | 64 | 35 | 29 | 17 (`allow_internal_unstable`) |
| Macro-generated | 0 | | | |
| Doc comments in the body | 0 | | | |
| `local_inner_macros` | 0 | | | |

- One more `pub(crate) macro`, `const_eval_select`, is not exported.
- **Builtin `macro_rules!` (23):** `assert`, `cfg`, `column`, `compile_error`,
  `concat`, `concat_bytes`, `const_format_args`, `env`, `file`, `format_args`,
  `format_args_nl`, `include`, `include_bytes`, `include_str`, `line`,
  `log_syntax`, `module_path`, `option_env`, `panic`, `pattern_type`,
  `stringify`, `trace_macros` and `unreachable`.
- **Ordinary `macro_rules!` (15):** `assert_eq`, `assert_ne`,
  `assert_unsafe_precondition`, `debug_assert`, `debug_assert_eq`,
  `debug_assert_ne`, `impl_partial_eq`, `impl_partial_eq_n`,
  `impl_partial_eq_ord`, `matches`, `r#try`, `todo`, `unimplemented`, `write` and
  `writeln`.
  - Today's profile admits only the 8 without definition-site permissions.
  - The other 7 (`assert_eq`, `assert_ne`, `debug_assert`, `matches`, `todo`,
    `unimplemented`, `writeln`) carry `#[allow_internal_unstable(...)]`.
- **Ordinary `pub macro` (29):** these include `assert_matches`, `pin`,
  `addr_of`, `offset_of`, `ready`, the `panic_2015`/`panic_2021` family and
  `simd_swizzle`. They have definition-site hygiene, which this envelope does not
  carry.

**Proposed next extension.** Builtin rows are the largest group: 58 of 102,
spread across both syntaxes. They need no image. A builtin row would transport:

- the exported name;
- the syntax (`macro_rules!` or `macro`);
- the builtin identity, from the closed `Builtin` vocabulary;
- its macro kind (bang, attribute or derive).

The consumer would check the identity against its own catalog and bind
`MacroTarget::Builtin`, exactly as the bootstrap prelude does, with no tokens.
The rows could live in the same tag-14 table under a row-kind byte. That is a
version bump, since today's decoder rejects unknown flags.

Two later steps would follow. First, typed `allow_internal_unstable` feature
lists, which the consumer re-checks against its own feature policy; this would
admit the other 7 ordinary rows. Second, macros-2.0 definition-site hygiene for
the 29 ordinary `pub macro` rows.

## Builtin rows and allowance lists (version 2)

Follow-up (Lane B, PA29). The integrator approved the row extension as tag-14
version 2. Version-one envelopes keep their exact bytes and decoding, and a crate
whose only exports are ordinary `macro_rules!` rows without allowance lists still
writes version 1.

**Wire layout.** The header and the wrapped inner artifact are those of version
1. The table is a `u32` count of rows sorted strictly by (placement, name).
Every row starts with a kind byte, its module placement (`u16` component count,
then each component as a string) and its name:

- **kind 0, declarative.** The version-1 fields follow (edition, flags, rule
  count, image, typed rows). After them comes the row's sorted, unique
  `allow_internal_unstable` list: a `u32` count of feature identifiers. Its
  placement must be empty ("declarative exported macros belong to the crate
  root").
- **kind 1, builtin.** No image. The syntax byte (0 `macro_rules!`, 1 `macro`)
  comes next, then the builtin identity as a bounded spelling (an identifier of
  at most 64 bytes), the macro kind (0 bang, 1 attribute, 2 derive) and the
  sorted, unique derive helper attributes. A `macro_rules!` builtin row belongs
  to the root, and only a derive row may declare helpers.

Unknown row kinds, syntaxes and kinds are rejected by name. So are unsorted or
duplicate rows, and a version-2 table that needs neither builtin rows nor a
feature list ("version-two exported macro table has only version-one rows"). A
version-1 decoder rejects version 2 as an unknown version. The identity travels
as a spelling because the support layer cannot depend on the expander catalog.
Budgets: 64 placement components, 64 features or helpers per row, and one
macro-table atom per row plus one per placement component.

**Provider.** `sema::crates::export::declarative_rows` now walks every module
reachable from the root through public, direct module bindings. A public macro
binding of a local definition becomes a row:

- A `#[rustc_builtin_macro]` definition publishes its module path, syntax,
  expander identity and kind, and its derive helpers. Its claimed header role
  must equal its body's role.
- A `#[macro_export] macro_rules!` row publishes its sorted feature list.
  `allow_internal_unsafe` and edition-panic contracts stay rejected by name.

Macro re-exports (core's prelude shape), macros 2.0 definitions (no hygiene
transport) and macro-generated definitions are rejected by name. Macro bindings
no longer reach the scalar public-name, authored-export or module-contract
tables. The source-library profile (`--source-core`) now wraps its publication
in the same macro table, so a no_core provider can export builtin rows.

**Consumer.** `resolve::crates::bound_macro_rows` binds each builtin row
publicly in the member scope of the provider's own module at its placement.

- A source artifact resolves the path by walking its public module declaration
  rows and then its origin table. An ordinary artifact resolves it by the
  module's external identity. Neither route uses a binding name or an alias, so
  a renamed `extern crate` or a consumer module of the same name changes
  nothing.
- `expansion::transported_macros` binds the consumer's own expander of that
  identity. It rejects an unknown identity ("names unknown expander"), a changed
  kind ("changed the macro kind of expander") and helpers on a non-derive by
  name.
- Only context-free expanders execute: the bootstrap prelude roles except
  `option_env` and the source-only roles. These are `cfg`, `line`, `column`,
  `file`, `module_path`, `stringify`, `concat`, `compile_error`, `env`,
  `include`, `include_str` and `include_bytes`. Any other imported identity
  (derives, `test`, `panic`, `assert`, `format_args`) binds, so resolution sees
  it, but its invocation fails with "imported builtin macro requires its
  provider's source declaration".

**Allowance lists.** A transported row's feature list is re-checked as feature
identifiers when the row is imported, and the consumer pays one unit per feature
plus its bytes. An expansion frame of that row has no source owner. The
allowance query (`permission_session`) answers from the row's list exactly where
a local definition's header answers, and charges `1 + query work + features` per
query. The frame never grants `allow_internal_unsafe`. An ordinary crate that
imports any listed row now owns a local allowance ledger even without local
macro features. That ledger accepts a crate root without an attribute owner
only when it exists for imports alone. Source clients and staged consumers
still have no ledger, so a gated site inside an imported expansion fails there
by name.

### Qualification (version 2)

All outcomes were declared in `pa29/tests/builtin-macro-rows-outcomes.json`
(plan sha256 `7048944…`, `obj/parallel/builtin-rows-v1/declared/`) before any
rustc or RustGM run of these files. The fixtures are in
`pa29/tests/spec/builtin-macro-rows/`: a no_core source-core provider, seven
source clients, two rejected providers, an ordinary allowance provider and
three clients.

- **Oracle.** Pinned rustc 1.98.0 matched all 14 declared statuses and codes
  (`E0433`, `E0658`) and the native value `7 9 1` (line, column and cfg at the
  consumer's call site); see `obj/parallel/builtin-rows-v1/oracle/round1/oracle.json`
  (sha256 `cf819ead…`).
- **Candidate.** `scripts/check_macro_metadata.py` now runs all 29 cases. It
  handles source-core providers, source clients, object clients linked with a
  frozen C driver, declared diagnostics and the published macro-table version.
  The native run passed 29 of 29 (`candidate/round2/complete.json`, CLI
  `dfbaed10…`). Three rows are requirements where rustc accepts and RustGM
  rejects by name: the `test` attribute row, a macros 2.0 provider and a
  re-exporting provider.
- **Integration tests.** `crate_linking::expander_rows` has three tests:
  - publication and round trip of the version-2 table, plus a version-1 control
    for declarative-only crates;
  - forged and relabelled rows: row kind, syntax, kind, order, an off-root
    `macro_rules!` builtin, helpers on a non-derive, an off-root declarative row,
    a needless version 2, an invalid feature, and an unknown or relabelled
    identity in a fresh consumer;
  - linear growth: the macro table of 8, 32 and 128 builtin rows grows by
    exactly the same bytes per row, and the consumer's LowIR does not change.
- **Receipts.** The six integration tests above passed
  (`obj/qualification/fast/1794056625422756880`). The private
  `macro_context`, `permission_session`, `token_images` and `cfg_gates`
  selections passed 11 tests under strict private Clippy (`1794056054956364057`).
  The text suites PA1–PA19 and PA21–PA24 passed (`1794056277519828131`,
  `1794056348682261921`). PA20 shows only the four known Lane A
  `assoc-bounds/{wide16,deep16,wide64,deep64}` storage-budget failures. Also
  passing: `make test-upstream`, `make test-harness`, `make test-target-units`
  with the same CLI, and the `legacy_editions` and `reexports` controls
  (`1794056577400502666`).
- **Amended controls.** `declarative_exports` forged version 2 as its
  unknown-version control. It now forges version 3, and also checks that v1 rows
  relabelled as v2 fail by name. The runner-count check in `transported_rows`
  rises from 15 to 29.

### Census: core macros that alloc and std invoke

Analysis only. `obj/parallel/builtin-rows-v1/census/census-uses.py` scans the
pinned `library/alloc/src` and `library/std/src` for invocations of the 102 core
export rows (`uses.json`), with comments and string contents blanked. A bang use
is `name!`, path-qualified or not. An attribute use is `#[name]`, and a derive
use is a name in `#[derive(...)]` or `#[derive_const(...)]`. Files named
`tests.rs`/`benches.rs`, or under `tests/` or `benches/`, count as test code.
The counts are lexical: std defines its own `panic`, so its 100 `panic!` sites
do not reach core's row.

| alloc (library code) | Names | Sites | Reaches alloc through | Runs on version 2 today |
|---|---|---|---|---|
| Builtin derives | 9 (`Clone`, `Copy`, `Debug`, `Default`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`) | 84 | prelude re-export | no: re-export, and the expander is not context-free |
| Builtin attribute `derive` | 1 | 29 | prelude re-export | no: re-export |
| Builtin root `macro_rules!` | `assert` 49, `panic` 17, `unreachable` 9, `format_args` 2, `cfg` 4 | 81 | `#[macro_use]` root rows | only `cfg` (4 sites) |
| Ordinary root `macro_rules!` | `debug_assert` 92, `debug_assert_eq` 27, `debug_assert_ne` 2, `write` 6, `assert_unsafe_precondition` 3, `impl_partial_eq*` 12 | 142 | root rows; `impl_partial_eq*` through `use core::bstr::{...}` | no: they expand to `assert!`/`panic!`/`format_args!`, and `impl_partial_eq*` needs a re-export |
| Ordinary `pub macro` | `const_error` | 1 | path | no: macros 2.0 |

Alloc test code adds `assert_eq` (1,406 sites), `assert_ne`, `matches`,
`assert_matches`, `#[test]` (308) and `#[bench]`.

std library code invokes 46 of the names. The largest groups are the builtin
derives (1,023 sites), `derive` (391), `cfg_select!` (88, a builtin
`pub macro`), `asm!` (27), and the ordinary `pub macro` `const_error!` (237).
After them come `write!` (70) and the `debug_assert*` family (100). Its
context-free builtin uses (`cfg`, `concat`, `stringify`, `include_str`, `line`,
`column`, `file`, `env`, `include`, `compile_error`) total 113 sites.

### Proposed next step

Macros 2.0 hygiene is not next for alloc: alloc library code uses only one
ordinary `pub macro` site (`const_error!`). It does matter for std, where
`const_error!` has 237 sites. The alloc path is blocked first by two other gaps:

1. **Macro re-export rows.** Core's prelude re-exports the derives, `derive`,
   `test` and every root builtin (`pub use crate::{assert, cfg, ...}`), and
   `core::bstr` re-exports the `impl_partial_eq` family. A re-export row would
   name its target row by provider placement and name, and the consumer would
   bind it through the same module-identity walk. This unblocks 125 alloc sites
   at the resolution level. It also removes the by-name publication failure
   that core's prelude would hit once core reaches publication.
2. **Executable non-context-free builtins.** `assert`, `panic`, `unreachable`,
   `format_args` and the derives expand to provider paths under internal
   permissions. Their rows would need the provider's allowance list too (a
   builtin-row feature list, which is another version-2 field). The consumer's
   expander would then run them with the imported frame permissions, as
   declarative rows now do. That covers 161 alloc sites directly, and the
   `debug_assert*`/`write` family that expands into them.

A third, smaller gap is the source-client allowance ledger. alloc is a source
client of core, and source clients have no macro context. So even a listed row
(`assert_unsafe_precondition` with `cfg_ub_checks`) would fail there by name.

## Re-export rows

Follow-up (Lane B, PA29). A `pub use` of a macro, including a glob, now
publishes a re-export row instead of failing by name. This covers core's
prelude (`pub use crate::{assert, cfg, ...}`, `pub use super::v1::*`), root
re-exports such as `pub use crate::macros::builtin::derive` and `core::bstr`'s
`impl_partial_eq*` imports. Row kind 2 is part of tag-14 version 2. Tables
without re-exports keep their exact version-1 or version-2 bytes.

**Wire layout.** After the shared kind byte, placement and name, a kind-2 row
holds its owner ordinal (`u32`), the target placement (`u16` count, then each
component) and the target name. Owner 0 is the artifact itself; owner `k`
is entry `k - 1` of the artifact's own sorted dependency list, as in the
public-name table. The target is the key of a definition row (kind 0 or 1) in
that owner's table, never another re-export. Rows of all three kinds share one
order, sorted strictly by (placement, name), so one scope never binds a name
twice. Decoding rejects by name:

- an owner ordinal past the dependency list ("names an unknown owner");
- a local target (owner 0) that is no definition row of the table, including a
  target that is itself a re-export ("names no definition row of its crate");
- unsorted or duplicate keys and unknown row kinds.

A table with only re-export rows also needs version 2. Budgets: re-export rows
count against the 4,096 rows and pay one atom per row plus one per placement
and target component.

**Provider.** `sema::crates::export::declarative_rows` walks every module that
public direct module declarations reach from the root, once, and collects every
public macro binding there, whatever its origin (direct, import or glob). Every
such binding is published or rejected by name; none is dropped. The scalar
public-name table now skips every public macro binding.

- A local definition gets exactly one definition row. It sits at the
  definition's own direct binding when that binding is published. Otherwise
  (the definition's module is private, like `core::fmt::macros::Debug`) the
  row sits at the least (placement, name) key among its public bindings. Every
  other public binding of the definition becomes a re-export row of owner 0
  naming that key.
- A binding of an imported definition becomes a re-export row naming the
  owner's definition row. Chains flatten: a re-export of a re-export names the
  defining row, because the consumer bound the intermediate re-export to the
  definition itself. The owner must be one of the crate's own dependencies
  ("macro re-export target's crate is not a direct dependency").
- Rejected by name: ambiguous public macro bindings; bindings of a non-macro
  definition such as a bootstrap builtin; a public macro binding in a module
  that only a public re-export alias reaches, since that module has no public
  placement (`aliased-provider`); and every definition form the earlier
  sections reject (macros 2.0, macro-generated definitions, edition-panic and
  `allow_internal_unsafe` contracts).

**Consumer.** `resolve::crates::bound_macro_rows` binds in two passes. First
every artifact's definition rows are bound and recorded by
(artifact, definition row). Then every re-export row is bound at its own
placement to the definition its target names. The owner is found by crate
identity among the imported artifacts, the target row by its key in that
owner's table. No binding name, alias or consumer path is consulted, and the
definition order of artifacts does not matter. So every path to one macro
reaches one `DefId`:

- explicit `use` of two paths to one name still collides
  (`duplicate imported macro name`, rustc E0252);
- two globs that bring one identity are not ambiguous, while two globs that
  bring two definitions sharing an expander are (`ambiguous macro name`,
  rustc E0659);
- `#[macro_use] extern crate` now enters every macro bound at the provider's
  root, re-exports included, as rustc's `for_each_child` does. It used to
  enter the definition rows whose scope was the root;
- allowance lists follow the definition, so a re-export of an unlisted row
  keeps its gates (`facade-unchecked-client`).

### Qualification (re-export rows)

All 15 outcomes were declared in `pa29/tests/macro-reexport-outcomes.json`
(plan sha256 `81ee2852…`, `obj/parallel/macro-reexports-v1/declared/`) before
any rustc or RustGM run. The fixtures are in `pa29/tests/spec/macro-reexport-rows/`:
a no_core source-core provider (`exporter.rs`) with a private-module builtin
reached only through re-exports, root and renamed re-exports and a core-shaped
prelude; eight source clients; a source-client bridge and its inventory-only
client; an ordinary facade over `internal-provider` with two binary clients;
and a requirement provider whose macro module is reached only through an alias.

- **Oracle.** Pinned rustc 1.98.0 matched all 15 declared statuses and codes
  (`E0659`, `E0603`, `E0252`, `E0658`), the native line/column/cfg value
  `7 9 1` and the facade client's exit 0 (`oracle/round1`, then `round2`
  after the bridge amendment below; `oracle.json` sha256 `237558ec…`).
- **Candidate.** `scripts/check_macro_metadata.py` now runs all 44 cases and
  passes `--dependency-metadata` for a case's `inventory`. Both runs passed
  44 of 44 with CLI `90a8d3e5…`: non-native
  (`obj/qualification/fast/1794059085991598230`) and native
  (`1794059133535519860`, evidence
  `obj/parallel/pa29-macro-metadata/maintainer-419851-true-1794059162635632131`).
  `exporter-path-client` prints `7 9 1` beside rustc, and `facade-client` exits
  0 beside rustc. The one requirement row (`aliased-provider`) is accepted by
  rustc and rejected by name.
- **Integration tests.** `crate_linking::expander_rows` gains three tests:
  - publication and round trip of the exporter's 7 builtin, 1 declarative and
    16 re-export rows, including the relocated private-module `concat`;
  - forged and relabelled rows: an unknown owner, a missing local target, a
    target that is a re-export, an unknown row kind 3, an unsorted key, and a
    bridge artifact whose re-export names a row its owner lacks, which a fresh
    inventory consumer rejects by name;
  - linear growth: 8, 32 and 128 re-export rows grow the table by equal bytes
    per row, and the consumer's LowIR does not change.
  All six `expander_rows` tests passed (`1794058592100564194`), as did
  `declarative_exports`, `reexports` and the other `transported_rows` tests
  (`1794058776842740197`). Strict private Clippy and the 11 private
  `macro_context`, `permission_session`, `token_images` and `cfg_gates` tests
  passed (`1794059192987416343`).

**Amended controls.**

- `builtin-macro-rows-outcomes.json`'s `reexport-provider` was a requirement
  row rejected by name. It now publishes version 2 (frontend 0); its rustc
  outcome is unchanged.
- `expander_rows`'s unknown-row-kind forgery used kind 2, which is now the
  re-export kind; it forges kind 3.
- `transported_rows`'s runner count rises from 29 to 44.
- Candidate round 1 changed two declarations of the new file, recorded in
  `declared/amendments.txt`:
  - `exporter-private-client` declared the diagnostic `unresolved macro`.
    RustGM exits 1 as declared, but with `private type name \`hidden\``, the
    reason rustc gives (E0603), so only the diagnostic substring changed.
  - `bridge` failed with `unresolved import` at `pub use exporter::forty_two`
    inside a nested module. A probe shows the same failure for a function
    (`pub use exporter::base`): no_core source clients do not reach the extern
    prelude from nested modules. That gap is unrelated to macro rows, so the
    fixture now names the root `extern crate` item
    (`pub use crate::exporter::forty_two`), and the rustc oracle was rerun.

## Stability rows and imported allowance ledgers (version 3)

Follow-up (Lane B, PA29). Tag-14 version 3 carries each definition row's
`#[unstable]` feature and each builtin row's `allow_internal_unstable` list. A
consumer now gates every call of an unstable row. Source clients and staged
consumers now own the allowance ledger that imported lists need. Tables with
neither field keep their exact version-1 or version-2 bytes.

**Wire layout.** Version 3 is the version-2 layout plus two fields:

- a declarative row appends its stability after its allowance list;
- a builtin row appends, after its helpers, its allowance list (a `u32` count
  of feature identifiers) and then its stability.

Stability is a tag byte: 0 when the definition has no `#[unstable]`
attribute, or 1 followed by the feature string. Re-export rows are unchanged,
because rustc checks the stability of the macro definition and not of the
re-export. `Macros::version` writes version 3 exactly when some row is
unstable or some builtin row has a list. Decoding rejects by name an unknown
stability tag, an invalid feature, an unsorted or invalid list, and a version-3
table with neither field. A stable `since` is not transported, since no
consumer check reads it.

**Provider.** `declarative_rows` looks up each published definition's regular
policy in the completed staged inventory, by declaration node. It publishes
only `Unstable` features. A builtin row now publishes its allowance list
through the same `exported_internal_unstable` check that declarative rows use.
Version 2 silently dropped a builtin's list. Builtins with
`allow_internal_unsafe` or edition-panic contracts (core's `format_args`,
`panic`, `assert` and `unreachable`) are now rejected by name, instead of being
published without their contract.

**Consumer.**

- Each imported definition records its `DefinitionRow`, declarative or builtin.
  A builtin frame therefore grants its own list, as a declarative frame does
  (`permission_session`).
- `resolve::expansion::unstable_calls` checks every call of an imported row
  after `bindings::finish`, once the library requests are complete. Either the
  crate requests the feature, or the call's own span lies in an expansion
  allowed to use it (`macro_allowances().allows`). Otherwise the call fails with
  "use of unstable library macro requires feature `F`" (rustc E0658). The check
  reads the definition row behind the call, so it covers every path, `use`,
  re-export and `#[macro_use]` route.
- A row's feature counts as a library feature of its dependency, so a source
  client's `#![feature(F)]` passes request validation.
- Source clients and staged consumers build the imported-only local ledger
  (`LocalMacros`) whenever a dependency row carries a list, as ordinary crates
  already did. Before this change a source client could not apply an imported
  list: `cfg(ub_checks)` inside a listed imported expansion failed. A side
  effect is that such a client has a macro context, which `#[prelude_import]`
  also requires.
- Not checked: a `use` of an unstable row that is never called. rustc's
  import-path check was not probed.

### Imported context-dependent builtins

These builtins are still not runnable when imported. They stay rejected by name
("imported builtin macro requires its provider's source declaration").
The blockers, in order:

1. `assert!(cond)` expands to `$crate::panicking::panic("assertion failed: …")`.
   The consumer's own expander could emit that path, anchoring `$crate` at the
   row's crate id (`Kind::CrateRoot(id)`, as imported declarative rows already
   do). But the callee cannot be imported. Source metadata has no transport for
   `&'static str` or `!` signatures ("str is outside the scalar metadata
   profile"), so a probe provider defining that function fails publication.
   This transport is outside the macro lane (`sema::crates::authored_export`).
2. `assert!(cond, msg)`, `panic!` and `unreachable!` delegate to `panic_2021` or
   `unreachable_2021`. Both are `pub macro` definitions with
   `#[rustc_macro_transparency = "semiopaque"]`, and macros 2.0 rows are still
   rejected. A semiopaque `pub macro` has `macro_rules!` hygiene, so it could
   travel as a declarative row with a syntax flag, without a new hygiene model.
   The core rows also need the edition-panic contract, which is rejected by
   name above.
3. `format_args!`, and `write!` through it, build typed `FormatConstructor`
   nodes, charge the source-core ledger and name `fmt::Arguments`
   constructors. Core's row also carries `allow_internal_unsafe`.
4. `debug_assert*`, `write` and `assert_unsafe_precondition` are declarative
   rows that already travel. They stop at the builtins they call.

### Qualification (version 3)

All nine outcomes in `pa29/tests/macro-stability-outcomes.json` (plan sha256
`b8f21bfc…`, `obj/parallel/imported-builtins-v1/declared/`) were declared
before any rustc or RustGM run of these files. Exploratory probes of similar
shapes (`obj/parallel/imported-builtins-v1/probe`) ran first. The fixtures in
`pa29/tests/spec/macro-stability-rows/` are:

- a staged no_core provider with an unstable declarative row (`answer`), an
  unstable builtin row (`line`), a builtin row with a list (`stringify`),
  listed wrappers, an unlisted wrapper and a staged re-export;
- seven source clients;
- a native object client driven by the `builtin-macro-rows` driver.

Results:

- **Oracle.** Round 1 rejected the declared provider: rustc requires a stability
  attribute on a staged crate's public re-export. The fixture was amended
  (`declared/amendments.txt`) and round 2 reran every case. Pinned rustc 1.98.0
  then matched all 53 macro cases natively, including four E0658 rejections and
  the native `6 7 1` (`oracle.json` sha256 `4475f61e…`).
- **Candidate.** `scripts/check_macro_metadata.py` runs all 53 cases. With CLI
  `d7af1f75…`, all 53 passed both non-native
  (`obj/qualification/fast/1794061449063891192`) and native
  (`1794061547550973425`, evidence
  `obj/parallel/pa29-macro-metadata/maintainer-1435107-true-1794061575021107648`).
  The native client prints `6 7 1`, as rustc does. RustGM's four rejections
  name the feature and frame, for example "use of unstable library macro
  requires feature `gated_answer`" in macro `leaky`.
- **Integration tests.** `crate_linking::expander_rows` gains three tests:
  - publication and round trip of the stability fields, the builtin list,
    gating through a re-export, and admission by a requested feature; the
    version-2 provider keeps version 2;
  - forged rows: an unknown stability tag, an invalid feature, an invalid
    builtin list, a version-3 table with neither field and a version-2
    relabel all fail. An unlisted wrapper and an unstable builtin are
    rejected, while listed wrappers and the `cfg(ub_checks)` row are accepted;
  - linear growth: 8, 32 and 128 unstable rows grow the table by equal bytes
    per row, and the consumer's LowIR does not change.
  All nine `expander_rows` tests passed (`1794061301030997286`).
- **Private tests.** Strict private Clippy and 12 private tests passed
  (`1794061632402438057`): `macro_context`, `permission_session`, `cfg_gates`,
  the source-client selection tests and an encoded-consumer test.

**Amended controls.**

- `declarative_exports` used version 3 as its unknown-version forgery. It now
  uses 4, and also checks that version-one rows relabelled as version 3 fail
  with "unknown exported macro row kind".
- `transported_rows`'s runner count rises from 44 to 53.
- Behavior: a builtin with `allow_internal_unsafe` or an edition-panic contract
  is now rejected at publication, where it used to publish without its contract.
