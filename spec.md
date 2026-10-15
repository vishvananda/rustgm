# Compile-time-first Rust frontend architecture

This is the normative architecture of the maintainer reference frontend.
It adapts the principles of cppgm-extended's `spec.md`, pinned in
[docs/cppgm-spec.md](docs/cppgm-spec.md). Preserve its attribution in
[shared/CPPGM-NOTICE](shared/CPPGM-NOTICE). The C++ implementation is a guide to
efficient representations and algorithms, not an implementation of Rust rules.

## Scope and pipeline

Target Rust 1.98.0, first edition 2024, on Linux x86-64. Freeze language release,
Unicode tables, edition, target, panic mode and overflow-check mode in each
relevant test manifest. Older editions arrive explicitly at PA36. Library,
Cargo and rustc-private binary/metadata compatibility are separate objectives.

```text
source image -> Rust lexer -> token-tree/parser/expansion/resolution worklist
             -> syntax with canonical semantic facts
             -> typed Rust body CFG -> initialization/ownership/region checks
             -> drops, demanded instances, closure/async transformations
             -> typed LowIR -> explicit text backend adapter -> CPPGM native
```

The external LowIR serialization is an authorized temporary backend boundary.
No Rust frontend phase may communicate by printing and reparsing token, syntax,
type or diagnostic text. After an in-process backend becomes available, pass
typed LowIR directly. All backend-visible facts must have a LowIR spelling.

## Source, lexing and parsing

Read source in bulk; retain immutable source images and compact file/offset
identities. An ordinary LF source requires no normalization copy. A CRLF file
may keep one normalized view beside its original for physical diagnostics.
Do not build per-character records. Source line/column lookup is cold and lazy.
Identifiers use pinned Unicode XID and NFC rules when interned; raw spelling
and semantic identity are different facts.

The lexer yields typed tokens with borrowed source ranges on demand. Ordinary
syntax consumes a bounded cursor. Retained token trees are allowed for macro
definitions, invocations, procedural-macro interfaces and source provenance;
they are not an excuse for successive owning token streams of the whole crate.
Raw strings and nested comments must scan linearly in input size, with bounded
delimiter lookahead. No token spelling allocation on the ordinary path.

Parse each source or newly produced expansion fragment once, apart from bounded
grammar checkpoints and macro matcher alternatives. Macro fragment matchers
call the ordinary Rust parser. Retain captured opaque fragments and hygiene
identities; never stringify them to recover syntax. Memoize repeated matching
states and diagnose resource limits rather than retrying exponentially.

Macro expansion and resolution form a worklist with explicit unresolved,
in-progress, complete and failed states. Do not repeatedly scan the whole crate
after each new item. Spans carry compact source and expansion context IDs.

## Semantic identity and demand

Intern names, types, declarations, trait goals, instances and layouts with
compact typed IDs. Canonical equality is O(1). Dense per-node facts belong on
nodes or dense side arrays; sparse composite query keys may use hash tables.
Rendered names/types are never semantic keys. Deterministic output must not
force ordered node-based containers onto the lookup hot path.

Built-in semantic traits retain their own definition identities; a user item
with the same spelling cannot acquire language behavior. Resolve aliases and
qualified paths through ordinary typed namespace facts. Demand external metadata
only when used, keep it separate from the crate root, and retain configured
prelude policy per module so lookup does not rescan attributes.

Trait member access is inherited in the resolver's owning scope, including
members installed by expansion. A macro invocation that shares its enclosing
scope must not reset that scope's associated-item metadata. Nested method-body
items retain their ordinary access. Anonymous type imports retain the final
binding by import identity, with scope and hygiene available through the existing
import records. Reprocessing an import withdraws its old anonymous target before
resolving the replacement; method lookup must consume these facts rather than
resolve source paths again.

Retain trait/impl owners and their direct associated declarations in one shared
inventory after name resolution. Keep owner, Self and member identities separate;
generic parameters and nested method-body items are not direct members. Preserve
resolver declaration order, including appended expansion definitions, independently
of hash-table iteration. Destructor discovery consumes this inventory rather
than rescanning all definitions. Receiver storage is shared by declaration checks,
body checking, scopes and CFG construction. Ownership alone is not evidence of
an elaborated signature, valid impl, selected method or executable body.

Elaborate each inherent impl target under its original type/const parameter
identities. Method environments inherit those identities and capture owner
lifetimes in a separate prefix before the method's own early and late slots.
Callable parameter lists validate an ordered declaration-owner path, including
foreign, repeated and reordered parameter rejection. Keep anonymous and unused
impl lifetime slots in the source proof environment.

Index inherent functions once by canonical nominal owner and resolved name after
signature elaboration. Qualified paths retain their resolved owner and typed
prefix; explicit type qualification uses the ordinary annotation worklist.
Selected methods enter ordinary function-item inference, template demand and
lowering. The full callable parameter order includes inherited impl arguments,
but only the method's own parameters consume its turbofish. Instantiate owner
lifetime captures as fresh body regions, including unused captures, then equate
the instantiated impl target with the typed prefix. Check inherited predicates
as well as method predicates. A declaration or required trait signature alone
does not authorize executable dispatch.

Validate unused inherent methods for overlap after completing owner signatures.
Use isolated existential header equations, preserving repeated type and const
parameters. Impl predicates constrain overlap; method predicates, visibility,
receiver spelling and lifetime restrictions do not disambiguate declarations.
Coherence asks whether a compatible instantiation can exist, separately from
whether a generic body may perform an operation. Local conditional derive
contracts constrain this possibility; absence of Copy/Clone on a foreign type
head is not itself a negative coherence fact.

Share each owner-pair proof across method names. Partition closed targets by
canonical lifetime-erased shape, including callable binder erasure, before
probing possible pairs. These erased shapes are index keys only. They must never
replace source proof types or executable lifetimes. Keep deterministic diagnostic
order independent of hash iteration and bound inference, folding and pair work.

Qualified method selection consumes only the typed prefix and caller environment.
Surrounding call arguments, result expectations and method turbofish must not break
an ambiguous prefix lookup. Filter inaccessible candidates and unsatisfied impl
predicates before selection; check the selected method's own requirements afterward.
Caller parameters remain rigid. Unknown prefix arguments may remain possible,
but they do not acquire permissions in the completed body.

Partial annotations, inferred aliases and ordered defaults use one inference
implementation shared by bodies and prefix queries. Import only the reachable
type/const graph into each isolated candidate context, preserving variable
equivalence classes and domains. Failed equations never constrain the source
context. Built-in candidate properties consume actual caller assumptions and
current implementations, separately from compatible-world coherence queries.
After selection, ordinary body inference commits owner arguments and performs
the complete ownership, lifetime and lowering checks.

Retain the candidate partition built for overlap checking instead of maintaining
a duplicate header table. Closed prefix lookup consumes that partition; unknown
prefixes use bounded probes within the nominal/name bucket. Reuse the same shape
folder across headers and prefixes, so repeated aliases do not repeatedly traverse
their type DAG. Property queries cache known and unknown subtrees only after a
probe's equations are fixed and under its immutable environment.

Impl applicability shares complete trait-goal equations with body checking. Cache
caller assumption indexes once per scope, normalizing body lifetime aliases before
indexing. A missing match for a rigid goal excludes a candidate; unresolved evidence
may remain possible until the selected call has all argument/result constraints.
Only proved evidence authorizes a body operation. Canonical nominal and trait
argument requirements use the same complete-goal matcher under rigid caller
parameters. Retain inferred lifetime equalities with their owning declaration or
body; prove both directions using the ordinary region engine and completed nominal
contracts. An outlives relation in one direction alone is not equality. Keep exact
assumption lookups cheap and build broader indexes only when needed. Apply builtin
property queries after trait equations fix their inference inputs. Method-own bounds remain checks
on the selected callable. Prove disjoint target shapes before requiring trait
coherence; missing local implementations do not establish negative evidence.

Dot calls retain the resolved receiver, canonical method name, lexical scope and
written arguments. Type the receiver before selection. For each built-in reference
dereference level, probe the value, shared borrow and mutable borrow in that order.
Reuse the inherent candidate partition and isolated applicability equations;
discard impossible receiver depths before importing an inference graph. Receiver
mutability, moves and loan legality remain obligations of the selected call.

Prepare written method type annotations and constant dependencies in the ordinary
declaration worklist without selecting a method. After selection, validate kinds
and arity against the method's own parameters, then evaluate constant arguments
through the shared capture and scalar-constant machinery. Hidden callable values
must satisfy the same nominal and well-formedness requirements as ordinary callee
expressions. Callable templates and complete owner arguments remain shared with
qualified function-item uses.

Retain selected callable identity, dereference count, borrow type and receiver
parameter type as semantic facts. Validate the hidden callable and actual/expected
receiver value types through the ordinary body path, including field preparation
for generic lifetime instances without their own expression node. Body construction
evaluates the receiver place once, before explicit arguments, and feeds its adjusted
value as the first ordinary
call argument. Implicit mutable receiver borrows use the existing two-phase loan
model; explicit source borrows retain their own earlier loan events. Borrow regions
keep expression provenance, and ordinary ownership, destruction, instance demand
and LowIR lowering consume the resulting body. Do not synthesize or reparse an AST
call to implement receiver adjustments.

An alias used as a value prefix with no written type/const arguments infers all
those arguments, including slots with defaults; explicit type qualification
retains ordinary type-annotation arity and default rules. Store this distinction
in the shared partial-annotation facts, not synthetic syntax. Enum variants take
precedence over same-named inherent functions for associated paths. Alias and
explicit type prefixes reuse the enum's resolved bindings and constructor facts.

Receiver syntax binds the whole value or reference. A reference directly to Self
supplies the output-elision lifetime; nested reference receivers use ordinary
input elision. Explicit receivers retain their terminal type and prove equality
with Self under declared and implied lifetime assumptions. Canonically identical
targets take the direct path. Other targets use the shared directed type-relation
engine in `types`, also consumed by body lifetime checking. Method bodies pass the
ordinary ownership/region pipeline even when no call demands their code.

Trait Self is an implicit possibly-unsized type parameter with its own owner.
Explicit trait parameters retain their source indices; method environments inherit
both owners. Required method signatures check well-formedness while permitting
unsized parameters and results. Default bodies require ordinary sizedness and
ownership proofs under their declared bounds. Required signatures never create
CFG templates or executable symbols. Validate owner bounds and closed parameter
defaults even when a trait or impl has no methods; unchecked trait implementations
must not pass merely because required methods have no bodies.

User trait applications have canonical declaration and argument identities, with
Self retained separately as the predicate subject. Preserve every type, const and
lifetime argument through inference, substitution, parameter-use queries and body
universal aliases. Arguments are not synthetic nominal types. Complete predicates
participate in closed/default checks even when their subject alone is concrete.

Prepare trait headers before dependent bounds. Share ordinary path argument and
default elaboration, then instantiate each trait contract through one cached,
capture-safe mapping of Self and its explicit parameters. Expand only requirements
on Self into caller assumptions; parameter and composite-subject bounds require
separate evidence. Diagnose supertrait cycles by declaration identity before
expansion, and deduplicate diamond paths under a bounded worklist. Arbitrary trait
names confer no builtin Copy, Clone or Sized behavior.

Trait argument well-formedness and declared requirements are checked even for
unused declarations. Lifetime requirements consume the completed nominal contracts
and shared directed relation engine. Assumptions from the original signature and
explicit bounds remain distinct from obligations imposed by a trait application.
Predicate inference shares the ordinary type/const equivalence classes, and
completion rejects unresolved arguments that appear only in an obligation.

Caller-bound trait goals participate in the same inference graph as call arguments.
Import each probe with one shared mapping across Self and all type/const roots.
Index assumptions by trait identity and the rigid outer subject shape; do not scan
unrelated parameter subjects. Deduplicate identical assumptions before probing.
Failed alternatives never constrain the caller. Commit a unique match and retain
its actual assumption identity and every required lifetime equality. The ordinary
region checker proves those equalities; matching never erases lifetime obligations.

An unknown whole Self is not inferred merely from an available caller assumption.
Other arguments may be inferred, and a bounded worklist revisits ambiguous goals
after another goal constrains their shared arguments. Check declaration ambiguity
even for unused owners. Distinct lifetime applications that remain type-compatible
are ambiguous; identical repeated bounds are one assumption. A declaration-owned
exact bound with no competing index entry needs no inference probe. Bodies without
user trait call obligations take the empty fast path.

Resolve a destructor's impl target and receiver once. Retain the nominal item,
method identity, whole-reference receiver binding and captured lifetime slots.
Anonymous impl lifetimes are real slots, not fabricated declaration IDs. A
destructor must cover every instantiation of its nominal type: target lifetimes
are distinct universals; unused impl lifetimes are existential during coverage
checking. Prove declared requirements against nominal well-formedness, including
field-implied bounds, using the shared directed outlives graph. A receiver has an
independent late lifetime; reference mutability and binding mutability are
separate facts. A typed destructor is not an ordinary callable function value
and is not evidence that cleanup has been proved or emitted.
Index declared lifetime membership separately from first-use slot assignment;
capturing a wide impl binder must not scan all its declarations per parameter.
Reserve each actual Drop owner before evaluating any impl-target constants.
An exact outer-constructor query follows resolved alias heads, parameter forwarding
and only the defaults needed for that head; it does not evaluate const arguments
or stand in for complete target elaboration. Supplied arguments substitute once,
simultaneously. Finite self applications in defaults remain valid, while genuine
head/default cycles fail. Constant eligibility consumes these owner identities;
method bodies and cleanup still require completed implementations.

Generic Drop headers must cover the complete nominal schema. Invert direct,
distinct impl type/const arguments into the nominal parameter environment before
proving that impl and method bounds add no requirements. Include implicit Sized;
never use the impl's own assumptions to prove its conformance. Captured lifetime
slots are separate from named argument indices, including anonymous impl slots.
Use the shared outlives graph for lifetime and type-parameter requirements;
unused impl lifetimes remain existential. Inherited method environments retain
each parameter's original declaration identity. Classify such bodies as templates
until demanded concrete instances and their destruction plans are available.

Keep the checked inverse impl-to-nominal argument mapping on the completed Drop
implementation. Selecting a method for a concrete target consumes that mapping;
it does not repeat source matching. A runtime callable retains only target
lifetime captures and an independent late receiver binder. Unused impl lifetimes
and their bounds remain in the original method proof, rather than receiving an
arbitrary concrete witness. Validate that the selected receiver matches its
concrete nominal target, including reordered type/const arguments and nested
reference binders.

Destruction recipes for type/const instances use full canonical type identities
and prepared concrete fields, while retaining declaration field/variant order.
Lifetime-only recipes may retain their existing shared representation. An
unresolved generic destructor is distinct from a selected callable instance.
Separate retained recipe/query state from short-lived semantic borrows so new
instances can be discovered without rebuilding completed recipes. Actual cleanup
selection, original template proofs and instance ownership/lifetime/loan proofs
remain separate requirements before executable generic destruction.

Prove each concrete body once as demand discovers it. Keep the immutable owned
body with its selected cleanup plan until executable assembly; match its source,
complete instance and original template proof. Assembly cannot discard cleanup
authorization or insert unproved bodies. Retain completed structural outlives
queries and recipe caches across append-only semantic discovery, including their
work budgets. Prove unused ordinary bodies without demanding their callees.

Discover destructor code from selected cleanup roots, following fields, variants
and nonempty array intervals with shared visited state. A cached recipe by itself
does not demand executable code. Enqueue selected methods on the ordinary instance
DFS stack before leaving their owner, so recursive destructor growth shares the
existing depth and storage limits. Prepare coercion destinations as well as
expression types: a distinct nominal lifetime instance needs its own field facts.

Keep dense liveness words for CFG joins, but enumerate only nonzero words when
recording point seeds. A compact summary tree preserves ascending identity order
without scanning every inactive entity at every cleanup event. Update summaries
on zero/nonzero transitions and rebuild them after dense joins; retain the same
work limits and charge enumeration and join work explicitly.
Forward loan states use the same index for pruning, conflict checks and optional
inspection. Rebuild the occupied-word index after dense CFG copies, clear it when
the last loan in a word expires, and preserve two-phase activation separately.

The executable CLI enters this template/instance pipeline before the ordinary
inspection-only body builder. It must not duplicate promotion, constant or proof
passes. Earlier inspection modes retain their own contracts. Once explicit and
referenced implicit constructor templates have been discovered and original
proofs completed, an empty sealed template set permits skipping the generic
dependency walk: ordinary lowering already follows ordinary call/cleanup uses.
All ordinary source bodies must still be proved before this fast path.

Analyze generic bodies without reparsing syntax. A monomorphization key must
include all type/const arguments and semantic context affecting its result.
Trait solver caches include parameter environment, binder universe and mode;
negative results need equally complete keys. Each demanded fact is computed
once; cycle detection and explicit solver budgets are part of the contract.

Separate generic parameter identity, alias expansion and default-template
readiness. Explicit arguments demand no unrelated defaults. Omitted arguments
demand each needed template through the bounded declaration worklist; a default
can use an explicit self application or a later default whose template depends
on an already supplied parameter. Detect actual default cycles independently
of alias-body cycles, and check every unused default before publishing facts.
Elaborate alias bound subjects after their bodies are available; parsing those
bounds does not make them enforced requirements at alias uses.

Reserve nominal identities separately from complete field templates. Demand fields
on the same bounded declaration worklist, publish a complete group only after its
annotations finish, and distinguish field/default cycles from pointer recursion.
Index constructor owners once; initialize tuple-callable facts only when demanded
or during final completion. Register representation and derive selections before
early semantic queries can cache absence. Constant dependencies include referenced
aliases, constructors, immutable statics and named initializers, by-value shapes,
enum discriminant expressions and local cast annotations in their lexical contexts.
These prerequisites make values available; final unused-declaration, variance,
derive, lifetime and layout checks remain mandatory.

Early expression paths use the ordinary argument scheduler, including alias
variant selection, function values, partial annotations and consumed trailing
defaults. Nested const arguments own their dependency work and lifetime context.
Determine value readiness from each nominal declaration's field templates once;
follow actual type arguments separately. Do not expand symbolic field instances
to discover these prerequisites. Instantiate concrete fields after inference.
Before evaluating a const argument, reject references to enclosing generic
parameters while allowing lifetimes bound inside the constant. Cache syntax
capture summaries using scope ancestry, preserving syntactic captures even when
type aliases erase them. Use compact node-indexed completion flags and sparse
capture records, so capture-free syntax needs no per-node hash entry. Standalone
const parameters retain their symbolic identity.

Retain runtime const operands as canonical const arguments, separately from
named constant definitions. Publish array index bounds after inference resolves
the receiver: a bound is either a target `usize` value or the original rigid
const parameter. A symbolic fixed array does not become a slice or a zero-length
array. Place overlap and move checks use the same conservative indexed-place
rules while the actual bound awaits substitution. Keep concrete node facts compact.

Resolve a use's defaults with one append-only substitution environment. Earlier
arguments never change, and inserted replacements are simultaneous. Successful
cached results therefore remain valid as later slots are appended, including
binder shifts. Intern the completed argument list once and retain that same
environment for the expanded body. Do not copy and intern every argument prefix
or create a separate cached substitution for each omitted parameter.

Infer nominal variance over completed declaration templates, with distinct
lifetime, type and const slots in declaration order. Compose variance variables
through nested nominal arguments using shared product expressions and reverse
dependencies; solve the finite lattice with bounded monotone propagation.
Never expand recursive field instances to infer variance. Callable binders
retain their own lifetimes. Recursive-only type/lifetime parameters remain
unused; const slots require exact equality even when no field uses them.
Directed region checking consumes the completed table for both lifetime and
type arguments, after structural inference retains the actual/expected types.

Generic nominal fields are instantiated from completed typed declarations, keyed
by the canonical nominal type including every type, const and lifetime argument.
Reuse one capture-safe substitution across an instance's fields. Share the
declaration's variant order, representation and final discriminants; retain only
substituted field lists per instance. Prepare demanded fields before immutable
layout and ownership queries. Follow owned fields and derive argument obligations
iteratively, stopping at pointer/reference referents; equality separately demands
reference referents. Do not expand growing recursive pointer instantiations.
Implicit field dereferences also demand their selected receiver's fields, even
without a separate syntax node. Retain receiver discovery order when publishing
these canonical types; hash-map iteration must not determine type identities.
Failed preparation cannot publish unfinished descendants as prepared. This
availability check does not prove declaration validity, bounds or finite layout.

Complete nominal parameter schemas and declaration contracts independently of
field instances. Prove field and use-site predicates in their owning immutable
parameter environment; a cached predicate list is not a cached proof across
environments. Generated Copy, Clone and equality implementations have separate
assumptions. Propagate by-value parameter requirements over declaration templates
to reject infinite storage without expanding growing recursive instances.
Zero-length arrays still require their element alignment. Infer field-implied
outlives requirements over the same symbolic arguments, retaining type-parameter
requirements as well as lifetime pairs.

Constructor and pattern inference retain unresolved mixed arguments. Reuse one
substitution session across the selected receiver's field templates; do not
resolve the receiver before its fields have constrained it. Substitution under
callable binders shifts inserted outer lifetimes without capturing inner ones.
Written partial arguments, omitted arguments and omitted trailing defaults are
distinct inputs. Alias-qualified variants retain their resolved namespace prefix
and selected member rather than repeating path lookup in body checking.

Validate symbolic representation constraints without inventing concrete layouts
for rigid parameters. Keep failed symbolic queries failed and permit unrelated
queries afterward. The pinned packed-declaration restriction follows direct
named struct fields before substitution; instance size/alignment remains a
separate numerical query. A generic callable template is never an executable
body merely because its declaration exists. Concrete lowering requires a
demanded instance with its complete arguments and proofs.

Preserve complete callable identities through function values, constants and
promoted data. Check every generic template, including unused bodies, before
specializing its retained CFG. Substitute parameters by their declaration
identities; callable argument positions need not include late lifetime slots.
Abstract lifetimes inside actual type arguments into a fresh instance proof
context while preserving nested binders and the template's own variables.
Pair each specialized body with its own region proof and complete instance
identity, even when multiple bodies share an original source location.

Schedule demanded instances iteratively, registering an instance before following
its dependencies so finite recursion terminates. Bound active recursive growth
separately from independent uses of one function. Check retained storage before
cloning a body. Follow actual calls, function-pointer reification and typed
constant allocations; a discarded zero-sized function item does not require its
body. Key promoted backing storage by concrete body identity, and preserve
runtime overflow and bounds checks when materializing its values.

Retain selected implicit tuple/variant constructors during type checking and
build their factory bodies only when referenced. These factories share the
ordinary field-transfer CFG builder. Their generic proof uses the nominal declaration's assumptions and
lifetime contract, with one shared immutable parameter environment per nominal.
Retain owned field transfers and returned-value scopes before specialization.
Prepare generic return fields in the original declaration context as well as
any expression context using free lifetime aliases.

Derived abilities constrain type arguments even behind raw/function pointers;
packed Clone and equality derives impose Copy bounds as well. Owned destruction follows
instantiated fields and skips zero-length arrays. Custom Drop instead retains
every argument's full outlives components, including Copy type parameters and
their reference lifetimes. Symbolic cleanup consumes the same field instances;
it never substitutes for selected concrete destructor methods and executable
recipes keyed by all relevant instance arguments.

Record selected methods, adjustments, casts, layouts and obligations where
later phases consume them. Lowering cannot repeat lookup or inference. Missing
facts are invariant failures, never a reason to fall back to text or rustc.

Declaration-time enum values have an explicit pending query registry, distinct
from finalized aggregate discriminants. Shape constants must demand those queries;
an empty pending registry never means a provisional zero is final. Reuse completed
values and the variant index across argument evaluations. Check all enum values,
including unused duplicates and overflows, before publishing final layouts.
Constant borrows consume retained typed alignment obligations, including dead
branches, without evaluating their places. Layout readiness shares the constant
work stack: complete required enum tags and by-value field layouts before caching
the layout, retaining zero-array alignment and stopping at pointers. Detect genuine
layout/value cycles. Reuse completed layouts and index appended borrow obligations
once; an unrelated enum inventory must not be rescanned for each argument.

Constant eligibility and body construction share lexical temporary-scope analysis.
Keep that analysis below body construction so declaration queries do not depend
on a later phase. Transferred results, static reads and references with static
lifetime extension do not imply destruction. Owned projection bases, update bases
and ordinary zero-length repeat operands retain cleanup obligations. The pinned
stable profile rejects cleanup of partially moved temporaries as well. Qualify
unexecuted branches too. Memoize resolved place roots and destruction requirements;
demand scope analysis only when a temporary may need destruction. Invalidate
absence-of-destruction results if the impl registry grows during declaration work.
Anonymous constants that emit lifetime obligations retain a typed proof body;
the ordinary region, loan and destruction checks must consume those obligations.
They are never emitted as native functions. Scalar constants without lifetime
facts need no additional body. Static lifetime extension supplies permanent
storage provenance independently of promotion, including for a value with Drop.
Annotations elaborated before constant-body inference already belong to that
constant's proof body, including inferred lifetimes subsequently erased by aliases.
Retain this ownership even when inference adds no new lifetime facts.

Constant patterns constrain types during ordinary body inference. Retain pending
value/range/coverage work in flat arenas with the originating file, module and
body; finish it after initializer typing in one shared query session. Reuse
resolved dependencies, selected operations and already computed discriminants.
Cache demanded constant-definition values across these phases in the current
monomorphic profile; leave ordinary node facts compact and allocate no query
storage for bodies without constant-backed patterns. Do not instantiate a
whole-crate checker per use, repeat initializer typing, or interpret pattern
views. Deferred checks must retain the matching module's visibility context.

Demanded tuple/array constant values use flat shared arenas. Repeats retain one
element and their length; field/index queries select from that representation
without materializing the repeated array. Translate constants to the existing
selected product/scalar patterns with use-site provenance. Share repeated
product subtrees within a use; retain each projection's distinct place identity
for decisions and execution. Check type-level PartialEq obligations even for
empty products, then validate actual scalar leaves (including NaN rejection).
Reserve expansion work before allocating siblings so pending work is bounded.
Neither value storage nor pattern translation grows dense ordinary node facts.

Nominal constant products retain selected constructor and field identities from
typing; updates fill only missing fields from the shared base value. Validate
constructor callees and update bases even for zero-sized values or completely
overridden fields. Equality obligations are a separate cached type query from
actual structural-value eligibility. Check every enum field for the former and
only the active variant's values for the latter. Derived reference cycles use
an iterative greatest fixed point with reverse dependency edges: publish all
positive and negative results only after propagation completes. Count actual
nodes, dependencies and propagation work under an explicit query budget.

Selected string and byte-string literals retain decoded bytes in one flat,
interned byte arena, addressed by sparse syntax-node facts. Constant evaluation
and body construction reuse those identities without decoding again.
Equality compares exact bytes; identifier Unicode
normalization must never affect text values. Share repeated text subpatterns
within a constant use while preserving each selected place. Byte-string patterns
translate to ordinary fixed-array or exact-slice decisions under the existing
preallocation work budget. String coverage partitions only the current query's
observed values plus one class for every other string. Do not scan unrelated
bodies or enumerate a string domain. Matching a value constant compares type
structure without equating its storage lifetime to the input lifetime; ordinary
expression constraints remain explicit. These pattern facts do not discharge
the region and loan proofs required for reference execution.

Demanded shared constant references use flat place records containing an optional value,
canonical type, access provenance and packed alignment. Separate place queries
from value reads so borrowing a non-Copy static or selecting its Copy field
does not move its container. Check Copy reads before consulting a definition's
cached value. Field/index queries consume selected identities and sparse
autodereference facts; they neither repeat lookup nor materialize repeated
arrays. Preserve provenance across projections and use the actual referent's
alignment after dereference. Borrowing requires alignment, while an owned field
read need not form a reference. Reference chains use bounded worklists for
evaluation, pattern translation and destruction. Keep ordinary node facts
compact; reference operations still require region/loan proofs before LowIR.
Intern static allocation identities before evaluating their initializers. Forming
an address or fixed projection uses typed storage facts; it must not introduce
a dependency on the stored value. Actual reads demand and cache location
contents through the bounded query worklist, after checking access and Copy
requirements. Complete reachable static backing with a growing identity queue
before lowering. Address cycles are valid storage graphs; initializer value
cycles remain errors. Constant patterns demand finite referent expansion and
diagnose cycles independently of executable pointer graphs.
Coverage derives root validity from the selected input place and retains it
through deferred binding checks. Explicit and automatic dereferences cannot
reuse owned-value inhabitation to prune variants with potentially invalid fields.

Floating pattern domains use constant-time bit/rank conversions. Numeric ranks
collapse signed zero, while a separate unordered class keeps NaN visible to
exhaustiveness. Reuse scalar interval indexes and composite coverage decisions;
do not enumerate floating values or add per-value domain storage. Execution
compares the original typed value and captures its original bits. Even a range
covering every numeric rank must test its input because it excludes NaN.

Dynamic-slice coverage retains exact/minimum lengths and typed start/end/rest
projections. Lazily collect tested positions from the current query's selected
roots, keyed by canonical slice type. Specialize only lengths at which a tested
start and end position can alias; keep the remaining length intervals shared.
Canonicalize aliases to one place identity, including nested patterns. Wildcards
and unconstrained captures add no alias points. Charge collection, position pairs,
specialization and graph work to the same bounded query; sort the completed alias
set once rather than using ordered node containers for hot-path membership.

## Rust body representation

A typed control-flow IR retains places, projections, moves, borrows, storage
lifetimes, initialization and abstract drop obligations. It is an executable
body representation, not a second copy of the complete syntax tree. Do not call
it MIR without qualification: CPPGM already uses MIR for machine IR.

Share live storage tails persistently. An exit records the suffix it ends,
without copying the surrounding live-local list or repeating every storage-end
statement at every failure branch. Preserve reverse lifetime order and skip
already-closed nested scopes. Storage compaction is distinct from ownership
proof complexity: charge the paths/words actually visited and retain explicit
dataflow memory and work budgets.

Compute destruction requirements from canonical types with a bounded iterative
query, memoizing shared shapes and errors. References do not own their referents;
zero-length arrays do not require element destruction. Array length must not
expand query work. Programs without custom destructors need no pending-drop
metadata in the current closed type profile.

Pending body facts retain destructor identities, type requirements and candidate
store/storage exits. Reuse persistent live-stack ranges and existing exit IDs;
do not copy every local at every exit. These candidates can overlap and are not
an execution plan: initialization, moves, owned rvalue scopes, ordering and
destructor-induced liveness still need proof. The ordinary ownership, region
and loan entry points reject pending elaboration. The destruction
verifier discharges those obligations and attaches immutable selected plans and
glue recipes to checked bodies; only that completed proof can authorize cleanup
lowering. Raw body views may expose pending work explicitly. A value needing
destruction cannot be promoted to permanent storage, even while its cleanup
plan remains pending.

Promotion eligibility and evaluated values are distinct facts. A generic
temporary can qualify for permanent storage while its value awaits concrete
arguments. Retain that eligibility against the original typed value DAG;
storage, region and cleanup proofs consume it without inventing bytes. Unknown
repeat lengths still require eligible operands, and potentially failing division
or value-level indexing cannot become eligible merely because an operand is
symbolic. Preserve runtime effects and checks around promoted roots. Concrete
verification rejects unresolved promotion values; demanded instantiation must
evaluate them before executable lowering.

Retain destructible rvalue creation scopes while building the body, including
discarded values that never become borrowed places. Forwarding a block result
forwards its temporary scope without inventing a second object. Pattern captures
use their retained binding scope. Mark returned values as transferred before
local cleanup, preserving order relative to storage exits at the same point.

Initialization has a distinct may-be-initialized plane; immutable-binding write
history cannot substitute for it after moves. Share the existing transfer and
fixed-point implementation, with additional state retained only on demand.
Expose bounded point queries over retained entry states for cleanup planning.
Preserve sparse array element/gap partitions from move-path numbering when that
analysis requests them; never reconstruct these mappings from output or expand
unselected array elements. A whole-path mixed state can include partial moves
and does not authorize dropping the entire object behind one flag.

Retained initialization also tracks destructible rvalues that never become
places. Derive their consumption once from typed aggregate/call operands,
constraints, stores and recorded returns. Share the existing solver and inspect
each consumed source before clearing its state. Preserve immediate disposal of
an ordinary zero-length repeat operand separately from transfer and constant
repeat instantiation. Retain the constant-path selection from type checking:
runtime coercions can wrap a constant, and blocks can forward its value without
being constant repeat operands. Neither array length nor text reconstruction
may expand this work. These consumption facts do not substitute for scope cleanup or its
destructor-induced loan and region uses.

Scope-aware initialization reuses the validated storage event index. Retain one
ordered owner list per scope, merging local declaration and temporary creation
order; traverse shared ancestor ranges at exits instead of copying live lists.
Merge return transfers with storage events at equal CFG points. Each successor's
cleanup starts from the same outgoing state. Scope and initializer endpoints
clear owned initialization, so loop reentry cannot resurrect an earlier value.
Materializing an already-owned value must not initialize it again.

Expose ordered cleanup candidates before their state changes, including scope
exits, explicit storage ends, replacement and discarded repeat operands. Overlapping
endpoints observe the state left by earlier ones. Borrowed and dynamically indexed
replacement destinations do not claim their containing pointer or array's move
path. These candidates still require field/array glue, conditional flags and
destructor-induced liveness before they become executable cleanup.

Generic cleanup reuses the initialization replay and ordered event schedule.
Select possible destructor access separately from original-type drop liveness:
field moves can remove every runtime call while retaining an aggregate's lifetime
obligation. Abstract plans contain no native glue and cannot authorize lowering.
Scoped drop queries preserve opaque owned type parameters as well as named
lifetimes; Copy assumptions suppress only the exact proved subject. Ordinary
coerced operands retain their input components until consumption. Unknown-length
constant repeats instantiate elements; ordinary repeats require Copy and cannot
supply an owned destructible prototype. Do not guess whether a symbolic length
is zero. Sparse move partitions, bounded iterative queries and shared component
lists apply to these proofs as they do to concrete cleanup.

Internal match-guard views retain selected place identity and delay owning
captures until success. Protect tested places without runtime loads or address
formation for inactive variants. Track guard protection liveness across control
flow; a diverging path does not imply a later guard use. Internal view proofs
are distinct from the pending loans and regions of user reference values.
Build guarded pattern retries with shared first-match and resume continuations,
not an expanded Cartesian product. Preserve written alternative/field order and
end attempt-local storage on failure. Static path offsets may be cached as typed
recipes, but addresses must be formed where they dominate their actual uses.

After region and loan proofs, sized references lower to thin pointers and their
lifetime binders have no runtime representation. Collect address-observable
places sparsely. Materialize borrowed scalar temporaries once at their defining
evaluation, preserve object snapshots, and use persistent backing for promoted
roots without deleting their original effects or runtime checks. Borrowed
zero-sized values require aligned, nonnull backing; their addressable fields
retain canonical offsets even though value loads and parameters remain erased.
Reference pattern paths share persistent dereference prefixes alongside ordinary
offset recipes. Load each pointer in the current use block, restoring referent
alignment after dereference. Representation-preserving lifetime conversions may
reuse values; thin-to-fat coercions must construct metadata explicitly.

The course x86-64 representation for references and raw pointers to slices or
`str` is a 16-byte, eight-aligned object: a data pointer at offset zero and an
unsigned 64-bit length at offset eight. This is an explicit course ABI, not a
claim of a stable Rust ABI. Slice length counts elements, including zero-sized
elements; string length counts UTF-8 bytes. Unsized referents themselves still
have no fixed storage layout. Use the existing object calling convention for
wide values and retain metadata sparsely beside unsized place projections.
Array-to-slice conversions use the checked source length and compatible element
layout. Dynamic indexing checks that length before address calculation, even
when the selected value is zero-sized. Reborrows retain the source metadata.

Literal backing is immutable and shared by canonical bytes. Empty strings and
byte strings receive nonnull backing with zero logical length. Persistent wide
values serialize both the address and length at their canonical offsets;
packed fields use the existing byte serializer and pointer startup fixups.
Metadata and literal identities must not enlarge ordinary scalar place facts.

Dynamic slice patterns test exact or minimum length before selecting any element.
Suffix projections retain the tested minimum length as a typed fact; rest
projections retain excluded prefix/suffix counts. Loan overlap uses those facts
to distinguish disjoint pattern captures. An arbitrary runtime index does not
receive the same disjointness proof. Normalize nested rest projections without
enumerating elements. Pattern metadata reads are distinct from element reads
and moves, and retain initialization, storage and liveness obligations.

Ordinary and guarded decisions consume the same selected constructor conditions.
Guard retry recipes rebuild both addresses and lengths in their actual use
blocks; cached recipes must not cache branch-local runtime values. String tests
compare exact interned bytes after checking length. Emit a bounded-size comparison
loop instead of one instruction sequence per literal byte, and never load bytes
on a failed length test. Constant text patterns obey the same execution path.

Borrow checking runs before references and moves lose their source semantics
in LowIR. Region/dataflow analysis uses typed worklists and bounded lattices,
with per-body storage and complete dependency edges. Pattern usefulness and
trait solving must expose work counters and explicit complexity limits.
PA15's region inspection exposes the retained directed contract graph separately
from PA16's checked loan inspection. Render deterministic IDs, direct edges and
sparse point/loan words; never use inspection text as proof input. Optional loan
snapshots observe the final checked transfer pass, after pruning and call
activation. Executable verification must not allocate those snapshots or retain
the per-body inspection arenas. Counts describe the same facts as the full view.

Destruction consumes initialization/move facts to elaborate conditional drops.
Async transformation accounts for borrows, pinning and cancellation cleanup.

## Lowering and backend facts

Lower only checked, typed facts. Give every emitted instance, drop shim, vtable,
closure and coroutine a stable emission identity and emit it once. Rust signed
overflow, saturating casts, zero-sized types, fat pointers and panic behavior
must not inherit C++ assumptions. In particular, do not blindly emit C++
`nowrap`, `typed` or alias facts. State only what Rust analysis proves.

Cleanup lowering consumes the proof's ordered statement, cleanup and flag-write
events. Preserve successor-specific cleanup on real CFG edges; imaginary proof
edges never generate runtime code. Retain no event tape for bodies with no
runtime cleanup. Share demand-emitted glue by canonical recipe identity, and
emit custom destructor calls before automatic field destruction. Variant tags
select only active fields. Array intervals use counted loops, including
zero-sized elements; array length must not expand emitted instructions. Use
bounded iterative emission, with field cursors preserving declaration order.
Give zero-sized destructor receivers stable aligned storage, and copy packed
fields into aligned backing before invoking their glue. Ordinary expressions
and cleanup share the same typed layout and address operations.

Use the shared LowIR model contract; serialized text is an output view. The
temporary backend adapter accepts LowIR only, validates it with CPPGM and then
builds native code. No oracle object or reference function may replace a failed
Rust compilation. General assembly and target intrinsics require an explicit
capability review before PA37; the initial backend is not assumed complete.

Persistent data consumes completed constant values through a read-only typed
interface. Reserve global identity before following address dependencies;
statics retain their own storage and projections retain their original root.
Serialize canonical field offsets and exact scalar bits with bounded flat work.
Memoize zero-filled products and keep repeated zero data compressed. By-value
constant aggregates still produce independent runtime snapshots. When the
external backend cannot relocate an unaligned pointer field, retain its packed
offset and install the address before Rust entry code; never silently realign it.

## Ownership, allocation and complexity

Use owned arenas/slabs/vectors and compact IDs instead of Rc/Arc pointer graphs,
per-node boxes or recursively owned trees on hot paths. Prefer slice/range
children and reuse per-query/per-body scratch capacity. Bulk destruction must
follow phase ownership. Avoid an allocation per token, edge or solver step.

Each function owns mutable body/LowIR records. Editing one function must not
rebuild another. Release per-function scratch promptly and discard frontend
ownership before whole-program LowIR optimization. A future O0 in-process
backend should consume and release functions incrementally.

Lexing should be O(source bytes + produced tokens); parsing near-linear except
explicit language-required matching. Resolution visits relevant scopes and
candidates, not unrelated declarations. Lowering should be linear in emitted
IR; fixed points must make monotonic progress in a bounded lattice/worklist.
Any broader bound needs a documented semantic reason and adversarial tests.

Telemetry observes work already being done: bytes/tokens, retained token bytes,
allocations or arena high water, solver goals, retries, region edges, demanded
instances and phase time. Disabled instrumentation must not change semantics.
Track throughput and peak RSS at several input scales. Compare frozen binaries
with identical inputs, A/A calibration and paired ABBA blocks; retain all raw
samples and verify output equality. Wall-time noise alone is not a regression.

## Safe Rust and audits

Forbid authored unsafe compiler code. The input language still includes unsafe
Rust; fixtures are not compiler implementation. Keep OS/process/environment
access at explicit driver/build boundaries, outside `dev/src/` environment
reads. Audit dependencies and feature sets before use.

Run rustfmt and Clippy all/pedantic with warnings denied. The file audit counts
code lines: 3000 per implementation module, 2400 for registered interfaces,
1800 for internal models; functions 240, or 360 for cohesive dispatch with at
least eight arms. Nesting warns past six and fails past eight. Do not hide logic
in macros, compressed lines, generated files or mechanically split modules.
