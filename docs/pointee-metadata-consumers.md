# Source Pointee selection and metadata normalization

This is the consumer successor under development after qualified declaration source
`12e87c240cc10d308364a0b832df4665e90b9e6e`. No consumer implementation or
execution is qualified by this document. The declaration handoff and its six
controls remain immutable; they are not repeated for this proposal.

## Selection and result authority

Pinned Rust revision `88d9e12ae178fab0fb5cc050a94da85685d449ea` separates
automatic Pointee selection from associated Metadata normalization. Its
`select/candidate_assembly.rs:81` admits the actual Pointee role for every
type, and `select/confirmation.rs:238` adds no Sized premise for that role.
Its associated projection confirmation at `project.rs:1555` separately
normalizes the storage tail and retains the resulting obligations. These
are different semantic operations: an unconstrained metadata result must
not prevent a generic Pointee proof or become unit metadata.

The private source is `selection::Source::Pointee(Selection)`,
where `Selection` privately shares the existing `Rc<Contracts>`. It does not
allocate another Rc owner or copy the goal/output into a new per-use record.
The complete canonical application is already retained by the proof node
or requesting projection query and must accompany every member-result use.
The constructor is available only to the Pointee proof completion path.
No implementation row is invented. No ordinary trait named Pointee gains
this source, and `Source::Builtin` remains insufficient for projection.

The shared Rc keeps the exact Contracts incarnation alive across suspension and
prevents a numerical arena/member coincidence from becoming authority.
This changes Source from Copy to Clone; existing arms clone only their
small existing values, and only the Pointee arm retains an Rc. This central API change is approved but remains unqualified here. Its alignment,
Source/Node/Evidence sizes and enclosing query layouts must be observed;
no zero-growth claim is made in advance. The ordinary path allocates no
Pointee owner, and the rare selection takes a prepaid existing-owner handle.

## Source seal and readiness

The current checked member query authenticates Report, Catalog, arena,
source incarnation, owner/effects, canonical headers and result contracts.
The solver and both normalizers currently receive only Declarations.
They cannot honestly claim to reread a separately mutated Report.

The approved phase boundary retains the Report-aware producer check while
Checker borrows its immutable Report. A private facts-side replay compares
the retained Contracts Rc, actual source-role declarations, arena, declared
predicate prefixes, canonical headers/results/regions and
the exact existing HeaderLedger SourceImage witnesses. It grants no new
attribute admission and never infers completion from a map miss. The public
Report-aware CheckedMember::current check remains authoritative for callers
that possess Report. This immutable phase contract needs explicit review
when a Report-free replay API is used. Later genuine Report-aware
pipeline/body/publication boundaries must authenticate again before results
can authorize publication or execution.

Unfinished source contracts carry the actual
`Dependency::PointeeContracts(actual_metadata_definition)`. A phased existing
worklist producer reuses required associated-type declaration, bounds and
outlives producers before sealing the same optional owner. Its ordinary entry
runs after genuine trait and sizing headers and before alias completion.
Later package preparation authenticates the already completed member instead
of recreating consumed reserved syntax. Global `complete_results` is unchanged.
Forward demand while an owning trait header is still unfinished remains an
explicit ordering opponent for this unqualified draft; readiness must come from
that genuine header producer, never an impl-header alias or guessed absence.

The normalizer authenticates DynMetadata's real head, source, owner and formals.
It constructs `DynMetadata<actual full object tail>` without reading its fields.
Ordinary nominal/type formation, regions and all defining-crate field checks
remain required before successful analysis/artifact publication. No extra field
snapshot, RefCell, field-seal wakeup or duplicated field interpreter is retained.
Future VTable extraction/construction/layout consumers must demand actual fields;
head authority grants none of those operations. Input Struct/Tuple tails retain
real existing Fields dependencies and canonical immediate-field substitution.

## One proof producer for both normalizers

The universal trait source shares the existing Contracts owner. A metadata
query has one shared Plan algorithm but uses each normalizer's existing proof
engine. Its continuation reuses each existing optional boxed inherited-result
slot through a disjoint Inherited/Metadata enum. An actual requested alias is
either the inherited trait result or the Metadata member; nested child queries
own their own slots. This adds no Frame/Query pointer or independent engine.
The cold enum's actual tag/max-size and any larger inherited allocation must
be paid and measured. Existing child-frame, wakeup, Availability, scoped
Obligations and original/instance RPIT semantics remain intact.

Ordinary source formation and lifetime duties remain, but Pointee itself must
not acquire an extra blanket nominal-input or Sized premise. A newly observed
projection of `RequiresSized<str>::Metadata` with an unused parameter is
accepted by pinned Rust even though directly admitting that nominal would
have a stronger input contract. The concrete metadata query follows the
actual typed tail; it cannot strengthen the erased caller input speculatively.

Metadata planning distinguishes these outcomes:

* A known sized concrete storage tail yields the canonical unit type.
* A slice or str tail yields canonical usize.
* A dynamic tail yields the actual DynMetadata nominal applied to the full
  dynamic type, including its object region and associated constraints.
  Its real generic/field formation is checked; no substitute nominal or
  fabricated field is allowed.
* A changed unresolved tail yields that actual tail's Metadata projection.
  The enclosing normalizer then demands it through the existing stack.
* An unchanged unresolved parameter/projection/opaque tail yields unit only
  after an actual caller-scoped Sized proof. Failed or unresolved evidence
  retains an opaque result or real pending dependency, never unit.

`abi::metadata::Queries` may report representation and actual tail
preparation, but it cannot supply the role, source member, formation or
trait proof. Unknown remains unknown. Tail visits, canonical substitution,
attempts, retained goals, result construction and failed resumes are charged
before allocation using the existing budget owners.

Both `nominals::traits::normal_form::Query` (and its Cursor caller) and
`traits::selection::search::Normalization` consume the same completed
Plan and completed result accessor. They revalidate the actual member and
full request against the same retained Contracts owner and parent proof.
The source handle proves the trait, while the result continuation proves
normalization demands before returning its resolution. Neither consumer gets a
second metadata algorithm or an unconditional builtin-to-type branch.
Their existing restoration and streamed scope/region proof append remain
unchanged. A method target cannot be formed from Pointee evidence.

## Required source and qualification boundaries

The complete pinned DynMetadata declaration contains NonNull<VTable> and
PhantomData<Dyn>. The producer's explicitly authored minimal providers are
declaration controls, not replacements for those unchanged source fields.
Foreign VTable currently lacks a semantic foreign-type representation;
NonNull's actual pattern_type expansion and the actual PhantomData role
also remain required source prerequisites. They need separate typed
representation/validator proposals before Kind/role changes. No fake
aggregate, erased generic formal or assumed foreign layout closes them.
Raw pointer metadata intrinsics, CTFE/runtime wide-pointer operations and
artifact transport are separate required consumers, not permissions granted
by this selection stage.

Before execution, freeze a small new independent set for concrete unit and
length results, generic changed tails, unresolved versus Sized parameters,
actual dynamic nominal application/region preservation, incompatible region
and result obligations, and ordinary lookalikes. Keep metadata expectations
separate from runtime claims. The candidate groups exercise both normalizers,
actual pending/resume and poisoned origin/arena/member/result opponents,
ordinary no-owner behavior, bounded work and actual layouts. No old six,
22-observation declaration set or predecessor layout run is repeated.

The first ten new sources produced twenty metadata observations in run
40342761935, with no infrastructure failures or warnings. Ten hypothesis
disagreements are retained: eight expected positives reached a missing Copy
language-item prerequisite, and two input errors had E0308 instead of E0277.
Six separately frozen setup sources then produced twelve observations in
40343925207. Genuine Copy/Clone declarations resolve the setup failure; unit,
length, changed-tail, Sized and dynamic-region positives pass, and dynamic
escape fails for an actual lifetime error. Two additional wrong input-WF
predictions are retained as accepted normalization positives. The source
result-bound use pair accepts its actual unit implementation and rejects the
missing implementation. All thirty-two observations are metadata-only.

Parent owns aliases/bounds/lookup and public integration; A owns the checked
import transaction; B owns source transmute/byte codecs. This successor
does not edit those owned algorithms or widen any imported/public profile.

## Focused candidate freeze

The current draft is UNQUALIFIED. The next gate selects exactly four new
source-consumer groups plus the changed Source/enclosing and declaration-cost
observers. It includes canonical Query, generic-argument Search normalization,
real body calls, concrete/changed/projected tails, erased nominal inputs,
Scoped Sized versus assumed-parent opacity and explicit equality, actual result
obligations, dynamic object regions, cumulative failed retention and ordinary
no-owner behavior. Existing 38 observations are not rerun. Source/trait/default
transport and complete pinned VTable/NonNull/PhantomData remain closed/required.
Actual new cold/hot sizes and work are unknown until the gate; old producer and
Source-before measurements remain retained historical observations.
