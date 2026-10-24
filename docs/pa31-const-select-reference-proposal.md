# Reached shared-reference callback selection proposal

This is a source-only successor to the qualified `736cb69a` compiler and its
`f8e67119` evidence document. Work starts on branch `pa28-ctfe-reference` from
that documentation checkpoint. No reference implementation, build, probe or
candidate result is claimed here. The original `reference-binder.rs` stays
byte-identical, SHA-256
`873e50d001036c158a16f771ab236571d2a25a5478575fcfc441e722e6022d45`.
Its pinned independent metadata/object/native observation already prints `7 1`;
its currently qualified candidate outcome is explicit unsupported.

## Existing authority and missing adapter

Ordinary direct calls already retain one caller-opened `Rvalue::Call.signature`.
`sema::bodies::calls::instantiate_call` creates caller-owned region identities,
opens all parameters and the result with one shared substitution and retains
that complete signature. Caller region and loan proof, actual const-function
eligibility and the independently proved callee body remain separate authorities.
`Machine::push` stores the actual Constant arguments in the selected Code's typed
parameter locals; reference allocation provenance and frame liveness remain
checked by ordinary memory/return handling.

`sema::traits::callables::match_signature` also opens one whole tuple containing
all parameters plus output. It equates the input tuple with actual FnOnce ARG
and its output with the actual Output equality. Its selected proof retains those
region obligations in the caller Contract. The selector's validated declaration
requires the exact F:const FnOnce<ARG,Output=RET> clause; G's ordinary obligation
stays required separately. An opening descriptor records this completed typed
Call relation. It is not an independently fabricated positive trait witness.

The current const Invoke adapter is deliberately limited to closed fn(u8)->u8.
It supplies no reference-profile permission. The selected-reference adapter must
reuse normal demanded function entry while keeping that Invoke frontier intact.

The current selected profile rejects nonempty callback binders and requires
closed ARG/RET. Its entry consistency helpers also compare Code.result with a
public result; that equality cannot represent a distinct owned opening. A new
shared-reference profile needs the caller's complete input/output relation and
separate public/owned Code consistency. Changing only Code.result to owned-result
equality would still establish no caller/result permission.

## First reference profile

Keep the existing closed/static-generic adapter unchanged. The new profile has
ordinary, safe Rust source const F, EMPTY generic arguments and exact evaluated
full F equal to the logical full F of this Call. Its public late binder remains
in that full Instance; no synthetic binder-erased executable target is created.
This excludes generic/early-lifetime remapping, imported functions, members,
selected intrinsics and conditional F until their existing owners close those
frontiers. Mutable-reference cleanup is separate. Shared thin and slice references
are exercised without adding slice indexing or library algorithm requirements.

The new caller may itself have a public late binder. Its exact owned Code and
Contract/region/loan proof provide universal caller authority. The existing
closed caller shortcut is retained for the old profile; the new path must require
the actual retained Row and exact Call descriptor. No descriptor means explicit
unsupported, not fallback to headers, signature shape or a cached body.

## Owned producer, storage and publication

1. After actual regions and loans, while exact Body/Contract/solved inputs are
   borrowed, prepare lazy per-Call data in the retained input owner. Derive the
   caller-namespace function type from the actual ARG fields and RET, together
   with the fixed actual logical F/selector relation. There are no fresh regions,
   second trait query, erased comparison or declaration-signature replay. Keep
   the whole input/output relation; never match independent fields in isolation.
2. Extend the existing cold `use_inputs::Row` with a lazy per-Call descriptor
   table. Ordinary rows and non-reference selector rows allocate no such table.
   Store only actual Call keys and necessary typed roots; derive other inputs
   from the immutable Code. Build no callback body during this preparation.
3. Row also shares an Rc clone of the actual finalized Code already owned by
   Catalog. This creates no new Code/Body object. Row validation uses actual Rc
   identity as well as source/key/instance/paired signatures/Contract; coincident
   metadata or matching cardinalities cannot retarget proof to another CFG.
4. CheckedInput owns Code plus its prepared retained-input plan. Finalize the
   existing ordinary/anonymous/source-initializer/paid-child mode before a Row
   retains that Rc. Prepare descriptor, Contract owner and any ordinary existing
   Requirements, and preflight their actual payload before either canonical
   publication. The separate existing Code and Inputs storage caps remain;
   retained total accounting includes both owners without double-counting Code.
5. Use a cold registration stage borrowing Catalog mutably: it owns finalized
   unpublished Rc<Code>, prospective canonical index/mode and byte plan. The
   retained-input plan is prepared against that exact Rc and index. Reserve both
   owners before an infallible paired commit; a failed allocation, wrong witness,
   wrong scope or Requirements failure publishes neither Code nor Row. No
   Rc::make_mut, unsafe address authority or post-publication descriptor install.
6. Named initializer demand constructs `Target::Initializer` before proof. The
   earlier semantic producer already recognizes an actual root through its typed
   `ValueScope::Initializer` and dense declaration index. Warm named roots validate
   exact source/type/kind/canonical Code, including when a root-cache entry exists.
   `Catalog::initializer` only validates that finalized owner read-only. Anonymous
   expression demand remains separate; a proved anonymous Code/Contract is never
   relabeled as a named declaration.

Concrete cold interfaces to settle during implementation are
`use_inputs::Source` (owned Contract plus lazy Call data),
`use_inputs::PreparedRow` (Rc-bound unpublished retained owner), and
`code::Registration<'catalog>` (the unpublished finalized Code/index/lease stage).
Their constructors remain private to the common completed Code-purpose path.
Original-purpose checks produce neither Code nor descriptors. Existing effect
Requirements and fresh Payments retain their current distinct APIs/authority.

## Reached consumer and entry

Machine currently borrows Catalog while Inputs lives in Engine.const_calls.
Add `CatalogRef::Source(&Catalog,&Inputs)` and a source constructor used by Engine;
keep Owned and Borrowed modes. The new variant borrows the two current owners,
adds no Machine/Code/Frame/State field and allocates no wrapper. Its payload is
smaller than existing Owned(Catalog); verify actual hot sizes rather than infer
layout from this observation. Ordinary/no-input paths retain Borrowed directly.

For a reached reference selector:

- Borrow the actual current Code/Frame/Call at the existing explicit suspended
  or executing phase. Validate live catalog index and exact Code object.
- Look up and validate the Rc-bound Row and this exact ValueId descriptor. Bind
  full selector instance, EMPTY-argument full logical/evaluated F, actual opened
  ARG fields/RET and already evaluated operands. Charge successful and failed
  lookups, including warmed-body uses. Copy only the minimal typed descriptor
  data into the existing private transient witness; the owners remain immutable.
- Enter the actual F through the common const/member/drop/eligibility gate and
  existing FunctionBody suspend/resume path. Validate actual source/key/owner,
  canonical index/full Instance and public/owned signature pair. Code.result must
  match the owned result as consistency; the authenticated caller Row supplies
  invocation/result permission. Preserve the callee's own late binder and proof.
- Spread owned ARG exactly once using ordinary field values. Keep shared-reference
  provenance/length and source allocation liveness; use the existing destination
  for the returned Constant. Do not create a selector or unselected G frame.

No proof descriptors go in Catalog.source_results, which remains a pure
layout/drop result cache. No new generic proof engine, IR/backend opcode or
hot Code/Frame field is proposed. Report cold Row/CheckedInput/Prepared sizes,
actual descriptor capacities and attempted work separately. The current cold
baseline is Row80, CheckedInput320 and Prepared312 bytes. Hot baseline is
Code304, Body448, Frame248, State448, Machine4728 and Value24 bytes.

## Predeclared qualification contract

`pa31/tests/spec/const-eval-select-ctfe/contract-reference.json` is written before
implementation or probes and binds all bytes and hypotheses. It retains the
original independent positive and adds nine sources: four positive targets and
five intended language negatives. New positives are local argument allocations,
a bound caller returning its input, two distinct input/result lifetimes and a
wide shared-slice identity with thin C observers. Expected outputs are `7 9 1`
for the first three and `7 1` for the slice program. These are hypotheses, not
acceptance observations. Unrelated prerequisites must be reported as unsupported;
a failed hypothesis is preserved rather than replaced silently.

Negatives distinguish callback-definition cross-lifetime failure, caller escape
of a local, a caller promising static from a general lifetime, ordinary non-const
F and reified function-pointer F. The first three must reach actual lifetime
proof rejection, not pass on the adapter's unsupported diagnostic. The latter
two retain actual const-callable obligations. Fresh/warm private controls must use
real proved positive rows; altered structures are rejection opponents only.
They cover foreign Catalog/Inputs/Code despite coincident headers, wrong Call/
selector/F/full Instance, swapped correlated signature data, public/owned entry
consistency, failed publication budgets with no partial indices, unrelated
allocations under warmed F Code and the old no-static generic frontier. Retain
all current closed/static-generic positives and runtime/ownership controls.

## Pinned upstream need

Inspection uses the shared read-only tree documented by the whole-core census,
revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`, not a modified provider.
Actual calls include:

- `core/src/slice/memchr.rs:54`: u8 plus shared &[u8] to Option<usize>.
- `core/src/slice/ascii.rs:461,598`: shared &[u8] to bool; the latter is the active
  x86_64/SSE2 branch. Both are shared references.
- `core/src/ffi/c_str.rs:390`: shared &[u8] to correlated &CStr. Its strlen helper
  at749 uses a raw pointer and is a separate profile.
- `core/src/str/mod.rs:70`: &str plus two indices to never.
- `core/src/panicking.rs:96`: fmt::Arguments<'_> plus bool to never.

The intrinsic macro at `intrinsics/mod.rs:2435` generates actual const/runtime
functions and forwards its capture tuple. This motivates true bound signatures,
shared-reference transport and later nominal lifetime cases; it does not establish
support for each surrounding library algorithm, unstable attribute or data model.

Pinned file SHA-256s are memchr
`c2bc761de734a9e25e02b45b199be8ec4cf3d8730da70b0d6847e4a02745eae7`,
ascii `d6629cb042952a616d94e7244c9189f1d84bc3a55dfeed4959c74101b5369da0`,
CStr `d70203ccebe03eb6b24666154f88951faa58b0d19b431510d1ce69dbf5a3daac`,
str `ad5d6274b831db659a5ea8a36462e8e7f59d4a89b4c22375c1938da776f3c6ed`,
panicking `805a33d479ebcdf2c640f43fb01b7a6707038bc248b4830d15f30a8a60d40303`
and intrinsics
`aab5c05ff3933d28cbab7914d71054f830311a888234da6201bf2f1a2959ebc3`.
