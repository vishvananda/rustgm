# Source const destruction definition checkpoint

This is intermediate progress from 5dc74570d. The unchanged retained VaList
cldef input now compiles through real static ownership, const destruction and
native lowering. Its clcall companion reaches the explicit interpreter cleanup
boundary before Code snapshot/publication. It does not execute a skipped destructor.

Actual Struct cleanup and explicit const Destruct bounds use the ordinary
source trait solver. The producer selects actual const Drop and recursively
proves semantic fields after full Bound::joint substitution. References and
pointers do not own their referents. ManuallyDrop/Never, suppressed fields,
unions, tuples and arrays retain their existing policies. A Copy premise can
suppress body cleanup; it does not invent a demanded const Destruct call premise.

The source Drop controls use the independently accepted required-member header
variant from the earlier source-c-enum fixture. They do not claim the complete
current upstream Drop/Pin declaration. Its original provided drop/pin_drop
module is retained unchanged and registered in third_party/sources.json. The
VaList consumer remains the previously approved attributed source/body cutout,
with actual lifetime markers, representation, traits and reached bodies.

## Shared proof boundary

- component_markers retains its existing heterogeneous Goal frontier. The new
  start_component_predicates adapter shares its paid constructor with uniform
  marker/capture goals. cparts adds actual const Drop and semantic field goals;
  no second solver, parser or authority registry exists.
- Verifier prepare/finish retains the one existing initialization Facts and
  symbolic cleanup plan. The template Batch retains owner indexes, region
  results and contracts, but holds only one initialized row at a time.
- The concrete cleanup kernel likewise retains its actual initialization Facts
  and native destruction Plan. Catalog returns to its existing Cache on every
  Result path before semantic selection. No CFG clone, initialization replay
  or second cleanup selection is used for this join.
- The transient seal checks the current type/definition arenas, immutable body
  allocation, verifier, original Copy evidence and exact environment. The owned
  concrete callback exposes only &Body and checks completion allocation before
  constructing Proved. Private foreign/replaced-arena, cloned-body, verifier and
  environment opponents are authored for CI; no cold private harness was linked.
- Pipeline owns semantic selection. Only affected contracts have their added
  lifetime obligations revalidated before normal loans finish. Contract uses Cow
  so a borrowed/no-active path does not acquire a new proof clone.
- Struct eligibility consumes live_type obligations, including a partially
  moved root with no physical cleanup node. This preserves the independently
  observed partial-Struct E0493. Actual access/node activity separately controls
  the Code-purpose interpreter fence; selected claims are not execution activity.

For an authenticated actual const Drop owner only, nominal runtime coverage
compares MaybeConstTrait as ordinary Trait. Original predicates remain unchanged
for conformance, const selection, effects and bodies. Stronger ordinary Clone/Foo,
unconditional const Foo, ordinary Drop and same-spelling non-role controls retain
their independently observed distinctions. AsyncDrop is unaffected.

## Qualification and limitations

source-const-drop-evidence.json binds every changed compiler/fixture file, the
final gate, reference history and native commands/artifacts. Final gate7 includes
strict CLI/integration Clippy, full formatting/audit, exact public controls,
actual C++ native observers, old VaList/enum/assignment/index cases, registration
and upstream verification. Complete private/release checks remain CI work.

The genuine const Drop observer writes 41 through its runtime pointer. The
conditional generic runtime observer executes an ordinary nonconst Drop and
writes runtime data 47. This latter path already has the real checked-caller
materialization witness in lower/instances/demand.rs: conditional runtime bodies
are distinguished only after authenticating original, instance, contract and
caller. CTFE materialization is separate. No redundant permission was added.

Canonical work for 0/8/32/128 actual unused const bodies is base 706 with growth
713/2849/11393. These are bounded-work observations, not an A/B wall-time claim.
On this 64-bit host cold defining State/Prepared is 112 bytes (was 104), Claims
48 (was 40), template Verifier 576 (was 568), incremental Verifier 528 (was 520).
The shared seal is 40, concrete Prepared 952, symbolic Prepared 784, Row 1008,
Batch 792 and LoanInput 32 bytes. Ordinary Body/NodeFacts/Code/wire layouts were
not widened. Existing retained-byte calculations include the enlarged cold state;
selected proof/map/vector growth is bounded before its private replacement commits.

These remain active required work, not parity negatives:

- selected enum/precise cleanup: known Good(ConstDrop) versus other
  Bad(NonConstDrop) is genuinely Rust-positive; whole-root Destruct cannot stand
  in for selected leaves. Improve the existing initialization/variant facts and
  authenticated semantic coordinates rather than run a second analysis.
- unconditional const generic runtime instances (rtonce) still need their genuine
  checked-original runtime consumer joined. The conditional rtinst/rtcnt path
  already works and must remain distinct.
- real interpreter cleanup scheduling/execution, actual const Drop callees and
  VaList cursor init/read/copy/end/early-exit/cleanup. Code rejects active cleanup
  before snapshot until this exists. No placeholder/native fallback substitutes.
- original unit-result conditional const payment positives, full unchanged
  standard module/macro and provided Drop/Pin closure, and existing imported
  variadic-local/operation transport profiles. Old wire fences remain.

The initial obsolete impl-const spelling, invalid struct [const] bound, missing
const Drop Destruct prerequisites and extern-string whitespace setup are kept
with original bytes/diagnostics. Array length zero, partial Struct and ordinary
Copy predictions were corrected from actual Rust outcomes without replacing the
original predictions. Infrastructure failures are never counted as rejections.

## Root integration

Preserve a3a2's future_arena witness and AsyncBody/AsyncApplication matches.
Batch constructs State before loans: initialize future_arena as None there,
then authenticate it in successful Batch::finish using current Store AFTER all
loans. Pass Store to finish from verify, context::verify and pipeline::cdrop.
Retained/suspended views must preserve the same witness and foreign-store checks.

Expected joins are borrow/templates.rs and context.rs, lower/instances.rs and
proof.rs, pipeline/execution.rs plus defining_heads/anonymous/parent_children,
module registrations, suite registration and the upstream manifest append.
Root has the exact component_markers SHA/API; preserve its new AsyncCallable
completion while retaining the one shared constructor and this adapter. Async
bodies are not const. No backend/pin, global spec/tracker or main changes belong
to this checkpoint.
