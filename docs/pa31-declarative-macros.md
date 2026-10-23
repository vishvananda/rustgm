# Scoped declarative macros: implementation boundary

This is a source/design checkpoint, not implemented macro2 support. The original
pinned whole-core root gets past `option_env` import resolution and reaches
`core/src/macros/mod.rs:170`, where `pub macro assert_matches` fails parsing.
The current public compiler checkpoint does not include the changes below.

The isolated branch now drafts typed concise/braced syntax and shared matcher
compilation, with parser/transfer/separator tests. It has only been formatted,
not audited, compiled or executed. Expansion and direct resolution still reject
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
code has not yet been audited, compiled or executed. It still uses the existing
context identity and must migrate to normalized LocalContext with the scoped
hygiene implementation. Whole-frontend allocation and timing remain unmeasured.
