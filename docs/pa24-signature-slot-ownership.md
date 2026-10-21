# PA24 assigned public signature slots

This repairs the two unchanged object-input programs `quantified-input.rs` and
`quantified-output.rs`. Selection already proved their inherited callable
principal, but committing its canonical inputs treated an assigned closure
signature placeholder as an enclosing-body variable. Under the public function
frame it shifted the original local Bound(0) to Bound(1), so strict comparison
rejected the independently valid program. The recorded debugger trace is in
`obj/parallel/pa24-object-inputs/followup/binder.log`.

Fresh parameter/result holes for a nonempty public closure binder now register
before any equation or child import. Registration checks the actual pending
Closure self, exact signature edges, distinct Any holes, unchanged roots/ranks
and absence of previous history. The Store-local ClosureId identifies origin;
frame and definition identities remain local to their Context. Generic instances
and completed/imported declarations cannot register these source holes.

The lazy catalog keeps immutable definitions, actual function-frame aliases and
a sparse union index over original placeholder identities. It never unions
histories merely because canonical constructor nodes are shared. Persistent
raw operands are considered before the equal-root shortcut, including an equation
between previously assigned placeholders. An external/unowned or different-owner
participant makes the entire class Mixed permanently. Rank and path compression
retain that history across later links without rescanning old records.

The private immutable `Term.was_variable` marker records allocation origin,
including a numeric hole later changed to Known; it grants no ownership. Term
size and alignment remain unchanged: 32 bytes in the private baseline comparison.
Unresolved imports allocate variables; complete canonical values are not new
placeholder definitions. Existing refinement pairs retain original placeholder
identity when an imported hole subsequently becomes assigned.

Only the actual binder-preserving region fold supplies a same-context frame
alias. Imported frames/definitions carry their origin with closed=false. Both
incoming refresh and outgoing publication merge loss into the exact retained
placeholder pairs, before parent recommit; neither can restore lost authority.
No imported body-opened frame acquires closed public-frame authority by arity.

Strict scoped comparison tracks owners separately on its two function-frame
stacks. An assigned local-only source slot shifts only by the extra nested depth
below its actual defining frame. An absent defining frame under nested comparison
rejects; depth-zero use retains ordinary behavior. Mixed escaping slots remain
conservative. The unresolved binding branches are unchanged in both operand
orders, so this repair grants no ability to capture a new local lifetime.
Empty public binders keep ordinary inward rebasing and allocate no catalog.

These are inference transport facts, not trait evidence, outlives assumptions,
body proof or execution permission. Exact proved-input equality, canonical
binder correlations and the complete ordinary borrow/lifetime checks remain.
No public wire, callable ABI, BodyKey or const permission changes.

## Declared controls

`object-input-outcomes.json` now has ten executable positives, six independently
Rust-invalid executable negatives and one unresolved-target boundary. The two
original quantified positive files are unchanged. The existing `output-escape`
negative now reaches its actual executable lifetime rejection; emit-types emits
the deferred facts rather than the earlier incompatible-types failure.

`signature-slot-outcomes.json` adds two repeated/independent-frame native
positives and three opposing lifetime rejections: captured outer output cannot
become a local public input lifetime, owned local backing cannot escape, and an
enclosing annotated input is not universally quantified. All three negatives
retain emit-types facts and reject LowIR/constants through ordinary lifetime
proof. Original source bytes and pinned full-object observations are retained.
The initial hypotheses expected the type view to reject; the staged-view
observations are recorded rather than represented as language acceptance.

Private adversarial checks cover same/different owners, both operand orders,
assigned equal-root equations, later external unions, shared canonical values,
actual fold aliases, duplicate imports, repeated refresh in both directions,
nested depth, zero-slot outer references, unresolved capture and ordinary-context
allocation. The logical retained limit charges 128 bytes per sparse catalog
entry, at most 64 MiB, independently of allocator capacity/RSS. Graph/index visits
are charged to existing inference work; no allocator or hash-comparison count is
claimed. The assigned-history chain at 16/64/256/1024 has 270/1038/4110/16398
charged work units, exactly 16*N+14 for this family, not a general timing claim.

Evidence is under `obj/parallel/pa24-signature-owners/`. Failed compile/style and
counter-hypothesis iterations remain recorded. The seventh snapshot passed all
40 inference tests and strict release all-target/all-feature Clippy plus 29
independent object/three-view probes. The final focused native/audit receipt is
the worker checkpoint evidence. No references, assignment activation or worker
push is part of this change; the integrator owns combined publication and timing.

The completed worker gates pass 40 inference tests (20 ownership tests), 17
focused source/native/scaling functions, strict all-target/all-feature release
Clippy, the 1468-file architecture audit (zero findings), layout38 and upstream
24+126. Four newly executable native programs comprise the two original promoted
quantified fixtures and the two new repeated-frame fixtures. Existing receiver
places, captured output frames, Copy, and object controls also pass. The final
style-only registration helper extraction was checked by strict Clippy/audits;
focused source/native checks ran immediately before that extraction.

Skipping ownership registration for empty public binders preserves the existing
ordinary object-family counters exactly: repeated 9494/37838/151214 and distinct
9851/39419/157691 at widths 8/32/128. This is work-counter evidence, not an RSS or
wall-time neutrality claim. The prior unexplained scalar RSS observation remains
unresolved; no benchmark was rerun here.

Retained-capacity follow-up: registration preflights the whole new frame plus
both records per fresh slot before allocating validation scratch or inserting
any catalog entries. Import preflights its exact missing class/definition/frame
records, actual folds preflight a missing frame, and loss/union operations
preflight their missing class records. Existing defensive postchecks remain.
The budget is logical retained storage, not allocator telemetry; existing
capacity and checked growth prevent batch overshoot or arithmetic wraparound.
Private controls cover a 262144-slot rejected registration with no catalog
allocation, exact remaining capacity, existing records and overflow.
The isolated capacity follow-up passes all 22 ownership tests, strict release
all-target/all-feature Clippy and the 1468-file architecture audit with zero
findings. Evidence: `obj/parallel/pa24-signature-owners/capacity/`, resource
receipt `79a76cc7840743c3b9d1d814621e0157`. No broad functional suite was repeated.
