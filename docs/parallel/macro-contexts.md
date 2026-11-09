# Macro-context provenance slice (B2 M)

This is Lane D's first B2 item, on `implement/asm-stream-stage-b` at
origin/main `f32b8d168`. It is a sub-item of LIB-059 ("Authenticated source
stability and provenance in library metadata") and covers only the provenance
the v13 export gate checks. Exported-macro provenance (LIB-007's subtask) and
general semantic-name provenance stay unassigned. No metadata row changes.

Before this slice, `require_export_mode` (authored_export.rs) rejected any
crate with more than one syntax context:

```rust
if report.resolution.bindings.has_contextual_items()
    || report.package.marks.context_count() != 1
```

So one private `macro_rules!` expansion anywhere stopped publication. The real
`_mm_stream_ps` module stopped there because `vps!("movntps", ",{a}")` and the
eager `concat!` inside it mark the template's tokens.

## Which provenance the gate needs

A mark matters to metadata only through a name that a row records or a consumer
re-resolves. `expand::marks` keeps two projections per context: the item
projection resolves item, type and generic-parameter names, and the local
projection resolves locals, parameters and labels (`resolve::hygiene::Query`).

- **Item names.** A semi-opaque mark (`macro_rules!`, eager builtins such as
  `concat!`) keeps its parent's item projection (`Data::intern`), so item
  resolution is the same as in physical source. Only an opaque mark (macro 2.0
  and the formatting builtins, eager.rs:376–383) creates a non-root item
  projection.
- **Locals.** Local, parameter and label definitions are keyed by the local
  projection of their name token (`resolve::index::define`). Body-local
  declaration rows carry a spelling (`Declaration.name`) without that context.

So the slice needs exactly these facts: every item projection is the root, and
no local, parameter or label is bound by a marked name token. If both hold, no
row depends on a mark and nothing new travels.

## Gate rule (version 13)

`has_contextual_items` stays. Under version 13 (`BodyTransport::Nodes`) the
crate-wide `context_count() != 1` test becomes two rejections, both with the
existing text "source macro/semantic-name provenance metadata is not
implemented":

- (a) `Marks::has_item_contexts`: some context has a non-root item projection;
- (b) `BindingTable::has_contextual_locals`: some hygienic local's context is
  not the root.

Older profiles keep the crate-wide test and their outcomes.

(c) Exported macro definitions. Lane B's envelope (tag 14) is unchanged. A
definition with introduction 0 lies in physical source, and
`unit::contexts::at` gives every physical offset the root context, so its
tokens carry no mark. Every other exported definition is already rejected by
name at `declarative_rows.rs` ("macro-generated exported macros are outside
the macro metadata profile"). That rejection is now located at the generated
definition, and both publication paths project it into the generating macro's
body: the source-core path where `lower::instances` reads `macro_table`, and
the ordinary library path in `export::with_envelopes`. Every other error the
macro table raises is at file 0: 0 and keeps that location. The text is
unchanged; on both paths the location moved from file 0: 0 to the generating
tokens, as declared in the outcomes JSON.

## Exported payloads hold no re-resolved identifiers

Lane B asked whether any exported payload holds tokens or identifiers that a
consumer re-resolves. Only the macro envelope does, and (c) covers it.

- **Bodies.** Every transported body, generic, inline, default or constant,
  whatever its owner kind (`owner_catalog::BodyKind`), is an `Original` row.
  Every operand the reader accepts (`ir_nodes.rs`
  `operand_read`, kinds 0–14) is a table index, flag, range, bit pattern or
  place: values, blocks, locals, projections, types, declarations,
  applications and regions. A field projection is `(owner, ordinal)`
  (`ProjectionKind::Field`), and a call or method target is a declaration or
  application row, never a name.
- **Texts.** The v13 texts column holds literal values (`Texts`, "each
  literal's UTF-8 contents, in value order"), not identifiers.
- **Locals.** A body-local declaration is identified by `(body, scope,
  ordinal)` (`LocalDeclaration`). Its name is checked only as a bounded
  spelling (`source_validation/lexical_forest.rs`). (b) keeps marked locals
  out anyway, because the spelling would drop their context.
- **Item names.** The consumer resolves names only when it installs public
  declarations (`installed_bindings.rs` compares each row's name with the
  installed binding's actual name). Those names come from root item keys:
  `has_contextual_items` and (a) reject the rest.

## Accepted flips

Crates that expand macros only internally and export plain items now get past
this gate. Before the candidate run I surveyed every file quoting the gate text:

| Input | Live check | Before | After |
| --- | --- | --- | --- |
| asm-native stream witness | `asm_lowering` | provenance | const parameters (declared) |
| pa41 float-reliability-cfg `native-target-values-v2.rs` | recorded only | provenance | 2024: source Copy language item is missing; 2021: source export requires defining Rust2024 edition |
| pa31 metavariable-expressions `execute-counts-owned.rs` | recorded only | provenance | 2024: conditional member requires its actual From declaration; 2021: source export requires defining Rust2024 edition |
| pa29 `spec/macro-contexts/provider.rs` (new) | `hygienic_exports` | provenance | exports |

No other test, outcome JSON or text reference quotes the gate. A crate that
passed the old gate passes the new one unchanged, since (a) and (b) each imply
more than one context. The two recorded inputs now stop at other fences and
publish no packet a consumer could read. The stream witness stops at its const
parameter. The new provider is the one flip that publishes, and four fresh
consumers show its resolution: its bodies run, and the client-side expansion
of `answer!` resolves `$crate` to the provider.

Correction, 2026-11-09, after the integrator's review: the first survey ran
pa31's `execute-counts.rs`, which also stops at "source Copy language item is
missing" now. The metavariable README's library consumer is
`execute-counts-owned.rs` (the source hash in `native-predictions.json`), and
the table now gives that file's stops, observed in both recorded editions with
the pinned baseline and candidate CLIs. Each record keeps its original
observation and gains a dated amendment naming the candidate's SHA-256:
`native-execution-v2.json` and the float README, the metavariable README and
its checkpoint. `native-execution-v1.json` records the same float boundary
and stays as written, because v2's `predecessor_contract_sha256` pins it. No
manifest pins those files' hashes.

## Fixtures and outcomes

`pa29/tests/spec/macro-contexts/` and `pa29/tests/macro-context-outcomes.json`,
declared at 2026-11-09T00:57:03Z before any oracle or candidate run. Two
amendments are recorded in the JSON, each before the run it governs: the
`[13, 0]` check reads the wrapped core inside Lane B's envelope, and the
source-core publication projects macro-table errors through expansion
provenance (`lower::instances`), as its other phase errors are. Without that
projection the generated-macro rejection named an offset in expanded text.
Lane B's review found the same gap on the ordinary library path. The JSON's
`generated_exported_macro_ordinary_path` entry, declared at
2026-11-09T01:32:17Z before any run of it, gives that control's exact
diagnostic: `file 0: 39`, at `macro_rules! made` inside `make!`'s body, with
the trace `in macro make at file 0: 67`.
`declarative_exports::generated_documented_and_permissioned_macro_exports_fail_by_name`
asserts it.

## Evidence

- **Independent Rust 1.98.0** (`obj/parallel/macro-contexts/rustc/receipt.txt`,
  SHA-256 `393e3db6…05b7f8`): the provider compiles; forty-two, plus-forty,
  relay and answer exit 42; mismatch fails with E0308; all seven generated
  controls compile.
- **Baseline** CLI built at origin/main `f32b8d168`
  (`obj/parallel/macro-contexts/baseline/rustgm`, SHA-256
  `deb2a3a2…e3d38c33`): the provider, the stream witness and both recorded
  inputs stop at the provenance text (`flips/baseline.txt`).
- **Candidate** (`obj/parallel/macro-contexts/candidate/rustgm`, SHA-256
  `e6d20708…fa53be6`): the provider exports, the stream witness stops at
  "source const parameters are outside the initial source profile" and the
  recorded inputs at "source Copy language item is missing"
  (`byte-identity/candidate.txt`).
- **Byte identity.** Baseline and candidate publish identical v13 packets for
  `spec/transparent-records/provider.rs` (`27fabdbc…`),
  `spec/panic-attributes/provider.rs` (`4afda652…`) and the unused `$crate`
  macro control (`0a1f6fa3…`). Their LowIR outputs are empty under both CLIs:
  a source-core library carries its bodies in the packet.
- **Fastcheck** 7/7 (`obj/qualification/fast/1794186995648562998`, receipt
  SHA-256 `8258899e…e1d2d4`): both `hygienic_exports` controls,
  Lane B's frozen macro-metadata outcomes and generated-export rejections,
  the two mark projection tests and the asm stream witness.
  - The provider packet wraps the `[13, 0]` core in Lane B's envelope with
    one row, `answer`; four clients exit 42 natively; mismatch reports
    `incompatible types`; v12 and v10 keep the provenance text.
  - The five fenced controls report the provenance text. The generated macro
    reports "macro-generated exported macros are outside the macro metadata
    profile" at `file 0: 5999`, the `macro_rules! made` tokens inside
    `make!`'s body, with the trace `in macro make`. The unused `$crate` macro
    control exports.

- **Lane B's frozen macro outcomes, native**: the ignored
  `transported_rows::frozen_macro_metadata_clients_match_rustc_natively` passes
  (`obj/qualification/fast/1794187253664706613`, receipt SHA-256
  `653a871c…0a2129`). Envelope tag 14 bytes are unchanged; this slice touches
  no codec.
- **Review fixups**, rerunning only what they invalidate: fastcheck 6/6
  (`obj/qualification/fast/1794188056110287901`, receipt SHA-256
  `c65e8cc8…ffd1000`) covers both `hygienic_exports` controls, the three
  `declarative_exports` tests with the exact declared ordinary-path diagnostic,
  and Lane B's frozen macro-metadata outcomes. The corrected survey of the
  recorded float and metavariable publications is in
  `obj/parallel/macro-contexts/fixup/{baseline,candidate}`.

## Remaining

- Rule (a) is crate-wide. Any opaque mark anywhere in the crate keeps the
  fence, even a private `format_args!` whose names no row records. Core
  expands `format_args!` through `panic!` and `assert!` throughout, so (a)
  will be core's library-export stop once core export is reached. Lifting it
  needs per-row context provenance, or a proof that no exported row depends
  on an opaque context.
- Marked locals keep the fence under (b); transporting them needs real context
  provenance in rows.
- Exported-macro provenance for generated definitions (LIB-007's subtask) and
  general semantic-name provenance stay unassigned.
