# PA26 primitive assignment methods

All ten canonical assignment traits now select primitive implementations with
owned or shared-reference primitive RHS values when the library identity has
been demanded. Integral arithmetic, shifts and bitwise operations, floating
arithmetic and remainder, and boolean bitwise operations use the existing typed
scalar matrix. Invalid primitive pairs remain absent. Source impl admission
runs first and keeps the ordinary orphan/coherence rules, including valid
primitive Self with a local nominal RHS.

Selection retains a distinct `Source::LanguageOperation` descriptor and the
normal trait proof. Its factory token is private to successful concrete
selection and keyed by the complete function-item Instance. Method identity,
receiver, unit result, primitive operands, Rust convention and safety, and the
canonical generic substitution are independently validated before body demand.
The canonical callable substitution handles shared RHS lifetimes underneath
the method binder; raw type-ID equality is insufficient across that binder.

The demand-loaded factory emits ordinary Project, Read, Binary, Store and
Return actions. No backend intrinsic or generated Rust source is involved.
Its receiver keeps its late-bound lifetime; nonstatic RHS lifetimes become
separate body universals, while a static RHS stays static. The checked instance
path runs normal region, ownership, loan, destruction and arithmetic checks.
Validation returns a body-owned private-constructor Permit for the exact
Instance. The borrow layer consumes that permit without depending on semantic
selection types. Only this validated factory route may omit an original source template;
ordinary/manual instances still require one. Paired invariants reject missing
templates, different selected instances, changed descriptors and changed
contracts while accepting the intended factory proof.

The declared matrix contains 59 cases: 43 accepted programs, 15 Rust language
rejections and one valid Rust boundary. Controls cover all ten families and
forwarding forms, u128/i128, floats, booleans, generic order, inference, UFCS,
reified method pointers, named/static RHS lifetimes and orphan rules. Explicit
forwarding succeeds while aliasing the mutable receiver and shared RHS rejects.
The earlier assignment suite's unchanged generic primitive boundary now accepts.

Implicit canonical hook availability remains unfinished: `x += &y` without any
written canonical trait demand still follows the earlier scalar boundary. The
paired explicit `use core::ops::AddAssign` control exercises the implemented
route. Direct library-trait dot-call lookup, other primitive operators,
comparison/Deref, iteration, question-mark/residual flow, additional derives,
formatting/runtime hooks and remaining library methods also remain. This is
not assignment activation or full cumulative release qualification.

Evidence is retained under `obj/parallel/pa26/primitive-assignment/`. The first
factory gate recorded missing projection dominance and incorrect cross-binder
RHS comparison; both were repaired through ordinary Project actions and typed
canonical substitution. The second gate passed 57 source controls, 42 native
programs and the paired factory proof invariant. The final gates also recorded strict-lint cleanup, a real mixed-unsuffixed
selection regression, and a forbidden borrow-to-sema dependency. Rigid
nonprimitive operands now exclude builtin evidence before unresolved scalar
operands cause deferral; the unchanged mixed user-impl control passes again.
The body-owned Permit resolves the phase dependency without relaxing proof.

All 21 focused groups passed, including 43 primitive and 25 assignment native
programs, existing IndexMut/operator/call controls, static permissions and
syntax transfer. The paired factory invariant and strict all-target/all-feature
Clippy passed after the permit handoff. Architecture audit covered 1,296 files
with zero findings; layout covered 38 stages; upstream verified 24 pinned
fixtures and 126 Reference files. Functional results are in
`factory-final-fourth.json`; the final proof/lint/audit/layout/upstream results
are in `factory-final-fifth.json`. Earlier failed observations remain separate.
Cumulative performance and full release qualification remain pending.
