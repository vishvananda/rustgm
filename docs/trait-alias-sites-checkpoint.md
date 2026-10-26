# Trait alias identity and source sites

This checkpoint implements the grammar and resolver identity of trait aliases,
with source feature checks and disabled-syntax diagnostics. Typed alias contract
expansion, associated lookup through expanded traits, object use and imported
transport remain required. An active alias stops explicitly at semantic Self
preparation; it cannot publish analyzed facts or become an ordinary trait owner.
LIB-031 and the original `core` artifact remain open. No new whole-core loading
observation was run for this partial checkpoint.

Aliases have distinct `DefKind::TraitAlias` identities in the type namespace and
real lexical generic/Self scopes. Neither the alias nor its Self binding exposes
that scope as associated members. Associated paths may remain deferred until
semantics; a written `Alias::U` never resolves to the alias's lexical parameter U.
Bounds and where clauses retain their typed parser nodes. Const aliases retain
their qualifier and independently require `const_trait_impl`, matching the
previous pinned observations rather than the original rejection hypothesis.

`Syntax::gate_sites` generalizes the existing sparse default-modifier column.
Aliases retain whole-item spans, including visibility and the semicolon;
default modifiers retain their exact keyword spans. They share the original
source-image witness, monotonic retention debt, original-prefix replay,
transactional syntax transfer, root replacement and sealed token-context
producer. `defaultness(item)` and `has_defaultness()` filter out alias sites.
Ordinary syntax adds no field or owner, and alias-only syntax cannot allocate
specialization marker state through the defaultness predicate.

Feature requests and actual macro allowances remain distinct. Literal allowance
and forwarded-caller opponents exercise the actual expansion graph. Inactive
ungated aliases retain the direct `unstable_syntax_pre_expansion` warning even
under allow/deny/forbid attributes. Changed source incarnations and altered
whole-item spans fail authentication. Wider mixed/opaque authority cases and the
complete diagnostic contract remain part of typed alias qualification.

[CI40251809477](https://github.com/vishvananda/rustgm/actions/runs/40251809477)
passes six exact private groups, strict production and private Clippy, formatting,
the global architecture audit, metadata, harness/upstream checks and all230 PA5
text cases without reference changes. The helper takes200.90 seconds, including
both lints, unoptimized library/CLI builds and text execution; actual selected
tests take0.064 seconds and PA5 text comparison takes0.515 seconds. All2,656 source
fingerprints match before/after. Hosted formatting precedes that snapshot, and
the integrator checks every final source hash before importing its one formatted
test file. No local compiler build, formatter or binary hydration was used.

Widths8/32/128 retain24-byte rows and a56-byte owner, with stable capacities and
two-replay work144/496/1856. The existing defaultness transfer, failed-budget and
source-incarnation controls and the real opaque Assert-transfer control pass.
Existing containing layouts remain Syntax312, Configured392, Unit688, Extent104,
Report3040 and Engine4088 bytes on the pinned x86-64 target; ItemKind28 and Node36
also remain unchanged. These are storage/work observations, not a wall-time
performance claim.

Input source is `b5252c56b17394338880d348106666694ce646ab`; the qualifying CLI SHA
is `78dd3728af7280af783a2738d5bd5ae63ed44a2ccc229f6a8e11533a0ef4638e`.
Binding SHA is `8cb9574c05f9a15cbb681571deac9f30ffdf5c16a70759db3692b88265b1513b`;
receipt SHA is `bb2d3b744476cdcff5ffc4c53bb97639f7d5c5264a6822621138474821ef91f7`.
Earlier failures remain retained: a missed renderer field access, a semantic
fence placed after associated inventory initialization, and a test that expected
semantic rejection during deferred name resolution. Both lint modes ran before
tests; no test build followed the first static failure. The source fence and
test-phase correction were independently checked against their actual producers.

Next, prepare alias Self/parameters without creating an associated owner or
synthetic Self:Alias predicate, and extend the declaration bound worklist from
one trait result to real multiple clauses. Keep nested alias contracts as a
shared typed dependency graph; expanding each declaration into a flattened
ancestor vector would repeat diamond work exponentially. Preserve quantified
clauses, equalities, outlives obligations, original/current arguments and actual
declaring-trait identities through substitution. The
[implementation plan](trait-alias-design.md) keeps the complete local and
transport acceptance requirements separate from this checkpoint.
