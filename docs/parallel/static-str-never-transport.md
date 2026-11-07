# `&'static str`, `!` and static text in provider bodies

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/static-str-never-20261107`. The branch is based on `e84a85783`
(main), with the provider-guards checkpoint rebased under it because main has
not joined that yet. It is a transport slice, not a library milestone, and
makes no claim about an original-core artifact or a whole assignment.

The aim was to run the shapes of core's `panicking::panic(&'static str) -> !`
and `panicking::assert_failed` from a fresh consumer. Lane B's alloc census
counts 120 and 22 library sites behind these two. Their bodies, attributes
and formatting are not covered (see the gaps below).

Version 13 now carries four things:

- the `str` and `!` types in signatures and bodies;
- static text literals;
- plain reborrow inputs;
- SSA snapshots in branching bodies, when each use stays in its block.

## Types (version 13)

Type recipe tag 10 is `str` and tag 11 is `!`, with no operands.

- **Export.** `recipe_emission.rs` writes them under `body_nodes()` only.
  Older stages keep "source scalar type is outside the supported profile"
  (`str`) and "source type form requires an explicitly supported transport
  row" (`!`).
- **Codec.** `recipe_columns.rs` reads tags 10 and 11 only under version 13.
- **Validation.** Both are formed leaves in the port plan and in the type
  domains.
- **Consumer.** `demanded_types.rs` maps them to the consumer's own
  `str` and never types.

## Static text (version 13)

| Form | Tag | Operands |
| --- | --- | --- |
| text literal | value node 7 | `[Flag(ordinal)]` |
| text column | after the guard columns | one UTF-8 string per literal |

The text column is present exactly when a decoded value is a text literal, so
originals without literals keep their exact bytes. Out of line, the
`Original` record holds it as `texts: Option<Box<Texts>>`.

- **Export.** `value_rows.rs` writes a literal only when its type is
  `&'static str`, and charges and reserves its bytes. Older stages reject it.
- **Codec.** The column is read only under version 13. Each string is bounded
  and charged per byte, and invalid UTF-8 is rejected ("source text literal
  is not UTF-8").
- **Validation.** A literal's value type must be a shared reference with the
  `'static` region to `str`. Ordinals are dense in value order, and the
  column has exactly one string per literal.
- **Consumer.** `encoded_texts.rs` copies the strings out of the sealed view
  while it is live, paying one step per literal and one per byte. It rebuilds
  each literal with a placeholder ID that no arena issues, and interns the
  bytes into the consumer's own text arena once the view is released. Cache
  validation compares each cached literal's bytes with its row.

## Plain reborrows (version 13)

Passing `&x` to a `&T` parameter, or a literal into a merge, records two
coercion inputs: a reborrow, with a reference conversion, and the same-shape
relation that the `Constrain` value requests. Every reborrow was rejected
before ("source adjusted/unsizing coercion requires a complete wider input
profile").

Version 13 admits a plain reborrow: at least one dereference, with no
unsizing and no two-phase borrow at either level.

- **Codec.** `relation_columns.rs` reads one under version 13. Every other
  adjusted form keeps "source adjusted coercion requires a wider source
  profile".
- **Port plan.** `recorded_inputs.rs` roots both types in the original's
  scope. No operation pays for the reborrow. The consumer derives the same
  bounds from its own borrowed places (`borrow/regions/places.rs` follows the
  dereference ancestry), and the `Constrain` relation is still requested.
- **Borrow port.** `value_requirements.rs` lets a borrow whose outermost
  projection is a dereference use an inference region without
  borrow-expression provenance, which is what an implicit reborrow is given.
  It still requires that region's variable input.
- **Consumer.** `reconstruction_requests.rs` demands every coercion type
  under version 13, because a literal flowing into a merge leaves the
  relation's region variables only in its input rows.

## Block-local snapshots (version 13)

The snapshot walk (`ssa_ports.rs`) followed one successor chain and rejected
every two-way exit, which `expect`'s branching body needs. Under version 13 a
body with a two-way exit is walked block by block instead. Every dependency
of a definition, and every place a store, check, mention, view binding or
selection uses, must be evaluated earlier in the same block, which then
dominates it. Each block forgets its definitions at its end, and the walk is
charged per statement and per definition. The graph and single-ownership
checks before the walk are unchanged. Older profiles keep the chain walk.

## Late-region trait methods now reach the consumer

The provider-guards checkpoint's `abcfea354` probes
(`obj/parallel/provider-guards/older-forge/`) stopped at the reborrow gate
under version 13. With plain reborrows admitted they go further. They were
rerun with the candidate, and the results are in
`obj/parallel/static-str-never/older-forge/`.

| Probe | Version 13 export | Fresh consumer |
| --- | --- | --- |
| `honest`, `variant` (trait `pick<'a, 'b>(&self, ..) -> &'a u8` or `&'b u8`) | accepted | "source named lifetime requires its actual provided original" |
| `t1` (static associated `first<'a, 'b>`) | accepted | the same |
| `t2`, `t2b` (`get<'a, 'b>(&'a self, ..)` through `Pick::get`) | "source operation omitted a required nontrivial formation input" | not reached |

Version 12 is unchanged: each probe still stops at "source original region
formals require their complete wider body profile" or at the reborrow gate.
Every consumer still rejects a late-region trait method, so the result-lifetime
hole stays unreachable. The trait-impl task must re-prove associated methods'
declared results when it opens that consumer path. The `t2` export failure is
an honest provider rejected by its own validation, which also belongs to that
task.

## Forgeries

`forged_text_and_reborrow_rows_reject_in_fresh_consumers` uses a generated
minimal provider: the shared foundation plus `name` and `pick`.

| Mutation | Fresh consumer |
| --- | --- |
| `pick`'s second literal relabelled to ordinal 0 | "source text ordinal leaves its value order" |
| first byte of `second` set to 0xFF | "source text literal is not UTF-8" |
| the `str` row relabelled `!` | "source text value is not a static str reference" |
| each plain reborrow made two-phase | "source adjusted coercion requires a wider source profile" |
| each plain reborrow given zero dereferences | "source adjusted/unsizing coercion requires a complete wider input profile" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |
| `second` rewritten to `sxcond` | accepted (sound) |

The rewritten literal is another valid program, and the client still reads
`s`. The block-local rule is covered by the private invariant
`branching_snapshots_are_block_local_only_in_version_thirteen`.

## Evidence

The fixtures are in `pa29/tests/spec/static-str-never/`, and their outcomes,
declared before the oracle and the candidate ran, are in
`pa29/tests/static-str-never-outcomes.json`. The integration tests are in
`crate_linking::str_never_transport`.

- **Oracle.** rustc 1.98.0 compiles the provider, the eight accepted clients
  exit 42 and the three opponents fail with E0308, E0061 and E0599. Every
  outcome matched its declaration on the first oracle and candidate runs.
- **Native.** The eight accepted clients exit 42 natively, as rustc's do:
  check, untaken, arm, name, literal, spin, expect and pick.
- **Opponents.** All three are rejected, as rustc rejects them:
  - name-type: E0308 / "incompatible types";
  - panic-arity: E0061 / "call argument count mismatch";
  - str-method: E0599 / "receiver has no inherent nominal method".

  The last two candidate texts were not predicted. They were recorded after
  the first candidate run, which rejected both as declared.
- **Older profiles.** Under versions 12 and 10, `str` and `!` on the
  source-nominals provider are rejected by name, and the full provider stops
  at the shared inherent-methods gate.
- **Scaling.** Packets are 34,550, 55,340 and 138,500 bytes for 6, 24 and 96
  functions, each with two literals, a branch and a `!` call. That is 1,155
  bytes per function, exactly linear. The client LowIR is 2,223 bytes at
  every width.
- **Byte identity.** The baseline `89c7ed47d` (sha `32bc59f0`) and the
  candidate `d1fa76423` (sha `eaf69733`) were compared over the v10, v12 and
  v13 fixtures, including provider-guards and this packet's older-profile
  exports. Across 585 metadata, LowIR, status and diagnostic files, none
  differ. The commit `7826962b5` has the candidate's code; it adds only the
  recorded observations to the outcome file.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `str_never_transport` integration (4) | 4/4 | `1794075689063913107` |
| private selection (16), including the hot-record invariant | 16/16 | `1794076290108885768` |
| neighbouring v13 suites (guard, inherent, constructor, match, node, branch, fn roles, statics) | 29/29 | `1794076583209215148` |
| native `--ignored` (this packet, guard, match, constructor, inherent) | 5/5 | `1794076679213405129` |
| PA1-PA12 | pass | `1794076749056217900` |
| PA13-PA19, PA21-PA24 | pass | `1794076814684505313` |
| PA20 | 1,451/1,451 | `1794076882663673433` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/static-str-never/test-upstream.log` |

The rustc 1.98.0 receipt is `obj/parallel/static-str-never/rustc/receipt.txt`,
and the identity receipt is `obj/parallel/static-str-never/identity/receipt.txt`.
The pinned binaries are `obj/parallel/static-str-never/{baseline,candidate}/`.

## Amended controls

None. No earlier outcome changed. The two private invariants that call the
consumer's operation validation (`literal_pools`, `effect_artifacts`) now
pass it the consumer's text arena.

## Edits outside body transport, for owner review

None outside crate transport. Within it, the consumer-side demand change in
`sema/crates/typed_headers/reconstruction_requests.rs` and the borrow-port
change in `port_plan/value_requirements.rs` widen shared paths under
version 13 only.

## Remaining gaps toward core's `panicking::panic` and `assert_failed`

- Attributes: `#[track_caller]` ("source caller tracking policy transport is
  not implemented"), `#[cold]` ("source cold advisory marker transport is not
  implemented") and `#[lang = "panic"]` on a function ("source trait role
  requires completed declaration validation"). `#[inline]`,
  `#[inline(never)]` and `const fn` already cross.
- The bodies: core's `panic` calls `panic_fmt(fmt::Arguments::new_const(..))`
  and `assert_failed` takes `fmt::Debug` arguments and an
  `Option<fmt::Arguments>`. Both need the fmt roles and constructors.
- A provider's `extern "C"` declaration (`abort`) is still rejected ("source
  exact callable requires actual version-five exported function/original").
- Reborrow inputs that unsize or are two-phase stay rejected by name.

## Remaining gaps toward `core::option::Option::map`

- const callable bounds (`[const] FnOnce + [const] Destruct`);
- lang and diagnostic items, stability attributes, derives and `must_use`;
- closures in provider bodies;
- callable bounds written by the consumer;
- trait impls with methods on provider nominals, including the late-region
  methods above.

`&'static str` and `!` in transported signatures and bodies are no longer a
gap.
