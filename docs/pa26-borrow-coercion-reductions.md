The [source implementation checkpoint](pa26-borrow-coercions.md) supersedes the
preimplementation candidate status below. Its57 independent outcomes retain all
original28 bytes/outcomes and qualify39 source checked bodies/four real source
native programs. Ordinary18/imports/native parity remain required.

# RefCell borrowed-wrapper coercion reductions

The next reached original-core blocker is RefCell::replace in cell.rs at
file112:35607,line1026: `mem::replace(&mut self.borrow_mut(), t)` reports
incompatible types. The returned RefMut wrapper needs an actual DerefMut call
before it can supply `&mut T`. The frozen callback compiler05cc already accepts
the corresponding direct reference and explicit `&mut *wrapper` typing controls.
It rejects all7 implicit source conversions, even with explicit type arguments.
This establishes a conversion gap independently of the later cloning records.

[PA26](../pa26/tests/spec/borrow-coercions/README.md) owns13 ordinary hook inputs,
10 independent accepts/3rejects. [PA30](../pa30/tests/spec/borrow-coercions/README.md)
owns the ordinary C ABI observer and its shared C driver. [PA31](../pa31/tests/spec/borrow-coercions/README.md)
owns14 source-role counterparts with the exact existing const-Deref prefix,
11accepts/3rejects. All28 independent object outcomes were declared before Rust1.98
execution and agree22accept/6reject. Every byte and owner is bound in its manifest.
Ordinary forms stop at existing core::ops imports; those positives remain required.

The two real Rust/C native observers print `borrow-coercion 23 41 1342`, status0.
The wrapper is evaluated once, actual DerefMut runs before evaluating the next
argument, stored23 becomes41, and actual Deref supplies a later shared read.
Candidate RustGM/CPPGM execution remains a REQUIRED POSITIVE. A layout cast or
empty result cannot replace this trace. Both escaping-reference opponents
independently reject with E0515; baseline incompatible types does not qualify
the required loan rejection after conversion works.

Receipt1794406262569032574 passes3 exact public tests/1target in48.30s: five new
complete checked-body controls (3direct/explicit positives and2actual mutability/
wrong-target negatives) plus two existing implicit-receiver contract groups.
It uses the existing prepared root-builder API; CLI source-library language
contract transport is a separate limitation, retained in raw probe outputs.
All7 implicit source positives, the actual escaping-loan cause and native parity
remain unqualified. Strict selected Clippy/full formatting/architecture pass.
Upstream128/Reference126 and315 harness tests/468roots13bins44stages pass.
No course oracle or assignment activation occurs. The
[machine record](pa26-borrow-coercion-reductions-evidence.json) retains limitations
and the first formatting-only failed selection as well as the final pass.

The implementation should reuse existing charged Deref/Target discovery and
actual dereference_call selection. Reference coercions need a retained actual
call chain distinct from the expression's original operation: an already selected
method, field or call can itself undergo conversion. Preserve those operations
and their existing plans. Reuse actual source-use effects, call instantiation,
closure capture loans and variance-aware region proof. The final borrow must
follow the actual selected return-reference lifetime. Checked-body lowering must
evaluate the original operand and each real call once through the existing
access-call machinery. Selecting an associated Target never grants DerefMut,
const-call or loan permission.

Then qualify the7 original implicit source positives, the genuine escape error,
and the real native trace before extending to deeper chains/unsizing/context
forms. Preserve bounded work/noise and all existing128-depth inference debt.
Keep ordinary bootstrap imports separate, without changing their required
outcomes. No new proof engine, source pin, observer limit, backend contract or
storage.py change is part of this reduction checkpoint.

No new original-core observer or compiler binary was needed for these inputs/tests.
The last original observation is still callback source5798's partial120s deadline,
first RefCell::replace after the reached LazyCell initializer failure cleared.
Full43034322443 on exact357d0bc5e/source5798 remains the SAME active run; input/test/
documentation descendants do not restart it. No original core/alloc/std artifact
exists, and every prior handoff requirement remains required.
