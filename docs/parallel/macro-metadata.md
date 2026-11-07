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
- the canonical image: the spelling of the single delimited body. Pieces
  (tokens, open and close delimiters) are separated by exactly one space. An
  empty group is spelled `()`, `[]` or `{}`;
- typed preorder rows. A token row is `(class, byte length)`, with 15 closed
  classes; punctuation identity is its spelling. A group row is
  `(0x80 | delimiter, descendant rows)`.

`support::crates::exported_macros` checks, without a lexer, that the rows lay
out the image exactly. This covers the one root group, every separator, every
delimiter byte, group containment and token boundaries. Unknown edition, flag,
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

## Consumer (design; not implemented)

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
  into the image, which needs a file-like image source owned by the package so
  that `contexts::at` and `editions::token` see the provider edition. A full
  parsed `Unit` does not fit: the image is a token body, not a Rust file.
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
  and `select`, but not `private_only`. The image of `call_f` is
  `{ () => { $ crate :: f () } ; }`, with its typed rows. Decode, re-encode and
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

1. **Consumer import.** Image ownership, authentication, compilation, root
   definitions, the `use`, path and `#[macro_use]` forms, and `$crate` ids.
2. **Native qualification.** A `check_macro_metadata.py` runner modelled on
   `check_public_names.py`, adding per-case editions, for the frozen
   `macro-metadata-outcomes.json`.
3. **cfg-if.** Rerun the original crate through metadata into the PA34
   `cfg-if-consumer.rs` consumers (2024 and 2015), and the target-unit executor's
   cfg_if unit.
4. **Spacing.** The canonical spacing loses punctuation jointness. Stringifying
   definition tokens (`stringify!` of definition-site punctuation) may then differ
   from rustc. A joint bit per punctuation row would close this if a frozen case
   needs it.
