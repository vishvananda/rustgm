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
2. Admit scoped syntax with `decl_macro` and the `rustc_macro_transparency`
   attribute with `rustc_attrs`, from actual enabled root language requests. Derive compact macro gate bits during the existing single
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

## Context-bearing binding development checkpoint

The focused-qualified successor to `3df65784` adds a lazy
forward/reverse context index into the existing Entry arena, without fields in
Entry or Local. Root slots count only root entries; their lookup also handles a
table populated solely by contextual entries. Removal/redeclaration retains the
same global entry ordinal and first-binding position. Import dependency keys,
candidate sets, provided sets, journals, selection records and withdrawal events
now retain ItemContext. Cold withdrawal ordering includes that context explicitly.
Existing root-only public binding iteration remains explicitly root-only; the
metadata seam `has_contextual_items` exposes retained contextual keys so the
metadata producer can reject the whole provider before provenance transport is
qualified. Wiring that rejection into the producer remains an integration task.

Review exposed an existing journal boundary issue: two unrecorded entries can
predate Work, both become ordinary bindings in one batch, then notify Work after
the batch. Seeding by creation ordinal duplicates the second declaration and
changes event order. The draft records each indexed scope's first-binding tail
once at Work creation, using four bytes per existing indexed scope, and seeds
only through that retained tail. It keeps lazy per-scope journals and adds no
per-entry publication field or per-declaration scope scan. The regression covers
the actual post-batch notification shape, not only one declaration at a time.
Standalone resolution with no imports returns before constructing Work, avoiding
the new snapshot allocation entirely. Expansion still retains its worklist when
later macro output may introduce imports; its existing bindings need a snapshot.

Ten new private controls cover root-only allocation, independent root/context
widths, empty root slots, alias identity, exact-context duplicates, removal and
re-declaration order, local/item separation, journal boundaries and exact-context
waiter wakeups, plus same-origin glob withdrawal with a direct binding masking
only one context. A scope-count by later-suffix-width control bounds journal
visits to at most five for four retained keys, including the current one-entry
peek, independently of the later suffix. Passing layout assertions establish
BindingTable120, Entry44, Local32 and Resolution2048 on this x86-64 profile.
The prior qualified source and receipts remain separate. Scoped source declaration/alias/path contexts,
module adjustment, visibility, mixed opaque context ranges, language gates and
cross-crate transport remain unfinished. Glob subscriptions are still scope-wide;
this draft makes no claim of eliminating a contexts-by-globs work product or
accepting any new scoped macro program.

Evidence `obj/parallel/pa31-declarative-macros/item-bindings-v2` records fresh
formatting, architecture/file audits, all-target strict Clippy, harness/upstream
verification, 135 frontend tests and 26 exact private controls. The rebuilt CLI
preserves PA6 218/218, PA7 181/181 and PA8 94/94 text outputs. Checks SHA-256 is
`7bd785428af51a3f1c612b252b210823ca4e4a83c95f10ca5fb1597d1a9f3106`;
the qualified source inventory SHA-256 is
`88062dbdee0d68d5a8f56d941afa4631f3133654540cda52897f625930fd7bb4`.
The first attempt's semicolon lint and subsequent preparation failure remain
recorded; v2 factors the shared alias key and reruns the audit freshly. This is
a focused development checkpoint, with combined release qualification and
end-to-end performance still required before publication.

## Configured root language requests

The next local checkpoint derives compact `decl_macro` and `rustc_attrs` request
bits during the existing root feature parse. Both ordinary and source-provider
loading retain them independently of library stability requests. Enabled root
`cfg_attr` requests apply; disabled requests and nested or misplaced attributes
grant no macro permission. Direct resolution and expansion check `decl_macro`
before the retained unsupported scoped-hygiene boundary. Disabled declarations
remain inert. Recognizing the request does not enable scoped expansion or the
transparency attribute's semantic consumer.

Five additional private controls pass with the existing 135 frontend and
26 private controls. Measured Package256, SourceCore72 and Features2 layouts
remain unchanged; MacroFeatures occupies one byte in available package padding.
Ordinary requests leave provider and library-policy boxes absent. Fresh format,
architecture/file audit, all-target/all-feature strict Clippy, harness/upstream
verification and PA6 218/218, PA7 181/181, PA8 94/94 all pass. Evidence is
`obj/parallel/pa31-declarative-macros/request-gates-v2`; its checks SHA-256 is
`a737986610f123ac88b0a65c82fa9db78466ada45af92133c751aec8a0226ecc` and source inventory SHA-256 is
`ade67210b646926bd083ed1648b370771f8228d1f3273f89b23cece6bea4a755`. The first attempt's filename-ownership audit failure is retained;
no compiler build or test ran in that attempt. The successor renamed only the
private test file and module path, then ran the full focused gate freshly.

Scoped item production, per-segment lookup, definition-site/module adjustment,
privacy and contextual metadata transport remain required. This checkpoint
is local development evidence, not release or whole-library qualification.

## Source identifiers and later member consumers

The next unqualified source draft derives declaration keys from each actual
identifier span: item and extern-alias names, generic parameters and enum
variants. Expansion publication and import notifications use that same complete
key. A generated item's keyword cannot substitute for its captured name. The
unused root-only table declaration convenience is removed; declarations and
private controls pass their full key. Three structural controls cover separate
root/opaque identifiers, genuine exact-key duplicates, and const parameters
versus implicit Self. They supply token provenance to parsed declarations and
do not qualify end-to-end scoped expansion. Formatting has run; compiler and
test qualification of this draft remains pending the combined worker gate.

The source-only draft also shares each retained import spelling sequence through
an immutable Arc slice. The worklist still owns the same declaration and retries
the same dependencies, but cloning its import record no longer allocates or
copies every segment on every retry. This representation change does not claim contextual path
lookup. Its compilation, import regression cases and performance remain unrun.

An explicit import alias now retains its actual identifier's item-normalized
context when collected. Publication, duplicate checking and withdrawal consume
that context with the destination spelling; they do not derive it again on each
retry. A fourth parsed-provenance control distinguishes an opaque alias from a
root declaration with the same spelling.

The typed use-tree now retains the actual glob star span, including through
fragment transfer. Its text view still renders the same boolean glob flag.
Two new transfer controls cover a star distinct from its prefix/alias tokens
and atomic rollback when mapping that star's provenance fails. This supplies
the token identity needed by later glob adjustment; synchronized mark stripping,
privacy adjustment and scoped lookup remain unimplemented. These source changes
and tests are formatted but have not run a compiler or test gate.

The subsequent unqualified collector draft shares the typed path visitor between
ordinary names and contextual segments. It flattens grouped imports iteratively
with one append/truncate buffer, publishes each leaf's spelling slice once, and
allocates a shared context record only when a segment or the actual glob star is
non-root. Explicit aliases remain independent. Unaliased destination publication
uses the retained final segment, after trimming `self` when required. Two further
parsed-provenance controls cover mixed nested groups, independent alias/star
contexts, root siblings after opaque siblings, and a non-root star on a root path.
These are structural controls: source lookup and glob adjustment still need the
retained contexts. The draft has not passed a compiler/test gate or layout/perf
measurement. Each ordinary group classifies only its newly appended spellings
and inherits its parent's bootstrap relevance; source-provider mode skips that
classification. Relevant bootstrap paths still perform full-prefix semantic
scans. Measure that remaining depth cost separately before claiming a general
linear bound for collection.

The first collector gate at source `8d3f4b0d` passes formatting and the
architecture/file audit, then stops at strict compilation: expansion's import
caller still used the former prefix/absolute arguments. No candidate tests ran.
Its logs remain in `obj/parallel/macro-identifiers-v1` in the integrator validation
checkout. The successor updates that caller to the same new entrypoint already
used by ordinary collection; fresh strict and focused qualification remain
required. The initial attempt in the macro checkout was refused by storage
admission before any runner started, and is preserved separately.

The second collector gate at `2f0efe37` again passes formatting and the audit,
then strict Clippy identifies the unused private-test root declaration wrapper.
All callers already use complete keys. The successor removes that wrapper
without adding a lint exception; no candidate tests ran in either failed gate.
The second failure and scoped cleanup receipt remain under
`obj/parallel/macro-identifiers-v2` and `obj/cleanup/1792817262704647706.jsonl`.
Fresh strict and focused qualification is still required.

Collector gate v3 at `67356ab6` passes formatting, audit, strict Clippy,
harness/upstream verification and all 137 selected frontend cases. Its private
lane reports 32/37: five new identifier provenance controls fail because their
fixture inserted origins on physical bytes, which correctly retain ROOT.
The passing duplicate-name control likewise exercised a root duplicate rather
than establishing its intended opaque-key case. CLI/text stages did not run.
The failure is retained in `obj/parallel/macro-identifiers-v3`; scoped cleanup
`1792818096946014054` removed no artifacts. After diagnosis, cache cleanup
`1792818123262540177` retired the rebuildable validation Cargo cache while
preserving current comparison pins and receipts.

The successor fixture now parses a generated fragment once and installs it at
an actual macro target through `Unit::install_expansion`. Every emitted token
has a retained origin, and each test changes the selected installed token's
context rather than appending overlapping origins. The helper verifies that
the selected span is generated, its context lookup matches, and physical bytes
remain ROOT. Compiler context lookup and test expectations are unchanged.
These remain supplied-provenance structural controls, not end-to-end scoped
expansion qualification. Their corrected execution is pending.

The separate ancestry successor is source-only and unqualified. It replaces
two linear parent-chain queries with one shared append-only index, consumed by
both textual lookup and resolved-binding ambiguity checks. Actual declarative
and builtin invocation producers, including eager argument reservation, populate
it; token syntax contexts do not.
Each record holds a parent, depth and one skew-binary jump. The intended cost is
amortized constant work per publication and logarithmic work per ancestry query,
with no retained query cache. The cold record grows from a single 4-byte parent
to three 4-byte fields; the Vec header and ordinary allocation-free root remain
unchanged. Layout assertions and correctness/work tests are drafted but unrun.

Four private controls compare chains, stars, balanced and irregular trees to an
independent linear parent oracle, reject malformed publication atomically, and
cross depth 8..8192 with query width 8/32/128 plus unrelated sibling publication.
One source integration control inspects actual builtin/declarative invocation
parents, including nested eager arguments. A peer source review caught the third
producer in `expansion/eager.rs` before any build of this draft; the successor
wires it to the same index and adds that explicit source control. These tests
do not qualify scoped lookup or claim an end-to-end speedup;
strict/focused/text and changed-path frozen-binary measurements remain required.

The pinned compiler intentionally treats implicit `Self` as root-normalized
(`rustc_resolve/src/ident.rs:316` and `late.rs:3503`). Type, const and lifetime
generic parameters instead use item normalization (`late.rs:3158`). This
distinction must survive declaration, lexical lookup and barrier checking.

Completing scoped names also requires the semantic consumers after resolution:

- Aggregate fields are indexed by their normalized declaration identity.
  Record initializers, patterns and field access adjust the use against the
  actual variant/container before matching; see `rustc_hir_typeck/src/expr.rs`
  around 1883 and 2722, and `pat.rs` around 2079. Existing `Field.origin`
  retains the source NodeKey needed to derive this identity.
- Method candidate lists may retain a coarse spelling index, but selection
  filters by container-adjusted hygienic equality before receiver equations and
  privacy. Existing call NodeKeys and `MethodCall`/`GenericMethodCall` spans
  retain the necessary use identity; no new Body, Value or Type field is needed.
- Associated type/const indexes and impl-to-trait member matching must use the
  same identity. A spelling-only single-winner map can discard distinct opaque
  members before selection. Preserve DefId/FieldId downstream; use cold
  contextual indexes only when non-root names exist.
- Legacy textual macro bindings also carry item-normalized names. The pinned
  reduced-graph macro publication uses `IdentKey::new` for both textual names
  and exported bindings (`build_reduced_graph.rs` around 1323). Keeping only
  NameId there would defeat the qualified mixed legacy/opaque opponents.

Share an adjustment result containing the normalized item context and privacy
definition scope. Cache only pure context/target-expansion adjustment, or include
the complete identity and relevant readiness in a semantic cache. Pending
headers cannot become permanent missing answers. Actual container introduction
is independent of its captured name context. This applies to named fields,
methods, associated suffixes, visibility paths, imports and aliases together.

Whole-provider metadata rejection must include contextual semantic field/member
indexes as well as `BindingTable::has_contextual_items()`: a root-captured type
name can contain opaque fields. Source descriptors remain disabled until their
full validation, remapping and fresh imported proof contract is complete.
