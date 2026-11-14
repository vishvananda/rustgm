# RustGM assignment roadmap

The [explicit const-dereference checkpoint](docs/pa26-explicit-const-dereference.md)
qualifies13 standalone source outcomes, two new native41 observations and all
seven prior indexing native controls. It addresses clean29's DropGuard::deref
result/place distinction. Ordinary bootstrap and original-library artifacts stay
open; repair the known mixed-profile declaration gate before the next freeze.

The [conditional-member checkpoint](docs/pa25-conditional-member-calls.md) qualifies
19 ordinary/source prepared outcomes, five actual native programs and bounded
post-lowering work. It addresses original DropGuard::dismiss's generic const
member check; clean original-library verification is next. Three reduced library
metadata positives remain open; no whole PA or original artifact is complete.

PA1–PA24 have active, independently qualified text-oracle suites. PA21–PA24 are
**implemented**, with their completed release evidence recorded in
assignments.json. PA1–PA20 retain implemented-draft while individual release
records are reconciled. PA25–PA44 remain planned as whole assignments. No
planned or empty suite counts as passing.

The [wide constant product checkpoint](docs/pa25-wide-constant-products.md)
qualifies98 new exact/classified results, real source Mul const calls and native
byte observers. Release21 clears the original division site and reaches
midpoint's MAX*0.5 constant; unchanged original verification of multiplication
is next. Runtime/backend/transport and all whole-assignment requirements remain.

The [wide constant division checkpoint](docs/pa25-wide-constant-division.md)
qualifies74 exact/classified results, four native image observers and bounded
scaling through the shared rounding kernel. It addresses clean release20's
original f128.rs799 completion failure, confirmed cleared by release21.
Wide runtime/ABI/transport and whole-assignment requirements stay open.

The [loop result checkpoint](docs/pa16-loop-result-identities.md) qualifies25
complete safe/raw/source outcomes, bounded scaling and four native comparisons.
It retains the ordinary unbounded manual Copy-wrapper positive as unclosed.
Clean release19 clears the anonymous lifetime error and reaches IterMut's loop
loan conflict; original verification of the replacement repair is next. No
original library artifact or additional whole assignment is complete.

The [adopted assignment-order review](docs/assignment-order.md) replaces the old
numeric work/release chain for incomplete assignments. Keep PA numbers and
fixture paths as ownership IDs; use the machine-checked
[dependency plan](assignment-plan.json) for actual implementation and component
publication order. It has37 phase units,10 implemented finite components and
46 explicitly retained PA25–PA30 requirement families. All component publications
are still pending. Whole-PA completion requires its entire handout and release
checks, not only one working subset.

Source-declaration binding precedes source consumers; runtime memory precedes
symbolic CTFE; declaration metadata precedes body/value transport; required
build/edition/SIMD/assembly slices precede original core. Core-only real crates
start before alloc, followed by alloc consumers and std. Broader closure and
every valid unsupported requirement remain named work. The
[library bootstrap plan](docs/library-bootstrap-roadmap.md) retains original
source pins and the full acceptance ladder; the
[live tracker](docs/library-build-blockers.md) owns concrete failures.

The [core census and transport plan](docs/core-census-plan.md), adopted
2026-11-07, is the current execution path: repair LIB-193, add an analysis-only
keep-going census of original core, and develop branching body transport and
the alloc/std build lanes in parallel. The backend stays frozen. Assignment
requirements and status are unchanged.

The [projected callback-result checkpoint](docs/pa24-projected-callback-results.md)
qualifies83 complete outcomes from84 frozen ordinary/source/native contracts,
including projected array payloads, generic method-value correlations, nonempty
lifetime proof and source native order23/12. One ordinary native artifact and eight override contrasts
remain required. This finite checkpoint does not activate a planned assignment
or establish an original core artifact; the bounded original probe is next.

The [wrapped callback-premise checkpoint](docs/pa24-wrapped-callback-premises.md)
qualifies55 complete outcomes and four additional source type outcomes from68
independent contracts, including source native order23/1234. Thirteen accepting
artifacts remain required. The next unchanged original observation is pending;
no original core artifact or planned-assignment completion is claimed.

The [try-capture checkpoint](docs/pa24-try-captures.md) adds value-use traversal
for captured try blocks,21 complete source outcomes, actual native7/43/23/123 and
one-field scaling. Fifteen ordinary contracts and all earlier requirements remain
open. Preceding CI's16 existing text contracts are independently reviewed and
updated without changing input or acceptance outcomes. The unchanged original
observer clears filter_try_fold and first reaches filter_map_fold at its match
expression. All pinned inputs/configuration/limits verify; no original artifact yet.

The [computed-match capture checkpoint](docs/pa24-match-captures.md) qualifies
40 complete API outcomes from49 independent contracts,19 new PA24 text outcomes,
actual ordinary/source native order7/9/12314 and one-field scaling. Nine entire
positive outcomes and one additional public-helper metadata artifact remain
required. The unchanged original observer clears all three filter-map capture
bodies; first remaining is FlattenCompat::iter_fold at its nested helper call.
Pinned inputs/configuration/limits verify; full successor qualification remains
pending. No original artifact or planned-assignment completion is claimed.

The [helper return-input checkpoint](docs/pa24-flatten-helper-inputs.md) relates
opaque helper callable inputs through published bounds:46 complete API outcomes
from52 independent contracts,23 new PA24 text outcomes, actual ordinary/source
native7/9/1234 and call-linear bounded work. Six entire contracts remain required.
The unchanged original observer clears iter_fold and its five fold/rfold siblings;
first remaining is iter_try_fold's projected helper input. Full 43171710575 passed.

The [helper return-output checkpoint](docs/pa24-flatten-helper-outputs.md) exposes
opaque helper callable outputs after their declared inputs settle. It covers 26
complete API outcomes from 27 independent contracts, 15 new PA24 text outcomes
and observer-linked native 7/9/1234. One borrowed-receiver executable stays
required at the existing opaque-storage gate. An exploratory observation clears
iter_try_fold, iter_try_rfold and four try_fold/try_rfold callers. The first
remaining failure is map_while try_fold's method candidate graph. Clean pin
0665938a confirms this; the full successor is pending.

The [projected inherent header checkpoint](docs/pa20-projected-inherent-headers.md)
normalizes method candidate headers such as `ControlFlow<R, R::Output>` into
projection-equality conditions. It has nine new PA20 text outcomes and seven
native positives. One unbound-output rejection stays required for its cause. An
exploratory observation clears the six adapter `.into_try()` bodies. The first
remaining failure is next_if_eq behind a diagnostic-formatting fence. Clean pin
05289eb6 confirms this, with 49 records. Full 43188445172 for the helper-output
commit stopped on one stale private delayed-Never expectation. Rust's E0308
qualifies the updated cause, and full successor 43196757856 on dd7d8e9d3 passed.

The [rigid candidate header checkpoint](docs/pa20-rigid-candidate-headers.md)
refutes a trait implementation header that needs a goal's rigid associated item
to equal a parameter or a concrete head, as rustc does. It adds five PA20 and
one PA24 text outcomes, four native positives and seven source-core reductions.
Clean pin 487e0881 (source ec763f142) clears next_if_eq and is_sorted with no
new reached failure, within its unchanged 120s observer (28 records). The first
remaining failure is step_by first_size, a quantified body requirement. The
measurement supports no speed claim either way.

The [source Copy capture checkpoint](docs/pa24-source-copy-captures.md) lets a
closure in a source library copy a capture that is Copy only through a source
`impl Copy`, such as core's `NonZero<usize>`, so the closure stays `Fn`. Twelve
declared source-core reductions agree with rustc. An exploratory observation
clears first_size and other_size; the first remaining failure is zip fold, with
unresolved call inputs. Clean pin 12358002 (source d0e926179) confirms this with
46 records, and its measurement supports no speed claim either way. The same
probe now accepts a repeated array operand that is Copy only through a source
impl. Nine declared reductions agree with rustc, and an exploratory observation
clears `DisplayBuffer::new` and six float-formatting bodies that repeat
`MaybeUninit`. Clean pin ed7d5ed7 (source a5fdccff7) confirms this with 41
records.

The [specialized trait input checkpoint](docs/pa20-specialized-trait-inputs.md)
lets specialization order proved impl candidates whose inferred trait inputs
agree, as core's `ZipImpl` impls do. Nine declared source-core reductions agree
with rustc. An exploratory observation clears zip fold; the first remaining
failure is iterator count, behind the diagnostic formatting fence. Clean pin
3a1a6772 (source bcbafa25a) confirms this with 43 records, and its measurement
supports no speed claim either way.

The [default const closure checkpoint](docs/pa25-default-const-closures.md)
lets a const closure defined in a const trait's default body meet a `[const]`
callable bound through that body's `Self: [const] Trait` premise, as core's
`Iterator::count` needs. Ten declared source-core reductions agree with rustc.
An exploratory observation clears count; the first remaining failure is
iterator `last`, which needs a scoped continuation for a const fn item. Clean
pin eacd5629 (source a468bcc0b) confirms this with 36 records, and its
measurement supports no speed claim either way.

The [conditional const item checkpoint](docs/pa25-conditional-const-items.md)
lets a conditional const fn item, or a const trait member's item, meet a
`[const]` callable bound through its substituted const conditions, as core's
`last`, `as_deref` and `get_or_insert_default` need. Eleven declared
source-core reductions agree with rustc. An exploratory observation clears ten
core bodies; the first remaining failure is iterator `max`. Clean pin 716cbe11
(source be48c2501) confirms this with 32 records, and its measurement supports
no speed claim either way.

The [abstract callable input checkpoint](docs/pa24-abstract-callable-inputs.md)
lets a function item whose inputs are inferred only through a callable bound
meet an abstract projection input such as `I::Item`. It checks identity for each
blocked projection comparison, not for the complete input tuple, as core's
`max` needs when it passes `Ord::cmp`. Nine graded PA24 outcomes and eleven
source-core reductions agree with rustc. An exploratory observation clears
`max`, `min`, `max_by_key` and `min_by_key`; the first remaining failure is
`read_radix_max_digits`. Clean pin ead8009c (source a7ce2784a) confirms this
with 20 records, and its measurement supports no speed claim either way.

The [let-condition capture checkpoint](docs/pa24-let-condition-captures.md)
lets a closure consume a computed `if let` or `while let` scrutinee through the
existing capture-use walker, as core's `read_radix_max_digits` needs. Seven
graded PA24 outcomes agree with rustc. An exploratory observation clears that
body; the first remaining failure is `Pin`'s `PartialEq`/`PartialOrd`
forwarding.

The [premise input checkpoint](docs/pa20-premise-inputs.md) lets a caller
premise select a path call's inferred `Rhs` as its own projection. The early
argument expectation treats that projection as unknown, and reference coercion
normalizes it on demand, as core's `Pin` comparison forwarding needs. Seven
graded PA20 outcomes and nine source-core reductions agree with rustc. An
exploratory observation clears the seven forwarding bodies; the first remaining
failure is `Result::cloned`.

The [negative receiver checkpoint](docs/pa31-negative-receivers.md) lets a
method receiver step skip a trait requirement that a negative impl refutes,
even when the negative's own lifetime equations are unpaid, as core's
`impl !Clone for &mut T` needs for `t.clone()` on `&mut T`. Ordinary
obligations keep the conditional boundary. Ten source-core reductions agree
with rustc. An exploratory observation clears `Result::cloned`; the first
remaining failure is `AtomicPtr::new`.

The [parameterized storage checkpoint](docs/pa31-parameterized-storage.md)
normalizes a parameterized nominal's projected fields for a generic transmute
in a premise-free session, as core's `AtomicPtr::new` needs for
`Atomic<*mut T>`. A body premise that could keep the projection rigid keeps the
closed boundary. Fifteen source-core reductions agree with rustc. An
exploratory observation clears `new` and `into_inner`; the first remaining
failure is `SyncView::poll`.

The [pinned method call checkpoint](docs/pa43-pinned-method-calls.md) lets a
source method call find an inherent method declared with a smart receiver such
as `self: Pin<&mut Self>` through the legacy LegacyReceiver/Deref chain, as
core's `SyncView::poll` needs. Twelve source-core reductions agree with rustc.
An exploratory observation clears `poll` and `resume`; the first remaining
failure is `small_sort_network`.

The [comparison right operand checkpoint](docs/pa31-comparison-rhs.md) gives a
source comparison's `Rhs` a fresh variable, selects the comparison's goals and
then coerces the right operand, as rustc does. Core's `small_sort_network`
needs this for `*const T != *mut T`. Twelve source-core reductions and a
three-file per-body control agree with rustc. An exploratory observation
clears `small_sort_network`; the first remaining failure is
`bidirectional_merge` (destructuring assignment).

The [destructuring assignment checkpoint](docs/pa31-destructuring-assignment.md)
assigns tuple and `_` assignees leaf by leaf, as rustc's desugaring does. Core's
`bidirectional_merge` needs this for `(left, right, dst) = merge_up(...)`.
Seventeen source-core reductions agree with rustc. Rest, array, struct and
tuple struct assignees fail closed. A clean candidate observation clears
`bidirectional_merge`; the first remaining failure is `as_underlying`.

The [branch unsizing checkpoint](docs/pa31-branch-unsizing.md) gives `if` and
`match` branches one reference type, as rustc's `CoerceMany` does. An expected
`&[T]` is the common type. Without an expectation, an earlier `&[_; N]` unsizes
to a later `&[T]`. Core's `Split::as_slice` and `SplitInclusive::next_back`
need this. Fourteen source-core reductions agree with rustc.

The [closure input region checkpoint](docs/pa31-closure-input-regions.md) lets a
closure without an expected signature export parameter types that its body
gave reference regions, as core's `partial_compare` and `compare` closures do.
Those regions become output ports, so the body's lifetime requirements reach
the enclosing body. Thirteen source-core reductions agree with rustc. The
[inferred-input repair](docs/pa24-inferred-inputs.md) adds typed noncapturing
generic port instantiation, source proof transport and distinct marker witnesses,
with75 PA20/24/31 outcomes and actual ordinary/source native consumers. Private
and full successor qualification remain pending; two const closure lowering
paths and original library artifacts retain their existing requirements.

The [array length pattern checkpoint](docs/pa31-array-length-patterns.md) lets
an array pattern without a rest fix an inferred array length, as core's
`is_sorted_by` closure `|[a, b]| compare(a, b)` fixes `array_windows`'s `N`.
Thirteen source-core reductions agree with rustc.

The [closure creator requirement checkpoint](docs/pa31-closure-creator-requirements.md)
lets core's `is_sorted_by` form lower. A closure passes a callee-region
parameter to a captured `FnMut(&'a T, &'a T)`. Its region requirements on
creator regions become creator obligations, and it assumes its creator's
implied bounds. Eleven source-core reductions agree with rustc. The checkpoint
also fixes a generic instantiation regression from the closure input region
change.

The [closure capture pattern checkpoint](docs/pa31-closure-capture-patterns.md)
lets a closure match a captured place, as core's `fmt_decimal` closure tests
`if let Some(integer_part) = integer_part`. Its selected patterns read, copy,
move or borrow the place. Uses below an enum variant or an index capture the
whole place, and a more precise capture fails closed. Twenty-nine source-core
reductions agree with rustc, and two PA31 match-capture positives qualify. An
uncaptured scrutinee's lowering and precise field and referent captures remain
recorded gaps.

The [always-const premise checkpoint](docs/pa31-always-const-premises.md) lets
a `const` premise prove a `[const]` requirement, as core's `simd.rs` `eq`
bodies 9611 and 9643 need from `SimdElement`'s `const PartialEq` supertrait.
Eighteen source-core reductions agree with rustc, including four E0277 effect
rejections and an E0284 ambiguity that the base accepted. Two pre-existing
prepared lowering gaps are recorded: an impl-proved trait member call in a
const body, and an `&T == &T` comparison's receiver region.

The [GAT region premise checkpoint](docs/pa31-gat-region-premises.md) lets
core's `SplitInclusive` `clone` 8624 prove `Searcher<'a>: Clone` for the
`SplitInternal` impl's fresh region. A body opening its declaration's
lifetimes kept no certificate when an elaborated `Sized` source was opened
too. Fifteen source-core reductions agree with rustc. Premises for another
lifetime now fail prepared lowering's lifetime proof, as rustc's borrow
checker rejects them.

The [bool niche checkpoint](docs/pa31-bool-niches.md) uses byte2 for a source
enum's companion beside bool, through CTFE, runtime matching, cleanup and static
initialization. Twenty-one active source outcomes, three new native observers
and private course/source cache isolation qualify it. Candidate12 clears actual
core `as_underlying`7631; only two closure lifetime failures and their four blocked
methods remain in the census. Additional unused values of an already niche-encoded
enum remain a required positive, and course layouts keep their existing profile.

The [zero-sized niche companion checkpoint](docs/pa31-zero-sized-niche-companions.md)
admits source enums with two single-field variants when one field is zero-sized
and the other has an existing niche carrier. Eleven declared source cases match
Rust, including reverse orientation, const matching and the incorrect-size
opponent. Tagged fallbacks and the historical course profile remain unchanged;
candidate11 clears the actual core size assertion. Bool's nonzero niche and
closure output lifetimes remain pending.

The [global clause premises checkpoint](docs/pa31-global-clause-premises.md)
selects a normalizer's concrete trait premise from implementations before the
caller's clauses, as rustc discards a global where clause when an impl
applies. portable-simd's integer `Div` and `Rem` impls declare
`where i8: SimdElement`, and their guard's `select` needs
`<i8 as SimdElement>::Mask` to normalize to `i8`. Twenty-five source-core
reductions agree with rustc, including four rejections.

The [body projection normalization checkpoint](docs/pa31-body-projection-normalization.md)
registers a resolved body annotation that names a projection, such as
`T::Unsigned`, for body normalization, as a qualified path already was. A
coercion into an unresolved expected type now relates an actual type with a
nested pending projection by equality, as it relates a projection head.
portable-simd's `first_set` and both `TryFrom<&[T]>` bodies for `Simd` need
them. Seventeen source-core reductions agree with rustc, including four
rejections.

The [blanket candidate winnowing checkpoint](docs/pa31-blanket-candidate-winnowing.md)
discards, as rustc's winnowing does, each compatible impl whose trait
condition has no candidate before a projection's symbolic result is exposed.
Core's `(&[]).into_iter()` in `slice::Iter`'s `Default` needs it, because the
blanket `IntoIterator` impl matches every header. Eleven source-core
reductions agree with rustc, including a subject the blanket impl may still
satisfy. The scan still stops at the second compatible header without trait
conditions, so a large unconditional family, such as core's integer `Shl`
impls, costs no instantiation per further impl.

The [derived field path bound checkpoint](docs/pa31-derive-projection-bounds.md)
bounds each derived field type path that begins with a type parameter, such as
`I::Item`, by the derived trait, as rustc does. Core's derived `Clone` and
`Debug` of `Peekable` and `ArrayChunks`, and `Debug` of `Intersperse`, need
it. Sixteen source-core reductions agree with rustc, including a use that only
the new predicate refuses and rustc's type macro rejection.

The [comparison body lifetime checkpoint](docs/pa31-comparison-body-lifetimes.md)
gives a source comparison's `Self` fresh regions, so core's derived `eq` of
`marker/variance.rs:107` no longer leaks a body lifetime into `Rhs`. An
unconverted right operand is now borrowed in its own type. That also closes
the `&T == &T` prepared lowering gap. Twenty-one source-core reductions
qualify it, including two invariant rejections in the lifetime proof.

The [configured block tail checkpoint](docs/pa31-configured-tails.md) makes a
block whose written tail is configured out take its last enabled statement
written without a semicolon as its tail, as core's `fmt/num.rs` `Display`
bodies need. Such a statement elsewhere must now have type `()`. Sixteen
source-core reductions qualify it, including three rustc rejections the base
accepted.

The [derived tuple field privacy checkpoint](docs/pa31-derive-field-privacy.md)
anchors a derived tuple index on its field, as rustc spans it, so `self.0` in
a derived impl is checked from the item's module. 31 derived core `fmt` and
`eq` bodies on private tuple fields depend on it. Seventeen source-core
reductions agree with rustc, including three E0616 outside accesses.

The PA25 [const-machine component](docs/pa25-const-machine-release.md) is the first
released work unit: 113 declared outcomes, including four new recursion/work/storage
limit controls, pass text, independent Rust and native checks with frozen a08ab1a4.
Full 43177635950 passed on its release documentation commit. PA25 and every other
parent assignment remain planned.

| PA | Assignment | Status |
| --- | --- | --- |
| [1](pa1/README.md) | Source and tokens | implemented-draft |
| [2](pa2/README.md) | Literals | implemented-draft |
| [3](pa3/README.md) | Token trees and attributes | implemented-draft |
| [4](pa4/README.md) | Expressions and statements | implemented-draft |
| [5](pa5/README.md) | Items types and patterns | implemented-draft |
| [6](pa6/README.md) | Modules and names | implemented-draft |
| [7](pa7/README.md) | Macro matching | implemented-draft |
| [8](pa8/README.md) | Expansion and hygiene | implemented-draft |
| [9](pa9/README.md) | Types and inference | implemented-draft |
| [10](pa10/README.md) | Typed bodies | implemented-draft |
| [11](pa11/README.md) | Procedural LowIR | implemented-draft |
| [12](pa12/README.md) | Aggregates and layout | implemented-draft |
| [13](pa13/README.md) | Pattern semantics | implemented-draft |
| [14](pa14/README.md) | Moves and initialization | implemented-draft |
| [15](pa15/README.md) | Lifetime contracts | implemented-draft |
| [16](pa16/README.md) | Borrow checking | implemented-draft |
| [17](pa17/README.md) | Destruction | implemented-draft |
| [18](pa18/README.md) | Generics | implemented-draft |
| [19](pa19/README.md) | Traits and methods | implemented-draft |
| [20](pa20/README.md) | Trait solving | implemented-draft |
| [21](pa21/README.md) | Advanced generic relationships | implemented |
| [22](pa22/README.md) | Unsized types | implemented |
| [23](pa23/README.md) | Trait objects | implemented |
| [24](pa24/README.md) | Closures | implemented |
| [25](pa25/README.md) | Constant evaluation | planned |
| [26](pa26/README.md) | Language and library hooks | planned |
| [27](pa27/README.md) | Unsafe language | planned |
| [28](pa28/README.md) | Intrinsics and atomics | planned |
| [29](pa29/README.md) | Crates and metadata | planned |
| [30](pa30/README.md) | Foreign interfaces | planned |
| [31](pa31/README.md) | Compiler/library bindings | planned |
| [32](pa32/README.md) | Pinned core bootstrap | planned |
| [33](pa33/README.md) | Allocator and alloc bootstrap | planned |
| [34](pa34/README.md) | Edition and crate build integration | planned |
| [35](pa35/README.md) | Upstream crate acceptance | planned |
| [36](pa36/README.md) | Std and platform bootstrap | planned |
| [37](pa37/README.md) | Procedural macro host | planned |
| [38](pa38/README.md) | Derive and attribute macros | planned |
| [39](pa39/README.md) | Panic and unwinding | planned |
| [40](pa40/README.md) | Edition and attribute closure | planned |
| [41](pa41/README.md) | Target-specific closure | planned |
| [42](pa42/README.md) | Async lowering | planned |
| [43](pa43/README.md) | Async ownership | planned |
| [44](pa44/README.md) | Frontend acceptance | planned |

The goal remains all assignment requirements needed to build original pinned
core, alloc and std and execute independent real-crate consumers. Actual active
source determines required language/runtime/target capabilities; a later number
never excuses a failed library path. Full unwinding and async remain separate
from the initial abort-profile graph. Optional PA45 reproducible self-hosting
remains outside this goal. CPPGM is the explicit LowIR backend; backend extensions
require separate reviewed capability work.

The [overflow/automatic/CI checkpoint](docs/parallel/overflow-auto-ci-main-join.md)
clears the authenticated const-closure overflow attribute frontier. The actual
joined CLI now reaches LIB189, a cold closure in str/pattern.rs1858,19.5022s with
all722 inputs/profile/limits unchanged. Nine focused controls, strict selected
Clippy, architecture and282 harness checks pass. Full successor CI, genuine
const-closure callable/capture/CTFE semantics, Field consumers and all original
library artifact gates remain required; no new whole assignment is complete.

The [actual Field operation checkpoint](docs/parallel/field-proof-projection-checkpoint.md)
qualifies builtin proof, Base/Type, generic/move/lifetime constraints and actual
native projection/callback values with11 focused controls and strict metadata.
Provided OFFSET/intrinsic/layout execution, full reflection and transport stay
required. Whole-PA statuses and original core/alloc/std artifact gates are unchanged.

The [provided-default join](docs/parallel/field-provided-default.md) executes the
actual alternative Field constant and qualifies cold positions and const closure
creation/originals. The [source-header allocation repair](docs/parallel/source-header-allocation.md)
clears the measured local byte failure with exact owner buffers and real-row
accounting. The unchanged-source probe now reaches ptr_guaranteed_cmp declaration
policy in18.1196s, with no limit increase. The [PA20 oracle profile repair](docs/parallel/pa20-oracle-profile-repair.md)
preserves all oracle bytes and passes complete independent checking. Field-offset
intrinsic/layout, actual captured const execution and original library artifacts
remain required; whole-PA statuses are unchanged.

The [complete Field offset/native checkpoint](docs/parallel/field-offset-execution.md)
executes the actual provided initializer and reached intrinsic/layout, preserving
ordinary/generic/lifetime proof. Its full original native observer, eight private
controls and15 regular/scaling/initializer checks pass. Full reflection/field_of,
transport and original library/whole-assignment gates remain required.

The [captured closure/pointer integration](docs/parallel/captured-pointer-main-join.md)
qualifies actual shared/mutable/owned const calls and existing-component reuse,
pointer CTFE and real native fallback bodies together with Field. Six private,
28 regular and6 native controls pass; the unchanged original-core source now
reaches LIB192 staged deprecation in19.1083s. Conditional/generic caller proof,
receiver adapters, bootstrap and authenticated transport remain required. No
whole-assignment status or original library artifact gate changes.

The [cold/deprecation integration](docs/parallel/cold-deprecation-main-join.md)
qualifies actual prototype owner binding and staged release semantics, clearing
LIB192 to an unlocated shared work-budget guard193. Four private,23 regular and
six native controls pass on the same compiler snapshot; no limit/source change
or original artifact is claimed. Assignment status is unchanged. Root handoff.md
records the user-requested pause and exact restart boundary.

The [source structural-pattern checkpoint](docs/pa31-structural-patterns.md)
qualifies19 prepared source outcomes and a three-export native observer, with
actual trait identities, value-specific variants and name-token API allowances.
It reduces candidate13's Alignment failure; a successor original-core observation
is still due. Ordinary controls and empty-list positives remain explicit
requirements. No original library artifact or whole-PA release is implied.

The [persistent layout work checkpoint](docs/pa31-layout-work-owner.md) fixes
completion's use of whole-crate observations as layout spending. Independent
PA31 sources, high-observation prepared/native programs and original Field
controls pass with unchanged quotas and source permissions. Private execution
and a fresh unchanged original-core observation remain due; no whole assignment
or original library artifact is complete.

The [source unchecked-shift checkpoint](docs/pa31-unchecked-shifts.md) reduces
release23's genuine usize::unchecked_shl completion failure. All576 constant
results, nine candidate guards, bounded preparation and five new/old native
programs pass. A reference f32 ICE remains an infrastructure failure. Final local
replay/harness/upstream and a new full CI/original-core attempt remain pending;
no whole assignment or original library artifact is complete.

The [initializer-storage checkpoint](docs/pa25-initializer-storage.md) qualifies
24 outcomes and seven native programs for computed extended backing, while
preserving ordinary expiry and mutable temporary restrictions. Two valid named
mutable-static controls and nominal-pointer metadata artifacts remain required.
Private full CI and a fresh original-core observation are pending. Whole
assignments remain planned and the released const-machine component is unchanged.

The [associated Copy root checkpoint](docs/pa31-root-copy-qualification.md) reuses
scoped qualification for projected roots and filters only actual NoDrop answers.
Thirteen outcomes, eighteen native programs and bounded work pass; three unused
negative definitions remain required. Original-core retry/full CI are pending;
whole assignments and original library artifact gates remain unchanged.


The [source unchecked arithmetic checkpoint](docs/pa31-unchecked-arithmetic.md)
adds genuine add/multiply/divide/remainder to the existing typed scalar path.
Twenty-five source outcomes are independently qualified; the floating-point
opponent remains unqualified after a Rust ICE, with its required integer-domain
rejection preserved. All240 constant and240 runtime values across12 integer types,
five new native programs and bounded8/32/128 work pass. A real signed i32 native
failure was repaired with explicit operand extension. Clean original-core retry
and successor full CI remain pending. Whole PA25–44 and original library artifact
requirements remain unchanged; the separate PA25 component stays independent.


Clean7db0c611e/release27 now completes unchanged original-core census analysis with
zero records across42310 declarations/25858 bodies/35120 attempts. Genuine library
output still fails at compiler-owned async_drop_in_place state/polling; metadata
is absent and LowIR empty. The [v11 CLI integration](docs/parallel/deferred-operations-v11.md)
exposes an existing authenticated symbolic contract and qualifies eight focused
transport/guard tests. It does not replace the required original v13 full-body
artifact or executable future/native consumers. Combined ordinary-body/operation
transport and genuine async state remain required. No whole PA advances status.


The [mixed body/operation checkpoint](docs/parallel/body-operations-v14.md)
qualifies explicit metadata version14, preserving ordinary v13 node bodies and
actual nonexecutable async-drop operations together. Both orderings execute their
ordinary native consumer41; typed Output/unsafe, static lifetime, wire forgery,
owned-domain and bounded8/32/128 controls pass. Legacy v11/v13 bytes remain exact.
Full43597538902 passed every preceding arithmetic stage. A successor clean release,
full CI and genuine original-core library retry remain required. This metadata
prerequisite supplies no real async constructor/poll/drop/cancellation/recursive
cleanup and does not close the older original-v13 positive or whole PA25–44.
