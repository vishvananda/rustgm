# Genuine trait aliases for original core

LIB-031 is the actual loading frontier after the defaultness checkpoint. Observer
40239001850 verifies722 original inputs, opens65 and stops at `ptr/metadata.rs:85`,
byte3311, the equals sign in `pub trait Thin = Pointee<Metadata = ()> + PointeeSized;`.
Source SHA is `90be519b27b46064f0bec607170ffce37aa9afb160a1edfab9c51641c8af15e2`.
This plan does not claim a trait-alias implementation or a core artifact.

Independent run40239493812 freezes16 authored sources before execution, then
observes32 metadata compilations and six actual native executions on pinned
rustc1.98.0 in editions2021/2024. Its44 commands take0.75 seconds, with zero
infrastructure failures. Actual native programs exercise associated equality,
lifetime/type arguments and nested aliases containing two ordinary traits; all
print `7\n`. Sixteen source fixtures and the complete command/diagnostic contract
are in `pa31/tests/spec/trait-aliases/`, outside the active manifest.

Plan SHA is `d38fe6af6ab66c04552b53f40c0dec3234ed7e5413cc8e20b891eb095c5acbff`.
Artifact12616985245 has ZIP SHA
`bd43c957290e0db79a70f91b0e3fadd962c132eaad1a710df34834fc3e966664`.
The original predictions remain frozen. Eight command observations disagree:
the pre-equals where-clause is rejected in both editions, a public alias over
private traits emits warnings in metadata/native builds, and disabled ungated
syntax emits `unstable_syntax_pre_expansion`. These are language observations,
not RustGM failures or reasons to remove diagnostic checks.

The unstable book describes aliases in bounds and objects. Pinned execution
specifies this milestone more precisely: multiple ordinary traits work in a
bound; after expansion, a trait object still allows only its ordinary principal
trait plus auto traits. Empty aliases and post-bounds where-clauses are accepted.
Cycles, using an alias as an impl owner, incorrect associated equality and
forwarded caller syntax lacking its own feature authority are rejected.
Primary background: [unstable book](https://doc.rust-lang.org/unstable-book/language-features/trait-alias.html),
[RFC1733](https://rust-lang.github.io/rfcs/1733-trait-alias.html). Pinned outcomes
above, rather than a broader proposed RFC profile, define these test requirements.

The current parser has a single real trait branch in `syntax/expression/items.rs`.
It reads generics/bounds/where before requiring an opening brace. This is the
observed failure. The existing explicit implementation-restriction alias fence
must remain. Introduce an actual `ItemKind::TraitAlias { bounds }` variant; retain
the existing Item generics, visibility, name and post-bounds where-clause. Accept
`=` only immediately after the name/generics, allow empty bounds and require `;`.
Reject inappropriate unsafe/auto/const/default/restriction forms explicitly.
Reuse `bounds()` and `where_clause()`. Walk, transfer, render and configured
expansion must preserve the real nodes and source context without reparsing.

Add a distinct resolved alias definition kind in the type namespace, with a real
lexical generic/Self scope. It is neither an ordinary trait owner nor a type
alias with a fabricated RHS. Do not create an implementation registry entry,
trait application, associated-member owner or vtable principal for the alias.
Ordinary imports/privacy retain the actual definition identity. Qualified member
lookup and bounds must expand to actual declaring traits; impl-owner lookup must
continue to reject an alias. Use the existing feature producer and token-context
allowance machinery for `trait_alias`; do not add a lexer spelling permission.
Share the direct unstable-syntax diagnostic projector with existing soft gates.

Semantic preparation should compile a real alias contract under its own rigid
Self, declared type/const/lifetime parameters and where-clause binders. Reuse the
declaration worklist, typed clauses and canonical substitutions. At each bound
use substitute the complete subject and arguments into those clauses, including
associated equalities and outlives obligations. Preserve quantifier depth and
caller/declaration environments. Never prove an invented `Self: Alias` trait or
treat it as an additional caller assumption. Ordinary bound proof, normalization,
object safety and region checks then consume the expanded actual contracts.
Unused aliases need full name/arity/WF/cycle checks as well as demanded calls.

Keep a cold, explicit preparation lifecycle with owner/arena/image identity,
pending dependency, ready typed contract and failed outcomes. Cycles need an
actual visiting-state diagnostic; a missing contract is not an empty alias.
Settle the exact private owner fields and worklist entry types against the
existing `declarations/generics/bounds.rs` before implementation. There is no
approval here for a second general trait registry or an unkeyed global cache.

Avoid expanding a diamond of nested aliases into exponentially repeated source
work. Parse each alias once and preserve shared typed dependencies. A demanded
expansion may reuse only identical canonical alias applications with complete
subject, arguments, quantifier context and relevant environment. Distinct
applications and obligations cannot collapse by name or rendered text. Charge
unique dependency edges, actual substitutions, emitted clauses, warm lookups,
failed attempts and transient coexistence before allocation. Diagnose bounded
depth/work/retention exhaustion while retaining cumulative payment. Ordinary
programs without aliases must allocate no alias owner. Measure any containing
owner/layout change and widths/depths8/32/128 plus a shared diamond and an
ordinary control; do not claim a wall-time improvement from oracle timings.

Implement grammar/identity, local typed expansion and genuine imported transport
as separate, explicitly bounded checkpoints. A syntax checkpoint must keep a
clear semantic unsupported error until the actual alias consumer is complete.
Export must reject unsupported alias identity/contracts rather than silently
publish an ordinary trait or omit a public alias. The existing public Provided,
defaultness, restricted-domain and legacy wire guards remain independent.

PA5 owns the grammar vocabulary and early parser regressions. PA21 owns shared
relationship/substitution algorithms; its existing stable contract does not gain
an unstable language requirement. PA31 owns the active unstable/source-provider
contract and complete reductions. PA29 must carry real alias contracts before
cross-crate aliases can count toward core consumers. Generate only independently
reviewed active text references after the corresponding implementation passes;
these planned fixtures do not activate PA31 or close LIB-031.
