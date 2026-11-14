# Explicit const dereference results

Clean release29's genuine original core library build clears DropGuard::dismiss,
then stops at mem/drop_guard.rs:118, `DropGuard::deref`'s `&*self.inner`, with
`const member changed its current signature or result`. The actual call returns
`&T`, while the dereference expression denotes the referent place `T`. The
source-request validator already handled that distinction for indexing, but
classified every such result as an Index expression.

The same checked-place validator now accepts the two actual syntax protocols:
Index/IndexMut with two parameters, and explicit Deref/DerefMut with one. It
retains the complete selected callable, actual language-trait owner, typed source
operand, receiver borrow, exact arity, output reference mutability/lifetime and
expression referent checks. Its operand validation and execution/const dispatch
requirements are shared with the existing indexing path. No new permission,
dense field, cache, wire format, backend or resource policy is introduced.

The [PA31 source packet](../pa31/tests/spec/explicit-const-dereference/manifest.json)
freezes13 complete standalone CLI and prepared outcomes:10 accept/3 reject,
including shared/mutable operators, original impl-body shapes, constant execution,
ordinary runtime control and8/32/128 scaling. Nonconst impl, shared mutation and
escaping lifetime opponents reject for their actual causes. Pinned Rust1.98 agrees
on all13 semantic counterparts. The two independent and actual emitted native
programs each return41: one mutates through the generic call, the other observes
its CTFE result. Seven prior indexing native programs remain unchanged and pass.

The ordinary standard-core counterparts live in
[PA26](../pa26/tests/spec/explicit-const-dereference/manifest.json).
All11 candidate ordinary commands currently stop at an unresolved core import;
this is an unqualified bootstrap boundary, including for the three rejecting
Rust counterparts. None is counted as candidate semantic rejection or inverted
into a negative. Actual source-role and standalone CLI qualification does not
replace those required ordinary-library outcomes.

Focused1794672298823814172 passes six exact tests across frontend/traits in155.92s,
including all73 source indexing cases, generic field-call controls and the future
helper CFG guard, with strict production/selected/CLI Clippy, formatting and
architecture checks. Native1794672574166858411 passes two groups/nine actual
programs in42.58s. Final authenticated replay binds3964 compiler/test source
hashes and both fixture manifests, repeats the two new regular tests and checks
all13 CLI outcomes. At8/32/128 functions, inference/canonical work after prepared
validation is3917/1919,15437/5785,61517/21225. No wall-time speed claim.
315 harness/495 roots/13 binaries/44 stages/one component and128 upstream+
126 Reference checks pass. Raw evidence is `obj/parallel/explicit-members/`.

The private forgery test preserves receiver/callable/borrow, referent and exact
arity opponents; the old Index-specific regression uses the shared validator.
These private changes await full CI rather than a cold local harness. A temporary
new-test reference to a nonexistent ListId::EMPTY was corrected to the existing
intern_list API before publication; it never supplied passing private evidence.

Original full-source verification remains pending. Before another optimized
compiler freeze, repair the independently reproduced v14 declaration gate so the
next original-core attempt covers both related blockers. Full43611467168 on the
preceding170c checkpoint passed fast/lint/harness/host/release/course and is in
native qualification; do not dispatch a duplicate. Original v13/v14 library
metadata, LowIR and native consumers, three earlier reduced metadata positives,
all future state/poll/drop work and every older handoff requirement stay open.
Whole PA25–PA44 remain planned; the113-case PA25 const-machine component is unchanged.
