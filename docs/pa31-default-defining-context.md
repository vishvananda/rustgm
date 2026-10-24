# Const-trait default defining context

This development successor separates defining const legality from executable
const capability. The call-checking scope passes focused qualification; the
independently established defining-drop obligation remains a required blocker.
It is not whole-core or cross-crate source acceptance.

The subsequent [defining cleanup checkpoint](pa31-defining-cleanup-preparation.md)
closes the direct rigid-parameter cleanup obligation in focused source/native
qualification. The earlier call-checking history below retains the boundary as
it stood at that checkpoint. The subsequent
[combined integration qualification](source-defining-producer-checkpoint.md)
passes full release/native/text checks and performance gates with the actual
source producer; it does not qualify imported-default execution.

`sema::function_context::FunctionContext` is a transient value derived from the
completed indexed Function/body and direct associated owner. Runtime,
ConstFunction and ConstTraitDefault remain distinct. The query uses bounded
indexed lookups, allocates nothing and stores no new record or map. Missing or
mismatched completed function data rejects. A nested function is classified by
its own direct owner.

`callee_use_effect` consumes that context and publishes Conditional through the
existing compact SourceCallUse for const-trait defaults. `Body.const_context`
continues to describe executable constness; destructor-loan checking and runtime
specializations retain its existing meaning. The readonly and mutable constants
eligibility entrypoints share one counted preparation for an exact original
default body. Neither path changes constant_functions or is_const_function.

The proposed profile rejects a nonconst free helper even when unused. An exact
derived Self:[const]ActualTrait clause belongs only to the actual default body's
environment. Only a copied implicit member Self requirement with its original
declaration site receives the call's requested effect; canonical clauses remain
unchanged. The existing normal selection and body proof paths retain all debts.

One scoped `DefiningCalls` view validates the original same-trait/Self member
application, typed Call operation, SourceCallUse owner/effect, full target and
opened signature/result. Its local environment preparation is shared across the
body's calls. It supplies defining legality only. Required acceptance includes
unchanged Clone::clone_from's paired original, ordinary-impl runtime calls and
runtime destructor cleanup. Direct/Invoke, pre-cache and root CTFE checks reject
actual const-trait member execution independently from instance/flag/cache state.

Pinned independent evidence preserves eleven original std-context hypotheses
and their four falsifications/eight raw rejection observations, then a distinct
thirteen-input no_core provider family. Those outcomes all match and six native
observers return 29. A separate sixteen-input std/provider implicit-Self matrix
also matches, including eight native 29 outcomes. The separately declared
runtime-cleanup context passes pinned Rust metadata, object and native 7 checks.
Its provenance is unchanged with no findings or infrastructure failures.

Development v1 stopped at audit naming; v2 stopped at a missing Result
propagation; v3 passed formatting/audit and stopped at the neutral Call input's
Copy lint. V4 passes strict and the debug build, 26/27 provider tests and 8/11
private controls. Its preserved failures expose two incorrect consumer
assumptions: an abstract member Call must not require an unrelated FnOnce role,
and known direct function calls do not ordinarily retain Invocation facts.
The unqualified recovery gates FnOnce only for actual Invoke sites and retains
the exact direct member receiver through the existing inference/publication
continuation. Its kind=None, unadjusted fact is data only; the Builder preserves
the ordinary Function Call path only after matching its actual default owner,
const-trait member and typed callee identity. Pointer/local-variable reads keep
their existing path. V5 stopped at new filename-prefix placement; V6 passed the
audit and stopped at one boolean-simplification lint. V7 passes format-check,
all-target/all-feature strict, 28 provider tests, the existing closure-pointer
semantic case, 14 exact private cases and 25 native programs. Registration and
upstream verification also pass. V6 audit is retained by exact inverse evidence
for the equivalent boolean rewrite. All failed receipts remain unchanged.

V7 evidence is under `obj/parallel/pa31/trait-default-effects-v1/development-v7`.
Its before/after source snapshot is byte-identical, SHA-256
`3ac2a960708f5ebdf5ee60f70a1843e96a9da6d1b62ce118074490ce29630a9b`.
The genuine Clone original passes paired proof/inspection. The direct-member
source negative independently rejects with E0282; the candidate instead first
rejects at a trait obligation, so this is not a claim of matching inference
diagnostics or reaching the same inference boundary.

One prepared Clone defining-call view costs 23 setup visits, then 112/448/1792
visits for 8/32/128 checks; a direct member view costs 22 setup visits, then
120/480/1920. These are readonly-check counters, not whole-source timing claims.
The transient view/input measure 24/20 bytes. Actual Invocation20, Operation16,
NodeFacts80, Declarations7064, Body448, Code304, Value24, Engine1696,
Machine4728 and State448 remain unchanged. Ordinary no-context checks retain
zero added work and no cold payment allocation.

Five separately declared defining-drop hypotheses also match pinned Rust:
std/provider defaults consuming Self without a conditional Destruct bound reject
with E0493, std with the bound executes runtime cleanup7, and std/provider
Copy-bound defaults are dropless runtime7 positives. The existing exact provider
conditional-Destruct cleanup positive is retained by source/tool/receipt hashes.
Those observations are under `defining-drop-v1/independent`; they do not prove
the candidate enforces the obligation. A subsequent check must consume proved
active cleanups under the exact defining environment, excluding moved/dropless
values, before this defining-legality checkpoint can close.

No Engine, Machine, State, Body, Call, Code, Invocation, Operation or NodeFacts
field was added. The existing counted const-Invoke defining validator charges
the additional bounded context lookup before access, including failure. Ordinary
nonconst bodies use an indexed owner-kind gate without a body/scope walk or new
allocation. Broader abstract/default execution, nonempty trait argument profiles,
conditional callback targets and CTFE cleanup remain required followups.
