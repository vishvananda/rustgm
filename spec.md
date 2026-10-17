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

Associated type projections retain their declaration identity, Self, complete
trait application and separate family arguments. Their immutable schema fixes
the declaring trait, result sizedness and declared Copy/Clone promises.
A projection is not a nominal type:
equal projected results do not establish equality of their inputs. Keep its
inputs invariant and preserve them through parameter/lifetime substitution at
the actual binder depth. An explicit dependency flag also identifies closed
projections that still need normalization. Structural properties and occurrence
sets are not implementation evidence or an injectivity proof. Concrete layout,
Copy/equality, destruction and lifetime queries must not infer result properties
from an unfamiliar constructor or silently treat it as a primitive.

Shape queries preserve the binder depth of captured associated inputs, including
empty callable binders. Match trait premises and explicit Sized subjects by
lowering only the surrounding prefix into existing canonical identities. Keep
nested local binders unchanged; a captured local cannot become an outer premise.
Share substitution's dependency walk and typed rebuild dispatch, but perform
these lookups without interning or cloning the assumption environment. An
immutable assumption already owns every canonical child of its root, so absence
can withhold permission without asserting that the type itself cannot exist.
Cache such misses only for that fixed environment. Keep ordinary root caches
separate from lazily allocated scoped caches and charge all lookup work.

Builtin promises are conditional declaration facts, not parent formation
or normalization authority. Register them before interning applications; never
change them beneath cached queries. Copy entails Clone and both entail Sized.
Copy queries may consume these promises without exposing the result's structure;
only Copy can remove possible destruction and opaque drop-region requirements.
Clone alone cannot remove those duties. Preserve separate input and lifetime
proofs, and never infer an input ability from the result. Close builtin promises
over resolved Self supertraits once, before projection schemas are registered.
Use the already decoded declaration bounds, with a bounded monotone worklist;
ordinary direct-builtin declarations need no closure allocation. Bounds on other
parameters or associated members grant no Self ability. An alias that later
normalizes to Self remains a separate required predicate, whereas parentheses
preserve the resolved Self identity. Keep the original complete predicates for
formation and implementation validation; these early bits are no substitute for
those proofs and must never be patched beneath cached type properties.

An associated type definition is checked under its impl's immutable environment,
including when no caller uses it. Prove its well-formedness, declared result
bounds and lifetime requirements; the definition itself cannot add assumptions.
Match required types by declaration identity and type namespace, retain the
selected definition with the validated implementation, and share lifetime queries
across all associated definitions of that impl. An ordinary alias's deferred
well-formedness policy must not weaken an associated type contract.

Shorthand associated type lookup keeps naming separate from projection proof.
Index written parameter bounds by lexical owner and resolved subject; elaborate
only a demanded bound's positional inputs, without publishing an unfinished
caller environment. This allows `where T: A, T::Out: B` independently of clause
order. Ordinary alias bounds can name a member but cannot authorize its use.
Trait Self also sees method-local Self bounds; trait-impl Self starts from its
own trait application and supertraits, excluding unrelated impl where-bounds.
Concrete types and inherent Self do not infer a trait qualifier. Generic defaults
require explicit trait qualification in the pinned profile.

Cache member candidates by complete predicate and canonical name, retaining
type, const and lifetime arguments. Rebase captured lifetime slots into the use
binder before lookup; duplicate identical applications are harmless while
distinct applications remain ambiguous. Run the selected typed projection through
the ordinary scoped normalizer and retain its formation and lifetime obligations.
The candidate cache grants no permission and does not select an implementation.

Opaque projected values in checked generic bodies retain move and possible-drop
requirements unless Copy is proved. Cleanup conservatively retains every input
lifetime and parameter, as runtime liveness already does; this sufficient route
does not infer reverse input bounds from an associated result. Concrete execution
still requires normalized storage and selected destruction. The conservative
input route is not the complete projected-result drop-checking contract.

Associated constant declarations retain their types independently of optional
initializers. Check missing and extra members by resolved owner and name. The
validated implementation publishes the final default/override identities. A
separate declaration-only member table may publish a checked constant after all
candidate headers are frozen; it grants neither method access nor whole-impl
validity. Final conformance must agree with every earlier selected identity.
Substitute the trait's Self and mixed arguments into the impl environment once;
normalize projected types using that impl's checked associated definitions.
Structural types must agree, while reference results may satisfy the trait by
outliving its declared result. Preserve invariant positions and every proof debt.

Constants introduce no parameters or predicates of their own, so their annotation
and initializer checks share the completed owner's immutable generic environment.
Retain captured lifetime binders in both semantic checking and initializer CFG
contracts. Required declarations may name unsized types, but actual initializers
must produce sized values. Check type formation and lifetime requirements even
for unused declarations with no value. Type checking is separate from demanded
evaluation: an unused associated constant must not be evaluated merely because
its declaration or implementation was checked.
Before executable lowering, reject non-const calls in every initializer, including
unused values and branches that evaluation would skip. Recognize tuple/enum
constructors by retained callable identity; a function pointer to a constructor
does not grant const-call permission. Earlier type inspection remains separate
from this eligibility proof.

Associated value uses retain a constant instance identity with the complete owner
arguments, including parameters absent from the result type. Solve those argument
lists with ordinary inference; do not invent function types for constants. Keep
inherited bounds, lifetime relations and each caller's selected implementation
proof separate from the structural value cache. Defaults and overrides enter the
selected initializer's own argument environment.

Build associated initializers as checked templates, including unused values and
closed owners. Promotion may retain an eligible but unevaluated value; required
constant evaluation still diagnoses its faults when demanded. The pinned profile
also diagnoses cycles among closed trivial initializer aliases before demand.
Selected dependency graphs include branches that value evaluation can skip.
Retain callable dependencies in those templates as complete function-item types.
Substitute the selected initializer's owner arguments before selecting trait
method overrides, including function pointers nested in products. Each caller
retains its own selection proof; a cached structural target grants no permission.
Callable bodies are executable dependencies, not constant-evaluation edges.

Share completed dependency closures by the selected instance's complete arguments
and implementation-availability mode when their proof is empty. Every transitive
initializer must have completed typing; an absent dependency list before that
point is not an empty graph. Propagate incompleteness through shared diamonds.
Caller environments, universal mappings and outstanding lifetime debts disable
this cross-caller reuse. Root selection still proves the requesting use, and
nonempty proofs still belong to their caller. This removes repeated traversal
of closed dependency chains without treating cached target identities as evidence.

Evaluate the retained expression template under a distinct instance context.
Node, place, contents and in-progress dependency states must use that context;
ordinary constants and statics retain their global identities. Substitute stored
types and const arguments without modifying shared source facts. Share repeated
values by selected identity and complete arguments, including persistent data,
while preserving independent callable, lifetime and destruction proofs. Bound
both instance discovery and the expanded evaluation work.

Finalize enum discriminants after implementation validation. Their associated
values use concrete dispatch and the shared instance evaluator; retain selection
obligations under the requesting anonymous constant body. Publish each fresh
region interval before lazily typing another initializer, which can allocate
additional regions. Apply the shared capture check before evaluation: a constant
initializer that ignores its arguments does not permit a discriminant to mention
an enclosing generic parameter. Declaration-time const arguments have separate
dependency requirements and cannot borrow this later phase's validated
implementation state.

Prepare qualified and implicit trait value paths with explicit declaration work
items for their prefix, selection and written arguments, including inherent
selection and its trait fallback. A nested const argument
must not drain an unfinished parent declaration continuation. Reuse completed
path facts, and prepare the selected initializer in its own lexical and generic
context. Do not pre-scan an associated declaration's default before value
selection: an override can remove that dependency, and eager preparation can
invent a cycle. Unselected initializer templates still undergo their ordinary
body and eligibility checks. Const arguments reached after implementation validation may
use concrete value dispatch, retaining proof under their anonymous constant body.
Queue typed const-argument evaluation ahead of the consuming application. The
application reads only a completed value keyed by source identity and expected
type. If evaluation discovers an unprepared selected initializer, suspend with
that exact definition and prepare its declaration dependencies in its own scope.
Preserve expression/instance caches, active cycle states, work continuations and
the work budget across suspension. Resume the requesting anonymous body without
retyping it or reselecting its already inferred dependencies. Initializer path
readiness is separate from body type checking and completed value caching.
Reuse cleared body work vectors, the expression scan buffer and call-typing
storage across suspension. Clear caller evidence before returning that storage;
neither active inference nor a previous caller's permissions belongs in scratch.
Probe completed method-path caches only after recognizing a relevant value path.
Nested shape queries must retain declaration-cycle detection across suspension.
Do not inspect unrelated implementation bodies or restart a long query for each
dependency. Early signature and default-argument queries use the frozen-header,
declaration-only mode explicitly. Suspend the existing value continuation for
exact member-type requests, preserving typed root facts and selected-instance
readiness. Normalize only that member's type dependencies: unrelated associated
definitions can depend on its value. Do not eagerly require all method signatures
or all associated definitions to authorize one constant. Whole-impl, unused-item
and coherence checks remain mandatory before completed facts escape. Impl-header
constant dependencies still require a separate pending-candidate readiness graph;
never ignore an unprepared compatible header or publish a partial environment.

Index inherent and trait member names before early value queries; these entries
carry identity and owner heads, not executable authority. Obtain a declaration
caller's immutable environment directly from its binder owner. A whole-crate
snapshot made during an early query can miss later declarations and must not be
repeatedly rebuilt to compensate.

Trait-header member naming uses a temporary graph of decoded Self supertrait
identities, separate from proof contracts. Reject identity cycles before expanding
arguments. Index declarations by member name and prepare complete positional inputs
only along routes reaching that name. This permits `trait A: B<Self::Out>` without
demanding B's arguments while naming A's own result. Map the trait's synthetic
Self binder to its actual declaration owner for bound lookup and proof scope.
All written header predicates must be typed before publishing immutable caller
environments. Retain raw projected roots for later formation proofs after impl
declarations are available, even if normalization erases the original inputs.

Classify an owned header result promise by its complete formal parent application,
including Self and every type, const and lifetime argument. Feed that promise to
the associated result contract and impl conformance, separately from required input
predicates. A bound on an inherited result remains an explicit caller requirement.
Closed builtin-property checks normalize in the existing declaration proof scope
and retain selected lifetime obligations; structural caches grant no permissions.
Associated type names participate in qualified member lookup without shadowing
enclosing lexical aliases, including hygienic lookup.

Associated result bounds retain complete predicates, including the separate
subjects of nested result equalities. Elaborate their arguments through the
ordinary bound worklist in the owning trait's lifetime scope. Validate the bound's
own type and lifetime requirements even when no implementation or caller exists.
Conformance substitutes trait Self and mixed arguments together; an already
elaborated implementation result stays in its original scope. Share that mapping
across the implementation's result contracts and retain every resulting proof
obligation. A result contract never adds assumptions to the implementing owner.

A user-trait bound on an opaque associated result is derived only after the
original parent application has been proved. Retain that parent evidence with
the result rule, including lifetime equalities and fresh region intervals.
A derived rule records its complete selected predicate separately from caller
assumptions and implementation bodies. Recursive result bounds are queried on
demand; they do not extend the eager ordinary-supertrait closure.

Close ordinary supertraits of result clauses once in their declaration scope,
and index these templates by the requested trait identity. Substitute only those
candidates into a query's projection; retain every same-trait alternative for
inference and ambiguity checks. Mapped clauses may share a cache keyed by the
complete canonical projection and requested trait. Enable these caches only
after all result clauses and trait headers are complete; invalidate them if the
headers change. They store typed clauses, never permission for a caller. Every
query still proves its own parent application and matches every type, const and
lifetime argument. Wide result contracts must not be expanded afresh for each
independent caller scope.

Symbolic value liveness may conservatively require every projection input to
remain live. This sufficient route preserves Self, trait and family arguments,
uses the bounded component walk, and respects empty runtime aggregates. It does
not reveal exact output components or imply new input assumptions from an output
bound. Closed property queries and explicit result-outlives proof retain their
separate normalization and scoped-evidence requirements.

Type-outlives queries retain opaque associated results as typed atoms, separately
from scalar lifetime and type-parameter components. A reference to an associated
result implies a bound on that result only. It grants no bound on Self, another
input or another associated member. Keep the original binder depth on each atom;
resolve captured lifetimes in the actual proof scope, including empty binders.
An atom involving a locally bound lifetime cannot be decomposed into input
assumptions when it escapes that binder.

Prove an opaque result bound with an applicable environment clause or with all
of its input components. Input components include reference referents even when
the reference lifetime is locally bound by a function pointer. Dropping that
lifetime must not erase an unbounded type parameter or nested associated result.
Match complete typed applications; if two applications
can match only when their lifetimes are equal, retain that equality as a condition,
not an inference equation. Introduce ordinary constraints for the forced input
route when no alternative is available. Keep alternatives in a shared, bounded,
iteratively evaluated Boolean graph. Verify them after ordinary lifetime
inference and again against actual runtime extents. The same alternative must
satisfy both phases; unrelated successful branches cannot supply half a proof
each. No type verification may turn a requirement into an assumption or grant a
projection a type parameter's implicit body extent. Ordinary graphs without
projected tests allocate no verification arena.

Directed callable relations may encounter the same opaque result beneath different
actual binder scopes after local inference freshens surrounding references.
Compare complete projection inputs in those scopes, preserving nested local
binders and every trait and family argument. Equal inputs suffice for equal
results; a result comparison cannot infer input lifetime equality. Retain any
required input equality as conditional verification through both symbolic and
runtime checking. An identical resolved input needs no verification allocation.

Projection normalization uses the common bounded type fold. Normalize inputs
before asking semantic selection for a result; suspend with typed requests when
selection needs the store or declaration worklist. Replacement types retain the
request's binder context and are normalized before their parents are published.
Cache only within one immutable proof scope, and diagnose recursive results or
exhausted work. A prepared impl may supply declaration work, but its complete
conformance remains mandatory before executable facts are published. Missing
lifetime or equality evidence must never be dropped to manufacture a normal form.

A suspended normalizer has already normalized its request's inputs. Carry that
fact into selection instead of starting another normalization of the same graph.
The enclosing query retains every input proof, including obligations erased from
the resulting type. Within trait search, a derived child retains those proof
nodes itself before it can enter the completed cache: discarding its original
candidate must not discard its lifetime equations. Include earlier existential
intervals when deciding whether that result is reusable without rebasing. Visit
shared evidence nodes once. Represent accumulated input evidence with persistent
prefix links; copying every previous child into each new result creates quadratic
proof storage. Newly discovered implementation premises still need
their own normalization; an input proof never grants a new trait premise.

Before opening lifetime aliases for a normalized opaque subject, consult the
immutable canonical premise indexes. A sole identical caller or result clause
needs no equations, fresh lifetime universe or lifetime-erased cycle key. Retain
the requesting normalizer's input evidence and publish only that exact premise.
Index projected caller clauses by trait and associated declaration; preserve all
competing clauses and rebuild the index when mapping an environment. Possible
caller or result equality rules, additional candidates and nonidentical arguments
require ordinary scoped selection. This shares structural work without caching
permission across callers or skipping a required lifetime relationship.

A qualified declaration projection consumes its subject's already checked type
fact in the same annotation scope. Rewalking every previously checked prefix
turns a linear source chain into quadratic normalization work. Retain the subject's
formation evidence in that scope and normalize the newly assembled outer node.
Substituted trait defaults and implementation replacements can introduce new
projections and still require ordinary checking. A root-scope fact does not
authorize the same graph beneath a new callable binder. Deferred alias and header
formation requirements retain their existing delayed proof boundaries.

Instantiating with no type or const bindings preserves the existing canonical
type graph. Do not materialize, copy and re-intern rigid nested inputs for an
identity substitution. Actual substitutions retain their usual simultaneous
mapping and binder shifts; a later substitution cannot change the original graph.

Callable boundaries in residual types create independent normalization frames.
Requests carry their actual scope identity, parent and local lifetime count;
depth alone cannot distinguish sibling universes. Keep suspended frames in a flat
worklist, charge their combined storage and work, and preserve empty binders.
Shift caller assumptions beneath the complete ambient prefix before selection.
Retain scoped evidence immediately in the actual declaration or body allocator,
reserving fresh intervals before another query can use them. Only contiguous
intervals for the same body can share an evidence object.

Open surrounding projection lifetimes with a single checked mapping shared by
the goal and its assumptions. Preserve nested local binders and reserve existing
body inference identities before allocating selection variables. Restore a
selected result only through equality components anchored in the caller or
static; an unanchored result lifetime is an error. Shift restored captures beneath
the result's nested binders without capturing their local slots. Substitution is
not proof: retain every equality, outlives and type-outlives obligation used during
selection, including when the result no longer mentions their lifetimes.

Group declaration projection obligations by their actual binder and discharge
them against the completed owning environment. Independent annotations in the
same scope use disjoint fresh intervals and share the directed proof. Local
annotations instead retain obligations in the body's allocator and ordinary
lifetime gate; they must not append to a declaration gate that has already run.
Keep plain, closed projection queries off the lifetime-fold allocation path.
Parameter-only environments also need no binder mapping when the projection has
no surrounding lifetime occurrences. Composite and trait predicates still pass
through the shared checked substitution.

Projection selection in a signature waits for its owner's immutable parameter
environment, including requirements inherited by methods. Generic lifetime
arguments also create this dependency. Nominal defaults demand the existing
nominal contract; an ordinary alias's written bounds do not grant permissions.
Retain worklist dependencies and completion states instead of selecting against
whichever assumptions happen to have been published so far.

When a bound subject or trait argument contains a projection, elaborate its typed
identity before normalizing it, including nominal and alias wrappers.
A private, immutable snapshot of all typed bounds supplies the
assumptions for normalization; publish the final environment only after the bound
roots complete. Struct and enum contracts use that same workflow, with an explicit
typed continuation that completes the owning nominal only after its bound roots.
Do not make an unfinished nominal environment visible merely to break a cycle.
Preserve the original predicate separately from its normal form.
Its original parameter/lifetime occurrences determine declaration-time versus
instantiation-time checking and early lifetime binding. A generic projected bound
may normalize to an unsatisfied concrete requirement without rejecting an unused
generic function; actual calls still must prove that requirement. An unrelated
generic parameter does not defer a genuinely closed written bound.
Normalize the retained type argument roots before rebuilding complete trait
applications; do not reconstruct parameters from rendered output or infer a
projection's inputs from its result. Bounds inside nested callable binders and
dependencies between unresolved projected assumptions require their own scoped
normalization support.

Associated result equalities are typed predicates with two retained roots. The
right-hand side participates in parameter/const occurrence checks, lifetime
capture queries, substitution and inference with the same mapping as the
projected left-hand side. Import both graphs into a probe through one mapping;
keep the probe's variables isolated from the caller. Normalizing either root
does not discard the equality or its remaining const arguments. Result equality
cannot establish equality of a projection's inputs.

Keep equality assumptions separate from builtin abilities and trait membership.
An equality alone grants neither Sized nor Copy/Clone, nor membership in its
projection's trait. Preserve competing results in an immutable environment;
unused contradictory assumptions need not have a unique normal form. Opening
the environment must map both roots and retain the actual lifetime evidence.
An unsupported equality proof must fail explicitly, never fall through a
builtin-only proof loop or disappear when importing candidate obligations.

Preserve graph sharing across the complete predicate, including repeated large
type arguments. Discover and rewrite projected inputs with one mapping per query;
resolve all predicate roots with one read-only traversal cache. Reuse successful
complete input equations only within the same relation and enclosing binder
scope. Compatible later equations preserve those equalities; a different root
pair still requires proof. Keep original projection roots and independent holes,
charge each input, and retain lifetime obligations. Sharing must not turn equal
projected results into evidence that their inputs are equal.

Ordinary aliases and generic defaults retain conditional projection requirements
as sealed typed dependency graphs. Instantiate a contract only at an actual use;
an overridden default contributes no requirements. Retain dependencies even when
an enclosing projection erases their result. Share identical instantiated uses
within the actual declaration binder or body, never across proof scopes, and
bound graph traversal and argument work. Freshen each template's private region
interval before substituting caller arguments; numeric overlap must not turn a
caller lifetime into a template variable. A failed expansion invalidates its scope.

Distinguish an unproved deferred projection from a proved symbolic result. An
ordinary alias may retain unproved type formation, including a closed projection,
until use. Dependent generic defaults similarly retain their requirements; closed
nominal and trait defaults still require declaration-time proof. Ordinary alias
bounds supply no assumptions. Its bare parameter scope identifies type and const
parameters without granting its written predicates. Declaration selection may use
registered derive contracts before fields finish, but final derive validation is
mandatory. Check retained type formation even if an outer alias erases the result
and the lifetime proof is empty; completing that check does not overwrite the
actual annotation's type.

Retain the implicit Sized premises of selected implementations and their recursive
conditions when alias normalization removes the original inputs. Alias parameters
can later receive unsized arguments despite their declaration spelling. Substitute
these premises at each use and check them in the actual caller environment. Track
their actual callable scopes across dependency graphs, separately from the local
substitution depth. Derive the total binder depth from that scope tree. Partial
annotations with only liftable Sized premises share the
result's inference variables and retain those premises through body completion;
they do not infer erased inputs from an associated result.

Nested callable annotations retain their enclosing alias/default proof owner and
an explicit universal scope tree. Complete each binder's slot count before sealing
the contract. Cache source scopes as they are discovered; captured-lifetime lookup
uses packed ancestor jumps rather than repeated parent walks. Instantiate each
private witness in its actual universe, keeping sibling scopes distinct and caller
inference variables outside younger binders. Empty binders preserve coordinates
and consume work without introducing new universal lifetimes.

Retain quantified evidence through body collection, specialization and selected
item rewriting. Substitute caller parameters beneath each block's local binder
prefix, and prove the resulting requirements in the same graph as ordinary caller
constraints. Scope IDs are local to one evidence object and must be remapped when
combining independent objects. Partial alias annotations retain deferred projections
and lifetime proofs until their actual inputs are determined. Residual projected
types and erased formation requirements normalize in their retained callable
scopes. Share an immutable binder plan with pending queries so moving or remapping
the original proof cannot change their scope identities. Import each required
scope prefix once per destination evidence object; never clone the full tree for
every requirement or drop its evidence to produce an executable type.

Body projection queries also observe partial input changes. A unique validated
implementation may determine type or const inputs from its header before numeric
fallback; an associated result cannot choose between competing headers. Watch
only the query's dependency leaves, coalesce queued notifications, and cancel
completed queries. A change notification is neither canonical readiness nor proof.
Keep canonical notifications in dependency-completion order. Deliver partial
notifications only after pending dependency updates, so a resolved input uses
the ordinary normalizer without a redundant partial query.
An entirely unknown Self cannot select a caller assumption or implementation;
wait for a known head or numeric domain before allocating query lifetime aliases
or opening its graph. A deferred attempt must not change source provenance.

Demand resolution may reuse scratch capacity within its inference body, but never
reuse resolved results across separate requests: equations and projection
publication can change the graph. Clear active markers and pending work after
failed traversals as well as successful ones. Bound retained hash-table capacity
relative to the current traversal before clearing it, so one large query cannot
make every later small query scan the same large allocation. Known-type queries
keep their allocation-free path.

Body calls normalize substituted `Copy`, `Clone`, `Sized` and type-outlives
predicate subjects through the
same deferred worklist before querying builtin properties. A property of the
selected result cannot choose its inputs or erase the original application's
formation and lifetime obligations. Composite subjects retain their own nested
scopes; ordinary known types and unresolved root variables need no traversal
allocation when they contain no present projection.

A full trait-proof retry can wait for input resolution when one candidate's
matched header preserves every input hole and its remaining premises are only
builtins. A single independent type parameter beneath already-known reference
or pointer heads, with no trait arguments, gives this certificate directly from
the checked header. Other layouts use an isolated probe: binding, domain
narrowing or merging original holes invalidates the certificate. Competing
headers, applicable caller assumptions and recursive trait premises retain
partial retries. The certificate only controls
scheduling; canonical selection must still prove the complete implementation.
Do not copy and probe a wide unchanged input graph after each leaf assignment.
Check scheduling before opening the full query lifetime environment, and do not
export a result graph merely to establish input preservation.
Symbolic result transport is a separate operation and may progress before that
complete proof; it must retain the original input and formation obligations.

Before exposing a symbolic result, count every compatible validated header,
including candidates whose equations would refine existing inputs. Result types
cannot remove competing headers. For a unique compatible header, instantiate
its associated definition with the same type/const bindings as its header and
export only an input-preserving graph. Scoped variable equations intersect numeric
domains in either argument order. They must not reject a numeric hole merely
because the implementation parameter is still unconstrained.
For a sole indexed candidate with one independent type parameter, exact known
reference/pointer heads can supply that parameter and its lifetime captures
directly. Repeated captures must agree exactly, and every capture must be supplied;
other cases retain the isolated probe. This shared header match supplies no premise
proof. Result substitution reuses the original input term and shifts captured
lifetimes beneath any result binders without copying the unchanged input graph.

Publication keeps an independent application root sharing the original input
identities, while redirecting the result root and waking its blocked equations.
The original dependency ticket and proof scope survive publication. Completing
the result does not cancel the input query: ordinary selection must still prove
its implementation and formation requirements, and its final normal form must
agree with the published result. Reserve and retain header lifetime equations
before exported types refer to their private identities. Premises remain pending,
even when the result erases all type or const inputs. This does not impose a
global requirement to solve unused const holes: ordinary aliases may erase them.

Partial alias applications use their retained unreduced definition so declaration
normalization cannot erase source-time inference or coercion context. An unresolved
projected expectation supplies no reference, array-unsizing or callable hint.
Ordinary result equalities may wait for normalization; implicit conversions and
reborrows cannot be introduced retrospectively when later inputs become known.

Freshening a stored function item reconstructs its signature from the immutable
callable template. Register newly constructed projection occurrences even when
no new lifetime was allocated; an earlier occurrence's proof does not authorize
them. Retain this work as a typed result of freshening, and register the whole
result in its enclosing scope so nested callable binders remain intact. Detect
projected templates and arguments during the existing traversal. Ordinary
callables must not allocate a normalization worklist merely because their
signatures were rebuilt. New normalization duties never discharge the original
parent application, implementation premises or caller lifetime obligations.

When a header requires a nested projection's normal form, register the nested
query at its actual callable scope and subscribe the enclosing application to
that result. Publication wakes only those dependents; it does not discharge the
nested query's original input proof. Recheck an enclosing application's current
inputs after that wakeup: an inner result may have erased holes still retained
by the inner proof. Cache input callable scopes by parent and inference-term
identity so retries preserve empty frames and never allocate replacement binders.

Reuse completed canonical body normal forms within the body's immutable caller
environment when the root has no ambient or free lifetimes and selection leaves
no lifetime obligations. Cache only after the complete selector succeeds; partial
results and failures supply no entry. Queries with scoped evidence retain their
ordinary proof path and private region intervals. Ordinary types bypass the
normalization worklist.

Run partial selection in an isolated input graph. Share one opening map with the
caller environment shifted beneath the actual ancestor binders. Restore the
selected input predicate before committing equations to the body: ambient slots
are rigid, nested callable slots may alpha-rename, and an outer inference hole
cannot capture a younger universal. Retain all selection requirements at the
original scope before installing the normalized result. If another query has
already normalized the same root, cancel its redundant notification without
selecting or publishing the evidence a second time.

Cache the body's projection environment with caller universals expressed as their
existing body aliases. Declaration predicates and their aliased copies describe
the same permissions; they must not become competing assumptions after opening.
Use only the established universal-to-alias mapping here, not arbitrary pending
region equations. Keep distinct caller lifetimes distinct and retain the ordinary
lifetime proof of any equality required by a selected implementation.

Opening a fully determined projected callable can introduce caller aliases
without a generic call or pending projection. Record that event in the body's
cleared call workspace and compare shape-equation operands in that same
representation, reusing the existing substitution cache. Ordinary nonprojected
calls need no new work. Otherwise an opaque projected argument may retain
bound slots while its callable parameter uses equivalent aliases, making valid
code depend on an unrelated earlier generic call. Preserve the written return
contract and ordinary lifetime proof; this alignment grants no equality between
different projection inputs or unrelated lifetimes.

Check constraining lifetime positions before erasing projections through
normalization. A lifetime occurring only inside a projected function-pointer
input does not constrain the pointer's output, even when a selected impl would
normalize that input to a reference using the lifetime. Preserve the unreduced
typed signature for this check, including projections introduced through aliases;
normal forms alone cannot establish this declaration constraint. Opaque input
positions still require valid binder slots.

Use original occurrences when detecting unconstrained impl lifetimes and when
classifying a function lifetime as early or late: projected inputs are
nonconstraining and original output occurrences remain significant after
normalization. Late instantiation then supplies the slots actually used by the
normalized executable signature. These are distinct queries over retained facts.

Retain unreduced annotation, alias and argument identities in sparse records
allocated only when projections occur; ordinary node facts keep their normal
types. Propagate those identities through completed typed children and cached
parameter/lifetime substitutions. Default arguments use their own append-only
unreduced prefix, so a preceding projected argument is not silently replaced by
its normal form in later defaults. Alias parameter-occurrence checks consume the
unreduced definition. Executable shapes, inference and lowering continue to use
normal forms, without reversing projected results into argument equations.

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

Trait-qualified callable uses keep the trait prefix's arguments separate from the
method's arguments. Infer inherited Self and omitted arguments through the ordinary
callable graph; explicit prefixes and ordered defaults constrain those same slots.
Generic bodies retain the trait declaration as a symbolic callable and prove its
complete requirements without guessing an implementation for a rigid parameter.

A type argument can introduce projections into a signature whose template contains
none. Record those occurrences during the existing argument binder-shift traversal
and register the instantiated signature for scoped normalization. Unknown variables
retain their dependency identities; the traversal summary is scheduling information,
never proof that a projected result exists. Ordinary generic calls keep their path
without a second graph scan.

Fully qualified `<T as Trait>::method` paths consume the resolver's retained Self
annotation, trait identity and canonical member name. Select a direct method from
the trait's existing member scope; inherited supertrait members require their own
trait qualifier. Bind the written Self annotation into the same callable inference
graph, preserving annotation holes, aliases and lifetime scopes. The trait prefix
has type-path arity and default rules: omitted required arguments fail, and defaults
apply even without explicit arguments. Reuse ordinary argument preparation and
dispatch after this qualification; no additional source parsing or impl search is
needed for selecting the named declaration.

Implicit `T::method`, `Self::method` and `<T>::method` lookup also retains and
binds the written Self annotation. Applicable inherent methods take precedence;
otherwise probe methods from parameter bounds before methods from lexical trait
bindings. These probes consume only the prefix and caller environment. Arguments,
return expectations and the method's own bounds do not disambiguate the name.
An implicit trait prefix infers trait arguments without applying type-path defaults.

Build trait visibility from final resolver bindings, including anonymous imports
and the retained candidate of an ambiguous glob import. Deduplicate aliases by
trait identity. Lazily index direct methods by name in each queried scope and
share inherited candidate lists across scopes that introduce no matching methods.
Index parameter-to-trait bounds only when a caller needs implicit lookup; preserve
the complete goals for applicability. Unimported homonyms must not multiply work
at each use. Bound indexing, scope traversal and candidate probes have explicit
work limits. Enum variants retain their existing lookup and constant-evaluation
paths before method fallback.

Before proving and lowering a concrete body, select its trait operands through the
validated impl registry. Map each declaration to its override or checked default
using the sorted member table. Keep the selected impl captures separate from the
method's own early lifetimes and type/const arguments. Normalize body universals
into fresh aliases for selection, retaining every equality, outlives requirement
and allocated variable in that body's proof graph.

Selected impl-method signatures can introduce projections after the original
trait operand was normalized. Normalize the selected function item before caching
or rewriting body operands, using the dispatcher's existing concrete proof and
region allocator. Preserve nested callable scopes and every formation obligation,
including those erased by the resulting type. A selected target with no projections
finishes directly; repeated uses share its checked target and retained evidence.

Replace actual function-item types and operand definitions throughout the concrete
CFG and its contract, including callable types inside generic arguments. A shared
type fold preserves nested binders and memoizes the type DAG. Promotion and demand
then consume these selected identities. Initializer selections also supply the
constant value reader's callable targets, so named constants and statics cannot
retain a declaration-only symbol. Cache dispatch only within its body proof scope;
generic templates and their source facts remain reusable by other instances.

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
parameters. A blocked opaque equation against one caller assumption does not
invalidate an independently equatable assumption. Continue probing alternatives
in isolated contexts; two equatable assumptions still defer selection. If no
assumption matches, retain the normalization requirement instead of reporting
absence or permitting an unrelated implementation. This does not normalize the
caller environment or infer a projection's inputs from its output. Unresolved
candidate inputs defer the query without committing any alternative. Prefix
applicability may remain possible until ordinary call argument/result constraints
settle; a completed body still needs a unique proof. Retrying the same goal after
inference must use its new inputs, not a cached provisional decision. Retain
inferred lifetime equalities with their
owning declaration or
body; prove both directions using the ordinary region engine and completed nominal
contracts. An outlives relation in one direction alone is not equality. Keep exact
assumption lookups cheap and build broader indexes only when needed. Drop coverage
folds the complete predicate into nominal parameter order. Map resulting lifetime
equations back to the original impl slots and prove them in the destructor coverage
graph, where target lifetimes are universal and extra impl lifetimes are existential.
Extra slots must not strengthen the nominal contract. Apply builtin property
queries after trait equations fix their inference inputs. Method-own bounds remain
checks on the selected callable. Prove disjoint target shapes before requiring trait
coherence; missing local implementations do not establish negative evidence.

Goal/header equations may contain inference variables on both sides. Share the
scoped callable relation with canonical assumption matching, preserving local
lifetime correlations while allowing alpha renaming and unused binder slots.
A variable on either side cannot capture a surrounding callable's local lifetime;
outer lifetimes are rebased when entering or leaving a callable. Equalities of
those outer lifetimes remain obligations, including explicit trait arguments.
Lifetime-erased header overlap equations establish possible compatibility only.

Body trait goals and inherent-method conditions consume the validated impl registry.
Reuse its complete closed-application and outer-head partitions; probe each possible
header in an isolated inference graph. Conditions are obligations under the caller's
environment, never additional caller assumptions. Traverse recursive trait requests
with an explicit worklist and a total work budget; an active cycle is not evidence.
Cache completed requests that allocate no new lifetime identities, and collect a
shared proof graph only once. Reusing evidence with new existential lifetimes requires
fresh rebasing before it can enter a different proof.

Memoize a fully resolved failed trait request only when its whole search completed
without an inductive cycle cutoff. A cutoff depends on the active proof stack;
neither it nor its failed ancestors is a reusable negative answer. Keep deferred
inference and exhausted-work errors separate from absence. The memo belongs to
one immutable caller environment, declaration/validated query mode and region
scope; its keys preserve complete trait arguments and binder identities. It must
not survive a change of assumptions or implementation availability.

Each candidate allocates lifetime captures from an isolated view of the caller's
next free identity. Only a selected proof publishes those variables, equalities,
outlives and type-outlives obligations into the ordinary region checker. Retain the
selected impl owner and its typed arguments at the body use. Failed alternatives
must not publish variables or constrain the caller; possible applicability alone
still cannot authorize an operation in a completed body.

Dot calls retain the resolved receiver, canonical method name, lexical scope and
written arguments. Type the receiver before selection. For each built-in reference
dereference level, probe the value, shared borrow and mutable borrow in that order.
Reuse the inherent candidate partition and isolated applicability equations;
discard impossible receiver depths before importing an inference graph. Receiver
mutability, moves and loan legality remain obligations of the selected call.

At each candidate receiver, inherent methods precede parameter-bound methods,
which precede lexical trait methods. Bound priority constrains Self to the parameter
that supplies the bound. Retain that origin for both dot calls and implicit type
paths. The selected callable's exact implicit `Self: Trait<parameters>` predicate
must use caller assumptions; matching only the trait name could incorrectly
replace its arguments with an unrelated blanket implementation. Multiple stated
applications can still be resolved by ordinary argument inference. Other predicates
of the selected method keep normal implementation proof, and an explicitly written
trait qualifier keeps normal trait selection. The same trait declaration may remain
a lexical candidate for another Self, such as a reference to that parameter. Peel the retained receiver
reference shape and consult existing assumption/implementation head indexes before
copying a probe graph. These filters can exclude impossible candidates; positive
matches still require complete equations and validated implementation evidence.
Integer and floating-point receiver variables keep their numeric domains and the
ordinary inference/fallback rules. A completely unknown Self is not guessed.

After the final built-in dereference, consider an array's slice type and its shared
and mutable borrows. Keep the actual place and borrow typed as an array; publish
the array-to-slice conversion explicitly for the region proof and body lowering.
Trait Self is inferred through the selected receiver signature, including when
Self is a reference. Only inherent calls bind the nominal impl target separately.
The hidden receiver argument belongs to every typed method-call node, regardless
of whether it has inherent-owner metadata.

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
separate evidence. Retain the decoded subject classification in each completed
header: an alias that reduces to Self remains a caller requirement, even though
its canonical type is identical. Result-rule closure and builtin/lifetime promise
queries must use that same distinction. Diagnose supertrait cycles by declaration
identity before expansion, and deduplicate diamond paths under a bounded worklist. Arbitrary trait
names confer no builtin Copy, Clone or Sized behavior.

Trait impl headers share generic/lifetime elaboration with inherent and builtin
Drop owners. Retain the complete trait predicate, owner and source once. Every
owned type/const parameter must constrain Self, the trait application, or an
associated result whose complete inputs are already determined. An ordinary
where bound alone does not constrain it. Validate target and argument well-formedness,
trait requirements and safety under the impl's own environment. Input types imply
lifetime well-formedness facts, never new Copy or user-trait assumptions.

Prepare raw typed impl bounds before normalizing their projected annotations, as
for function bounds. An opaque projection can finish this declaration pass after
its formation proof; retaining the projection must not reschedule the completed
bound environment. Structural shape checking requires its exact Self and trait
application in the immutable environment or a separately retained normalization
proof for the complete root. Keep those structural query caches separate: a
cached shape cannot authorize a raw query without parent evidence. A complete
shape under the caller's exact assumptions finishes directly; only a missing
proof or an insufficient structural property demands scoped normalization. Check
every type input, and use the associated schema, completed conditional declaration
promises or an explicit Sized assumption for the result.
Projection input well-formedness retains the surrounding callable depth. Input
lifetime requirements are not the opaque result's outlives or live-value
components; result properties still need their own proofs.

Associated result contracts do not expand recursively into caller environments.
Close ordinary Self supertraits first, then their immediate owned result clauses
and those clauses' ordinary implications. Retain their finite equality, builtin
and lifetime facts, while user-trait result applications use demand proofs instead
of additional caller assumptions. Further associated results are queried at the
finite projection demanded by the program. Recursive declarations such as
`trait A { type Out: A; }` and their equivalent owned header clauses must terminate
without raising work limits or withholding promises at deeper demanded results.

Bindings on an owned result, such as `Self::Out: Other<Value = X>`, belong to
that result contract, including bindings completed after header member lookup.
Retain the complete projected equality subject and both sets of trait arguments.
Index direct result equalities and ordinary Self-supertrait equalities by member;
map a demanded candidate set once per complete result application. A cached rule
is declaration data, not permission: retain the parent proof before applying it.
Select that bucket from the outer projection member before resolving or
materializing its input graph. An absent bucket needs no input traversal,
substitution or allocated empty rule list. Metadata lookup leaves unresolved
inputs and normalization permissions untouched; later publication can change
which outer member, if any, the term denotes.

Equality candidates match complete parent inputs in isolated probes. Identical
rules from explicit and derived evidence are harmless; conflicting complete
matches remain ambiguous. A blocked equation involving an opaque input supplies
no input equality and must not hide a different exact match. Without a complete
match, keep the projection opaque; this is not negative trait evidence. Never
infer equality of Self or its arguments from equality of associated outputs.

Projected arguments in associated member bounds retain their written identities
until the complete result contracts exist. Reuse the header formation worklist
and original binder to discharge those requirements afterward. Keep this proof
separate from result equations, including unused declarations and recursively
related results; an equation cannot manufacture a missing parent implementation.

Keep immutable canonical projection schemas separate from properties learned
while typing complete headers. Share the completed conditional Copy/Clone/Sized
table across immutable environments; it supplies no parent formation evidence.
Both declaration and body property checks retain their scoped normalization debts
before using a derived opaque result's structural properties.

Decode member-written and owned-header result outlives bounds through the ordinary
lifetime binder. Close only the result's ordinary Self supertraits when preparing
its finite lifetime templates. A scoped outlives query maps the declared regions
through the actual parent arguments and adds verification alternatives for that
opaque result. These alternatives never become bounds on Self or trait inputs.
Concrete normal forms retain their independent formation debts, so a conditional
implementation cannot use its output promise to prove its own prerequisites.
Unused implementations must establish every promised result lifetime under their
actual bounds; unconstrained impl lifetimes remain rejected before normalization
can erase their occurrences. Preserve the separate sufficient proof through live
input components and all existing projection-identity and binder-scope guards.

Determine impl parameters from unreduced typed occurrences. Seed injective
positions in Self and the trait application. An associated equality contributes
a directed dependency from every type, const and lifetime input to the injective
positions of its result; projected result inputs are non-injective and cannot be
inferred backwards. Exclude an equality on the implementation's own complete
trait predicate. A worklist wakes only dependents of newly determined parameters,
so declaration order is irrelevant and unseeded cycles remain unconstrained.
Use the same analysis for inherent and trait implementations, after completing
their bound environments. Skip dependency-graph construction when the header
already constrains every owned type/const parameter. This declaration check
determines parameter identity; actual selection must still prove every equality
and carry its type and lifetime evidence through the ordinary body gates.

Keep prepared headers separate from validated implementations. Declaration proofs
may use prepared headers as conditional rules, through an explicit declaration-only
selection entry point. This permits closed bounds, forward supertraits and cyclic
declaration well-formedness without depending on source order. Every header still
passes coherence, contract and member validation before body analysis receives the
facts; every executable body is checked before lowering. An active cycle of impl
conditions remains insufficient evidence. Select using the original declared bounds,
not the additional supertrait assumptions derived from those bounds, which can
otherwise manufacture circular conditions. Preserve their prefix before elaboration.

Normalize a declaration's captured lifetime binder to fresh free aliases before
inference, including the same substitution in its immutable assumption environment.
Nested callable binders remain local. Reserve disjoint lifetime intervals for proofs
contributing to the same owner and after existing method-conformance existentials.
Retain alias equations, selected impl equations, outlives and type-outlives obligations
together. Closed bounds use no caller assumptions and cannot justify themselves.
Fields, parameter defaults, signatures and initializer types all retain their proofs.
Drop coverage maps the complete proof back from nominal to impl parameter coordinates,
while unused impl lifetimes remain existential. Body checks that allocate aliases
after inference completion publish their equations under that same body's identity.

Member-bearing implementations require matching associated declarations and signature
conformance. Retain overrides separately
from default declaration identities; executable demand must dispatch through the
selected implementation. Index overlap candidates by trait and
rigid subject, then by complete lifetime-erased application for closed headers.
Share existential bindings across Self and every type/const argument when probing
compatibility. Lifetime erasure may establish structural disjointness, never body
permissions. Charge candidate-pair storage before allocation and equation work
during probes; predicate-sensitive coherence remains a separate proof.

Method conformance checks unused implementations too. Match direct members by
owner and name, require every nondefault method, and reject extra members. The
trait method's type/const parameters are rigid proof inputs; using the impl's
parameter identities would assume a stronger implicit Sized requirement. Prove
the impl method's requirements under the impl owner and substituted trait method
contract, while checking its body under its own declared assumptions.

Construct that conformance environment from the actual impl's bounds and the
trait method's own predicates. The impl-header check already proves the containing
trait's requirements. Reintroducing those inherited templates as substituted
caller assumptions can hide concrete supertrait associated definitions. Preserve
explicit method predicates, even when they repeat a parent bound: generic and
lifetime-dependent applications can remain opaque under those predicates.

Validate associated definitions before using their normal forms in method
conformance. Keep a local index of those checked definitions, shared by the impl's
methods. A local result applies only to its exact declaring item, Self and complete
trait application, shifted into the request's callable scope. This index does not
publish a validated implementation; all method checks still precede that publication.

Normalize each component of the already-opened conformance signature without
introducing another outer function binder. Share completed normalization work
within one fixed environment, proof scope, implementation availability and local
definition provider. Retain nested binder mappings and every ordinary selection
obligation in that session's proof. Repeated parameters must not rewalk the same
captured impl receiver graph. Body callable instantiation also schedules projection
normalization when the signature itself contains projections, including when this
is the first projection work in the body.

Match early lifetime arguments positionally. Instantiate impl late lifetimes
existentially against universal trait late lifetimes; unused late declarations
do not alter the contract. Share capture-safe substitution, shape equality and
directed lifetime relations. Preserve nested callable binders. Trait input
well-formedness supplies implied lifetime facts for contravariant parameter and
covariant result checks; structural types, receiver presence and safety still
must agree. This permits weaker input lifetimes and stronger results without
granting unproved bounds to the implementation body.

Trait argument well-formedness and declared requirements are checked even for
unused declarations. Lifetime requirements consume the completed nominal contracts
and shared directed relation engine. Assumptions from the original signature and
explicit bounds remain distinct from obligations imposed by a trait application.
Predicate inference shares the ordinary type/const equivalence classes, and
completion rejects unresolved arguments that appear only in an obligation.

Inferred associated applications retain Self and every trait/family input in
the type graph, including shared const variables and the argument-kind boundary.
Parameter substitution, binder mapping and isolated imports preserve these
identities. A projection remains a noninjective operation: its result cannot
equate its inputs or inherit their layout, Copy or lifetime properties.
Freshening its inputs is not a substitute for freshening a selected result.

An isolated inference query may transport symbolic result structure back only
while preserving every original type and const hole. Refuse input bindings,
domain narrowing, merged distinct inputs and escaping fresh query holes before
allocating destination terms. Reuse original variable identities through one
shared reverse mapping, with bounded traversal of shared structure. Lifetime
equations and semantic proofs stay with the caller; transporting a result does
not authorize implementation selection or discharge a formation requirement.

Canonical-input readiness uses dependency watches on unresolved type and const
leaves. Unions, bindings and explicit fallback wake only affected watches; newly
exposed compound terms add their own leaves without rescanning resolved ancestors.
Deliver each readiness event once and charge the shared inference work budget.
Watches retain no proof authority: the semantic caller still owns source identity,
binder scope, normalization, evidence and suspended equations. Collect dependencies
from every retained proof, normalization and Sized root, including erased inputs.
Wait only on the parameters those roots use: waiting on an unrelated output or a
dependent default can prevent an otherwise solvable query from making progress.
Schedule individual projected occurrences rather than whole output aggregates.
Traverse shared ordinary structure once per scope, preserve callable binder counts,
and leave a projection's input normalization to that query. An input also used
outside the projection has its own occurrence. Deduplicate an identical projected
term in the same scope without discarding its separate formation contracts.
Lifetime and parameter substitution must register the projected terms they create,
including projections supplied through arguments or defaults when the original
outer template contains none. A copied graph cannot inherit a finished ticket
from its original term.

Deferred equality and shape equations constrain ordinary siblings while retaining
blocked associated-result leaves. Never bind an output hole to a projection merely
to avoid suspension, and never infer projection inputs from an output equation.
Preserve the original relation kind and resume only awakened leaves; unchanged
siblings must not repeat work or lifetime equalities. Completion rejects remaining
equation or formation debt, including when a normalized result erased its inputs.

A proved opaque associated result has an inference-term identity distinct from
an unresolved projection operation. Retain each occurrence's original application
and scope even when results share a canonical type. Determined inputs remain
immutable; unresolved inputs retain their actual inference variables. Publishing
an unchanged opaque result must still wake suspended equations. Equal opaque
results never imply equations between distinct projection inputs. Region mapping
creates new formation obligations; value-lifetime freshening does not rewrite an
opaque operation's inputs as if they were exposed reference fields.

Concrete body instantiation normalizes substituted associated results before
layout, cleanup and lowering. Compare the substituted callable signature with
the selected signature after normalization, and retain implementation lifetime
debts in the instance contract. Map caller lifetimes using the same substitution
context as parameter arguments. The body's signature binder is its existing
universal scope; callable types nested inside it introduce their own binders.
Do not copy an existing proof's quantified ancestors to a sibling proof and reuse
its private variables there. Extensions of such scoped projection debts remain
rejected until their original scope and allocation ownership can be preserved.

Body annotations retain source callable scopes before descending into their typed
children. Alias-local binders extend that ambient prefix. Rebase proof-private
identities before substituting actual arguments, and preserve both prefixes when
aliasing caller universals. A fresh query copies only its ancestor chain, not every
previous sibling scope. Scoped proof publication preserves gaps in private region
intervals and the ownership of nested witnesses. Synchronous body normalization
reserves intervals in its exclusive local proof before publishing the contiguous
result once; it uses validated implementations and the existing lifetime gate.

Projection and trait progress run before numeric fallback and body publication.
Normalization never substitutes for the original well-formedness proof. Isolated
method-prefix queries still reject pending projection proofs until that query mode
supports them; body proofs cannot leak into a speculative candidate. General
opaque-result properties, quantified premises in implementation search,
associated constants and further PA20/PA21 work remain required.

Associated bindings in trait bounds retain resolved member identities separately
from positional arguments. Complete ordinary trait headers before resolving
header bindings through the supertrait graph, then publish all equality predicates
before freezing environments. Discard declaration-lookup application caches at
that transition. A member declared directly by the named trait takes precedence;
otherwise every inherited declaration participates in ambiguity. Preserve trait
arguments, defaults, raw identifier identity and the original projected roots.
Equalities on Self's associated types are implied by the trait's contract; a
requirement on another type parameter is not automatically implied.

Unused generic equality assumptions may disagree, including with a result's
declared Sized or Copy bound. Retain them without choosing a result. A requested
normal form needs a unique matching equality and a separate trait-premise proof.
Match complete input equations in isolated contexts; commit only the selected
equations and retain their lifetime obligations. Never equate rigid input or
result parameters merely because competing assumptions mention the same output.
Closed declaration equalities need independent implementation proofs without
using themselves as assumptions. Their normalization owns an explicit declaration
proof scope. Body equalities use the actual body's allocator and lifetime gate.

After proving a projection's trait premise, explicit equality assumptions still
take precedence. If no equality supplies a result, a completely concrete application
may use an independently selected impl despite a repeated caller trait bound.
Require the entire Self/trait application to contain no type or const parameters,
external lifetime variables or unresolved projections before this fallback. Static
lifetimes and locally bound callable lifetimes can be concrete. Cache this
distinction separately from the existing substitution flags, which also count
static lifetimes as free. Select without caller assumptions, respect
declaration/body implementation availability,
and retain all selection evidence and result-region restoration. This rule does
not remove explicit generic bounds or infer projection inputs from a result.

Instantiate both sides of a call's equality in the shared inference graph.
Schedule their normalizations and a suspended equality before inference finishes;
normalization may determine an output hole without guessing a projection's inputs.
The result has its own well-formedness and lifetime requirements. Its separately
retained trait premise supplies projected-subject formation. A proved opaque
normalizer response does not bypass any remaining shape, lifetime, ownership or
execution gate.

Index equality assumptions by associated declaration, then distinguish their full
inputs within that bucket. Remapping an environment preserves declaration order,
deduplicates newly identical assumptions and rebuilds its index. Cache whether
outer-region mapping may be necessary, including result-only captures. An
environment certified to contain no such regions can use canonical inputs
directly; unrelated equalities must not be cloned for each wide-list projection.
The type store's region summary remains conservative for static lifetimes.

Implementation selection treats associated equalities as proof work on its existing
iterative stack. A normalizer suspends at each trait premise and resumes from that
selected implementation or caller assumption; it must not start a fresh recursive
solver with independent cycle/cache state. Preserve the original projected query
and commit its complete input equations separately from the normalized result.
Result equations can determine output holes, never unrelated projection inputs.
Undetermined inputs defer until other premises provide enough information.

The same stack normalizes projected subjects and type arguments of trait premises,
and projected builtin conditions after other equations settle. Retain the original
input graph beside the selected normal-form view. Replace only independently
proved projection roots; keep ordinary type/const holes shared, including holes
beside a projection inside a tuple or nominal argument. A normalized result must
never infer its projection's inputs. Exact canonical operation identity may prove
congruence without exposing an opaque result's structure or properties.
Preserve a typed normalization-needed result from the early assumption lookup,
normalize the query and retry against the caller's concrete assumptions. A still
incomparable opaque operation defers; it cannot establish cached absence.

An input wrapper retains the original query, every normalization child and the
selected implementation or assumption. Allocate implementation-private regions
after normalization children, and preserve their complete interval through the
parent proof. Cache the original canonical predicate, not merely the normalized
result; partial queries become canonical only after independent holes settle.
Builtin conditions with ordinary subjects keep their direct proof path. Suspend
only projected ones on the shared stack, retaining pending Sized and outlives
requirements for the caller's existing gates. General opaque outlives properties
and original callable formation contracts remain required; normalizing a premise
does not complete those separate declaration and lifetime responsibilities.

Equality memo keys retain both canonical roots in the immutable caller environment.
Cycle-dependent failures remain uncached, and evidence with private lifetime
allocations cannot be reused without fresh identities. Selected result lifetimes
stay in their child proof intervals until the enclosing equality and caller's
lifetime gate prove them. Receiver applicability may discover a candidate with
pending requirements; the ordinary callable still proves all selected bounds.
Normalization beneath callable binders in implementation equality premises still
requires scoped selection evidence; the current explicit boundary must remain
until that proof representation is integrated.

Apply each tuple field's coercion before checking the next field. Context inferred
from an earlier field can establish a later mutable-reference reborrow, including
through a block tail or nested tuple. A reference consumed before that context
exists remains a move; independent type holes do not acquire a shared context.
Array elements retain their common-type join before final element coercions.

Caller-bound trait goals participate in the same inference graph as call arguments.
Import each probe with one shared mapping across Self and all type/const roots.
Index assumptions by trait identity and the rigid outer subject shape; do not scan
unrelated parameter subjects. Deduplicate identical assumptions before probing.
Failed alternatives never constrain the caller. Commit a unique match and retain
its actual assumption identity and every required lifetime equality. The ordinary
region checker proves those equalities; matching never erases lifetime obligations.

Function-pointer types inside trait applications use scoped alpha equality, not
function subtyping. Pair each callable's local lifetime slots bijectively as they
are used; renaming, reordering and unused declarations do not change equality.
Keep nested binder frames distinct, including callables with no local slots.
Only outer lifetime equalities may reach the caller's region proof, lowered by
the number of enclosing callable binders. An inference variable cannot capture
one of those local slots. Share ordinary shape/const equations and region mapping,
and cache rebasing for repeated variable uses. Canonical dependency flags keep
ordinary trait arguments on the existing equality path without scanning types.

An unconstrained whole Self is not inferred merely from an available caller assumption
or impl. Numeric variables retain their integer or floating-point domain: search
only compatible scalar heads, and infer a unique matching type when possible.
Apply numeric defaults only after ordinary equations and trait progress stop,
then retry the remaining goals. Candidate probes never apply numeric defaults.
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

Template loan checking consumes complete scoped destruction requirements. When
no local, value or projection can need destruction and no cleanup sites remain,
reuse the checked initialization and storage facts with the ordinary loan checker.
Do not allocate an abstract cleanup plan for that case. Custom-Drop field move
legality, region proofs and initialization checks remain independent requirements.

When cleanup candidates do remain, inspect their proved initialization before
demanding recursive destruction properties. A definitely moved root or sparse
child has no cleanup access. Keep conditional and live paths on the ordinary
type, destructor and lifetime checks; this ordering cannot replace their earlier
typing, move legality or initialization proof.

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

The concrete instance scheduler also retains one unscoped destruction-property
cache across its demanded bodies. Full canonical types and completed normalized
fields determine these answers under one immutable custom-Drop registry. Each
body must still normalize its storage and retain its own formation debts before
consulting that cache. Independent symbolic environments keep separate queries,
and manual instantiation entrypoints start their own concrete cache. Sharing
structural answers never transfers a caller's lifetime or trait permission.

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

Collect initial runtime lifetime seeds into one queued entry per region and
point word before propagating them. Seed collection already precedes propagation;
combine its bits in first-seen order instead of walking shared static or external
obligations separately for every point. Subsequent propagation retains only new
deltas. Keep the sparse result representation, local-sentinel exclusion and all
work/storage limits; charge both seed collection and completion of queued words.

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

An equality-normalized field retains its original declaration dependencies for
variance. Preserve sparse original field lists by nominal and variant, with the
same arity and ordering as storage fields. Original projection inputs remain
invariant even if their result becomes a scalar; bounds alone cannot manufacture
field occurrences. Multiple declarations may share a normalized type without
sharing their original inputs. Layout and value operations consume the normalized
storage fields. Ordinary variants allocate no retained field lists, and inference
reads the original lists through the same bounded variance graph without field
expansion or mutation of canonical types.

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

Keep raw substituted fields distinct from normalized concrete storage fields.
Normalize concrete fields through independently validated implementations before
ownership, destruction or layout queries can observe them. An unresolved concrete
field is a preparation error, not a conservative cached layout or Copy answer.
Lifetime-only nominal arguments also require substitution when their fields
contain projections. Cache that structural requirement once per declaration and
instance instead of scanning every field during each property query.

A normalized field list never grants another caller permission to form its raw
projection. Index the structural dependency graph once per canonical type and
retain its raw projection leaves in a persistent recipe DAG. Empty dependencies
vanish; a single dependency shares its child's recipe without copying its path or
flattening its descendants. Each body or concrete instance proves those leaves in
its own scope, including lifetime debts erased from scalar results. Share one
completed-recipe set and normalization session within that scope. Traverse owned
fields and derive argument dependencies with bounded worklists, stop at pointer
and reference referents, and publish completed structural lists separately from
the caller's evidence. Demand published body values and selected receivers, not
discarded provisional inference templates. Index projected declaration field
flags so signature-only projections do not trigger storage traversal. Keep
generic fields symbolic until actual instantiation;
template representation checks treat unresolved projections as unknown layouts.
Concrete instance discovery must include projections hidden in nominal fields,
not just projections visible in the nominal's argument graph.

Projected field inference retains its caller lifetime context after the pending
normalization worklist drains. Later shape comparisons use the same universal
aliases; queue emptiness controls progress, not the meaning of existing terms.
For record constructors with omitted lifetimes, a known expected nominal with the
same owner can supply those input lifetimes before field substitution. Preserve
explicit written lifetimes and prove the usual formation and coercion debts.
An associated output equality cannot invent equations between its inputs.

Complete nominal parameter schemas and declaration contracts independently of
field instances. Prove field and use-site predicates in their owning immutable
parameter environment; a cached predicate list is not a cached proof across
environments. Generated Copy, Clone and equality implementations have separate
assumptions. Propagate by-value parameter requirements over declaration templates
to reject infinite storage without expanding growing recursive instances.
Zero-length arrays still require their element alignment. Infer field-implied
outlives requirements over the same symbolic arguments, retaining type-parameter
requirements as well as lifetime pairs. Retain opaque associated results as
separate predicates, with complete Self, trait and family arguments. Propagate
these templates through nominal uses using shared joint substitution; do not
normalize during inference or infer input bounds from a result requirement.
Captured callable predicates lift to the owning scope, while predicates involving
callable locals remain deferred.

Prepare projected requirements for requested canonical roots before immutable
lifetime proofs, including generated constructor and specialized body contracts.
Cache substitutions by complete nominal arguments and charge traversal, capture
queries and mapping to bounded work. Preparation follows structural inputs without
expanding nominal fields; ordinary contracts allocate no projected state. Cache
requirements, never proof permission. Static field requirements remain obligations
against explicit and independently implied premises. Scalar-only query interfaces
must reject hidden projected debts even when the nominal root has closed arguments.

Nominal requirements inside function pointer types retain their callable binder
depth. Defer predicates mentioning a surrounding late-bound lifetime, as required
by [RFC 1214](https://github.com/rust-lang/rfcs/blob/main/text/1214-projections-lifetimes-and-wf.md#checking-conditions).
Prove independent predicates in the caller's scope after capture-safe lowering;
a predicate's own nested binders do not make it deferred. Cache occurrence and
lowering queries within each immutable scope, with a shared work budget. Deferred
predicates grant no body permissions. At a call, check the instantiated inputs
and result even if an argument diverges or the result is discarded.

Constructor and pattern inference retain unresolved mixed arguments. Reuse one
substitution session across the selected receiver's field templates; do not
resolve the receiver before its fields have constrained it. Substitution under
callable binders shifts inserted outer lifetimes without capturing inner ones.
Written partial arguments, omitted arguments and omitted trailing defaults are
distinct inputs. Alias-qualified variants retain their resolved namespace prefix
and selected member rather than repeating path lookup in body checking.

Infer partial callable shapes by opening source lifetime binders only where the
destination supplies matching callable structure around its holes. A hole for an
entire higher-ranked function-pointer type retains that type's own binders. Keep
the original source for the separate directed subsumption proof. Give sibling
callable occurrences distinct fresh lifetimes, and memoize unchanged shared
subtrees without merging those occurrences. Charge this iterative shape walk and
its substitutions to the same inference work budget. Detect source binders from
the current typed graph before attempting resolution on the generic path; this
query must not intern partial shapes or freeze answers across later bindings.
Register projections in
instantiated generic signatures and nominal fields in their resulting scopes;
an earlier query for the input argument does not cover a substituted occurrence.

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
Reconnect each imported early argument with its original body slot, including
captured owner slots and noncontiguous early slots after late declarations.
Inspect the existing abstraction map; static and unused arguments need no fresh
universal. Drop callables use their checked nominal coverage mapping to reconnect
permuted captures, preserving unused impl proof slots.
Pair each specialized body with its own region proof and complete instance
identity, even when multiple bodies share an original source location.

Demand from a fully proved caller may retain the callee's exact declared trait
premises during projected specialization. Select their validated implementations
in a separate scope after substituting complete arguments. Retain existential
lifetime witnesses as independent constrained universals, with every equality,
outlives and type-outlives condition; keep nested callable binders local. These
givens belong only to that specialized body. A manually constructed instance
has no caller authority and must still discharge the selected implementation's
lifetime debts. Ordinary bodies without projected specialization do not perform
this selection, and lifetime-free predicates need no witness scope.

After a caller's complete ownership, lifetime and cleanup proof, retained proved
outlives obligations may establish that an input lifetime equals static. Resolve
selected callable identities through those equalities before discovering demanded
instances. Store only changed targets on that sealed body; dependency discovery
and executable lowering consume the same table. Preserve unrelated free lifetimes
and nested binders. Neither an unproved body nor another caller may supply this
authority. The selected callee still proves its original template and every
implementation requirement. Requirements absent from its declared premises
remain obligations, never new assumptions supplied by normalization.
An empty static-obligation frontier must require no traversal allocation.

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
body and immutable caller environment; finish it after initializer typing in one
shared query session. Reuse resolved dependencies, selected operations and already
computed discriminants.
Cache demanded ordinary definitions and complete associated instances across
these phases; leave ordinary node facts compact and allocate no query
storage for bodies without constant-backed patterns. Do not instantiate a
whole-crate checker per use, repeat initializer typing, or interpret pattern
views. Deferred checks must retain the matching module's visibility context.

Associated pattern paths share expression argument inference and selection facts.
Their values must be known before function specialization. An unused rigid type
or const argument does not prevent a fixed initializer from supplying a pattern;
an actual dependency on that parameter does. Prepare selected dependencies,
including skipped branches, without evaluating skipped arithmetic. A caller's
trait assumption cannot identify an initializer when it retains type/const
parameters or free lifetimes, including `'static`. Fully global bounds can use
their known implementation.
Retain fresh selection obligations under the requesting body's lifetime proof;
cached values never supply another caller's permissions. Translate nominal fields
from the completed instantiated shape while preserving original field identities.

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

Retain explicit whole-body runtime extents compactly. They mean exactly the
caller's seeds at every CFG point, not a permission inferred from a region kind
or an assumption. Propagate their closure along the proved directed requirements
before ordinary sparse seeds; the contract-only LOCAL sentinel never connects
runtime regions. Bound retained full-region entries, sparse words and pending
work. Membership and logical word counts must match expanded point propagation,
including empty bodies and partial final words. Sort the compact representation
only for inspection, then expand full extents lazily while rendering; never
recreate a region-by-word matrix just to print the view.

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


## Associated semantic inspection

The maintainer semantic view consumes completed declaration/body facts. Reuse the
canonical type, definition, argument and region writers; do not reconstruct
semantic decisions from syntax or output. Owner/member inventories and per-owner
predicate vectors retain construction order. Sort sparse source-keyed maps only
at the view boundary. Count mode consumes the same facts without text rendering.

Expose trait/impl ownership, declared parameters and defaults, complete predicates,
validated method/default mappings, qualified paths, receiver adjustments and
selected implementation arguments. Preserve bound-selected lookup origin through
this view. Region/loan constraints remain explicitly pending: inspection succeeds
at the inference boundary, while LowIR emission retains every template, lifetime,
ownership, loan, cleanup and demanded-instance proof. The new view must not change
that execution boundary or mutate the analyzed facts.
