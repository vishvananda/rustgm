# Generic closure body proofs

Closure CFG construction now shares one constructor between ordinary bodies and
templates. A closure in a generic owner retains its source closure identity and
shares the owner's immutable permission environment. Every original template,
including unused closures and nested closures, passes the existing ownership,
loan, promotion, lifetime and destruction checks. The checked template token
keeps a separate closure index; function and initializer lookup remain unchanged.

Formal closure signatures restore inference aliases to their declared body
universals before those signatures contribute assumptions. The original equality
obligations remain required. Already closed signatures take a constant-time
property check and bypass equation construction. This fixes the generic
`|value: &T| -> T { *value }` proof for `T: Copy` without weakening the
corresponding move and escaping-reference rejections.

Capture-free closures defined in generic owners now use the same instance demand,
substitution, ownership, promotion, destruction and lowering pipeline as generic
functions. An instance retains its complete canonical closure type, including
unused enclosing type and const arguments. Equal arguments reuse one body;
different arguments remain distinct even when their call signatures are equal.
A compact source-identity index provides constant-time template lookup. The
existing 128-active-instance, 100,000-instance and 64 MiB demand limits apply to
closure families as well as functions. Ordinary closure names remain unchanged.

Required nested bodies exchange constraints through their own lifetime universes.
The receiving body registers and normalizes projected inputs before reconciling
those constraints. Transport does not confer normalization evidence, and the
transactional validation used by speculative trait queries remains unchanged.
This permits projected annotations, contextual function pointers, partial tuple
annotations and nested closures while still rejecting incompatible projected
results. Declared opaque callable bounds also supply contextual parameter types;
the original bound obligations must subsequently be proved.

Inherited enclosing lifetime arguments now retain an explicit prefix in the
closure body proof. Concrete references supplied through type parameters also
work. Captures, environment receivers and dynamic callable adapters remain
unfinished. PA24 is not active.

The focused `closure-templates-v2` batch passed 30 tests across generic closure
templates, source closures, closure lifetimes/pointers and template loan/region/
cleanup groups. Fourteen declared source outcomes were independently checked
with pinned rustc. The initial borrowed-copy proof failure remains in the v1
logs. A fifteenth contextual-borrow control and a formal-signature invariant
were added afterwards, together with the closed-signature fast path.

The v3 gate stopped at a test-only Clippy spelling error, which was corrected.
The v4 formatter and strict all-target/all-feature release Clippy command passed;
the guard was interrupted during its historical cleanup scan before audit,
focused and native stages. That incomplete receipt is not a passing gate.
The subsequent executable specialization checkpoint passed formatting, strict
all-target/all-feature release Clippy, the syntax-aware file audit (1,251 files,
zero findings), 59 focused release test groups and four native groups. Its 21 new
positive programs agree with pinned rustc and execute through the locked CPPGM
backend. Six independently rejected programs check permissions and result types;
unsupported diagnostics cannot satisfy those rejection checks. Instance reuse,
growing recursion and the inherited-lifetime boundary have separate controls.

The specialization evidence is in
`obj/parallel/closure-specializations-gate-v17/`, including the frozen source
manifest and candidate SHA-256
`0cc5e9d885671fc5433ee11908420cda8695f1fb46a0da4f7c85e20c8d3910ac`.
Earlier failed projection experiments and the v16 lint failure are retained;
they are not passing coverage. The matched development study is recorded in
[the specialization measurements](pa24-specialization-performance.md).
Combined publication checks remain pending. No assignment release claim is made.
Evidence remains under `obj/parallel/closure-templates-v*/` and the earlier
v1/v2 logs. Ordinary native and active text coverage for the preceding compiler
snapshot is recorded separately in the publication checkpoint.

## Inherited lifetime specialization

The source callable and its body proof use separate binders. Each closure records
how many leading body universals come from its immediate enclosing body. Instance
selection reconstructs the call signature with the actual inherited arguments,
then applies type/const substitution. The body imports those arguments and any
reference-bearing type arguments through one fresh lifetime folder, connecting
the original inherited slots to their actual identities in its checked contract.
Nested closure input binders remain distinct. The unproved body-only API rejects
instances that need this inherited lifetime contract.

Inference preserves a closure's invariant nominal identity across inferred value
bindings. Previously, freshening its captured arguments and signature separately
could give the same inherited lifetime two independent IDs. The ordinary source
coercion retained their equality, but instance selection correctly rejected the
inconsistent callable identity. Keeping the original closure graph retains that
relationship without extra lifetime allocations or a later whole-body equality
scan. A focused invariant covers both canonical and materialized inference roots,
including constant work and preserved arguments/signature identity.

Seven additional independently accepted source/native programs cover late and
early inherited lifetimes, nested closures, inherited plus elided inputs,
returned function pointers, reference-bearing type arguments and unused inherited
parameters. Two additional independently rejected programs cover escaping an
inherited reference and moving from a borrowed generic value. The specialization
matrix now has 28 positive programs and eight semantic rejection controls.

The combined gate in `obj/parallel/closure-inherited-gate-v4/` passes formatting,
strict release all-target/all-feature Clippy, the 1,272-file zero-finding audit,
86 focused release groups and six native groups. It includes the integrated
scalar named-constant metadata tests. The compiler SHA-256 is
`5d62c74e1046eb4aa04c821f08b1fbb38459424981ee4686ec0813fcfd277dc9`.
The earlier failing v1/v2 signature checks and passing focused v3 remain in the
logs. These checks do not constitute full publication or assignment release;
matched performance qualification of this newer snapshot remains pending.
