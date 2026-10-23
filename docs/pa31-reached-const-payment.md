# Reached conditional const obligations

A closed direct source const call now pays its declared Conditional obligations
as Const for each actual interpreter use, including unused and inherited clauses.
A previously materialized Code does not supply a payment. Defining predicates,
caller environments, exact predicate equality and proof keys keep their effects.
Ordinary runtime calls keep ordinary obligations; explicit Const is not weakened.

The executable profile is a complete function-item Instance with no value
arguments, a closed signature, primitive result and closed contract namespace.
Selection must complete without new lifetime, quantified or retained-size debts.
Unsupported pending/open cases fail explicitly. This does not implement
const-callable29, general Invoke, lifetime-bearing calls, Pin/Deref or CTFE cleanup.

## Authority and storage

Actual const impl owners publish ConstTrait capability in their canonical header.
Trial instantiation copies that capability under the requested use effect and
projects copied Conditional requirements accordingly. Source owner/polarity/effect
are rechecked at preparation, selection and completed-proof retrieval. Ordinary
impls do not satisfy Const. Coarse candidate/coherence shape keys alone erase the
effect so ordinary/const competing headers reach the same bucket; exact goals,
headers, inference equations and proof/memo keys remain distinct.

Actual const impl overrides inherit const eligibility through their retained
associated owner. Their unused bodies pass the existing complete original/CFG
const-legality phase. `pipeline::analyze_source` alone does not promise that CFG
check, for either explicit const functions or inherited const methods.

One optional Engine owner retains immutable Code inputs and unmet requirements.
A stack-only proof purpose distinguishes original validation from Code publication.
Only publication that needs a row moves its actual already source-Copy/region/loan
proved Contract into an Arc. CheckedInput is one pointer; original-only and
ordinary no-row paths create no new contract Arc or row owner. Requirements are
prepared once for a complete materialization. Existing Environment trait indexes
are reused as immutable premise structure, never as a paid interpreter token.

Each reached use establishes fresh selection under its exact caller environment.
A private one-use Box binds payment to frame/code/block/statement, complete
Instance, paired public/owned signatures and immutable Contract/Environment rows.
The same Box moves through suspend/restore and is consumed before invocation.
Replay, changed caller/site/callee, missing rows and insufficient retained slots
reject. Continuation payload participates in suspension limits; retained row and
requirement payloads participate in the existing Code budget. Charges follow the
existing deterministic payload policy, not allocator/RSS accounting.

Measured successor layouts are Engine1696, Machine4728, State448, Row80 and
Continuation72 bytes. The source diff adds one cold optional pointer to each of
Engine/Machine/State. Body448, Code304 and Value24 remain unchanged; CheckedInput
is8 bytes. No Body/Call/Code/Value fields or quota increases were introduced.

A's imported-function/header and cached-Code lineage checks remain in place.
C's source drop_glue CTFE denial still precedes cached code use. Actual intrinsic
identities are excluded from this ordinary-function payment profile: layout and
needs_drop retain their separate fresh-use witnesses and kernels. A valid
conditional size_of with fallback0 independently returns1 in Rust, while an
ordinary same-name function returns0. The former remains a required unsupported
positive; no source placeholder executes as an intrinsic.

## Focused qualification

Evidence is rooted at
`/home/vishvananda/rustgm-pa26-work/obj/parallel/pa31/reached-const-payment-v2`.
The six-fixture sizing repair119d is already integrated in the base and remains
unchanged. Original source Copy/Clone/FnOnce and generic inspector/metadata gaps
from the prerequisite remain explicit; this uses checked library API roots and
is not whole-crate metadata or upstream core acceptance.

- `static-v1`: format/strict and all3 independently declared legality outcomes
  passed; an existing declarations function exceeded placement limits after the
  owner test was added. Extracting that exact predicate fixed placement.
- `development-v1`: format/audit passed; two local factoring lints stopped strict.
- `development-v2`: format/audit/strict, all9 private controls and3 source controls
  passed. The fourth source assertion stopped at analysis instead of the declared
  complete lowering phase. That observation is retained.
- `recovery-v1`: the exact corrected source case passes full lowering. Ordinary/
  const overlap rejects, unused invalid const body rejects at the existing
  non-const-call rule, and the valid counterpart lowers. Production was unchanged.
- `closing-v1`: format/audit passed; one new test wildcard lint stopped strict.
- `closing-v2`: format/audit/strict, the new work control, layout observations and
  all17 native programs pass. Source hashes match before/after. Successful native
  objects/executables are hashed, then pruned; logs, LowIR and C observers remain.

The source controls bind independently qualified Rust1.98/edition2024/target
outcomes. Three new legality sources and two intrinsic/name sources have separate
independent object/native receipts. Fourteen work sources are exact copies of
`reached-const-payment-v1/inputs`, reusing their unchanged independent observations
by SHA-256. Checked-in contracts record explicit modes and distinguish source
rejection from valid unsupported use. No text oracles were generated.

## Work observations

A second manual interpreter replay uses already materialized Code, but counts a
new successful payment for every reached call. Both replays produce7. Each pays
28 canonical work per use and retains exactly the same row storage afterward.

| Axis | Width | Payments per replay | Work per replay | Requirement rows | Logical row bytes |
|---|---:|---:|---:|---:|---:|
| Repeated use, one target |8|8|224|1|1264|
| Repeated use, one target |32|32|896|1|1264|
| Repeated use, one target |128|128|3584|1|1264|
| Distinct caller definitions |8|16|448|9|7472|
| Distinct caller definitions |32|64|1792|33|26096|
| Distinct caller definitions |128|256|7168|129|100592|
| Distinct complete Instances |8|8|224|8|6248|
| Distinct complete Instances |32|32|896|32|23336|
| Distinct complete Instances |128|128|3584|128|91688|

Caller-definition controls call both the wrapper and the common target, hence two
payments each. Total row count includes the initializer in addition to requirement
rows. Adding128 unrelated definitions to the8-call input leaves work224,
requirements1 and logical row bytes1264 unchanged. These are work/storage controls,
not wall/RSS performance claims; combined release/performance remains integrator work.

The unchanged original paid source has SHA256
445583c77f030b50bbb6e16e1f634c0649bcb47f23dac413df2d9087127a2840;
the unpaid opposite is
b25fb0dab492f7ade3fc1989b105ab846fcaee4ab0b22b44a567d8e3b5c0ad29;
the runtime opposite is
50f5a24b42ee12552d2f9ca05ed4b19fec9aeba040483caf5b682d63473f1c25.
The old unqualified90cc prototype and all failed development receipts remain
preserved separately. Conditional intrinsic use, rich/open payment, actual const
callable adapters and CTFE source cleanup remain required followups.
