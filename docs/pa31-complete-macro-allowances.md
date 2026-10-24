# Complete macro allowance lists and borrowed query sessions

This supersedes the three-name declaration restriction proposed in
pa31-internal-macro-allowances.md. It does not overwrite those hypotheses.
Branch pa31-allowance-sessions preserves the 21 inputs
at contract SHA 1cc6626e306b367d468e3db987cb71d069c5052d6372270837565a6c2f648f31.
The old three-bit prototype is retained separately in stash 54c8fa03; it is not
applied here. The isolated spelling repair bd8053d3/be1c769b remains preserved on
its own branch; its combined qualification is integrator-owned.

## Names are data, consumers own meaning

Pinned rustc_attr_parsing/src/attributes/allow_unstable.rs parses arbitrary
identifier symbols and combines repeated attributes. A complete list, including
unknown names, duplicate entries, repeated attributes and empty lists, is inert
macro declaration data. It does not add a root request, enable a language
capability, prove a trait bound, grant const execution or implement a foreign
library API. Existing malformed-list, authorization and owner checks remain;
nonmacro target semantics retain their separate observed boundary.

Keep known consumer bits for EditionPanic and the initially inspected named
features, but retain every actual name independently. Generic membership accepts
a feature name from the actual typed consumer: the edition checker supplies its
fixed name; a future completed API checker supplies its validated Policy.feature.
A new allowance name never requires a compiler capability or whitelist entry.
The membership result is only a provenance fact. Foreign API checking and
stability/macro metadata remain explicitly unsupported.

## Exact lexemes and canonical feature symbols

Forest::name strips r# but retains original Unicode code points. Membership must
use NFC, matching the existing support::names::Names interner and the pinned
Rust symbol rule. The source header builder normalizes once using the same
unicode-normalization dependency; it does not refactor the hot name interner.
Preflight normalization work/scratch/output before producing the canonical
buffer, then check actual retained capacity before publication.

Retain the original complete token lexeme separately for witness comparison,
including raw prefix and original Unicode sequence. A temporary builder string
pool can share buffers between canonical names and identical source lexemes;
ASCII nonraw names use one shared Rc. Only raw or noncanonical forms need a
different lexeme buffer. Drop the builder index after HeaderData publication;
its attempted work/storage remains charged. Feature membership compares NFC
symbols, while witness authentication compares exact source bytes. Canonically
equivalent source replacement must not silently validate an old source witness.

Queries accept FeatureName<'a>, a borrowed canonical symbol, rather than an
unprepared source spelling. Known consumer names have static constructors;
Report::feature_name(NameId) borrows the already NFC name arena; the currently
ASCII-validated library Policy exposes its existing feature symbol read-only.
A future metadata producer must validate canonical symbols before publication.
No token query reparses or normalizes a name, and preparing a borrowed query
symbol allocates nothing. A feature symbol is data, not an execution permit.

HashSet iteration never defines external ordering. Future stability/macro
metadata export must sort canonical names deterministically while preserving
actual declaration/attribute provenance, before authenticated serialization.
That transport remains unsupported here.

## Owned cold header data

Proposed representation, inside existing SourceCore Contracts ownership:

- Entry is Plain(Declaration) or Owned(Rc<HeaderData>); its summary accessor
  returns the existing Copy declaration summary. There is no duplicate summary.
- HeaderData owns that complete declaration summary, the existing Witness vector
  and a complete HashSet<Rc<str>> of canonical identifier names, plus precomputed
  validation work/payload counts.
- Spelling keeps static attribute/builtin identities; Feature(Rc<str>) retains
  the exact parsed feature identity in the existing witness row. A feature buffer
  is allocated once per distinct name per macro; witness occurrences clone its
  Rc. No full source image is cloned or hashed per query.
- Each relevant attribute keeps its original configured effect index, full
  Attribute, capture/tree identities and typed lexemes. Empty lists still have
  an attribute witness. Combining membership does not discard source witnesses.

Unannotated macros retain no HeaderData allocation. Header-only builtin macros
still retain their actual claim witness. Parsing consumes configured typed
forests once; unknown identifiers are accepted as names. Feature hashing and
name allocation are charged before work. Capture reads and witness comparisons
are also charged. HashSet growth, Rc/header allocation, witness row capacities,
feature string payload and declaration-map capacity are preflighted together
before publishing a usable Entry. Rejected attempts retain work charges and
sticky budget failure. Payload accounting is distinct from a whole-heap claim.

The declaration summary records has_allowance separately from known membership.
Both Engine::define and definition authentication must compute edition_panic from
actual EditionPanic membership, never from has_allowance. Empty/other-feature
lists therefore do not skip edition frames.

Replace the one-attribute summary permit with a checked header:

  checked(package: &Package, owner: NodeKey) -> Result<CheckedHeader<'_>, Error>
  CheckedHeader::summary() -> Declaration
  CheckedHeader::allowance_at(effect_index, Attribute) -> Result<Option<Allowance>, Error>

CheckedHeader owns an Rc clone and borrows the exact Package; it retains no
RefCell borrow. It is created only after complete source/tree/lexeme validation.
Witnesses are already ordered by configured effect index; allowance_at uses that
index and exact full Attribute/kind to produce the existing private per-attribute
permit. It does not scan attributes or reparse a list. The claims traversal
threads its actual effect index (an indexed retained-attribute iterator can
preserve the existing iterator API). The stability collector receives a permit
for each exact repeated attribute, never a general owner/name exemption.

## Expansion ownership versus frozen report ownership

A frozen Report borrow can authenticate a header once; a per-token wrapper that
revalidates its full list every time still costs list-length times uses. Avoid
that false O(1) claim explicitly.

For the mutable expander, propose a small optional Rc<HeaderData> in the
existing cold Definition record. It is constructed together with that actual
Definition/body from the completed header producer. SourceCore Entry and the
Definition share this same immutable allocation; ordinary non-source macros and
unannotated source macros need no allocation. No Package, Report, Body, Code,
Value, Type, invocation/frame or token record gains a field. The cold Definition
pointer cost must be measured, not assumed to fit padding.

This owned snapshot supplies the expander's authority during its private phase:
Unit::install only replaces active Kind::Macro targets, appends source/configured
arenas, and cannot rewrite an already-published Kind::Item macro definition.
Loader/Engine do not expose their Package for caller mutation during expansion.
Frame checks compare actual definition/node/module/role/edition and the snapshot
identity retained by its SourceCore entry before consuming known bits. They
charge every reached use, but need not rescan an immutable published source
header on each panic callback. No later source is used to retarget a snapshot.

Report fields become publicly editable after expansion, so a report session
must reauthenticate that actual snapshot against current Source/typed witnesses
on first use. The producer invariant is not an unconditional shortcut on a
possibly altered Report. The known equal-width spelling and foreign-Unit
opponents must continue to reject at this boundary.

## Borrowed report session

The public read-only provenance interface is:

  Report::macro_allowances(&self) -> MacroAllowanceSession<'_>
  MacroAllowanceSession::allows(&mut self, site: Location, feature: &FeatureName<'_>)
      -> Result<bool, Error>

The session holds &'report Report and one optional boxed cold memo owner. Its
constructor allocates nothing. No direct convenience query creates a new session
for every token. A semantic traversal retains the session over its read-only
phase. Location is the genuine typed consumer's use span, not an enclosing call,
diagnostic origin, container name or guessed terminal identifier span.

Each query charges before work and resolves the actual raw token context through
the existing retained-origin helper, counting traversed links. It inspects only
Marks::outer: ROOT is false, and there is no ancestor union or normalization to
item/local hygiene. On first reached frame, shared FrameView authenticates the
existing invocation/index/call-site/ancestry/definition/module/scope records.
The session memoizes verified frames and authenticates each reached immutable
source header once, retaining its checked Rc. Authenticated empty/metadata-only
headers are retained too. A memo value of None records a validated Plain header;
absence of a memo entry remains distinct and cannot satisfy a warm frame.
Those are the actual existing
indexes, not a second solver or a speculative all-Code/all-token scan.

Holding &Report prevents source, definitions, invocations and resolution from
changing while the memo is usable. No RefCell borrow crosses a query or callback.
An Rc header is copied only once into the session for a reached header. Every
later hit still charges the SourceCore owner and verifies that its current Entry
retains this exact Rc; foreign same-numbered rows or substituted cached data
cannot reuse another owner. Global or session budget failure denies warmed hits.
Memo reserves, actual capacities and retained row payload are checked before
publishing entries; failure cannot expose partially authenticated rows.

Ordinary packages without source bindings return false with no allocation/work.
Physical tokens retain an empty memo. A reached unannotated macro frame retains
its authenticated Plain negative row, without allocating HeaderData.
Definition-literal tokens carry the actual applied invocation mark; captured
expr/path/tt tokens keep their own contexts, including pre-granted tokens from
an outer definition. No transcription, registry payload identity or captured
token provenance changes are proposed.

## Work and opponents

Measure feature-list length 8/32/128 against distinct and repeated use counts
8/32/128, with one and several genuine source owners/expansion frames. Include
same site repeated, many literal sites in one expansion, nested forwarding and
ordinary/no-allowance controls. Record original attribute-token count as well as
unique feature count: deduplication cannot erase validation work for repeated
source entries. Charge feature-name hashing/comparison bytes, source-origin
links, first header authentication, first frame checks and memo growth. Expected
session cost includes each reached header's full source-token work once plus
per-frame validation and each actual name query; a known membership bit alone
is not evidence for total O(1) behavior.

Private controls must distinguish unannotated inner literals from allowed outer
captures; ordinary callers from macro literals; coincident NodeKey/ExpansionId
across reports; substituted Rc rows; wrong source/full Attribute/tree/lexeme and
frame/definition/owner; exact effect index for repeated attributes; and cold/warm
work/storage exhaustion. Existing accepted-profile spelling controls stay intact.
Future API consumers must check their actual caller/callee contract separately.

The 21 original hypotheses remain unobserved. Before interpreting them, use a
coordinated independent-only guard and retain every disagreement. Additive
contracts will declare arbitrary-name, empty, duplicate and combined-attribute
admission; old valid-but-unsupported results remain historical evidence. The
mixed captured-terminal hypothesis remains exploratory until observed. No C
heavy run is authorized by this document.

## Source checkpoint and qualification boundary

The initial implementation was frozen source-only and unqualified. Its subsequent
qualified development record is in [the handoff](pa31-internal-allowances-handoff.md).
It wires complete lists into
the existing declaration/stability collector, preserves per-attribute owner
permits, and consumes actual EditionPanic membership in definition publication
and frame authentication. The Report session exposes provenance facts only;
foreign unstable API checking and macro/stability metadata remain unfinished.
Accepting a declaration or returning true from allows() does not close LIB014.

The initial twelve new private controls cover literal/captured/pre-granted contexts,
Unicode and raw spelling, same-numbered foreign reports and substituted memo
rows, source/frame/definition changes, missing warm headers, exhausted budgets,
positive and negative width/use matrices, metadata-only headers, Plain headers,
and per-use retained-origin traversal. Layout output will measure Session,
Memo, Definition and SourceCore; arithmetic predictions are not measurements.
SourceCore's old accepted-profile assertion updates use the retained measured
176-byte baseline. The existing Package and hot records gain no fields.

The held independent-only plan authenticates all original 21, additive seven
and Unicode three inputs before running 106 declared observations, including
22 native programs. Their actual independent results and later candidate checks are recorded in the
handoff, including the retained six harness-link failures. The candidate plan
requires one debug
private binary, execute its entire nonignored inventory plus exact affected
names, then four frontend functions (43 source contexts) and 34 native programs.
The independent-only gate must precede that costly candidate build.

Integration must preserve root's fourth Paths::invocation publication fix in
panic_dispatch; the older C base still contains ancestry.append there. The
Attributes witness identity added by the integrator's release repair is also
present here. Field consumers inherited from b937 remain independently
unqualified and have no production dependency from this allowance slice.
