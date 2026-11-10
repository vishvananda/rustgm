# Scoped quantified method lookup

After the [closed ordinary call prerequisite](pa21-universal-call-consumer.md),
lookup retains each nonempty quantifier with its inferred predicate. Both
qualified paths and receiver lookup open the actual universal scope, retain it
across declaration waits, and reject impossible closed lifetime debts before
trying a trait fallback. No lookup result supplies body proof authority.
Unresolved/caller-dependent conditions remain possible; selected calls still
prove their complete obligations. Outer captured binders and quantified
outlives lookup clauses keep their existing fences.

Compiler222d5174b first factors ordinary constness/twin handling without changing
behavior: four exact tests and127 complete immutable outcomes agree. Compiler
a52d5161b adds scoped lookup. A borrowed universal query retains a caller only
on suspension. The shared lifetime classifier recognizes a bound slot inside
its own retained scope. Zero-slot function signatures keep their owner context
when they have a declared environment: a concrete universal where clause still
belongs to that caller. Lookup shares that environment's existing typed premise
index and certificates instead of rebuilding it.

Seven PA21 sources independently match Rust1.98 objects before implementation.
They cover valid inherent selection, static-only rejection, static/diagonal
trait fallbacks, receiver fallback, a real caller premise and the function-pointer
Split shape. All run through types, generic semantics and complete LowIR. The
six retained PA31 bound-region controls now accept; their source bytes stay
unchanged. Existing const/empty-binder work and trace invariants still pass.
The suspension invariant covers ordinary and quantified queries at depths8/32/128,
including intervening body allocations and no published probe lifetime facts.

Full f01 CI42782379010 failed three real scaling controls: one captured-vtable
private case and two closure-context cases. Other3048 private tests and the other
regular targets passed; later native/debug/independent stages were skipped.
The early call consumer had included computed callable and unrelated builtin/
equality results. Their existing callable, normalization and final body-region
consumers retain those duties. Compiler48b0db2ae restores computed ownership;
61efef104 limits early consumption to ordinary implementation/caller-premise
selections. The static-only ordinary f01 still rejects. PA24 owns three concrete
width128 regressions, independently accepted by Rust and rejected by immutable
e8a01f0c with the old work-budget error. All three now pass both intermediate
semantics and LowIR, plus three actual native Rust/CPPGM executable comparisons.

Production1794301908630707445 passes12 exact checks across three targets,
including all original CI failures, ordinary f01, all scoped lookup cases,
the existing higher-ranked LowIR group, caller/equality negatives and work
invariants. Final ordinary-selection repair1794302856348070798 passes8 affected
checks; frozen PA24 inputs1794302473407168988 pass through both modes. Strict
selected/private Clippy, formatting and architecture pass.

All867 PA21 and493 PA24 course outcomes pass. All existing oracle bytes remain
unchanged. One new Split semantic oracle's internal IDs were regenerated after
independently checking byte-identical complete LowIR against the previous
candidate.313 harness tests/467 roots/44 stages and upstream128/Reference126 pass.
The127-input survey permits exactly the six declared PA31 accepts and preserves
every other complete outcome. Thirty immutable A/A+twoABBA observations preserve
ordinary lookup LowIR at8/32 functions and74 ordinary outcomes. Ratios are
1.036/1.022/1.015, A/A0.721/1.173/0.950; no speed claim.

Final source-bound dev-fast CLI
f24650d0dc28d39f0741377b48a50f0a02fd875921310c15e4eac4e1d2058a83
binds committed compiler61efef104, all compiler/test Rust hashes and an empty
patch. The unchanged722-input core census opens375 in83.96s and clears all
declaration analysis:42310 declarations checked, none failed/blocked. It enters
bodies, reporting checked0/failed67/blocked0 before terminal record68, a budget
stop at cell.rs:1393 (file112, byte47857), `!is_writing(self.borrow.get())`:
source unary declaration query exceeds its work budget. TSV SHA256
0bfa9f46206eafc260aa1533b21888b86f5c479c6a21971039ba3aa5a7a3602d.
This is analysis-only; no original library artifact exists.

Group the67 body failures:16 source dyn-upcast/3 nominal-tail Unsize fences,
12 inclusive-range call-plan fences,12 unavailable receiver methods,5 projection
normalizations,4 unknown field subjects and smaller groups. Attribute the unary
budget first to expose the rest of the census, then repair shared causes with
reductions in their owning assignments. Limits and original inputs stay fixed.
Full successor CI and optimized nested-proof measurements remain pending.

Raw failed attempts, independent objects, reference receipts, immutable binaries,
paired observations and census remain under pa26-work/obj/parallel/scoped-lookup.
Native evidence is obj/native/pa24/selected/1794303683196999869.
