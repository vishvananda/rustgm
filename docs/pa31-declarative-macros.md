# Scoped declarative macros: implementation boundary

This is a source/design checkpoint, not implemented macro2 support. The original
pinned whole-core root gets past `option_env` import resolution and reaches
`core/src/macros/mod.rs:170`, where `pub macro assert_matches` fails parsing.
The current public compiler checkpoint does not include the changes below.

The isolated branch implements typed concise/braced syntax and shared matcher
compilation, with parser/transfer/separator tests. Development snapshot
`b270dd1266354f5b71f2f7239c1c1456d21c0e66` passes formatting, architecture/file
audits, all-target strict Clippy, harness/upstream verification, 135 focused
frontend tests and seven private invariant tests. Expansion and direct resolution still reject
scoped macro semantics explicitly while normalized hygiene is being implemented;
this is not a passing semantic checkpoint or a parser-frontier success claim.

## Evidence and corrected hypothesis

The predeclared 19 inputs have 60 independent rustc observations, including
11 successful native answers. One hypothesis was false: definition-site lookup
finds a private helper but does not authorize exposing its private function type
across that module boundary. Both original rejection observations remain in
`pa31/tests/spec/declarative-macros/observations.json`; do not rewrite the original
contract as though it predicted them. The separately declared public-helper
variant passes metadata, object, linking and native output `3`. Its complete
binding is recorded in `followup-observations.json` beside the original contract.

The pinned implementation is Rust revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`.
`compiler/rustc_span/src/hygiene.rs` caches both opaque-only and
opaque-plus-semiopaque context normalizations. Its `apply_mark` additionally
preserves a macro2 caller context when invoking legacy macros.
`compiler/rustc_resolve/src/ident.rs` distinguishes item/type normalization from
local-value normalization; `lib.rs` wraps normalized item contexts in a separate
type. These are algorithm guidance, not code or acceptance oracles to copy.

## One coherent implementation

1. Retain a typed item flavor and token forests for concise `macro name(args)
   {body}` and comma-separated `macro name {rules}`. Reuse the existing compiled
   matcher, fragment grammar, repetition checking and transcription. Capture each
   group once; do not synthesize a string containing a legacy macro or relex it.
   Preserve legacy semicolon grammar and exact existing text outputs.
2. Admit `decl_macro` and `rustc_macro_transparency` from actual enabled root
   language requests. Derive compact macro gate bits during the existing single
   feature-request parse; retain them in Package's available padding if verified
   by layout controls. No second attribute scan per definition, library-feature
   permission, or new ordinary heap allocation. Keep provider/client role claims
   separate from these syntax gates. Disabled declarations grant nothing.
3. Publish scoped macros in their actual item namespace with ordinary access,
   restricted visibility, forward lookup and the existing import worklist.
   A scoped macro does not enter legacy textual scope or become a root export.
   Retain existing macro_rules textual/export behavior and invocation wakeups.
4. Separate expansion identity from token syntax context. The current ContextId
   doubles as an invocation index and a token mark; opaque and mixed transparency
   need distinct typed identities. Retain invocation identity for introduction
   ancestry, diagnostics and fixed-point ordering. Intern token contexts from
   parent context, actual expansion and transparency; cache opaque-only and
   opaque-plus-semiopaque normalized views at insertion. Preserve captured token
   contexts, original source locations and defining edition independently.
5. Keep normalized opaque item identities through definition, import and use.
   Existing binding keys omit this information and therefore cannot distinguish
   the two hidden `helper` items in the independently qualified repeated-expansion
   fixture. A definition-site fallback alone is insufficient. Prefer a cold
   context-keyed table for non-root item bindings, while ordinary entries retain
   their present representation and lookup. The import dependency key must carry
   the same normalized context, so unrelated expansions do not wake or collide
   with every same-spelling binding. Do not rename source spellings, conflate
   semantic identity with display names, or scan prior expansions per lookup.
6. Preserve real visibility checks after hygienic selection. Normalize locals
   with opaque plus semiopaque marks; normalize items/types with opaque marks.
   Transparent literals use caller lookup, semiopaque literals retain current
   legacy local behavior, opaque literals use definition-site lookup. Captures
   remain caller-owned. Qualified/imported paths need each retained segment's
   actual token context, not one context guessed from an entire mixed path.

Before broad implementation, add independently declared mixed legacy/opaque
nesting and captured path-segment controls. Original rustc hygiene tests including
`tests/ui/hygiene/legacy_interaction.rs`, `items.rs`, `lexical.rs` and
`rustc-macro-transparency.rs` identify further obligations; importing fixtures
must preserve complete original bytes, attribution and source hashes.

The five mixed hypotheses now have 18 independent observations and four native
answers, retained in `mixed-observations.json`. The original textual-only
`legacy-inside-opaque` hypothesis fails macro lookup in rustc; it must not become
an accepted RustGM contract. A separate `exported-legacy-inside-opaque` input adds
an actual exported item binding and declares the expected answer before probing.
The original source, expectation and rejection remain unchanged. The captured
generated macro name, captured module segment, captured import/alias and
opaque-inside-legacy cases independently pass.

The exported legacy followup also independently passes metadata, object, linking
and native answer `7`; `mixed-export-observations.json` binds those four results.
This isolates mixed-policy rebasing from textual-macro visibility, without
changing the original negative observation.

## Qualification and bounded work

Run the authored source/native corpus with the corrected private-helper contract
explicitly distinguished from its original hypothesis. Include no-feature,
private-sibling, malformed separator, captured local, transparent local and
repeated introduced-item opponents. Existing PA7/PA8 and edition suites must stay
unchanged. Add operation/storage controls for wide repeated expansions, nested
mixed transparency and independent same-spelling bindings; context normalization
must be O(1) after insertion and dependency wakeups must follow exact keys.

Measure ordinary macro_rules controls and opaque width/depth growth with the
usual immutable A/B, A/A and output-equality protocol. Preserve hot AST/type/body
record layouts; measure any per-import key growth explicitly. No fake origins,
source rescanning or speculative name resolution. After the coherent source
contract passes, rerun the original whole-core root with the complete pinned
opened-source/configuration inventory. Passing one parser frontier does not
qualify all of core or an assignment release.

## Resolver review refinements

The read-only second review identifies the following required distinctions:

- Use separate `ExpansionId`, `SyntaxContextId`, normalized `ItemContext` and
  normalized `LocalContext` types. Only the mark arena constructs normalized
  wrappers; cache both projections. A proposed context node contains parent,
  expansion, opaque projection, local projection and transparency (logical
  20-byte estimate, not a measured Rust layout). Preserve the actual call-site
  syntax context independently from expansion ancestry.
- Value const-generic parameters use item/parameter normalization, rather than
  local-variable normalization. Keep parameter kind and lexical barriers when
  choosing the lookup policy; the value namespace alone is insufficient.
- Adjust a path segment against the target module's expansion, retaining its
  definition-site/privacy context. Preserve distinct segment and alias-destination
  contexts. Import candidate/provided sets, glob/reexport journals and declaration
  notifications need the same normalized key as waiting maps and direct lookup.
- Apply literal marks to each definition token's retained raw context. A generated
  macro's captured name may be root-owned while its literal body retains an outer
  opaque mark. The present one `Environment.context` for all definition pieces
  cannot represent this. Captured pieces preserve their contexts without literal
  marking. Keep source/edition locations as a separate provenance axis.
- Cache mixed-policy prefix rebasing separately from the final mark. Prefix keys
  may share raw token context, normalized call-site context and policy; final
  interning must include the actual expansion identity. Two repeated introductions
  must never collapse. Count first-miss depth by distinct call-site contexts;
  cached O(1) projection alone does not prove total linear construction.
- Current local histories walk unrelated contexts both for lookup and duplicate
  checks. Add a cold `(entry, normalized local context)` index to ordered LocalIds:
  bucket presence checks duplicates and binary predecessor finds the latest
  `DefId < visible`. Preserve older same-context shadows and lexical barriers.
  Validate monotonic insertion rather than silently assuming it. Measure width
  by contexts by shadows by historical visibility, including transparent context
  collapse, absent contexts and unrelated declarations. No Entry/Local/Token
  record growth is needed for this index; any aggregate pointer/allocation cost
  still needs measurement.

Local scoped expansion does not by itself provide cross-crate macro export,
imported definition-token provenance, or macro stability allowances. Keep these
requirements explicit in the source-library metadata/bootstrap roadmap; no local
fixture or parser frontier can stand in for the original alloc/std crate graph.

## Isolated local-history draft

The source draft adds a lazy context index only after an entry gains its second
local. A single-local entry retains its existing direct lookup and allocates no
index. Indexed histories keep the newest LocalId inline and allocate a vector
only for older shadows in that same entry/context. Duplicate checks and ordinary
newest lookup use the context hash; historical visibility uses a binary
predecessor over strictly increasing actual declaration identities. Publication
rejects an out-of-order definition before changing local records or histories.
Hash lookup retains the compiler's ordinary expected-time hash-table model; it
is not a collision-resistant worst-case bound.

Entry and Local fields are unchanged. BindingTable and its containing Resolution
add one optional boxed index pointer (expected eight bytes on the qualified
x86-64 profile); the three existing Resolution layout contracts account for it.
Private draft controls cover single-local no-allocation, 128 unrelated contexts
by 32 shadows with bounded predecessor comparisons, historical strict visibility,
duplicate/out-of-order atomicity, and separate scope/namespace histories. This
code passes the focused development checks above. The three existing Resolution
layout controls pass at 2040 bytes; the prior record was 2032 bytes. The later token-context checkpoint below migrates it to normalized LocalContext.
Scoped item/import keys remain a separate required consumer. Whole-frontend allocation and timing remain
unmeasured. This development gate does not replace release/native/performance
qualification of the coherent scoped-macro implementation.

Development evidence is retained under
`obj/parallel/pa31-declarative-macros/development-v5`; its checks receipt SHA-256 is
`e542edba148ec2a4af8186966bf313a4ae11c71d5e0a14b0662301426d5b284e`.
The earlier formatting, audit, naming and Clippy failures remain alongside it.

Sixteen independently declared legacy-history scaling sources vary width
8/32/128/512 across distinct introduced contexts, historical predecessors,
ordinary unrelated locals and newest shadows. Their metadata and object checks
pass, but the initial C links lack the upstream overflow-panic runtime. Those
failures remain under `obj/parallel/local-history-scaling-v1`. A separately bound
pinned-Rust native consumer supplies that runtime with every original source and
overflow-check setting unchanged: all 64 metadata/object/link/native observations
pass, including 16 native answers. The v2 results SHA-256 is
`e393996137a27f937d35c3871a2aff0792a491848e26a9d35f51bc009a4d9f58`.
These inputs are independently qualified, not measured RustGM performance results.

## Normalized mark arena draft

`expand::marks` now implements distinct expansion and token-context identities,
constructor-only item/local normalized wrappers, and cached projections. At snapshot `5919c938` it was
not connected to Package, token production or lookup. That isolated module passed
fresh formatting, architecture/file audits, all-target strict Clippy,
harness/upstream verification and five focused private controls. The later
checkpoint below connects the live legacy path; neither checkpoint grants
scoped-macro acceptance.

The arena keeps the physical root allocation-free and interns the
actual `(definition-token parent, invocation, transparency)` key. Projection
construction recurses only into already-normalized parents, giving constant
depth. Mixed-policy rebasing shares each definition prefix for a normalized
call-site context; its cache excludes the final actual invocation mark so
independent expansions stay distinct. First misses cost definition depth per
distinct call-site context, rather than per output token. Passing controls cover
the 128-mark first miss, cache hits across 128 actual invocations, a distinct
call-site context, captured-name/literal-parent differences, transparency
projections and invalid inputs. Measured layouts are Marks8, Node20, Key12 and
normalized context wrappers4 bytes. Private work counters record 128 rebasing
steps for the first 128-mark prefix, no additional steps for 128 distinct actual
invocations at that call-site context, and another 128 steps for a different
call-site context. This is an arena construction bound, not a whole-frontend
throughput or semantic compatibility claim. Evidence is retained in
`obj/parallel/pa31-declarative-macros/marks-development-v1`.

The next checkpoint connects these identities to actual token origins, definition
sites and local histories. Item/import dependency keys and scoped lookup remain
required before scoped expansion can be enabled. Cross-crate macro transport
remains a separate required consumer of these identities.


## Live token-context checkpoint

The isolated implementation now connects the arena to Package, actual expansion
production and legacy local lookup. Each definition token receives a mark from
its retained raw context; captured lexical tokens retain their own context.
Declaration-name context is no longer substituted for every body token.
Invocation ancestry remains a distinct ExpansionId for diagnostics, textual
scope ordering and physical introduction. Ordinary macro_rules uses semiopaque
marks, and builtin production selects the actual terminal callee segment.

Origin keeps a four-byte optional nonzero syntax context beside a four-byte
introduction identity. Measured sizes remain Origin32 and Token12; Package adds
one eight-byte cold mark arena handle and is 256 bytes, with no allocation for
physical-only sources. Normalized local histories retain Resolution2040 and
avoid scans through unrelated contexts. Their definition-site fallback follows
the token's own normalized parent. Captured opaque slices still follow bounded
physical-origin ancestry to resolve their component contexts. This is a retained
performance obligation: materialize or share mixed context ranges without
splitting the physical/edition origins or assigning one guessed context to an
entire mixed fragment. No end-to-end speed or allocation improvement is claimed.

Evidence `obj/parallel/pa31-declarative-macros/token-contexts-v4` records fresh
formatting, all-target strict Clippy, harness/upstream checks, 135 frontend tests,
16 exact private controls and unchanged PA7 181/181 plus PA8 94/94 text outputs.
The private controls include mixed captured contexts, reversed/cyclic provenance,
normalization/rebasing work and actual layout checks. The prior fresh architecture
and file audit is retained through an authenticated one-semicolon correction;
its exact inverse restores every audited byte. Earlier factoring/type/lint
failures remain in v1–v3. Checks SHA-256 is
`1d547c3fd2105744fd707ec795b20ad2cc8b900559db2ab29e260c8f260be39e`,
and the qualified source inventory SHA-256 is
`6a7fea4becfc0a578ee7c5035d0f7d72070cd7ebd27b32e5f680b5deb2f5f3e5`.

Scoped declarations still reject explicitly. Root language gates, normalized item
and import keys, module/path adjustment, visibility and cross-crate provenance
remain required. This development checkpoint is not a release qualification or
an original whole-core/alloc/std build.
