# User-defined iteration checkpoint

PA26 remains planned. This checkpoint implements canonical `Iterator`,
`IntoIterator`, their identity and mutable-reference blankets, and `for` over
user-defined implementations. Arrays, slices and ranges remain required next
iteration contracts, alongside further iterator adapters and provided methods.

The resolver owns demand-only canonical trait, associated type, impl and method
identities. It reserves `for` dependencies after source traversal, preserving
source definition identities. Written aliases, imports and lexical lookalikes
still use ordinary resolution. The two canonical blanket impls have actual
library-owned identities and declared prerequisites; no generated user syntax
or diagnostic node supplies ownership or assumptions. Their overlap filter uses
the existing declaration-only may-overlap query, keeping deferred, cyclic and
open prerequisites possible. It publishes no execution witness.

Semantic preparation retains `Iterator::Item: Sized` and
`IntoIterator::IntoIter: Iterator<Item=Self::Item>`, including all associated
formation proofs. `Iterator` itself accepts `?Sized`; the identity blanket needs
sized owned `I`, while mutable-reference forwarding accepts `I: ?Sized`.
Both canonical blanket methods get ordinary generic original CFGs and contracts,
keyed by their real method declarations. Independent factory validation checks
the exact registry owner, parameter schema, prerequisite, associated projections,
safe Rust signature and lifetime binder before constructing a body. The originals
then pass the same promotion, region, ownership, loan and drop proof as source
bodies. There is no new permit, body owner or IR operation.

Each checked `ForPlan` retains two separate complete callable/signature contexts,
selected `IntoIter` and `Item`, and the actual canonical `Option` variants. Plans
are published only after ordinary trait selection and associated relationships
are proved. Lowering evaluates `into_iter` once, stores the hidden iterator for
the entire loop, reborrows it for each selected `next`, and matches its returned
`Option`. Pattern bindings and the option belong to the iteration scope; the
iterator and expression temporaries belong to the enclosing loop extent.
Existing transfer cleanup handles normal repetition, `continue`, `break`, labels
and return. Forwarding through `&mut I` projects both reference layers and forms
a fresh mutable reborrow through the normal borrow operation;
it never reads or moves an intermediate mutable-reference value.

The frozen source matrices are under `obj/parallel/pa26/iteration/declared-v1`,
`declared-v2` and `declared-v3`. All 48 declared outcomes independently match
Rust 1.98.0/2024: 29 accepted executable programs and 19 rejections. The original
43-case set passes LowIR and all 26 accepted originals execute through pinned
CPPGM. Permanent focused tests additionally cover the five later controls.
The four destructor-counter observers contain unsafe raw-pointer operations and
therefore live under PA27; the other fixtures belong to PA26. Draft outcome files
use explicit LowIR mode and native schema and do not activate an assignment or
claim published text references.

Private invariants check both original contract collectors, ordinary proof,
forged owners/parameters/prerequisites/binders, sparse zero-demand storage,
canonical call identities and scaling. Ordinary declarations grow from 8,416 to
8,424 bytes; the existing boxed call-state record grows from 720 to 728 bytes.
Both added slots are optional owners, with no new allocation on a scalar-only
source. Work stays 40, TypeId 4, Kind 24, Value 24 and NodeFacts 80 bytes.
The lazy registry is 304 bytes; each retained ForPlan is 36 bytes and each
original record 56 bytes. With 8/32/128 loops, actual inference work is
1,930/7,546/30,010, trait selections 16/64/256, and body work 86/326/1,286.
With one loop and 8/32/128 unrelated functions, selections stay at two and
inference work is 356/548/1,316. These are logical-work bounds, not whole-pipeline
performance claims. Frozen scaling sources and hashes are in
`obj/parallel/pa26/iteration/scaling-v1/plan.json`; integration owns immutable
release timing and full qualification.

The known cleanup/liveness quadratic remains a separate measured dependency:
`borrow/liveness/cleanup.rs` enumerates entities times points and repeatedly
requests destructor components and region seeds. This checkpoint preserves
cleanup semantics rather than mixing in that algorithmic change.

Other required PA26 work remains explicit: array/slice/range iteration;
question-mark; unit, tuple and str comparisons; remaining derives and formatting;
float/char and further associated constants; and the retained unsuffixed
AddAssign RHS inference case. Earlier accepted source-wrapper alternatives do
not close those contracts. PA30 broader ABI work is likewise not completed by
this handoff.

## Closing qualification

`obj/parallel/pa26/iteration/close-v1` is terminal zero. Strict release Clippy
(all targets and features), architecture audit (1,633 files, zero findings),
layout, five private factory/proof/scaling tests, the compact call-state test,
48 source outcomes, 28 grouped native controls, discovery and upstream checks
pass. The no-prelude original also has retained exact-source native success in
`execute-v1`. Nine frozen scaling inputs independently compile under pinned
Rust; all six new iteration inputs reject on the MAX baseline and pass the
candidate, while three ordinary controls have exact full output equality.
The frozen candidate SHA-256 is
`d67b6e4fc5847f6a565c46f91c86fc577836b6ed6a0b14f5425968ac07256e95`.

All 8,360 active PA1–24 exit statuses match. There are 8,356 exact text triples
and four differences against this branch's older committed references. Three
are inherited: PA6 `let_chain` names and PA21 `hidden-equality` diagnostics in
semantic and LowIR modes. For both PA21 sources, pinned Rust independently
rejects with E0369; the immutable MAX baseline, iteration candidate and public
`0b995f43` references all give the identical line:
`ERROR: file 0: 48: trait obligation requires an unambiguous matching assumption or implementation`.
The older local reference says `operation requires a scalar type` at the same
position. Exact source and public-reference hashes/observations are in
`opaque-review-v1/review.json`; iteration introduces no PA21 difference.

The one new delta is PA6 `for_binding`'s names view. Reserving canonical
iteration dependencies appends library names, scopes, definitions and bindings;
every old record stays exact and in order, including source name IDs 0–20,
scopes 0–4, definitions 0–19 and both references. The EOF definition count grows
from 20 to 51. The unchanged source independently compiles as a Rust library;
its original bytes, exact append-only identity check and both binary hashes are
in `names-review-v1/review.json`. All difference artifact paths are enumerated
in `close-v1/differences.json`. No references were edited or generated here.

Retained failed/preparation observations are explicit: initial probes lacked
the source-free owner/associated-definition handoffs, then needed the correct
captured impl-parameter index and prepared generic Option fields. A private
proof test initially omitted mandatory promotion analysis and was corrected;
the implementation always used it in the production assembly path. Two queued
or early test guards were cooperatively interrupted, with receipts preserved.
The final source/native/private and strict checks run after the corrections.
This is a functional handoff; integration owns combined release and performance.
