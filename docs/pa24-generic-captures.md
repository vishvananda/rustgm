# Captured generic closure execution

PA24 remains planned. This checkpoint specializes captured closure bodies while
retaining their complete enclosing type, const and inherited lifetime arguments.
Unused arguments remain part of identity; equal complete instances reuse one
body. Generic receiver adapters, dynamic callable objects and assignment release
qualification remain unfinished. A captured lifetime-bearing associated projection
also has a retained source-normalization failure described below.

The complete execution Self contains the actual capture environment and public
call signature. Parameter argument lifetimes occupy fresh slots after the entire
source execution binder, including its hidden receiver. Structural matching
preserves nested public binders and supplies source identities; repeated slot
matches become equality debts. Projection normalization retains private witnesses,
selected lifetime obligations and nested proof scopes. Private witnesses are
rebased past existing caller allocations before actual caller slots are restored.

Original closure templates still pass the complete region, loan, move and cleanup
checks, including unused templates. Their conditional output summaries retain
formal lifetime requirements. The borrow layer exposes these checked summaries
through a visitor; lowering publishes the exact source-keyed requirements into
semantic state. This avoids a backwards borrow-to-semantics dependency.

Every selected generic invocation maps those requirements into caller outlives
obligations. Only demand from a sealed, fully checked caller may use the original
formal requirements as callee preconditions. Public/manual instantiation retains
the Required path. Specialized bodies still prove their original constraints,
formation requirements, normalized projection debts and ordinary borrow rules.
The original enclosing parent's conclusions never become blanket permission.

Condition pairs share an immutable Arc between the checked summary and catalog.
A checked unconditional entry differs from an absent, unchecked entry. Repeated
publication must preserve exact requirements. A 64,000,000-byte logical budget
charges conservative hash entry capacity, the pair payload and Arc counters once;
it is not an allocator/RSS measurement. Ordinary functions allocate no catalog.

Capture formation retains both the physical parent field type and the exported
field type. An actual read/borrow uses the physical type, with a separately retained
ordinary lifetime coercion to the exported alias. This applies equally to moved,
shared, mutable and unique immutable captures and preserves exact physical place
checks. The additional retained formation vector costs one TypeId per capture.

## Focused evidence

`pa24/tests/generic-capture-outcomes.json` declares 21 programs: fourteen valid
programs and seven genuine rejections. Pinned Rust 1.98 full object generation
qualifies each outcome. The three frontend views distinguish ordinary lifetime/
move/loan validation from the earlier type-only view. Cases cover move/shared/
mutable capture, const arguments, returned opaque closures, captured results,
nesting, public input binders, unused arguments, reference-valued type arguments,
associated scalar captures and destruction. Local-result and escaping-reference
controls must fail at lifetime proof, not an unimplemented formation boundary.

Before promoting the final three followups, focused-seventh passed all 552 private
library tests, 52 source test groups across closure_lifetimes/environment_execution/
source_closures, fourteen explicit native groups, strict all-target/all-feature
Clippy, formatting, the file/architecture audit (1,409 files; zero findings) and
upstream checks. The original eighteen outcomes and four independent followup
controls matched. Evidence is `obj/parallel/pa24/generic-captures/focused-seventh/`.
The successful scoped guard removed 358 generated files (0.65 GiB of unshared
blocks), retaining diagnostics, source inputs and receipts. The final promoted
batch passed all 21 outcomes in all three views and all fourteen positive native
executions, formatting, strict Clippy, file/architecture and layout audits, and
upstream checks. Evidence is `obj/parallel/pa24/generic-captures/promoted/`;
its scoped cleanup removed another 42 generated files (0.06 GiB).

At widths 8/32/128, the ordinary control records zero closure canonical steps.
Repeated calls to one demanded captured generic body record 38 steps and one
closure body at every width. Independent captured bodies record 304/1,216/4,864
steps and 8/32/128 bodies. These count the instrumented canonical work, not total
compiler work or elapsed time. Full wall-time/RSS/instruction measurement and
combined release qualification are pending; no throughput claim follows here.

The earlier prototype copied source conditions, omitted some normalization work
charges and compared borrowed exported aliases directly with physical places.
The focused checks exposed those gaps. A separate attempted rewrite of all child
imported bindings to inference aliases failed existing exact borrow-placement
controls and did not repair the projected-reference failure; it was reverted.
All failed observations remain in `focused/`, `focused-second/` through
`focused-sixth/` and the numbered probe directories.

`extras-declared.json` and `extras-second/` retain the independently valid
`Pick<'a>::Out = &'a u8` captured-reference program and its escaping counterpart.
An ordinary function using the same projection passes. Before the scoped correction below, the captured variant failed source
normalization before its intended lifetime proof; that earlier negative result
was not counted as semantic validation. No generated oracle or reclassification
as a supported rejection accompanied the probe.

## Projected captured-reference transport

The scoped query fix retains the original `Pick<'a>::Out = &'a u8` captured
result and escape sources in `pa24/tests/spec/projected-captures/`. The positive
now passes all three checked views and native value execution. The escape reaches
the ordinary lifetime proof. `projected-capture-outcomes.json` declares each
independent full-object Rust outcome and the separate frontend modes.

A canonical written projection uses the canonical caller clauses, while a resolved
free-alias projection uses the current body's query environment. A closure's query
environment transports parent premises through the same `Interface.opening` as its
inputs. This imports no selection witnesses, parent proof, or execution permission.
Imported parent bindings already use the parent's established aliases. Their
existing incoming transport opens readonly views through the child Interface.
Only an inward equation batch belongs to the child's complete lifetime namespace;
it folds both roots into that body's existing alias namespace before normalization.
Outward batches retain the existing exact Interface.closing path, including public
signature roots whose binder does not belong to the parent body.
Original bindings, closure metadata and source roots remain intact; every resulting
equation still requires the receiving body's normalization and ordinary proof.
The common refresh retains a typed inward/outward direction and one prepared
inward batch across dependency waits. Each inward batch shares a `RegionMapping`
and the body's cached `Bound` fold across all roots.
Distinct slots and nested callable binders remain distinct.

The expanded fixtures retain three frontiers. A written projected closure parameter
(`written-input`) remains valid Rust but fails the lifetime proof. Creating a
nested quantified function pointer from a closure (`nested-binder`) remains valid
Rust but hits the conditional nested output shape boundary. Its invalid escaping
counterpart (`nested-input-escape`) hits that same boundary and is not counted as a
validated semantic negative. The named-function nested-binder control succeeds.
No generated references or assignment activation change with this fix.

The first broad fold attempted to align outward and parent-source roots too. The
existing quantified-input output tests exposed an invalid inference identity:
a parent with no body binder received a standalone root owned by the nested
callable's public binder. The correction uses the typed interface direction,
not a numeric-slot heuristic. Parent-source folding proved redundant and was
removed. The quantified-input controls and the additional generic-parent/public-
child-binder fixture retain their independent acceptance expectations.

Focused qualification retains eleven independently compiled Rust fixture objects:
five supported positives, three intended semantic negatives and three explicitly
uncovered frontiers. The five supported positives also execute independently and
through pinned CPPGM. Three private interface tests cover readonly source roots,
canonical/alias views, distinct slots, local callable binders and shared mapping
work at 16/64/256/1,024 roots. At 8/32/128 projected captures, the changed-path
family records 8,938/34,810/138,298 inference, canonicalization and projection steps.
These are instrumented work counts for this family, not total heap, wall time or
a general complexity claim. Existing adjusted-place, public-output signature,
generic-capture, source receiver and inline-constant suites pass on the scoped fix.
Evidence, including failed broad-fold observations, is retained under
`obj/parallel/pa24-projected-repair/`.

The unchanged ordinary-function control is retained only in generated evidence
(`controls/ordinary.rs` and the earlier probe directory), rather than adding a
closure-free required fixture to PA24. Its independent object, three-view and
native observations remain preserved with the superseded twelve-case batch.
