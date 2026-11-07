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
